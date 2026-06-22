#!/usr/bin/env bash
set -euo pipefail

# V140-003: live-alpha dry-run order gate.
# This script is CI-safe. It proves the default path fails closed and the
# owner-gated dry-run path records intent only, without network, execution
# adapter calls, order endpoint access, production submission, or mutation.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_V14_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V14_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V14_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi

GATE_ROOT="${NTPRO_V14_LIVE_ALPHA_DRY_RUN_GATE_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v14-live-alpha-dry-run-gate.XXXXXX")}"
OUTPUT_DIR="$GATE_ROOT/command-output"
mkdir -p "$OUTPUT_DIR"

BLOCKED_JSON="$OUTPUT_DIR/blocked-live-alpha-dry-run-order-gate.json"
BLOCKED_STDOUT="$OUTPUT_DIR/blocked.stdout.log"
BLOCKED_STDERR="$OUTPUT_DIR/blocked.stderr.log"
READY_JSON="$OUTPUT_DIR/ready-live-alpha-dry-run-order-gate.json"
READY_STDOUT="$OUTPUT_DIR/ready.stdout.log"
READY_STDERR="$OUTPUT_DIR/ready.stderr.log"

"$NAUTILUS_BIN" live production-live-alpha-dry-run-order-gate \
  --run-id v140-live-alpha-dry-run-blocked \
  --session-id session-blocked \
  --strategy-id ema_cross_btcusdt_v1 \
  --symbol BTCUSDT \
  --side BUY \
  --order-type LIMIT \
  --quantity 0.001 \
  --notional 10.00 \
  --output "$BLOCKED_JSON" \
  >"$BLOCKED_STDOUT" \
  2>"$BLOCKED_STDERR"

if [[ -s "$BLOCKED_STDERR" ]]; then
  echo "v14 live-alpha dry-run blocked path wrote stderr" >&2
  cat "$BLOCKED_STDERR" >&2
  exit 1
fi

python3 - "$BLOCKED_JSON" <<'PY'
import json
import sys
from pathlib import Path

report = json.loads(Path(sys.argv[1]).read_text())

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require(report["schema_version"] == "ntpro.v140_live_alpha_dry_run_order_gate.v1", report)
require(report["status"] == "blocked_missing_gate", report)
require(report["mode"] == "production_live_alpha_dry_run", report)
require(report["order_type"] == "LIMIT", report)
require(report["owner_gate_required"] is True, report)
require(report["manual_gate_required"] is True, report)
require(len(report["missing_cli_flags"]) == 8, report)
require(report["dry_run_order_gate_ready"] is False, report)
require(report["dry_run_order_intent_recorded"] is False, report)
require(report["production_order_submission_allowed"] is False, report)
require(report["production_order_mutation_allowed"] is False, report)
require(report["production_order_state_reads_allowed"] is False, report)
require(report["listen_key_lifecycle_allowed"] is False, report)
require(report["production_order_submissions_attempted"] == 0, report)
require(report["production_orders_submitted"] == 0, report)
require(report["production_order_mutations_attempted"] == 0, report)
require(report["production_order_state_reads_attempted"] == 0, report)
require(report["listen_key_lifecycle_attempted"] == 0, report)
require(report["cancel_replace_amend_attempted"] is False, report)
require(report["order_endpoint_access_attempted"] is False, report)
require(report["execution_adapter_called"] is False, report)
require(report["matching_engine_submission"] is False, report)
require(report["actual_submission_count"] == 0, report)
require(report["automatic_correction_orders_submitted"] == 0, report)
require(report["dashboard_order_controls_enabled"] is False, report)
require(report["external_venue_connection"] is False, report)
require(report["network_attempted"] is False, report)
require(report["real_orders_submitted"] is False, report)
require(report["real_funds"] is False, report)
require(report["production_trading_enabled"] is False, report)
require(report["values_are_exchange_truth"] is False, report)
PY

