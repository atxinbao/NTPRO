#!/usr/bin/env bash
set -euo pipefail

FEATURES="${NAUTILUS_RUST_FEATURES:-arrow,ffi,high-precision,streaming,defi}"

exclude_args=()
if cargo metadata --no-deps --format-version=1 2>/dev/null | grep -q '"name":"nautilus-pyo3"'; then
  exclude_args+=(--exclude nautilus-pyo3)
fi

echo "== verify_full: fast checks =="
scripts/ai/verify_fast.sh

echo "== verify_full: clippy =="
cargo clippy --workspace "${exclude_args[@]}" --lib --tests --features "$FEATURES" -- -D warnings

echo "== verify_full: rust tests =="
if cargo nextest --version >/dev/null 2>&1; then
  cargo nextest run --workspace "${exclude_args[@]}" --lib --tests --features "$FEATURES" --no-fail-fast
else
  cargo test --workspace "${exclude_args[@]}" --lib --tests --features "$FEATURES"
fi

echo "== verify_full: golden trace validation =="
scripts/ai/run_golden_traces.sh

echo "== verify_full: rust docs =="
cargo doc --workspace "${exclude_args[@]}" --features "$FEATURES" --no-deps

echo "== verify_full complete =="
