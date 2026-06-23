#!/usr/bin/env bash
set -euo pipefail

# V160-004: v0.16 single LIMIT GTC production order request builder.
# This verifier is local/offline only. It constructs a redacted request-object
# artifact and proves no signed query, signature, signed URL, raw body, network
# call, production order, or Dashboard order control is persisted or executed.

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

GATE_ROOT="${NTPRO_V16_REQUEST_BUILDER_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v16-request-builder.XXXXXX")}"
OUTPUT_DIR="$GATE_ROOT/command-output"
mkdir -p "$OUTPUT_DIR"

ORDER_GATE="$OUTPUT_DIR/live-alpha-order-gate.json"
RISK_INPUT="$OUTPUT_DIR/live-alpha-risk-input.json"
RISK_PREFLIGHT="$OUTPUT_DIR/live-alpha-risk-preflight.json"
MANUAL_APPROVAL="$OUTPUT_DIR/manual-approval-lifecycle.json"
REQUEST_PREVIEW="$OUTPUT_DIR/production-request-preview.json"
MARKET_REQUEST_PREVIEW="$OUTPUT_DIR/market-request-preview.json"
SIGNING_APPROVAL="$OUTPUT_DIR/signing-approval.json"
KILL_SWITCH_APPROVAL="$OUTPUT_DIR/kill-switch-approval.json"
KILL_SWITCH_GATE="$OUTPUT_DIR/kill-switch-runtime-gate.json"
RUNTIME_GATE="$OUTPUT_DIR/runtime-gate.json"
MISSING_FLAGS_BUILDER="$OUTPUT_DIR/missing-flags-request-builder.json"
READY_BUILDER="$OUTPUT_DIR/ready-request-builder.json"
MARKET_BUILDER="$OUTPUT_DIR/market-request-builder.json"

PRODUCTION_API_KEY="ntpro_v160004_production_like_api_key_value"
PRODUCTION_API_SECRET="ntpro_v160004_production_like_api_secret_value"

"$NAUTILUS_BIN" live production-live-alpha-dry-run-order-gate \
  --run-id v160-request-builder \
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
  --run-id v160-request-builder-risk \
  --order-gate "$ORDER_GATE" \
  --input "$RISK_INPUT" \
  --output "$RISK_PREFLIGHT" \
  --confirm-hypothetical-dry-run-only \
  --confirm-no-execution-adapter-call \
  --confirm-no-production-order-submission \
  --confirm-no-production-order-mutation \
  --confirm-dashboard-order-controls-disabled >/dev/null

"$NAUTILUS_BIN" live production-live-alpha-manual-approval-lifecycle \
  --run-id v150-live-alpha-request-preview \
  --strategy-id ema_cross_btcusdt_v1 \
  --symbol BTCUSDT \
  --notional 10.00 \
  --approval-state approved \
  --manual-approval-id owner-approval-v160-004 \
  --approved-by owner \
  --now-unix-ms 1718400000000 \
  --expires-at-unix-ms 1718400060000 \
  --output "$MANUAL_APPROVAL" \
  --confirm-dry-run-request-preview-only \
  --confirm-one-time-approval \
  --confirm-no-production-mutation \
  --confirm-dashboard-order-controls-disabled >/dev/null

NTPRO_ALLOW_PRODUCTION_MUTATION_SIGNING_MATERIAL=1 \
NTPRO_OWNER_APPROVED_MUTATION_SIGNING_DRY_RUN=1 \
NTPRO_V160004_API_KEY="$PRODUCTION_API_KEY" \
NTPRO_V160004_API_SECRET="$PRODUCTION_API_SECRET" \
  "$NAUTILUS_BIN" live production-live-alpha-order-request-preview \
    --run-id v150-live-alpha-request-preview \
    --order-gate "$ORDER_GATE" \
    --manual-approval-lifecycle "$MANUAL_APPROVAL" \
    --endpoint-path /api/v3/order \
    --price 10000.00 \
    --time-in-force GTC \
    --timestamp-ms 1718400000000 \
    --recv-window-ms 5000 \
    --api-key-env NTPRO_V160004_API_KEY \
    --api-secret-env NTPRO_V160004_API_SECRET \
    --credential-material production_live_alpha \
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

