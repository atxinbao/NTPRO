#!/usr/bin/env bash
set -euo pipefail

# V150-003: v0.15 production live-alpha execution adapter isolation.
# Safe for local development and CI. It routes strategy intent through local
# artifacts into a dry-run execution adapter proof only. It never instantiates
# production adapters, opens network connections, or submits production orders.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_V15_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V15_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V15_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi

ISOLATION_ROOT="${NTPRO_V15_ISOLATION_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v15-isolation.XXXXXX")}"
OUTPUT_DIR="$ISOLATION_ROOT/command-output"
mkdir -p "$OUTPUT_DIR"

ORDER_GATE="$OUTPUT_DIR/live-alpha-order-gate.json"
RISK_INPUT="$OUTPUT_DIR/live-alpha-risk-input.json"
RISK_PREFLIGHT="$OUTPUT_DIR/live-alpha-risk-preflight.json"
MANUAL_APPROVAL="$OUTPUT_DIR/manual-approval-lifecycle.json"
REQUEST_PREVIEW="$OUTPUT_DIR/live-alpha-request-preview.json"
KILL_SWITCH_APPROVAL="$OUTPUT_DIR/kill-switch-approval.json"
KILL_SWITCH_RUNTIME_GATE="$OUTPUT_DIR/kill-switch-runtime-gate.json"
BLOCKED_REPORT="$OUTPUT_DIR/blocked-execution-dry-run.json"
READY_REPORT="$OUTPUT_DIR/ready-execution-dry-run.json"
BLOCKED_STDOUT="$OUTPUT_DIR/blocked.stdout.log"
BLOCKED_STDERR="$OUTPUT_DIR/blocked.stderr.log"
READY_STDOUT="$OUTPUT_DIR/ready.stdout.log"
READY_STDERR="$OUTPUT_DIR/ready.stderr.log"

SYNTHETIC_API_KEY="ntpro_v151003_synthetic_api_key_value"
SYNTHETIC_API_SECRET="ntpro_v151003_synthetic_api_secret_value"

"$NAUTILUS_BIN" live production-live-alpha-dry-run-order-gate \
  --run-id v150-execution-dry-run \
  --session-id session-v150 \
  --strategy-id ema_cross_btcusdt_v1 \
  --symbol BTCUSDT \
  --side BUY \
  --order-type LIMIT \
  --quantity 0.001 \
  --notional 10.00 \
  --output "$ORDER_GATE" \
  --allow-production-live-alpha-dry-run \
  --confirm-owner-approved-dry-run \
  --confirm-no-production-order-submission \
  --confirm-no-production-order-mutation \
  --confirm-no-execution-adapter-call \
  --confirm-no-listen-key-lifecycle \
  --confirm-dashboard-order-controls-disabled \
  --confirm-no-real-funds >/dev/null

python3 - "$RISK_INPUT" <<'PY'
import json
import sys
from pathlib import Path

Path(sys.argv[1]).write_text(json.dumps({
    "schema_version": "ntpro.v140_live_alpha_risk_preflight_input.v1",
    "session": {"state": "running"},
    "market": {
        "symbol": "BTCUSDT",
        "last_event_at_unix_ms": 1000,
        "now_unix_ms": 1500,
        "max_age_ms": 1000,
    },
    "account": {
        "readable": True,
        "account_id": "BINANCE-001",
    },
    "order_state": {
        "readable": True,
        "open_order_count": 0,
        "last_read_at_unix_ms": None,
        "now_unix_ms": None,
        "max_age_ms": None,
    },
    "risk": {
        "kill_switch_active": False,
        "allowed_symbols": ["BTCUSDT"],
    },
    "order": {
        "symbol": "BTCUSDT",
        "side": "BUY",
        "order_type": "LIMIT",
        "quantity": "0.001",
        "notional": "10.00",
    },
    "limits": {
        "max_order_notional": "25.00",
        "current_position_notional": "50.00",
        "max_position_notional": "100.00",
        "max_open_orders": 5,
        "max_clock_skew_ms": 100,
        "observed_clock_skew_ms": 25,
    },
}, indent=2) + "\n")
PY

"$NAUTILUS_BIN" live production-live-alpha-risk-preflight \
  --run-id v150-execution-risk \
  --order-gate "$ORDER_GATE" \
  --input "$RISK_INPUT" \
  --output "$RISK_PREFLIGHT" \
  --confirm-hypothetical-dry-run-only \
  --confirm-no-execution-adapter-call \
  --confirm-no-production-order-submission \
  --confirm-no-production-order-mutation \
  --confirm-dashboard-order-controls-disabled >/dev/null

