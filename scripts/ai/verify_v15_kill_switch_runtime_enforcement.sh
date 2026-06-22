#!/usr/bin/env bash
set -euo pipefail

# V150-004: v0.15 production live-alpha kill-switch runtime enforcement.
# Safe for local development and CI. It evaluates local artifacts only and must
# never open network connections, instantiate production adapters, or submit
# production orders.

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

GATE_ROOT="${NTPRO_V15_KILL_SWITCH_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v15-kill-switch.XXXXXX")}"
OUTPUT_DIR="$GATE_ROOT/command-output"
mkdir -p "$OUTPUT_DIR"

ORDER_GATE="$OUTPUT_DIR/live-alpha-order-gate.json"
RISK_INPUT="$OUTPUT_DIR/live-alpha-risk-input.json"
RISK_PREFLIGHT="$OUTPUT_DIR/live-alpha-risk-preflight.json"
READY_MANUAL_APPROVAL="$OUTPUT_DIR/ready-manual-approval-lifecycle.json"
BLOCKED_MANUAL_APPROVAL="$OUTPUT_DIR/blocked-manual-approval-lifecycle.json"
READY_REQUEST_PREVIEW="$OUTPUT_DIR/ready-request-preview.json"
BLOCKED_REQUEST_PREVIEW="$OUTPUT_DIR/blocked-request-preview.json"
ACTIVE_APPROVAL="$OUTPUT_DIR/active-kill-switch-approval.json"
PENDING_APPROVAL="$OUTPUT_DIR/pending-kill-switch-approval.json"
APPROVED_APPROVAL="$OUTPUT_DIR/approved-kill-switch-approval.json"
ACTIVE_GATE="$OUTPUT_DIR/active-kill-switch-runtime-gate.json"
PENDING_GATE="$OUTPUT_DIR/pending-kill-switch-runtime-gate.json"
BLOCKED_PREVIEW_GATE="$OUTPUT_DIR/blocked-preview-kill-switch-runtime-gate.json"
READY_GATE="$OUTPUT_DIR/ready-kill-switch-runtime-gate.json"
ACTIVE_EXECUTION_REPORT="$OUTPUT_DIR/active-kill-switch-execution-dry-run.json"
ACTIVE_EXECUTION_STDOUT="$OUTPUT_DIR/active-execution.stdout.log"
ACTIVE_EXECUTION_STDERR="$OUTPUT_DIR/active-execution.stderr.log"

SYNTHETIC_API_KEY="ntpro_v151003_synthetic_api_key_value"
SYNTHETIC_API_SECRET="ntpro_v151003_synthetic_api_secret_value"

"$NAUTILUS_BIN" live production-live-alpha-dry-run-order-gate \
  --run-id v150-kill-switch-runtime \
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
  --run-id v150-kill-switch-risk \
  --order-gate "$ORDER_GATE" \
  --input "$RISK_INPUT" \
  --output "$RISK_PREFLIGHT" \
  --confirm-hypothetical-dry-run-only \
  --confirm-no-execution-adapter-call \
  --confirm-no-production-order-submission \
  --confirm-no-production-order-mutation \
  --confirm-dashboard-order-controls-disabled >/dev/null

write_manual_approval() {
  local run_id="$1"
  local output="$2"
  "$NAUTILUS_BIN" live production-live-alpha-manual-approval-lifecycle \
    --run-id "$run_id" \
    --strategy-id ema_cross_btcusdt_v1 \
    --symbol BTCUSDT \
    --notional 10.00 \
    --approval-state approved \
    --manual-approval-id owner-approval-v150-005 \
    --approved-by owner \
    --now-unix-ms 1718400000000 \
    --expires-at-unix-ms 1718400060000 \
    --output "$output" \
    --confirm-dry-run-request-preview-only \
    --confirm-one-time-approval \
    --confirm-no-production-mutation \
    --confirm-dashboard-order-controls-disabled >/dev/null
}

write_manual_approval v150-kill-switch-ready-request-preview "$READY_MANUAL_APPROVAL"
write_manual_approval v150-kill-switch-blocked-request-preview "$BLOCKED_MANUAL_APPROVAL"

NTPRO_V150004_API_KEY="$SYNTHETIC_API_KEY" \
NTPRO_V150004_API_SECRET="$SYNTHETIC_API_SECRET" \
  "$NAUTILUS_BIN" live production-live-alpha-order-request-preview \
    --run-id v150-kill-switch-ready-request-preview \
    --order-gate "$ORDER_GATE" \
    --manual-approval-lifecycle "$READY_MANUAL_APPROVAL" \
    --endpoint-path /api/v3/order \
    --price 10000.00 \
    --time-in-force GTC \
    --timestamp-ms 1718400000000 \
    --recv-window-ms 5000 \
    --api-key-env NTPRO_V150004_API_KEY \
    --api-secret-env NTPRO_V150004_API_SECRET \
    --output "$READY_REQUEST_PREVIEW" \
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