"$NAUTILUS_BIN" live production-mutation-signing-approval \
  --run-id v160-production-mutation-signing-approval \
  --request-preview "$REQUEST_PREVIEW" \
  --approval-state approved \
  --manual-approval-id owner-approval-v160-004 \
  --approved-by owner \
  --now-unix-ms 1718400000000 \
  --expires-at-unix-ms 1718400060000 \
  --output "$SIGNING_APPROVAL" \
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
  --run-id v160-request-builder-runtime \
  --session-id session-v160 \
  --strategy-id ema_cross_btcusdt_v1 \
  --output "$KILL_SWITCH_APPROVAL" \
  --kill-switch-active false \
  --approval-state approved \
  --manual-approval-id owner-approval-v160-004 \
  --approved-by owner \
  --confirm-dry-run-only \
  --confirm-no-production-mutation \
  --confirm-dashboard-order-controls-disabled >/dev/null

"$NAUTILUS_BIN" live production-live-alpha-kill-switch-runtime-gate \
  --run-id v160-request-builder-runtime \
  --kill-switch-approval "$KILL_SWITCH_APPROVAL" \
  --risk-preflight "$RISK_PREFLIGHT" \
  --request-preview "$REQUEST_PREVIEW" \
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
  --request-preview "$REQUEST_PREVIEW" \
  --kill-switch-runtime-gate "$KILL_SWITCH_GATE" \
  --signing-approval "$SIGNING_APPROVAL" \
  --output "$RUNTIME_GATE" \
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

run_request_builder() {
  local request_preview="$1"
  local output="$2"
  shift 2
  "$NAUTILUS_BIN" live production-mutation-request-builder \
    --run-id v160-production-mutation-request-builder \
    --runtime-gate "$RUNTIME_GATE" \
    --signing-approval "$SIGNING_APPROVAL" \
    --request-preview "$request_preview" \
    --api-key-env NTPRO_V160004_API_KEY \
    --api-secret-env NTPRO_V160004_API_SECRET \
    --timestamp-ms 1718400000000 \
    --recv-window-ms 5000 \
    --max-notional 10.00 \
    --output "$output" \
    "$@"
}

run_request_builder "$REQUEST_PREVIEW" "$MISSING_FLAGS_BUILDER" >/dev/null

NTPRO_ALLOW_PRODUCTION_MUTATION_SIGNING_MATERIAL=1 \
NTPRO_OWNER_APPROVED_MUTATION_SIGNING_DRY_RUN=1 \
NTPRO_V160004_API_KEY="$PRODUCTION_API_KEY" \
NTPRO_V160004_API_SECRET="$PRODUCTION_API_SECRET" \
  run_request_builder "$REQUEST_PREVIEW" "$READY_BUILDER" \
    --allow-production-mutation-request-builder \
    --confirm-owner-approved-request-builder \
    --confirm-single-limit-gtc \
    --confirm-tiny-notional \
    --confirm-signing-approval-ready \
    --confirm-memory-only-signing \
    --confirm-no-secret-persistence \
    --confirm-no-network \
    --confirm-no-production-order-submission \
    --confirm-no-production-order-mutation \
    --confirm-dashboard-order-controls-disabled \
    --confirm-no-listen-key-lifecycle \
    --confirm-no-retry >/dev/null

python3 - "$REQUEST_PREVIEW" "$MARKET_REQUEST_PREVIEW" <<'PY'
import json
import sys
from pathlib import Path

source = Path(sys.argv[1])
target = Path(sys.argv[2])
artifact = json.loads(source.read_text())
artifact["order_type"] = "MARKET"
target.write_text(json.dumps(artifact, indent=2) + "\n")
PY

NTPRO_ALLOW_PRODUCTION_MUTATION_SIGNING_MATERIAL=1 \
NTPRO_OWNER_APPROVED_MUTATION_SIGNING_DRY_RUN=1 \
NTPRO_V160004_API_KEY="$PRODUCTION_API_KEY" \
NTPRO_V160004_API_SECRET="$PRODUCTION_API_SECRET" \
  run_request_builder "$MARKET_REQUEST_PREVIEW" "$MARKET_BUILDER" \
    --allow-production-mutation-request-builder \
    --confirm-owner-approved-request-builder \
    --confirm-single-limit-gtc \
    --confirm-tiny-notional \
    --confirm-signing-approval-ready \
    --confirm-memory-only-signing \
    --confirm-no-secret-persistence \
    --confirm-no-network \
    --confirm-no-production-order-submission \
    --confirm-no-production-order-mutation \
    --confirm-dashboard-order-controls-disabled \
    --confirm-no-listen-key-lifecycle \
    --confirm-no-retry >/dev/null