"$NAUTILUS_BIN" live production-live-alpha-dry-run-order-gate \
  --run-id v140-live-alpha-dry-run-ready \
  --session-id session-ready \
  --strategy-id ema_cross_btcusdt_v1 \
  --symbol BTCUSDT \
  --side BUY \
  --order-type LIMIT \
  --quantity 0.001 \
  --notional 10.00 \
  --output "$READY_JSON" \
  --allow-production-live-alpha-dry-run \
  --confirm-owner-approved-dry-run \
  --confirm-no-production-order-submission \
  --confirm-no-production-order-mutation \
  --confirm-no-execution-adapter-call \
  --confirm-no-listen-key-lifecycle \
  --confirm-dashboard-order-controls-disabled \
  --confirm-no-real-funds \
  >"$READY_STDOUT" \
  2>"$READY_STDERR"

if [[ -s "$READY_STDERR" ]]; then
  echo "v14 live-alpha dry-run ready path wrote stderr" >&2
  cat "$READY_STDERR" >&2
  exit 1
fi

python3 - "$READY_JSON" <<'PY'
import json
import sys
from pathlib import Path

report = json.loads(Path(sys.argv[1]).read_text())

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require(report["schema_version"] == "ntpro.v140_live_alpha_dry_run_order_gate.v1", report)
require(report["status"] == "ready_dry_run_no_submission", report)
require(report["missing_cli_flags"] == [], report)
require(report["dry_run_order_gate_ready"] is True, report)
require(report["dry_run_order_intent_recorded"] is True, report)
require(report["order_submission_mode"] == "dry_run_no_submission", report)
require(report["order_type"] == "LIMIT", report)
require(report["quantity"] == "0.001", report)
require(report["notional"] == "10.00", report)
require(report["production_order_submission_allowed"] is False, report)
require(report["production_order_mutation_allowed"] is False, report)
require(report["production_order_state_reads_allowed"] is False, report)
require(report["listen_key_lifecycle_allowed"] is False, report)
require(report["production_order_submissions_attempted"] == 0, report)
require(report["production_orders_submitted"] == 0, report)
require(report["production_order_mutations_attempted"] == 0, report)
require(report["production_order_state_reads_attempted"] == 0, report)
require(report["listen_key_lifecycle_attempted"] == 0, report)
require(report["cancel_replace_amend_attempted"] is False, report)
require(report["order_endpoint_access_attempted"] is False, report)
require(report["execution_adapter_called"] is False, report)
require(report["matching_engine_submission"] is False, report)
require(report["actual_submission_count"] == 0, report)
require(report["automatic_correction_orders_submitted"] == 0, report)
require(report["dashboard_order_controls_enabled"] is False, report)
require(report["external_venue_connection"] is False, report)
require(report["network_attempted"] is False, report)
require(report["real_orders_submitted"] is False, report)
require(report["real_funds"] is False, report)
require(report["production_trading_enabled"] is False, report)
require(report["values_are_exchange_truth"] is False, report)
require(report["no_production_order_submission_confirmed"] is True, report)
require(report["no_production_order_mutation_confirmed"] is True, report)
require(report["no_execution_adapter_call_confirmed"] is True, report)
require(report["no_listen_key_lifecycle_confirmed"] is True, report)
require(report["dashboard_controls_disabled_confirmed"] is True, report)
require(report["no_real_funds_confirmed"] is True, report)
PY

if grep -En \
  '"production_order_submission_allowed": true|"production_order_mutation_allowed": true|"execution_adapter_called": true|"order_endpoint_access_attempted": true|"dashboard_order_controls_enabled": true|"production_orders_submitted": [1-9]|"production_order_mutations_attempted": [1-9]' \
  "$READY_JSON" >/dev/null; then
  echo "v14 live-alpha dry-run ready artifact contains enabled production mutation fields" >&2
  exit 1
fi

echo "v14_live_alpha_dry_run_order_gate status=ok root=$GATE_ROOT default=blocked owner_dry_run=ready production_orders_submitted=0 production_order_mutations_attempted=0 execution_adapter_called=false order_endpoint_access_attempted=false network_attempted=false dashboard_order_controls_enabled=false"
