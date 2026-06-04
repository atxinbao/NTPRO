#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

FEATURES="${NAUTILUS_RUST_FEATURES:-arrow,ffi,high-precision,streaming,defi}"
VERIFY_FAST_CARGO_CHECK="${VERIFY_FAST_CARGO_CHECK:-0}"
VERIFY_FAST_CLIPPY="${VERIFY_FAST_CLIPPY:-0}"

echo "== verify_fast: toolchain =="
cargo --version
rustc --version

echo "== verify_fast: rust fmt =="
cargo fmt --check

if [ "$VERIFY_FAST_CARGO_CHECK" = "1" ]; then
  echo "== verify_fast: optional cargo check workspace without Python bridge product path =="
  cargo check --workspace --features "$FEATURES"
else
  echo "== verify_fast: cargo check skipped; set VERIFY_FAST_CARGO_CHECK=1 to run the legacy mixed-workspace check =="
fi

if [ "$VERIFY_FAST_CLIPPY" = "1" ]; then
  echo "== verify_fast: optional clippy =="
  cargo clippy --workspace --lib --tests --features "$FEATURES" -- -D warnings
else
  echo "== verify_fast: clippy skipped; set VERIFY_FAST_CLIPPY=1 to run it in fast mode =="
fi

echo "== verify_fast complete =="
