#!/usr/bin/env bash
set -euo pipefail

: "${HEAVY_RUST:?HEAVY_RUST is required}"
: "${CHANGES_RESULT:?CHANGES_RESULT is required}"
: "${CORE_RESULT:?CORE_RESULT is required}"
: "${RUST_LINT_RESULT:?RUST_LINT_RESULT is required}"
: "${RUST_TESTS_RESULT:?RUST_TESTS_RESULT is required}"

require_result() {
  local name="$1"
  local actual="$2"
  local expected="$3"
  if [[ "$actual" != "$expected" ]]; then
    echo "$name result mismatch: expected=$expected actual=$actual" >&2
    exit 1
  fi
}

require_result changes "$CHANGES_RESULT" success
require_result core "$CORE_RESULT" success

case "$HEAVY_RUST" in
  true)
    require_result rust-lint "$RUST_LINT_RESULT" success
    require_result rust-tests "$RUST_TESTS_RESULT" success
    ;;
  false)
    require_result rust-lint "$RUST_LINT_RESULT" skipped
    require_result rust-tests "$RUST_TESTS_RESULT" skipped
    ;;
  *)
    echo "invalid HEAVY_RUST classification: $HEAVY_RUST" >&2
    exit 1
    ;;
esac

printf 'required_smoke=pass heavy_rust=%s core=%s rust_lint=%s rust_tests=%s\n' \
  "$HEAVY_RUST" "$CORE_RESULT" "$RUST_LINT_RESULT" "$RUST_TESTS_RESULT"
