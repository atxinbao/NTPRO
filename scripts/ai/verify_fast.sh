#!/usr/bin/env bash
set -euo pipefail

FEATURES="${NAUTILUS_RUST_FEATURES:-arrow,ffi,high-precision,streaming,defi}"
VERIFY_FAST_CLIPPY="${VERIFY_FAST_CLIPPY:-0}"

echo "== verify_fast: toolchain =="
command -v cargo >/dev/null

echo "== verify_fast: rust fmt =="
cargo fmt --check

echo "== verify_fast: cargo check workspace without Python/PyO3 product path =="
exclude_args=()
if cargo metadata --no-deps --format-version=1 2>/dev/null | grep -q '"name":"nautilus-pyo3"'; then
  exclude_args+=(--exclude nautilus-pyo3)
fi
cargo check --workspace "${exclude_args[@]}" --features "$FEATURES"

if [ "$VERIFY_FAST_CLIPPY" = "1" ]; then
  echo "== verify_fast: optional clippy =="
  cargo clippy --workspace "${exclude_args[@]}" --lib --tests --features "$FEATURES" -- -D warnings
else
  echo "== verify_fast: clippy skipped; set VERIFY_FAST_CLIPPY=1 to run it in fast mode =="
fi

echo "== verify_fast complete =="