"$NAUTILUS_BIN" live production-live-alpha-manual-approval-lifecycle \
  --run-id v150-execution-request-preview \
  --strategy-id ema_cross_btcusdt_v1 \
  --symbol BTCUSDT \
  --notional 10.00 \
  --approval-state approved \
  --manual-approval-id owner-approval-v150-005 \
  --approved-by owner \
  --now-unix-ms 1718400000000 \
  --expires-at-unix-ms 1718400060000 \
  --output "$MANUAL_APPROVAL" \
  --confirm-dry-run-request-preview-only \
  --confirm-one-time-approval \
  --confirm-no-production-mutation \
  --confirm-dashboard-order-controls-disabled >/dev/null

NTPRO_V150003_API_KEY="$SYNTHETIC_API_KEY" \
NTPRO_V150003_API_SECRET="$SYNTHETIC_API_SECRET" \
  "$NAUTILUS_BIN" live production-live-alpha-order-request-preview \
    --run-id v150-execution-request-preview \
    --order-gate "$ORDER_GATE" \
    --manual-approval-lifecycle "$MANUAL_APPROVAL" \
    --endpoint-path /api/v3/order \
    --price 10000.00 \
    --time-in-force GTC \
    --timestamp-ms 1718400000000 \
    --recv-window-ms 5000 \
    --api-key-env NTPRO_V150003_API_KEY \
    --api-secret-env NTPRO_V150003_API_SECRET \
    --output "$REQUEST_PREVIEW" \
    --allow-production-live-alpha-request-preview \
    --confirm-owner-approved-request-preview \
    --confirm-memory-only-signature \
    --confirm-no-production-order-submission \
    --confirm-no-production-order-mutation \
    --confirm-no-execution-adapter-call \
    --confirm-no-network \
    --confirm-no-listen-key-lifecycle \
    --confirm-dashboard-order-controls-disabled \
    --confirm-no-real-funds >/dev/null

"$NAUTILUS_BIN" live production-kill-switch-approval-artifact \
  --run-id v150-execution-kill-switch \
  --session-id session-v150 \
  --strategy-id ema_cross_btcusdt_v1 \
  --output "$KILL_SWITCH_APPROVAL" \
  --kill-switch-active false \
  --approval-state approved \
  --manual-approval-id owner-approval-v150-004 \
  --approved-by owner \
  --confirm-dry-run-only \
  --confirm-no-production-mutation \
  --confirm-dashboard-order-controls-disabled >/dev/null

"$NAUTILUS_BIN" live production-live-alpha-kill-switch-runtime-gate \
  --run-id v150-execution-kill-switch-runtime-gate \
  --kill-switch-approval "$KILL_SWITCH_APPROVAL" \
  --risk-preflight "$RISK_PREFLIGHT" \
  --request-preview "$REQUEST_PREVIEW" \
  --output "$KILL_SWITCH_RUNTIME_GATE" \
  --allow-production-live-alpha-kill-switch-runtime-gate \
  --confirm-owner-approved-runtime-gate \
  --confirm-no-production-order-submission \
  --confirm-no-production-order-mutation \
  --confirm-no-network \
  --confirm-no-listen-key-lifecycle \
  --confirm-dashboard-order-controls-disabled \
  --confirm-no-real-funds >/dev/null

"$NAUTILUS_BIN" live production-live-alpha-execution-dry-run \
  --run-id v150-execution-dry-run-blocked \
  --order-gate "$ORDER_GATE" \
  --risk-preflight "$RISK_PREFLIGHT" \
  --request-preview "$REQUEST_PREVIEW" \
  --kill-switch-runtime-gate "$KILL_SWITCH_RUNTIME_GATE" \
  --output "$BLOCKED_REPORT" \
  >"$BLOCKED_STDOUT" \
  2>"$BLOCKED_STDERR"

if [[ -s "$BLOCKED_STDERR" ]]; then
  echo "v15 execution dry-run blocked path wrote stderr" >&2
  cat "$BLOCKED_STDERR" >&2
  exit 1
fi
grep -q "status=blocked_missing_gate" "$BLOCKED_STDOUT"
grep -q "dry_run_execution_adapter_called=false" "$BLOCKED_STDOUT"
grep -q "production_adapter_called=false" "$BLOCKED_STDOUT"
grep -q "network_attempted=false" "$BLOCKED_STDOUT"