"$NAUTILUS_BIN" live production-live-alpha-order-request-preview \
  --run-id v150-kill-switch-blocked-request-preview \
  --order-gate "$ORDER_GATE" \
  --manual-approval-lifecycle "$BLOCKED_MANUAL_APPROVAL" \
  --endpoint-path /api/v3/order \
  --price 10000.00 \
  --time-in-force GTC \
  --timestamp-ms 1718400000000 \
  --recv-window-ms 5000 \
  --api-key-env NTPRO_V150004_MISSING_API_KEY \
  --api-secret-env NTPRO_V150004_MISSING_API_SECRET \
  --credential-material production_live_alpha \
  --output "$BLOCKED_REQUEST_PREVIEW" \
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
  --run-id v150-kill-switch-active \
  --session-id session-v150 \
  --strategy-id ema_cross_btcusdt_v1 \
  --output "$ACTIVE_APPROVAL" \
  --kill-switch-active true \
  --approval-state approved \
  --manual-approval-id owner-approval-v150-004 \
  --approved-by owner \
  --confirm-dry-run-only \
  --confirm-no-production-mutation \
  --confirm-dashboard-order-controls-disabled >/dev/null

"$NAUTILUS_BIN" live production-kill-switch-approval-artifact \
  --run-id v150-kill-switch-pending \
  --session-id session-v150 \
  --strategy-id ema_cross_btcusdt_v1 \
  --output "$PENDING_APPROVAL" \
  --kill-switch-active false \
  --approval-state pending \
  --confirm-dry-run-only \
  --confirm-no-production-mutation \
  --confirm-dashboard-order-controls-disabled >/dev/null

"$NAUTILUS_BIN" live production-kill-switch-approval-artifact \
  --run-id v150-kill-switch-approved \
  --session-id session-v150 \
  --strategy-id ema_cross_btcusdt_v1 \
  --output "$APPROVED_APPROVAL" \
  --kill-switch-active false \
  --approval-state approved \
  --manual-approval-id owner-approval-v150-004 \
  --approved-by owner \
  --confirm-dry-run-only \
  --confirm-no-production-mutation \
  --confirm-dashboard-order-controls-disabled >/dev/null

run_runtime_gate() {
  local approval="$1"
  local request_preview="$2"
  local output="$3"
  "$NAUTILUS_BIN" live production-live-alpha-kill-switch-runtime-gate \
    --run-id v150-kill-switch-runtime-gate \
    --kill-switch-approval "$approval" \
    --risk-preflight "$RISK_PREFLIGHT" \
    --request-preview "$request_preview" \
    --output "$output" \
    --allow-production-live-alpha-kill-switch-runtime-gate \
    --confirm-owner-approved-runtime-gate \
    --confirm-no-production-order-submission \
    --confirm-no-production-order-mutation \
    --confirm-no-network \
    --confirm-no-listen-key-lifecycle \
    --confirm-dashboard-order-controls-disabled \
    --confirm-no-real-funds >/dev/null
}

run_runtime_gate "$ACTIVE_APPROVAL" "$READY_REQUEST_PREVIEW" "$ACTIVE_GATE"
run_runtime_gate "$PENDING_APPROVAL" "$READY_REQUEST_PREVIEW" "$PENDING_GATE"
run_runtime_gate "$APPROVED_APPROVAL" "$BLOCKED_REQUEST_PREVIEW" "$BLOCKED_PREVIEW_GATE"
run_runtime_gate "$APPROVED_APPROVAL" "$READY_REQUEST_PREVIEW" "$READY_GATE"

"$NAUTILUS_BIN" live production-live-alpha-execution-dry-run \
  --run-id v150-active-kill-switch-execution-dry-run \
  --order-gate "$ORDER_GATE" \
  --risk-preflight "$RISK_PREFLIGHT" \
  --request-preview "$READY_REQUEST_PREVIEW" \
  --kill-switch-runtime-gate "$ACTIVE_GATE" \
  --output "$ACTIVE_EXECUTION_REPORT" \
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
  >"$ACTIVE_EXECUTION_STDOUT" \
  2>"$ACTIVE_EXECUTION_STDERR"

if [[ -s "$ACTIVE_EXECUTION_STDERR" ]]; then
  echo "v15 kill-switch execution blocked path wrote stderr" >&2
  cat "$ACTIVE_EXECUTION_STDERR" >&2
  exit 1
