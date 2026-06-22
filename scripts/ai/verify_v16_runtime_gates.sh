#!/usr/bin/env bash
set -euo pipefail

# V160-002: v0.16 owner-approved production mutation runtime gates.
# This verifier is local/offline only. It must not read real credentials, open
# network connections, submit production orders, mutate exchange state, or enable
# Dashboard order controls.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_V16_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V16_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V16_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi

GATE_ROOT="${NTPRO_V16_RUNTIME_GATE_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v16-runtime-gates.XXXXXX")}"
OUTPUT_DIR="$GATE_ROOT/command-output"
mkdir -p "$OUTPUT_DIR"

ORDER_GATE="$OUTPUT_DIR/live-alpha-order-gate.json"
RISK_INPUT="$OUTPUT_DIR/live-alpha-risk-input.json"
RISK_PREFLIGHT="$OUTPUT_DIR/live-alpha-risk-preflight.json"
READY_MANUAL_APPROVAL="$OUTPUT_DIR/ready-manual-approval-lifecycle.json"
BLOCKED_MANUAL_APPROVAL="$OUTPUT_DIR/blocked-manual-approval-lifecycle.json"
READY_REQUEST_PREVIEW="$OUTPUT_DIR/ready-request-preview.json"
BLOCKED_REQUEST_PREVIEW="$OUTPUT_DIR/blocked-request-preview.json"
APPROVED_KILL_SWITCH_APPROVAL="$OUTPUT_DIR/approved-kill-switch-approval.json"
ACTIVE_KILL_SWITCH_APPROVAL="$OUTPUT_DIR/active-kill-switch-approval.json"
READY_KILL_SWITCH_GATE="$OUTPUT_DIR/ready-kill-switch-runtime-gate.json"
ACTIVE_KILL_SWITCH_GATE="$OUTPUT_DIR/active-kill-switch-runtime-gate.json"
BLOCKED_PREVIEW_KILL_SWITCH_GATE="$OUTPUT_DIR/blocked-preview-kill-switch-runtime-gate.json"
MISSING_FLAGS_GATE="$OUTPUT_DIR/missing-flags-production-mutation-runtime-gate.json"
ACTIVE_KILL_SWITCH_PRODUCTION_GATE="$OUTPUT_DIR/active-kill-switch-production-mutation-runtime-gate.json"
BLOCKED_PREVIEW_PRODUCTION_GATE="$OUTPUT_DIR/blocked-preview-production-mutation-runtime-gate.json"
MISSING_SIGNING_GATE="$OUTPUT_DIR/missing-signing-production-mutation-runtime-gate.json"

SYNTHETIC_API_KEY="ntpro_v160002_synthetic_api_key_value"
SYNTHETIC_API_SECRET="ntpro_v160002_synthetic_api_secret_value"

"$NAUTILUS_BIN" live production-live-alpha-dry-run-order-gate \
  --run-id v160-runtime-gate \
  --session-id session-v160 \
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
    "account": {"readable": True, "account_id": "BINANCE-001"},
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
  --run-id v160-runtime-gate-risk \
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
    --manual-approval-id owner-approval-v160-002 \
    --approved-by owner \
    --now-unix-ms 1718400000000 \
    --expires-at-unix-ms 1718400060000 \
    --output "$output" \
    --confirm-dry-run-request-preview-only \
    --confirm-one-time-approval \
    --confirm-no-production-mutation \
    --confirm-dashboard-order-controls-disabled >/dev/null
}

write_manual_approval v150-live-alpha-request-preview "$READY_MANUAL_APPROVAL"
write_manual_approval v150-live-alpha-request-preview "$BLOCKED_MANUAL_APPROVAL"

NTPRO_V160002_API_KEY="$SYNTHETIC_API_KEY" \
NTPRO_V160002_API_SECRET="$SYNTHETIC_API_SECRET" \
  "$NAUTILUS_BIN" live production-live-alpha-order-request-preview \
    --run-id v150-live-alpha-request-preview \
    --order-gate "$ORDER_GATE" \
    --manual-approval-lifecycle "$READY_MANUAL_APPROVAL" \
    --endpoint-path /api/v3/order \
    --price 10000.00 \
    --time-in-force GTC \
    --timestamp-ms 1718400000000 \
    --recv-window-ms 5000 \
    --api-key-env NTPRO_V160002_API_KEY \
    --api-secret-env NTPRO_V160002_API_SECRET \
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
  --run-id v150-live-alpha-request-preview \
  --order-gate "$ORDER_GATE" \
  --manual-approval-lifecycle "$BLOCKED_MANUAL_APPROVAL" \
  --endpoint-path /api/v3/order \
  --price 10000.00 \
  --time-in-force GTC \
  --timestamp-ms 1718400000000 \
  --recv-window-ms 5000 \
  --api-key-env NTPRO_V160002_MISSING_API_KEY \
  --api-secret-env NTPRO_V160002_MISSING_API_SECRET \
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
  --run-id v160-runtime-gate-ready \
  --session-id session-v160 \
  --strategy-id ema_cross_btcusdt_v1 \
  --output "$APPROVED_KILL_SWITCH_APPROVAL" \
  --kill-switch-active false \
  --approval-state approved \
  --manual-approval-id owner-approval-v160-002 \
  --approved-by owner \
  --confirm-dry-run-only \
  --confirm-no-production-mutation \
  --confirm-dashboard-order-controls-disabled >/dev/null

