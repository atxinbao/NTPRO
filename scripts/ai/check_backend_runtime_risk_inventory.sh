#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
INVENTORY="$ROOT/docs/rust-cutover/quality/backend_runtime_risk_inventory.json"
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
    rg -n --no-heading -e "$pattern" crates -g '*.rs'
  else
    find crates -type f -name '*.rs' -print0 \
      | xargs -0 grep -nH -E -- "$pattern"
  fi
}

cd "$ROOT"
[[ -f "$INVENTORY" ]] || fail "missing inventory: $INVENTORY"

{
  scan_rust_sources '#\[cfg\(test\)\]' || true
} | awk -F: '!seen[$1]++ { print $1 "\t" $2 }' >"$TMP_DIR/test_starts.tsv"

{
  scan_rust_sources \
    '[.]unwrap[[:space:]]*\(|[.]expect[[:space:]]*\(|panic![[:space:]]*\(|todo![[:space:]]*\(|TODO|unsafe|dead_code|unused' \
    || true
} >"$TMP_DIR/raw.txt"

awk -F '\t' '
  NR == FNR {
    first_test_line[$1] = $2
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
    if (path ~ /\/tests[.]rs$/) test_owned = 1
    if (path ~ /_test[.]rs$/) test_owned = 1
    if ((path in first_test_line) && line_number >= first_test_line[path]) test_owned = 1

    if (path ~ /\/generated\//) {
      ownership = "generated"
    } else if (path ~ /\/src\/ffi(\/|[.]rs$)/) {
      ownership = "ffi"
    } else if (test_owned) {
      ownership = "test"
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
' "$TMP_DIR/test_starts.tsv" "$TMP_DIR/raw.txt" \
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
    fail "unknown mode '$MODE'; expected verify or --summary"
    ;;
esac
