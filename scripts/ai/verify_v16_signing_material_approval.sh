#!/usr/bin/env bash
set -euo pipefail

# V160-003: v0.16 production signing material approval artifact.
# This verifier is local/offline only. It uses dummy environment values to prove
# production_live_alpha signing material approval can be evidenced without
# persisting API keys, secrets, signatures, signed queries, raw request bodies,
# opening network connections, or submitting production orders.

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

GATE_ROOT="${NTPRO_V16_SIGNING_APPROVAL_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v16-signing-approval.XXXXXX")}"
OUTPUT_DIR="$GATE_ROOT/command-output"
mkdir -p "$OUTPUT_DIR"

ORDER_GATE="$OUTPUT_DIR/live-alpha-order-gate.json"
RISK_INPUT="$OUTPUT_DIR/live-alpha-risk-input.json"
RISK_PREFLIGHT="$OUTPUT_DIR/live-alpha-risk-preflight.json"
SYNTHETIC_MANUAL_APPROVAL="$OUTPUT_DIR/synthetic-manual-approval-lifecycle.json"
PRODUCTION_MANUAL_APPROVAL="$OUTPUT_DIR/production-manual-approval-lifecycle.json"
SYNTHETIC_REQUEST_PREVIEW="$OUTPUT_DIR/synthetic-request-preview.json"
PRODUCTION_REQUEST_PREVIEW="$OUTPUT_DIR/production-request-preview.json"
SYNTHETIC_SIGNING_APPROVAL="$OUTPUT_DIR/synthetic-signing-approval.json"
MISSING_FLAGS_SIGNING_APPROVAL="$OUTPUT_DIR/missing-flags-signing-approval.json"
READY_SIGNING_APPROVAL="$OUTPUT_DIR/ready-signing-approval.json"
KILL_SWITCH_APPROVAL="$OUTPUT_DIR/kill-switch-approval.json"
KILL_SWITCH_GATE="$OUTPUT_DIR/kill-switch-runtime-gate.json"
RUNTIME_GATE_WITH_SIGNING="$OUTPUT_DIR/runtime-gate-with-signing-approval.json"

SYNTHETIC_API_KEY="ntpro_v160003_synthetic_api_key_value"
SYNTHETIC_API_SECRET="ntpro_v160003_synthetic_api_secret_value"
PRODUCTION_API_KEY="ntpro_v160003_production_like_api_key_value"
PRODUCTION_API_SECRET="ntpro_v160003_production_like_api_secret_value"

"$NAUTILUS_BIN" live production-live-alpha-dry-run-order-gate \
  --run-id v160-signing-approval \
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
  --run-id v160-signing-approval-risk \
  --order-gate "$ORDER_GATE" \
  --input "$RISK_INPUT" \
  --output "$RISK_PREFLIGHT" \
  --confirm-hypothetical-dry-run-only \
  --confirm-no-execution-adapter-call \
  --confirm-no-production-order-submission \
  --confirm-no-production-order-mutation \
  --confirm-dashboard-order-controls-disabled >/dev/null

write_manual_approval() {
  local output="$1"
  "$NAUTILUS_BIN" live production-live-alpha-manual-approval-lifecycle \
    --run-id v150-live-alpha-request-preview \
    --strategy-id ema_cross_btcusdt_v1 \
    --symbol BTCUSDT \
    --notional 10.00 \
    --approval-state approved \
    --manual-approval-id owner-approval-v160-003 \
    --approved-by owner \
    --now-unix-ms 1718400000000 \
    --expires-at-unix-ms 1718400060000 \
    --output "$output" \
    --confirm-dry-run-request-preview-only \
    --confirm-one-time-approval \
    --confirm-no-production-mutation \
    --confirm-dashboard-order-controls-disabled >/dev/null
}

write_manual_approval "$SYNTHETIC_MANUAL_APPROVAL"
write_manual_approval "$PRODUCTION_MANUAL_APPROVAL"

