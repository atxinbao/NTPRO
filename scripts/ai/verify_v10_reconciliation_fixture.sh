#!/usr/bin/env bash
set -euo pipefail

# V100-008: v0.10 Binance testnet reconciliation/orphan-order fixture.
# Safe for local development and CI. It renders offline inconsistent-state
# fixtures only; it does not open network connections and does not submit or
# cancel orders.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_V10_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V10_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V10_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
CONFIG="${NTPRO_V10_CONFIG:-$ROOT_DIR/configs/nodes/btc-ema-shadow.toml}"

if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi
if [[ ! -f "$CONFIG" ]]; then
  echo "missing v0.10 strategy config: $CONFIG" >&2
  exit 1
fi

RECON_ROOT="${NTPRO_V10_RECONCILIATION_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v10-reconciliation.XXXXXX")}"
OUTPUT_DIR="$RECON_ROOT/command-output"
mkdir -p "$OUTPUT_DIR"

ALL_REPORT="$OUTPUT_DIR/reconciliation-all.json"
ONE_REPORT="$OUTPUT_DIR/reconciliation-cancel-timeout.json"
ALL_STDOUT="$OUTPUT_DIR/all.stdout.log"
ONE_STDOUT="$OUTPUT_DIR/one.stdout.log"

"$NAUTILUS_BIN" live testnet-reconciliation-fixture \
  --config "$CONFIG" \
  --scenario all \
  --output "$ALL_REPORT" \
  >"$ALL_STDOUT"

grep -q "live.testnet_reconciliation_fixture status=risk_halted" "$ALL_STDOUT"
grep -q "scenario=all" "$ALL_STDOUT"
grep -q "scenario_count=4" "$ALL_STDOUT"
grep -q "risk_halted=true" "$ALL_STDOUT"
grep -q "new_orders_blocked=true" "$ALL_STDOUT"
grep -q "network_attempted=false" "$ALL_STDOUT"
grep -q "real_orders_submitted=false" "$ALL_STDOUT"
grep -q '"schema_version": "ntpro.v100_reconciliation_fixture_report.v1"' "$ALL_REPORT"
grep -q '"status": "risk_halted"' "$ALL_REPORT"
grep -q '"scenario_count": 4' "$ALL_REPORT"
grep -q '"name": "submit_without_local_ack"' "$ALL_REPORT"
grep -q '"name": "cancel_timeout"' "$ALL_REPORT"
grep -q '"name": "local_open_exchange_filled"' "$ALL_REPORT"
grep -q '"name": "restart_unfinished_order"' "$ALL_REPORT"
grep -q '"risk_halted": true' "$ALL_REPORT"
grep -q '"new_orders_blocked": true' "$ALL_REPORT"
grep -q '"testnet_orders_submitted": 0' "$ALL_REPORT"
grep -q '"testnet_orders_canceled": 0' "$ALL_REPORT"
grep -q '"production_orders_submitted": 0' "$ALL_REPORT"
grep -q '"production_orders_canceled": 0' "$ALL_REPORT"
grep -q '"manual_submit_cancel_proof_observed": false' "$ALL_REPORT"
grep -q '"matching_engine_submission": false' "$ALL_REPORT"
grep -q '"order_submission_remains_disabled": true' "$ALL_REPORT"
grep -q '"network_attempted": false' "$ALL_REPORT"
grep -q '"real_orders_submitted": false' "$ALL_REPORT"
grep -q '"production_endpoint_allowed": false' "$ALL_REPORT"
grep -q '"dashboard_order_controls": false' "$ALL_REPORT"

"$NAUTILUS_BIN" live testnet-reconciliation-fixture \
  --config "$CONFIG" \
  --scenario cancel-timeout \
  --output "$ONE_REPORT" \
  >"$ONE_STDOUT"

grep -q "scenario=cancel_timeout" "$ONE_STDOUT"
grep -q "scenario_count=1" "$ONE_STDOUT"
grep -q '"scenario": "cancel_timeout"' "$ONE_REPORT"
grep -q '"scenario_count": 1' "$ONE_REPORT"
grep -q '"name": "cancel_timeout"' "$ONE_REPORT"
if grep -q '"name": "submit_without_local_ack"' "$ONE_REPORT"; then
  echo "single-scenario reconciliation fixture included an unrelated scenario" >&2
  exit 1
fi

echo "v10_reconciliation_fixture status=ok root=$RECON_ROOT"