"$NAUTILUS_BIN" live production-live-alpha-execution-dry-run \
  --run-id v150-execution-dry-run \
  --order-gate "$ORDER_GATE" \
  --risk-preflight "$RISK_PREFLIGHT" \
  --request-preview "$REQUEST_PREVIEW" \
  --kill-switch-runtime-gate "$KILL_SWITCH_RUNTIME_GATE" \
  --output "$READY_REPORT" \
  --allow-production-live-alpha-execution-dry-run \
  --confirm-owner-approved-execution-dry-run \
  --confirm-dry-run-adapter-only \
  --confirm-no-production-adapter \
  --confirm-no-production-order-submission \
  --confirm-no-production-order-mutation \
  --confirm-no-network \
  --confirm-no-listen-key-lifecycle \
  --confirm-dashboard-order-controls-disabled \
  --confirm-no-real-funds \
  >"$READY_STDOUT" \
  2>"$READY_STDERR"

if [[ -s "$READY_STDERR" ]]; then
  echo "v15 execution dry-run wrote stderr on pass path" >&2
  cat "$READY_STDERR" >&2
  exit 1
fi
grep -q "live.production_live_alpha_execution_dry_run status=ready_dry_run_execution_adapter_only" "$READY_STDOUT"
grep -q "dry_run_execution_adapter_called=true" "$READY_STDOUT"
grep -q "production_adapter_called=false" "$READY_STDOUT"
grep -q "production_adapter_instantiated=false" "$READY_STDOUT"
grep -q "execution_command_route=dry_run_adapter_only" "$READY_STDOUT"
grep -q "production_orders_submitted=0" "$READY_STDOUT"
grep -q "production_order_mutations_attempted=0" "$READY_STDOUT"
grep -q "network_attempted=false" "$READY_STDOUT"
grep -q "dashboard_order_controls_enabled=false" "$READY_STDOUT"

python3 - "$BLOCKED_REPORT" "$READY_REPORT" <<'PY'
import json
import sys
from pathlib import Path

blocked = json.loads(Path(sys.argv[1]).read_text())
ready = json.loads(Path(sys.argv[2]).read_text())

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require(blocked["schema_version"] == "ntpro.v150_live_alpha_execution_dry_run.v1", blocked)
require(blocked["status"] == "blocked_missing_gate", blocked)
require(blocked["execution_decision"] == "blocked_no_adapter_route", blocked)
require(blocked["execution_boundary_contract_version"] == "ntpro.v151_execution_dry_run_adapter_boundary.v1", blocked)
require(blocked["execution_boundary_flow"] == "StrategyIntent -> RiskDecision -> ExecutionCommand -> DryRunExecutionAdapter", blocked)
require(blocked["execution_boundary_contract_ready"] is False, blocked)
require(blocked["strategy_intent_boundary"] == "StrategyIntent", blocked)
require(blocked["risk_decision_boundary"] == "RiskDecision", blocked)
require(blocked["execution_command_boundary"] == "ExecutionCommand", blocked)
require(blocked["execution_command_created"] is False, blocked)
require(blocked["execution_command_route"] == "blocked_before_execution_command", blocked)
require(blocked["execution_command_destination"] == "none", blocked)
require(blocked["dry_run_adapter_boundary"] == "DryRunExecutionAdapter", blocked)
require(blocked["dry_run_adapter_route_allowed"] is False, blocked)
require(blocked["production_adapter_boundary"] == "ProductionExecutionAdapter", blocked)
require(blocked["production_adapter_route_allowed"] is False, blocked)
require(blocked["production_adapter_instantiation_allowed"] is False, blocked)
require(blocked["dry_run_execution_adapter_called"] is False, blocked)
require(blocked["production_adapter_instantiated"] is False, blocked)
require(blocked["production_adapter_called"] is False, blocked)
require(blocked["network_attempted"] is False, blocked)
require(len(blocked["missing_cli_flags"]) == 10, blocked)
require(len(blocked["source_artifact_issues"]) == 0, blocked)

