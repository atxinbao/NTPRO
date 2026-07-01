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

echo "== verify_fast: scope =="
echo "fast smoke only: toolchain + cargo fmt by default"
echo "not release validation and not release evidence"
echo "does not replace workspace cargo check, clippy, golden traces, release gates, or strict provenance"
echo "compile/lint check: use VERIFY_FAST_CARGO_CHECK=1 VERIFY_FAST_CLIPPY=1 scripts/ai/verify_fast.sh"
echo "full test check: use scripts/ai/verify_full.sh"
echo "release gate: use scripts/ai/verify_release.sh"
echo "strict provenance gate: use scripts/ai/verify_release_strict.sh v18, v19, v20, or v21"

echo "== verify_fast: rust fmt =="
cargo fmt --check

if [ "$VERIFY_FAST_CARGO_CHECK" = "1" ]; then
  echo "== verify_fast: optional cargo check workspace without Python bridge product path =="
  cargo check --workspace --features "$FEATURES"
else
  echo "== verify_fast: cargo check skipped by fast-smoke default; set VERIFY_FAST_CARGO_CHECK=1 for compile coverage =="
fi

if [ "$VERIFY_FAST_CLIPPY" = "1" ]; then
  echo "== verify_fast: optional clippy =="
  cargo clippy --workspace --lib --tests --features "$FEATURES" -- -D warnings
else
  echo "== verify_fast: clippy skipped by fast-smoke default; set VERIFY_FAST_CLIPPY=1 for lint coverage =="
fi

echo "== verify_fast complete: fast smoke only; release work still requires verify_release.sh and strict provenance when applicable =="