fi
grep -q "status=blocked_source_artifact" "$ACTIVE_EXECUTION_STDOUT"
grep -q "dry_run_execution_adapter_called=false" "$ACTIVE_EXECUTION_STDOUT"
grep -q "production_orders_submitted=0" "$ACTIVE_EXECUTION_STDOUT"
grep -q "network_attempted=false" "$ACTIVE_EXECUTION_STDOUT"

python3 - "$ACTIVE_GATE" "$PENDING_GATE" "$BLOCKED_PREVIEW_GATE" "$READY_GATE" "$ACTIVE_EXECUTION_REPORT" <<'PY'
import json
import sys
from pathlib import Path

active, pending, blocked_preview, ready, active_execution = [
    json.loads(Path(path).read_text()) for path in sys.argv[1:]
]

def require(condition, message):
    if not condition:
        raise SystemExit(message)

def require_no_mutation(artifact):
    for key in [
        "production_order_submissions_attempted",
        "production_orders_submitted",
        "production_order_mutations_attempted",
        "production_order_state_reads_attempted",
        "listen_key_lifecycle_attempted",
        "actual_submission_count",
        "automatic_correction_orders_submitted",
    ]:
        require(artifact[key] == 0, (key, artifact[key], artifact))
    for key in [
        "production_order_submission_allowed",
        "production_order_mutation_allowed",
        "production_order_state_reads_allowed",
        "listen_key_lifecycle_allowed",
        "cancel_replace_amend_attempted",
        "order_endpoint_access_attempted",
        "execution_adapter_called",
        "production_adapter_called",
        "matching_engine_submission",
        "dashboard_order_controls_enabled",
        "external_venue_connection",
        "network_attempted",
        "real_orders_submitted",
        "real_funds",
        "production_trading_enabled",
    ]:
        require(artifact[key] is False, (key, artifact[key], artifact))

for artifact in [active, pending, blocked_preview, ready]:
    require(artifact["schema_version"] == "ntpro.v150_live_alpha_kill_switch_runtime_gate.v1", artifact)
    require_no_mutation(artifact)

require(active["status"] == "blocked_kill_switch_active", active)
require(active["runtime_gate_open"] is False, active)
require(active["kill_switch_active"] is True, active)
require(active["manual_approval_recorded"] is True, active)
require("kill_switch_active" in active["runtime_gate_reasons"], active)

require(pending["status"] == "blocked_missing_manual_approval", pending)
require(pending["runtime_gate_open"] is False, pending)
require(pending["kill_switch_active"] is False, pending)
require(pending["approval_state"] == "pending", pending)
require(pending["manual_approval_recorded"] is False, pending)
require("manual_approval_missing_or_not_approved" in pending["runtime_gate_reasons"], pending)

require(blocked_preview["status"] == "blocked_request_preview", blocked_preview)
require(blocked_preview["runtime_gate_open"] is False, blocked_preview)
require(blocked_preview["request_preview_built"] is False, blocked_preview)
require("request_preview_blocked" in blocked_preview["runtime_gate_reasons"], blocked_preview)

require(ready["status"] == "ready_runtime_gate_open_for_dry_run_only", ready)
require(ready["runtime_gate_open"] is True, ready)
require(ready["kill_switch_active"] is False, ready)
require(ready["manual_approval_recorded"] is True, ready)
require(ready["request_preview_built"] is True, ready)

require(active_execution["schema_version"] == "ntpro.v150_live_alpha_execution_dry_run.v1", active_execution)
require(active_execution["status"] == "blocked_source_artifact", active_execution)
require(active_execution["dry_run_execution_adapter_called"] is False, active_execution)
require(active_execution["production_adapter_called"] is False, active_execution)
require(active_execution["production_adapter_instantiated"] is False, active_execution)
require(active_execution["network_attempted"] is False, active_execution)
require(active_execution["production_orders_submitted"] == 0, active_execution)
require("kill_switch_runtime_gate_not_ready" in active_execution["source_artifact_issues"], active_execution)
require("kill_switch_runtime_gate_closed" in active_execution["source_artifact_issues"], active_execution)
require("kill_switch_runtime_gate_active" in active_execution["source_artifact_issues"], active_execution)
PY

if grep -R -q "$SYNTHETIC_API_KEY\|$SYNTHETIC_API_SECRET" "$OUTPUT_DIR"; then
  echo "v15 kill-switch runtime gate leaked a synthetic secret into output artifacts" >&2
  exit 1
fi
if grep -R -q "network_attempted\":[[:space:]]*true\|production_orders_submitted\":[[:space:]]*[1-9]" "$OUTPUT_DIR"; then
  echo "v15 kill-switch runtime gate recorded forbidden network or order mutation evidence" >&2
  exit 1
fi

echo "v15_kill_switch_runtime_enforcement status=ok root=$GATE_ROOT active_blocked=true missing_approval_blocked=true request_preview_blocked=true production_orders_submitted=0 network_attempted=false"
