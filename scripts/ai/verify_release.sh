#!/usr/bin/env bash
set -euo pipefail

FEATURES="${NAUTILUS_RUST_FEATURES:-arrow,ffi,high-precision,streaming,defi}"
export REQUIRE_GOLDEN_REPLAY="${REQUIRE_GOLDEN_REPLAY:-1}"

exclude_args=()
if cargo metadata --no-deps --format-version=1 2>/dev/null | grep -q '"name":"nautilus-pyo3"'; then
  exclude_args+=(--exclude nautilus-pyo3)
fi

echo "== verify_release: full checks =="
scripts/ai/verify_full.sh

echo "== verify_release: release build =="
cargo build --workspace "${exclude_args[@]}" --release --features "$FEATURES"

echo "== verify_release: Rust CLI product surface =="
if cargo metadata --no-deps --format-version=1 | grep -q '"name":"nautilus-cli"'; then
  cargo run -q -p nautilus-cli -- --help >/tmp/nautilus_cli_help.txt
  grep -Ei 'backtest|live|sandbox|data|database|blockchain' /tmp/nautilus_cli_help.txt >/dev/null
else
  echo "nautilus-cli package is missing" >&2
  exit 1
fi

echo "== verify_release: Rust-only runtime check =="
scripts/ai/check_rust_only_runtime.sh

echo "== verify_release: final Cython removed check =="
scripts/ai/check_cython_removed.sh

echo "== verify_release complete =="