NTPRO_V160003_SYNTHETIC_API_KEY="$SYNTHETIC_API_KEY" \
NTPRO_V160003_SYNTHETIC_API_SECRET="$SYNTHETIC_API_SECRET" \
  "$NAUTILUS_BIN" live production-live-alpha-order-request-preview \
    --run-id v150-live-alpha-request-preview \
    --order-gate "$ORDER_GATE" \
    --manual-approval-lifecycle "$SYNTHETIC_MANUAL_APPROVAL" \
    --endpoint-path /api/v3/order \
    --price 10000.00 \
    --time-in-force GTC \
    --timestamp-ms 1718400000000 \
    --recv-window-ms 5000 \
    --api-key-env NTPRO_V160003_SYNTHETIC_API_KEY \
    --api-secret-env NTPRO_V160003_SYNTHETIC_API_SECRET \
    --output "$SYNTHETIC_REQUEST_PREVIEW" \
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

NTPRO_ALLOW_PRODUCTION_MUTATION_SIGNING_MATERIAL=1 \
NTPRO_OWNER_APPROVED_MUTATION_SIGNING_DRY_RUN=1 \
NTPRO_V160003_PRODUCTION_API_KEY="$PRODUCTION_API_KEY" \
NTPRO_V160003_PRODUCTION_API_SECRET="$PRODUCTION_API_SECRET" \
  "$NAUTILUS_BIN" live production-live-alpha-order-request-preview \
    --run-id v150-live-alpha-request-preview \
    --order-gate "$ORDER_GATE" \
    --manual-approval-lifecycle "$PRODUCTION_MANUAL_APPROVAL" \
    --endpoint-path /api/v3/order \
    --price 10000.00 \
    --time-in-force GTC \
    --timestamp-ms 1718400000000 \
    --recv-window-ms 5000 \
    --api-key-env NTPRO_V160003_PRODUCTION_API_KEY \
    --api-secret-env NTPRO_V160003_PRODUCTION_API_SECRET \
    --credential-material production_live_alpha \
    --output "$PRODUCTION_REQUEST_PREVIEW" \
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

run_signing_approval() {
  local request_preview="$1"
  local output="$2"
  shift 2
  "$NAUTILUS_BIN" live production-mutation-signing-approval \
    --run-id v160-production-mutation-signing-approval \
    --request-preview "$request_preview" \
    --approval-state approved \
    --manual-approval-id owner-approval-v160-003 \
    --approved-by owner \
    --now-unix-ms 1718400000000 \
    --expires-at-unix-ms 1718400060000 \
    --output "$output" \
    "$@"
}

run_signing_approval "$SYNTHETIC_REQUEST_PREVIEW" "$SYNTHETIC_SIGNING_APPROVAL" \
  --allow-production-mutation-signing-approval \
  --confirm-owner-approved-signing-material \
  --confirm-env-only-signing-material \
  --confirm-memory-only-signing \
  --confirm-no-secret-persistence \
  --confirm-no-network \
  --confirm-no-production-order-submission \
  --confirm-no-production-order-mutation \
  --confirm-dashboard-order-controls-disabled \
  --confirm-no-listen-key-lifecycle >/dev/null

run_signing_approval "$PRODUCTION_REQUEST_PREVIEW" "$MISSING_FLAGS_SIGNING_APPROVAL" >/dev/null

run_signing_approval "$PRODUCTION_REQUEST_PREVIEW" "$READY_SIGNING_APPROVAL" \
  --allow-production-mutation-signing-approval \
  --confirm-owner-approved-signing-material \
  --confirm-env-only-signing-material \
  --confirm-memory-only-signing \
  --confirm-no-secret-persistence \
  --confirm-no-network \
  --confirm-no-production-order-submission \
  --confirm-no-production-order-mutation \
  --confirm-dashboard-order-controls-disabled \
  --confirm-no-listen-key-lifecycle >/dev/null