"$NAUTILUS_BIN" live production-kill-switch-approval-artifact \
  --run-id v160-runtime-gate-active \
  --session-id session-v160 \
  --strategy-id ema_cross_btcusdt_v1 \
  --output "$ACTIVE_KILL_SWITCH_APPROVAL" \
  --kill-switch-active true \
  --approval-state approved \
  --manual-approval-id owner-approval-v160-002 \
  --approved-by owner \
  --confirm-dry-run-only \
  --confirm-no-production-mutation \
  --confirm-dashboard-order-controls-disabled >/dev/null

run_kill_switch_gate() {
  local approval="$1"
  local request_preview="$2"
  local output="$3"
  "$NAUTILUS_BIN" live production-live-alpha-kill-switch-runtime-gate \
    --run-id v160-kill-switch-runtime-gate \
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

run_kill_switch_gate "$APPROVED_KILL_SWITCH_APPROVAL" "$READY_REQUEST_PREVIEW" "$READY_KILL_SWITCH_GATE"
run_kill_switch_gate "$ACTIVE_KILL_SWITCH_APPROVAL" "$READY_REQUEST_PREVIEW" "$ACTIVE_KILL_SWITCH_GATE"
run_kill_switch_gate "$APPROVED_KILL_SWITCH_APPROVAL" "$BLOCKED_REQUEST_PREVIEW" "$BLOCKED_PREVIEW_KILL_SWITCH_GATE"

run_production_runtime_gate() {
  local runtime_gate="$1"
  local request_preview="$2"
  local output="$3"
  shift 3
  "$NAUTILUS_BIN" live production-mutation-runtime-gate \
    --run-id v160-production-mutation-runtime-gate \
    --order-gate "$ORDER_GATE" \
    --risk-preflight "$RISK_PREFLIGHT" \
    --request-preview "$request_preview" \
    --kill-switch-runtime-gate "$runtime_gate" \
    --output "$output" \
    --max-notional 10.00 \
    "$@"
}

run_production_runtime_gate "$READY_KILL_SWITCH_GATE" "$READY_REQUEST_PREVIEW" "$MISSING_FLAGS_GATE" >/dev/null
run_production_runtime_gate "$ACTIVE_KILL_SWITCH_GATE" "$READY_REQUEST_PREVIEW" "$ACTIVE_KILL_SWITCH_PRODUCTION_GATE" \
  --allow-production-mutation-runtime-gate \
  --confirm-owner-approved-production-mutation \
  --confirm-single-limit-gtc \
  --confirm-tiny-notional \
  --confirm-signing-approval-required \
  --confirm-no-network-before-send \
  --confirm-dashboard-order-controls-disabled \
  --confirm-no-listen-key-lifecycle \
  --confirm-no-retry >/dev/null
run_production_runtime_gate "$BLOCKED_PREVIEW_KILL_SWITCH_GATE" "$BLOCKED_REQUEST_PREVIEW" "$BLOCKED_PREVIEW_PRODUCTION_GATE" \
  --allow-production-mutation-runtime-gate \
  --confirm-owner-approved-production-mutation \
  --confirm-single-limit-gtc \
  --confirm-tiny-notional \
  --confirm-signing-approval-required \
  --confirm-no-network-before-send \
  --confirm-dashboard-order-controls-disabled \
  --confirm-no-listen-key-lifecycle \
  --confirm-no-retry >/dev/null
run_production_runtime_gate "$READY_KILL_SWITCH_GATE" "$READY_REQUEST_PREVIEW" "$MISSING_SIGNING_GATE" \
  --allow-production-mutation-runtime-gate \
  --confirm-owner-approved-production-mutation \
  --confirm-single-limit-gtc \
  --confirm-tiny-notional \
  --confirm-signing-approval-required \
  --confirm-no-network-before-send \
  --confirm-dashboard-order-controls-disabled \
  --confirm-no-listen-key-lifecycle \
  --confirm-no-retry >/dev/null

python3 - "$MISSING_FLAGS_GATE" "$ACTIVE_KILL_SWITCH_PRODUCTION_GATE" "$BLOCKED_PREVIEW_PRODUCTION_GATE" "$MISSING_SIGNING_GATE" <<'PY'
import json
import sys
from pathlib import Path

expected = {
    "missing-flags": (Path(sys.argv[1]), "blocked_missing_gate", "missing_owner_runtime_gate_confirmation"),
    "active-kill-switch": (Path(sys.argv[2]), "blocked_kill_switch_active", "kill_switch_active"),
    "blocked-preview": (Path(sys.argv[3]), "blocked_request_preview", "request_preview_blocked_or_sent"),
    "missing-signing": (Path(sys.argv[4]), "blocked_signing_approval", "signing_approval_missing"),
}

for name, (path, status, reason) in expected.items():
    artifact = json.loads(path.read_text())
    assert artifact["schema_version"] == "ntpro.v160_production_mutation_runtime_gate.v1", (name, artifact)
    assert artifact["capability"] == "Minimum Owner-Approved Production Order Mutation Candidate", (name, artifact)
    assert artifact["status"] == status, (name, artifact["status"])
    assert artifact["default_fail_closed"] is True, (name, artifact)
    assert artifact["runtime_gate_open"] is False, (name, artifact)
    assert artifact["send_consideration_allowed"] is False, (name, artifact)
    assert reason in artifact["runtime_gate_reasons"], (name, artifact["runtime_gate_reasons"])
    assert artifact["request_sent"] is False, (name, artifact)
    assert artifact["network_attempted"] is False, (name, artifact)
    assert artifact["production_order_submission_allowed"] is False, (name, artifact)
    assert artifact["production_order_mutation_allowed"] is False, (name, artifact)
    assert artifact["production_order_submissions_attempted"] == 0, (name, artifact)
    assert artifact["production_orders_submitted"] == 0, (name, artifact)
    assert artifact["production_order_mutations_attempted"] == 0, (name, artifact)
    assert artifact["dashboard_order_controls_enabled"] is False, (name, artifact)
    assert artifact["listen_key_lifecycle_attempted"] == 0, (name, artifact)
    assert artifact["retry_attempted"] is False, (name, artifact)
    assert artifact["cancel_attempted"] is False, (name, artifact)
    assert artifact["replace_attempted"] is False, (name, artifact)
    assert artifact["amend_attempted"] is False, (name, artifact)
    assert artifact["flatten_attempted"] is False, (name, artifact)
    assert artifact["signature_recorded"] is False, (name, artifact)
    assert artifact["signed_query_recorded"] is False, (name, artifact)
    assert artifact["signed_url_recorded"] is False, (name, artifact)
    assert artifact["api_key_value_recorded"] is False, (name, artifact)
    assert artifact["api_secret_value_recorded"] is False, (name, artifact)

missing_signing = json.loads(Path(sys.argv[4]).read_text())
assert missing_signing["owner_approval_consumed"] is True, missing_signing
assert missing_signing["kill_switch_checked_before_send"] is True, missing_signing
assert missing_signing["kill_switch_runtime_gate_open"] is True, missing_signing
assert missing_signing["risk_preflight_decision"] == "dry_run_approved", missing_signing
assert missing_signing["single_order_candidate"] is True, missing_signing
assert missing_signing["tiny_notional_gate_ready"] is True, missing_signing
assert missing_signing["order_type"] == "LIMIT", missing_signing
assert missing_signing["time_in_force"] == "GTC", missing_signing
assert missing_signing["signing_approval_required"] is True, missing_signing
assert missing_signing["signing_approval_ready"] is False, missing_signing
assert missing_signing["explicit_send_gate_open"] is False, missing_signing
PY

if grep -RE \
  "request_sent=true|network_attempted=true|production_orders_submitted=[1-9][0-9]*|production_order_mutations_attempted=[1-9][0-9]*|dashboard_order_controls_enabled=true|production_adapter_called=true|production_adapter_instantiated=true|real_orders_submitted=true|real_funds=true|production_trading_enabled=true|retry_attempted=true|cancel_attempted=true|replace_attempted=true|amend_attempted=true|flatten_attempted=true|signature_recorded=true|signed_query_recorded=true|signed_url_recorded=true|api_key_value_recorded=true|api_secret_value_recorded=true" \
  "$OUTPUT_DIR" >/tmp/ntpro-v16-runtime-gate-forbidden.txt; then
  echo "v16 runtime gate observed forbidden production mutation evidence:" >&2
  cat /tmp/ntpro-v16-runtime-gate-forbidden.txt >&2
  exit 1
fi

echo "v16_runtime_gates status=ok root=$GATE_ROOT request_sent=false network_attempted=false production_orders_submitted=0 production_order_mutations_attempted=0 dashboard_order_controls_enabled=false signing_approval_ready=false"
