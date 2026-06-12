#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

FEATURES="${NAUTILUS_RUST_FEATURES:-arrow,ffi,high-precision,streaming,defi}"
export REQUIRE_GOLDEN_REPLAY="${REQUIRE_GOLDEN_REPLAY:-1}"

echo "== verify_release: full checks =="
scripts/ai/verify_full.sh

echo "== verify_release: release build =="
cargo build --workspace --release --features "$FEATURES"

echo "== verify_release: Rust CLI product surface =="
if ! cargo metadata --no-deps --format-version=1 | grep -q '"name":"nautilus-cli"'; then
  echo "nautilus-cli package is missing" >&2
  exit 1
fi

NAUTILUS_RELEASE_BIN="$ROOT/target/release/nautilus"
NTPRO_NODE_RELEASE_BIN="$ROOT/target/release/ntpro-node"

if [[ ! -x "$NAUTILUS_RELEASE_BIN" ]]; then
  echo "missing release nautilus binary: $NAUTILUS_RELEASE_BIN" >&2
  exit 1
fi
if [[ ! -x "$NTPRO_NODE_RELEASE_BIN" ]]; then
  echo "missing release ntpro-node binary: $NTPRO_NODE_RELEASE_BIN" >&2
  exit 1
fi

require_help_contains() {
  local file="$1"
  shift
  local pattern
  for pattern in "$@"; do
    if ! grep -Eiq -- "$pattern" "$file"; then
      echo "help output missing required pattern '$pattern' in $file" >&2
      cat "$file" >&2
      exit 1
    fi
  done
}

"$NAUTILUS_RELEASE_BIN" --help >/tmp/nautilus_cli_help.txt
require_help_contains /tmp/nautilus_cli_help.txt \
  'backtest' \
  'live' \
  'sandbox' \
  'data' \
  'supervisor'

"$NAUTILUS_RELEASE_BIN" supervisor --help >/tmp/nautilus_supervisor_help.txt
require_help_contains /tmp/nautilus_supervisor_help.txt \
  'register' \
  'list' \
  'start' \
  'stop' \
  'pause' \
  'resume' \
  'reconnect-data' \
  'reconnect-execution' \
  'status' \
  'logs' \
  'metrics' \
  'sandbox-only'

"$NTPRO_NODE_RELEASE_BIN" --help >/tmp/ntpro_node_help.txt
require_help_contains /tmp/ntpro_node_help.txt \
  'config' \
  'run-id' \
  'output' \
  'stop-file' \
  'sandbox-only'

echo "== verify_release: Rust-only runtime check =="
scripts/ai/check_rust_only_runtime.sh

echo "== verify_release: final Cython removed check =="
scripts/ai/check_cython_removed.sh

echo "== verify_release: v0.2 two-node supervisor smoke =="
if [[ "${NTPRO_V02_009_SKIP_BUILD:-0}" == "1" ]]; then
  echo "release gate must not set NTPRO_V02_009_SKIP_BUILD=1" >&2
  exit 1
fi
NTPRO_RELEASE_GATE=1 NTPRO_V02_009_SKIP_BUILD=0 scripts/ai/v02_two_node_supervisor_smoke.sh

echo "== verify_release: v0.3 supervisor control smoke =="
NTPRO_V03_CONTROL_SKIP_BUILD=1 \
  NTPRO_V03_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
  NTPRO_V03_NODE_BIN="$NTPRO_NODE_RELEASE_BIN" \
  scripts/ai/v03_supervisor_control_smoke.sh

echo "== verify_release: v0.3 dashboard control smoke =="
NTPRO_V03_010_SKIP_BUILD=1 \
  NTPRO_V03_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
  NTPRO_V03_NODE_BIN="$NTPRO_NODE_RELEASE_BIN" \
  scripts/ai/v03_dashboard_smoke.sh

echo "== verify_release complete =="