"$NAUTILUS_BIN" live production-kill-switch-approval-artifact \
  --run-id v160-signing-approval-runtime \
  --session-id session-v160 \
  --strategy-id ema_cross_btcusdt_v1 \
  --output "$KILL_SWITCH_APPROVAL" \
  --kill-switch-active false \
  --approval-state approved \
  --manual-approval-id owner-approval-v160-003 \
  --approved-by owner \
  --confirm-dry-run-only \
  --confirm-no-production-mutation \
  --confirm-dashboard-order-controls-disabled >/dev/null

"$NAUTILUS_BIN" live production-live-alpha-kill-switch-runtime-gate \
  --run-id v160-signing-approval-runtime \
  --kill-switch-approval "$KILL_SWITCH_APPROVAL" \
  --risk-preflight "$RISK_PREFLIGHT" \
  --request-preview "$PRODUCTION_REQUEST_PREVIEW" \
  --output "$KILL_SWITCH_GATE" \
  --allow-production-live-alpha-kill-switch-runtime-gate \
  --confirm-owner-approved-runtime-gate \
  --confirm-no-production-order-submission \
  --confirm-no-production-order-mutation \
  --confirm-no-network \
  --confirm-no-listen-key-lifecycle \
  --confirm-dashboard-order-controls-disabled \
  --confirm-no-real-funds >/dev/null

"$NAUTILUS_BIN" live production-mutation-runtime-gate \
  --run-id v160-production-mutation-runtime-gate \
  --order-gate "$ORDER_GATE" \
  --risk-preflight "$RISK_PREFLIGHT" \
  --request-preview "$PRODUCTION_REQUEST_PREVIEW" \
  --kill-switch-runtime-gate "$KILL_SWITCH_GATE" \
  --signing-approval "$READY_SIGNING_APPROVAL" \
  --output "$RUNTIME_GATE_WITH_SIGNING" \
  --max-notional 10.00 \
  --allow-production-mutation-runtime-gate \
  --confirm-owner-approved-production-mutation \
  --confirm-single-limit-gtc \
  --confirm-tiny-notional \
  --confirm-signing-approval-required \
  --confirm-no-network-before-send \
  --confirm-dashboard-order-controls-disabled \
  --confirm-no-listen-key-lifecycle \
  --confirm-no-retry >/dev/null

python3 - \
  "$SYNTHETIC_SIGNING_APPROVAL" \
  "$MISSING_FLAGS_SIGNING_APPROVAL" \
  "$READY_SIGNING_APPROVAL" \
  "$RUNTIME_GATE_WITH_SIGNING" <<'PY'
import json
import sys
from pathlib import Path

synthetic = json.loads(Path(sys.argv[1]).read_text())
missing = json.loads(Path(sys.argv[2]).read_text())
ready = json.loads(Path(sys.argv[3]).read_text())
runtime = json.loads(Path(sys.argv[4]).read_text())

def assert_no_secret_or_mutation(name, artifact):
    for field in [
        "api_key_value_recorded",
        "api_secret_value_recorded",
        "signature_recorded",
        "signed_query_recorded",
        "signed_url_recorded",
        "request_body_recorded",
        "raw_request_body_recorded",
        "request_sent",
        "network_attempted",
        "dashboard_order_controls_enabled",
        "real_orders_submitted",
        "real_funds",
        "production_trading_enabled",
    ]:
        assert artifact.get(field, False) is False, (name, field, artifact.get(field), artifact)
    for field in [
        "production_order_submissions_attempted",
        "production_orders_submitted",
        "production_order_mutations_attempted",
        "production_order_state_reads_attempted",
        "listen_key_lifecycle_attempted",
    ]:
        assert artifact.get(field, 0) == 0, (name, field, artifact.get(field), artifact)