require(ready["schema_version"] == "ntpro.v150_live_alpha_execution_dry_run.v1", ready)
require(ready["status"] == "ready_dry_run_execution_adapter_only", ready)
require(ready["execution_decision"] == "dry_run_adapter_artifact_only", ready)
require(ready["execution_boundary_contract_version"] == "ntpro.v151_execution_dry_run_adapter_boundary.v1", ready)
require(ready["execution_boundary_flow"] == "StrategyIntent -> RiskDecision -> ExecutionCommand -> DryRunExecutionAdapter", ready)
require(ready["execution_boundary_contract_ready"] is True, ready)
require(ready["strategy_intent_boundary"] == "StrategyIntent", ready)
require(ready["risk_decision_boundary"] == "RiskDecision", ready)
require(ready["execution_command_boundary"] == "ExecutionCommand", ready)
require(ready["execution_command_created"] is True, ready)
require(ready["execution_command_route"] == "dry_run_adapter_only", ready)
require(ready["execution_command_destination"] == "ntpro_local_artifact_dry_run_execution_adapter", ready)
require(ready["dry_run_adapter_boundary"] == "DryRunExecutionAdapter", ready)
require(ready["dry_run_adapter_route_allowed"] is True, ready)
require(ready["production_adapter_boundary"] == "ProductionExecutionAdapter", ready)
require(ready["production_adapter_route_allowed"] is False, ready)
require(ready["production_adapter_instantiation_allowed"] is False, ready)
require(ready["dry_run_execution_adapter"] == "ntpro_local_artifact_dry_run_execution_adapter", ready)
require(ready["dry_run_execution_adapter_called"] is True, ready)
require(ready["dry_run_execution_adapter_wrote_artifact"] is True, ready)
require(ready["dry_run_adapter_artifact_only"] is True, ready)
require(ready["real_execution_adapter_called"] is False, ready)
require(ready["production_adapter_instantiated"] is False, ready)
require(ready["production_adapter_called"] is False, ready)
require(ready["strategy_intent_reaches_risk_preflight"] is True, ready)
require(ready["strategy_intent_reaches_dry_run_adapter"] is True, ready)
require(ready["strategy_intent_reaches_production_adapter"] is False, ready)
require(ready["order_gate_ready"] is True, ready)
require(ready["risk_preflight_decision"] == "dry_run_approved", ready)
require(ready["request_preview_built"] is True, ready)
require(ready["request_sent"] is False, ready)
require(ready["kill_switch_runtime_gate_status"] == "ready_runtime_gate_open_for_dry_run_only", ready)
require(ready["kill_switch_runtime_gate_open"] is True, ready)
require(len(ready["source_artifact_issues"]) == 0, ready)
require(len(ready["missing_cli_flags"]) == 0, ready)
for key in [
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "production_order_state_reads_allowed",
    "listen_key_lifecycle_allowed",
    "cancel_replace_amend_attempted",
    "order_endpoint_access_attempted",
    "network_attempted",
    "matching_engine_submission",
    "dashboard_order_controls_enabled",
    "external_venue_connection",
    "real_orders_submitted",
    "real_funds",
    "production_trading_enabled",
    "production_adapter_route_allowed",
    "production_adapter_instantiation_allowed",
    "order_state_values_are_exchange_truth",
    "shadow_values_are_exchange_truth",
    "portfolio_values_are_exchange_truth",
    "values_are_exchange_truth",
]:
    require(ready[key] is False, (key, ready[key], ready))
for key in [
    "production_order_submissions_attempted",
    "production_orders_submitted",
    "production_order_mutations_attempted",
    "production_order_state_reads_attempted",
    "listen_key_lifecycle_attempted",
    "actual_submission_count",
    "automatic_correction_orders_submitted",
]:
    require(ready[key] == 0, (key, ready[key], ready))
PY

if grep -R -q "$SYNTHETIC_API_KEY\|$SYNTHETIC_API_SECRET" "$OUTPUT_DIR"; then
  echo "v15 execution dry-run leaked a synthetic secret into output artifacts" >&2
  exit 1
fi
if grep -R -q "production_adapter_called=true\|production_adapter_instantiated=true\|network_attempted=true\|production_orders_submitted\":[[:space:]]*[1-9]" "$OUTPUT_DIR"; then
  echo "v15 execution dry-run recorded forbidden production adapter, network, or order mutation evidence" >&2
  exit 1
fi

echo "v15_execution_adapter_isolation status=ok root=$ISOLATION_ROOT execution_command_route=dry_run_adapter_only dry_run_execution_adapter_called=true production_adapter_called=false production_adapter_instantiated=false production_adapter_route_allowed=false network_attempted=false production_orders_submitted=0"
