#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

run_step() {
  local label="$1"
  shift
  echo "== verify_v04_binance_sandbox: ${label} =="
  "$@"
}

run_step "Binance fixture replay" \
  cargo test -p nautilus-binance --test v04_replay

run_step "EMA strategy smoke" \
  cargo test -p nautilus-trading --test v04_ema_smoke

run_step "RSI strategy smoke" \
  cargo test -p nautilus-trading --test v04_rsi_smoke

run_step "mock order lifecycle" \
  cargo test -p nautilus-binance --test v04_mock_lifecycle

run_step "risk rejection smoke" \
  cargo test -p nautilus-risk --test v04_binance_risk_rejection

run_step "dashboard Binance sandbox read-model" \
  cargo test -p nautilus-cli one_node_supervisor_artifacts_populate_dashboard_sections --lib

echo "== verify_v04_binance_sandbox complete =="
echo "scope=Binance sandbox-only no_real_funds=true no_production_trading=true real_orders_submitted=false"
