#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECK="$ROOT/scripts/ai/check_ignored_tests_current_register.sh"
CURRENT="$ROOT/docs/rust-cutover/quality/ignored_tests_register.md"
HISTORICAL="$ROOT/docs/rust-cutover/verification/ignored_tests_risk_register.md"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

fail() {
  echo "ignored tests current register self-test failed: $*" >&2
  exit 1
}

run_check() {
  local current="$1"
  local historical="$2"
  local scope="${3:-$ROOT/docs/rust-cutover}"

  NTPRO_IGNORED_CURRENT_REGISTER="$current" \
    NTPRO_IGNORED_HISTORICAL_REGISTER="$historical" \
    NTPRO_IGNORED_REGISTER_SCOPE="$scope" \
    "$CHECK"
}

expect_failure() {
  local name="$1"
  local current="$2"
  local historical="$3"
  local scope="${4:-$ROOT/docs/rust-cutover}"

  if run_check "$current" "$historical" "$scope" >"$TMP_DIR/$name.out" 2>&1; then
    fail "$name unexpectedly passed"
  fi
}

run_check "$CURRENT" "$HISTORICAL" >/dev/null

cp "$CURRENT" "$TMP_DIR/stale-count.md"
sed -i.bak 's/^Direct ignored attributes: 18$/Direct ignored attributes: 17/' \
  "$TMP_DIR/stale-count.md"
expect_failure stale_count "$TMP_DIR/stale-count.md" "$HISTORICAL"

mkdir "$TMP_DIR/duplicate-scope"
cp "$CURRENT" "$TMP_DIR/duplicate-scope/current.md"
cp "$HISTORICAL" "$TMP_DIR/duplicate-scope/historical.md"
sed -i.bak 's/^Register status: HISTORICAL_EXTENSION$/Register status: CURRENT/' \
  "$TMP_DIR/duplicate-scope/historical.md"
expect_failure duplicate_current \
  "$TMP_DIR/duplicate-scope/current.md" \
  "$TMP_DIR/duplicate-scope/historical.md" \
  "$TMP_DIR/duplicate-scope"

cp "$CURRENT" "$TMP_DIR/missing-path.md"
sed -i.bak 's#`crates/plugin/tests/load_example_cdylib.rs#`removed/plugin/path.rs#' \
  "$TMP_DIR/missing-path.md"
expect_failure missing_path "$TMP_DIR/missing-path.md" "$HISTORICAL"

echo "ignored tests current register self-test OK: cases=4"