python3 - "$MISSING_FLAGS_BUILDER" "$READY_BUILDER" "$MARKET_BUILDER" <<'PY'
import json
import sys
from pathlib import Path

missing = json.loads(Path(sys.argv[1]).read_text())
ready = json.loads(Path(sys.argv[2]).read_text())
market = json.loads(Path(sys.argv[3]).read_text())

def assert_no_send(name, artifact):
    for field in [
        "api_key_value_recorded",
        "api_secret_value_recorded",
        "api_key_header_value_recorded",
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
        "execution_adapter_called",
        "production_adapter_called",
        "production_adapter_instantiated",
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

assert missing["schema_version"] == "ntpro.v160_production_mutation_request_builder.v1"
assert missing["status"] == "blocked_missing_gate", missing
assert missing["request_builder_ready"] is False, missing
assert missing["request_object_built"] is False, missing
assert "--allow-production-mutation-request-builder" in missing["missing_cli_flags"], missing
assert "NTPRO_ALLOW_PRODUCTION_MUTATION_SIGNING_MATERIAL" in missing["missing_env_vars"], missing
assert_no_send("missing", missing)

assert ready["schema_version"] == "ntpro.v160_production_mutation_request_builder.v1"
assert ready["status"] == "ready_request_object_built_no_send", ready
assert ready["request_builder_ready"] is True, ready
assert ready["request_object_built"] is True, ready
assert ready["runtime_gate_status"] == "blocked_explicit_send_gate", ready
assert ready["runtime_gate_open"] is False, ready
assert ready["send_consideration_allowed"] is False, ready
assert ready["signing_approval_ready"] is True, ready
assert ready["explicit_send_gate_open"] is False, ready
assert ready["request_method"] == "POST", ready
assert ready["request_target"] == "/api/v3/order", ready
assert ready["order_type"] == "LIMIT", ready
assert ready["time_in_force"] == "GTC", ready
assert ready["single_order_candidate"] is True, ready
assert ready["tiny_notional_gate_ready"] is True, ready
assert ready["source_artifact_issues"] == [], ready
assert ready["missing_cli_flags"] == [], ready
assert ready["missing_env_vars"] == [], ready
assert_no_send("ready", ready)

assert market["status"] == "blocked_source_artifact", market
assert market["request_object_built"] is False, market
assert market["order_type"] == "MARKET", market
assert "request_preview_not_limit" in market["source_artifact_issues"], market
assert_no_send("market", market)
PY

if grep -R \
  -e "$PRODUCTION_API_KEY" \
  -e "$PRODUCTION_API_SECRET" \
  "$OUTPUT_DIR" >/tmp/ntpro-v16-request-builder-secret-leak.txt; then
  echo "v16 request builder observed persisted secret material:" >&2
  cat /tmp/ntpro-v16-request-builder-secret-leak.txt >&2
  exit 1
fi

if grep -RE \
  "request_sent=true|network_attempted=true|production_orders_submitted=[1-9][0-9]*|production_order_mutations_attempted=[1-9][0-9]*|dashboard_order_controls_enabled=true|production_adapter_called=true|production_adapter_instantiated=true|real_orders_submitted=true|real_funds=true|production_trading_enabled=true|signature_recorded=true|signed_query_recorded=true|signed_url_recorded=true|api_key_value_recorded=true|api_secret_value_recorded=true" \
  "$OUTPUT_DIR" >/tmp/ntpro-v16-request-builder-forbidden.txt; then
  echo "v16 request builder observed forbidden production mutation evidence:" >&2
  cat /tmp/ntpro-v16-request-builder-forbidden.txt >&2
  exit 1
fi

echo "v16_request_builder status=ok root=$GATE_ROOT request_builder_ready=true request_object_built=true request_sent=false network_attempted=false production_orders_submitted=0 production_order_mutations_attempted=0 dashboard_order_controls_enabled=false"
