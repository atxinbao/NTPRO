#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
INVENTORY="${NTPRO_RISK_INVENTORY:-$ROOT/docs/rust-cutover/quality/backend_runtime_risk_inventory.json}"
SCAN_ROOT="${NTPRO_RISK_SCAN_ROOT:-crates}"
INLINE_RANGE_SCANNER="$ROOT/scripts/ai/inline_test_ranges.awk"
MODE="${1:-verify}"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

fail() {
  echo "backend runtime risk inventory failed: $*" >&2
  exit 1
}

hash_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

scan_rust_sources() {
  local pattern="$1"

  if [[ "${NTPRO_RISK_SCAN_FORCE_GREP:-0}" != "1" ]] && command -v rg >/dev/null 2>&1; then
    rg -n --no-heading -e "$pattern" "$SCAN_ROOT" -g '*.rs'
  else
    find "$SCAN_ROOT" -type f -name '*.rs' -print0 \
      | xargs -0 grep -nH -E -- "$pattern"
  fi
}

cd "$ROOT"
[[ -f "$INVENTORY" ]] || fail "missing inventory: $INVENTORY"
[[ -f "$INLINE_RANGE_SCANNER" ]] || fail "missing inline test range scanner: $INLINE_RANGE_SCANNER"

while IFS= read -r -d '' source_file; do
  awk -v source_path="$source_file" -f "$INLINE_RANGE_SCANNER" "$source_file"
done < <(find "$SCAN_ROOT" -type f -name '*.rs' -print0) \
  >"$TMP_DIR/inline-test-lines.tsv"
printf '\t0\n' >>"$TMP_DIR/inline-test-lines.tsv"

{
  scan_rust_sources \
    '[.]unwrap[[:space:]]*\(|[.]expect[[:space:]]*\(|panic![[:space:]]*\(|todo![[:space:]]*\(|TODO|unsafe|dead_code|unused' \
    || true
} >"$TMP_DIR/raw.txt"

awk -F '\t' '
  FNR == NR {
    inline_test[$1 SUBSEP $2] = 1
    next
  }

  {
    first_colon = index($0, ":")
    remainder = substr($0, first_colon + 1)
    second_colon = index(remainder, ":")
    path = substr($0, 1, first_colon - 1)
    line_number = substr(remainder, 1, second_colon - 1) + 0
    text = substr(remainder, second_colon + 1)

    test_owned = 0
    if (path ~ /\/tests\//) test_owned = 1
    if (path ~ /\/benches\//) test_owned = 1
    if (path ~ /\/test_kit\//) test_owned = 1
    if (path ~ /\/src\/testing\//) test_owned = 1
    if (path ~ /\/src\/stubs\//) test_owned = 1
    if (path ~ /\/tests[.]rs$/) test_owned = 1
    if (path ~ /_test[.]rs$/) test_owned = 1

    if (path ~ /\/generated\//) {
      ownership = "generated"
    } else if (path ~ /\/src\/ffi(\/|[.]rs$)/) {
      ownership = "ffi"
    } else if (test_owned) {
      ownership = "test"
    } else if (inline_test[path SUBSEP line_number]) {
      ownership = "test_inline"
    } else {
      ownership = "production"
    }

    if (text ~ /[.]unwrap[[:space:]]*\(/) {
      print ownership "\tunwrap\t" path "\t" line_number
    }
    if (text ~ /[.]expect[[:space:]]*\(/) {
      print ownership "\texpect\t" path "\t" line_number
    }
    if (text ~ /panic![[:space:]]*\(/) {
      print ownership "\tpanic\t" path "\t" line_number
    }
    if (text ~ /todo![[:space:]]*\(/) {
      print ownership "\ttodo_macro\t" path "\t" line_number
    }
    if (text ~ /TODO/) {
      print ownership "\tTODO\t" path "\t" line_number
    }
    if (text ~ /(^|[^[:alnum:]_])unsafe([^[:alnum:]_]|$)/) {
      print ownership "\tunsafe\t" path "\t" line_number
    }
    if (text ~ /dead_code/) {
      print ownership "\tdead_code\t" path "\t" line_number
    }
    if (text ~ /(^|[^[:alnum:]_])unused([^[:alnum:]_]|$)/) {
      print ownership "\tunused\t" path "\t" line_number
    }
  }
' "$TMP_DIR/inline-test-lines.tsv" "$TMP_DIR/raw.txt" \
  | LC_ALL=C sort -t $'\t' -k3,3 -k4,4n -k2,2 -k1,1 \
  >"$TMP_DIR/canonical.tsv"

total_matches="$(wc -l <"$TMP_DIR/canonical.tsv" | tr -d ' ')"
matched_files="$(cut -f3 "$TMP_DIR/canonical.tsv" | LC_ALL=C sort -u | wc -l | tr -d ' ')"
scan_sha256="$(hash_file "$TMP_DIR/canonical.tsv")"

summary="$(
  jq -n \
    --argjson total_matches "$total_matches" \
    --argjson matched_files "$matched_files" \
    --arg scan_sha256 "$scan_sha256" \
    --slurpfile ownership <(
      awk -F '\t' '{ count[$1]++ } END { for (key in count) print key "\t" count[key] }' \
        "$TMP_DIR/canonical.tsv" \
        | LC_ALL=C sort \
        | jq -Rn '[inputs | split("\t") | {(.[0]): (.[1] | tonumber)}] | add // {}'
    ) \
    --slurpfile signals <(
      awk -F '\t' '{ count[$2]++ } END { for (key in count) print key "\t" count[key] }' \
        "$TMP_DIR/canonical.tsv" \
        | LC_ALL=C sort \
        | jq -Rn '[inputs | split("\t") | {(.[0]): (.[1] | tonumber)}] | add // {}'
    ) \
    '{
      total_matches: $total_matches,
      matched_files: $matched_files,
      by_ownership: $ownership[0],
      by_signal: $signals[0],
      canonical_sha256: $scan_sha256
    }'
)"

case "$MODE" in
  --canonical)
    cat "$TMP_DIR/canonical.tsv"
    ;;
  --summary)
    jq -S . <<<"$summary"
    ;;
  verify)
    expected="$(jq -S -c '.scan.summary' "$INVENTORY")"
    actual="$(jq -S -c . <<<"$summary")"
    [[ "$actual" == "$expected" ]] || {
      echo "expected: $expected" >&2
      echo "actual:   $actual" >&2
      fail "scan summary drift"
    }
    echo "backend runtime risk inventory OK: $total_matches signals in $matched_files files"
    ;;
  *)
    fail "unknown mode '$MODE'; expected verify, --summary, or --canonical"
    ;;
esac
