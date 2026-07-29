#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CURRENT_REGISTER="${NTPRO_IGNORED_CURRENT_REGISTER:-$ROOT/docs/rust-cutover/quality/ignored_tests_register.md}"
HISTORICAL_REGISTER="${NTPRO_IGNORED_HISTORICAL_REGISTER:-$ROOT/docs/rust-cutover/verification/ignored_tests_risk_register.md}"
REGISTER_SCOPE="${NTPRO_IGNORED_REGISTER_SCOPE:-$ROOT/docs/rust-cutover}"
SCAN_ROOTS_TEXT="${NTPRO_IGNORED_SCAN_ROOTS:-crates tests}"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

fail() {
  echo "ignored tests current register failed: $*" >&2
  exit 1
}

count_matches() {
  local pattern="$1"
  shift
  {
    rg --json -U --pcre2 "$pattern" "$@" --glob '*.rs' || true
  } | jq -s '[.[] | select(.type == "match") | .data.submatches[]] | length'
}

matching_paths() {
  local pattern="$1"
  shift
  {
    rg --json -U --pcre2 "$pattern" "$@" --glob '*.rs' || true
  } | jq -r 'select(.type == "match") | .data.path.text' | LC_ALL=C sort -u
}

read -r -a scan_roots <<<"$SCAN_ROOTS_TEXT"
cd "$ROOT"

[[ -f "$CURRENT_REGISTER" ]] || fail "missing current register: $CURRENT_REGISTER"
[[ -f "$HISTORICAL_REGISTER" ]] || fail "missing historical register: $HISTORICAL_REGISTER"

direct_pattern='(?m)^[[:space:]]*#\[[[:space:]]*ignore(?:[[:space:]]*=[[:space:]]*"[^"]*")?[[:space:]]*\]'
conditional_pattern='#\s*\[\s*cfg_attr\((?s:[^]])*\bignore\s*='

direct_count="$(count_matches "$direct_pattern" "${scan_roots[@]}")"
conditional_count="$(count_matches "$conditional_pattern" "${scan_roots[@]}")"
total_count="$((direct_count + conditional_count))"

declared_direct="$(sed -n 's/^Direct ignored attributes: \([0-9][0-9]*\)$/\1/p' "$CURRENT_REGISTER")"
declared_conditional="$(sed -n 's/^Conditional ignored attributes: \([0-9][0-9]*\)$/\1/p' "$CURRENT_REGISTER")"
declared_total="$(sed -n 's/^Total ignored attributes across configurations: \([0-9][0-9]*\)$/\1/p' "$CURRENT_REGISTER")"

[[ "$declared_direct" == "$direct_count" ]] \
  || fail "direct count drift: declared=${declared_direct:-missing} actual=$direct_count"
[[ "$declared_conditional" == "$conditional_count" ]] \
  || fail "conditional count drift: declared=${declared_conditional:-missing} actual=$conditional_count"
[[ "$declared_total" == "$total_count" ]] \
  || fail "total count drift: declared=${declared_total:-missing} actual=$total_count"

current_markers="$(rg -n '^Register status: CURRENT$' "$REGISTER_SCOPE" | wc -l | tr -d ' ')"
[[ "$current_markers" == "1" ]] || fail "expected exactly one CURRENT register, found $current_markers"
rg -q '^Register status: CURRENT$' "$CURRENT_REGISTER" \
  || fail "quality register is not CURRENT"
rg -q '^Register status: HISTORICAL_EXTENSION$' "$HISTORICAL_REGISTER" \
  || fail "verification register is not HISTORICAL_EXTENSION"

{
  matching_paths "$direct_pattern" "${scan_roots[@]}"
  matching_paths "$conditional_pattern" "${scan_roots[@]}"
} | LC_ALL=C sort -u >"$TMP_DIR/paths"
paths_file="$TMP_DIR/paths"

while IFS= read -r path; do
  [[ -z "$path" ]] && continue
  grep -Fq "\`$path" "$CURRENT_REGISTER" \
    || fail "current register does not reference ignored-test source: $path"
done <"$paths_file"

source_files="$(wc -l <"$paths_file" | tr -d ' ')"
echo "ignored tests current register OK: direct=$direct_count conditional=$conditional_count total=$total_count source_files=$source_files"