assert synthetic["schema_version"] == "ntpro.v160_production_mutation_signing_approval.v1"
assert synthetic["status"] == "blocked_request_preview", synthetic
assert synthetic["credential_material"] == "synthetic", synthetic
assert synthetic["signing_approval_ready"] is False, synthetic
assert "request_preview_not_production_live_alpha_material" in synthetic["source_artifact_issues"], synthetic
assert_no_secret_or_mutation("synthetic", synthetic)

assert missing["schema_version"] == "ntpro.v160_production_mutation_signing_approval.v1"
assert missing["status"] == "blocked_missing_gate", missing
assert missing["credential_material"] == "production_live_alpha", missing
assert missing["signing_approval_ready"] is False, missing
assert "--allow-production-mutation-signing-approval" in missing["missing_cli_flags"], missing
assert_no_secret_or_mutation("missing", missing)

assert ready["schema_version"] == "ntpro.v160_production_mutation_signing_approval.v1"
assert ready["status"] == "ready_signing_material_approval", ready
assert ready["credential_material"] == "production_live_alpha", ready
assert ready["owner_approved_signing_material"] is True, ready
assert ready["signing_approval_ready"] is True, ready
assert ready["production_signing_material_gate_required"] is True, ready
assert ready["production_signing_material_gate_open"] is True, ready
assert ready["production_signing_material_env_read"] is True, ready
assert ready["production_signing_material_missing_gate_env_vars"] == [], ready
assert ready["source_artifact_issues"] == [], ready
assert ready["missing_cli_flags"] == [], ready
assert_no_secret_or_mutation("ready", ready)

assert runtime["schema_version"] == "ntpro.v160_production_mutation_runtime_gate.v1", runtime
assert runtime["status"] == "blocked_explicit_send_gate", runtime
assert runtime["signing_approval_required"] is True, runtime
assert runtime["signing_approval_ready"] is True, runtime
assert runtime["signing_approval_status"] == "ready_signing_material_approval", runtime
assert runtime["explicit_send_gate_open"] is False, runtime
assert runtime["runtime_gate_open"] is False, runtime
assert runtime["send_consideration_allowed"] is False, runtime
assert "explicit_send_gate_closed" in runtime["runtime_gate_reasons"], runtime
assert_no_secret_or_mutation("runtime", runtime)
PY

if grep -R \
  -e "$SYNTHETIC_API_KEY" \
  -e "$SYNTHETIC_API_SECRET" \
  -e "$PRODUCTION_API_KEY" \
  -e "$PRODUCTION_API_SECRET" \
  -e "signature=" \
  "$OUTPUT_DIR" >/tmp/ntpro-v16-signing-approval-secret-leak.txt; then
  echo "v16 signing approval observed persisted secret or signature material:" >&2
  cat /tmp/ntpro-v16-signing-approval-secret-leak.txt >&2
  exit 1
fi

if grep -RE \
  "request_sent=true|network_attempted=true|production_orders_submitted=[1-9][0-9]*|production_order_mutations_attempted=[1-9][0-9]*|dashboard_order_controls_enabled=true|production_adapter_called=true|production_adapter_instantiated=true|real_orders_submitted=true|real_funds=true|production_trading_enabled=true|signature_recorded=true|signed_query_recorded=true|signed_url_recorded=true|api_key_value_recorded=true|api_secret_value_recorded=true" \
  "$OUTPUT_DIR" >/tmp/ntpro-v16-signing-approval-forbidden.txt; then
  echo "v16 signing approval observed forbidden production mutation evidence:" >&2
  cat /tmp/ntpro-v16-signing-approval-forbidden.txt >&2
  exit 1
fi

echo "v16_signing_material_approval status=ok root=$GATE_ROOT signing_approval_ready=true runtime_status=blocked_explicit_send_gate request_sent=false network_attempted=false production_orders_submitted=0 production_order_mutations_attempted=0 dashboard_order_controls_enabled=false"
