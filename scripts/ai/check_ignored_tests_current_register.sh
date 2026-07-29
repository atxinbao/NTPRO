#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CURRENT_REGISTER="${NTPRO_IGNORED_CURRENT_REGISTER:-$ROOT/docs/rust-cutover/quality/ignored_tests_register.md}"
HISTORICAL_REGISTER="${NTPRO_IGNORED_HISTORICAL_REGISTER:-$ROOT/docs/rust-cutover/verification/ignored_tests_risk_register.md}"
REGISTER_SCOPE="${NTPRO_IGNORED_REGISTER_SCOPE:-$ROOT/docs/rust-cutover}"
SCAN_ROOTS_TEXT="${NTPRO_IGNORED_SCAN_ROOTS:-crates tests}"
RUST_SOURCE_SCANNER="$ROOT/scripts/ai/inline_test_ranges.awk"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

fail() {
  echo "ignored tests current register failed: $*" >&2
  exit 1
}

scan_ignored_attributes() {
  local scan_root source_file

  for scan_root in "$@"; do
    while IFS= read -r -d '' source_file; do
      awk \
        -v scanner_mode=ignored_attributes \
        -v source_path="$source_file" \
        -f "$RUST_SOURCE_SCANNER" \
        "$source_file"
    done < <(find "$scan_root" -type f -name '*.rs' -print0)
  done
}

read -r -a scan_roots <<<"$SCAN_ROOTS_TEXT"
cd "$ROOT"

[[ -f "$CURRENT_REGISTER" ]] || fail "missing current register: $CURRENT_REGISTER"
[[ -f "$HISTORICAL_REGISTER" ]] || fail "missing historical register: $HISTORICAL_REGISTER"
[[ -f "$RUST_SOURCE_SCANNER" ]] || fail "missing Rust source scanner: $RUST_SOURCE_SCANNER"

scan_ignored_attributes "${scan_roots[@]}" >"$TMP_DIR/ignored-attributes"
awk -F '\t' '$1 == "direct" { print $2 }' \
  "$TMP_DIR/ignored-attributes" >"$TMP_DIR/direct-paths"
awk -F '\t' '$1 == "conditional" { print $2 }' \
  "$TMP_DIR/ignored-attributes" >"$TMP_DIR/conditional-paths"
direct_count="$(wc -l <"$TMP_DIR/direct-paths" | tr -d ' ')"
conditional_count="$(wc -l <"$TMP_DIR/conditional-paths" | tr -d ' ')"
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

current_markers="$(
  find "$REGISTER_SCOPE" -type f -name '*.md' -print0 \
    | xargs -0 grep -h -E '^Register status: CURRENT$' \
    | wc -l \
    | tr -d ' '
)"
[[ "$current_markers" == "1" ]] || fail "expected exactly one CURRENT register, found $current_markers"
grep -Eq '^Register status: CURRENT$' "$CURRENT_REGISTER" \
  || fail "quality register is not CURRENT"
grep -Eq '^Register status: HISTORICAL_EXTENSION$' "$HISTORICAL_REGISTER" \
  || fail "verification register is not HISTORICAL_EXTENSION"

{
  cat "$TMP_DIR/direct-paths"
  cat "$TMP_DIR/conditional-paths"
} | LC_ALL=C sort -u >"$TMP_DIR/paths"
paths_file="$TMP_DIR/paths"

while IFS= read -r path; do
  [[ -z "$path" ]] && continue
  grep -Fq "\`$path" "$CURRENT_REGISTER" \
    || fail "current register does not reference ignored-test source: $path"
done <"$paths_file"

source_files="$(wc -l <"$paths_file" | tr -d ' ')"
echo "ignored tests current register OK: direct=$direct_count conditional=$conditional_count total=$total_count source_files=$source_files"
