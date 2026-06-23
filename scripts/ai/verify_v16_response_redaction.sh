#!/usr/bin/env bash
set -euo pipefail

# V160-006: v0.16 production mutation response redaction contract.
# This verifier is local/offline. It feeds synthetic order responses into the
# redaction artifact builder and proves raw responses, headers, secrets,
# balances, fills, and unrestricted payloads are not persisted.

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

GATE_ROOT="${NTPRO_V16_RESPONSE_REDACTION_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v16-response-redaction.XXXXXX")}"
GUARDED_SEND_ROOT="$GATE_ROOT/guarded-send"
OUTPUT_DIR="$GATE_ROOT/command-output"
mkdir -p "$OUTPUT_DIR"

NTPRO_V16_SKIP_BUILD=1 \
NTPRO_V16_NAUTILUS_BIN="$NAUTILUS_BIN" \
NTPRO_V16_GUARDED_SEND_ROOT="$GUARDED_SEND_ROOT" \
  scripts/ai/verify_v16_guarded_send_path.sh >/dev/null

GUARDED_SEND="$GUARDED_SEND_ROOT/command-output/ready-offline-guarded-send.json"
ACTUAL_GUARDED_SEND="$OUTPUT_DIR/actual-guarded-send-http-result.json"
READY_RESPONSE="$OUTPUT_DIR/synthetic-order-response.json"
FORBIDDEN_RESPONSE="$OUTPUT_DIR/forbidden-order-response.json"
MISSING_FLAGS_REDACTION="$OUTPUT_DIR/missing-flags-response-redaction.json"
READY_REDACTION="$OUTPUT_DIR/ready-response-redaction.json"
ACTUAL_REDACTION="$OUTPUT_DIR/actual-response-redaction.json"
FORBIDDEN_REDACTION="$OUTPUT_DIR/forbidden-response-redaction.json"

if [[ ! -f "$GUARDED_SEND" ]]; then
  echo "guarded-send setup did not produce expected input" >&2
  exit 1
fi

python3 - "$READY_RESPONSE" "$FORBIDDEN_RESPONSE" <<'PY'
import json
import sys
from pathlib import Path

Path(sys.argv[1]).write_text(json.dumps({
    "symbol": "BTCUSDT",
    "orderId": 123456789,
    "clientOrderId": "owner-approved-v160-single-shot",
    "transactTime": 1718400000000,
    "workingTime": 1718400000001,
    "status": "NEW",
    "type": "LIMIT",
    "side": "BUY",
    "timeInForce": "GTC",
}, indent=2) + "\n")

Path(sys.argv[2]).write_text(json.dumps({
    "symbol": "BTCUSDT",
    "orderId": 123456789,
    "clientOrderId": "owner-approved-v160-single-shot",
    "status": "NEW",
    "type": "LIMIT",
    "side": "BUY",
    "headers": {"X-MBX-APIKEY": "must_not_persist"},
    "signature": "signature=must_not_persist",
    "balances": [{"asset": "USDT", "free": "100.0"}],
    "payload": {"raw": "unrestricted"},
}, indent=2) + "\n")
PY

python3 - "$GUARDED_SEND" "$ACTUAL_GUARDED_SEND" <<'PY'
import json
import sys
from pathlib import Path

source = Path(sys.argv[1])
target = Path(sys.argv[2])
artifact = json.loads(source.read_text())
artifact.update({
    "status": "manual_online_send_attempt_recorded",
    "manual_online_requested": True,
    "request_sent": True,
    "network_attempted": True,
    "production_order_request_attempted": True,
    "http_send_attempted": True,
    "exchange_ack_observed": True,
    "confirmed_production_order_submission": True,
    "production_order_submissions_attempted": 1,
    "production_orders_submitted": 1,
    "production_order_mutations_attempted": 1,
    "real_orders_submitted": True,
    "production_trading_enabled": False,
})
target.write_text(json.dumps(artifact, indent=2) + "\n")
PY

run_response_redaction() {
  local guarded_send="$1"
  local response="$2"
  local output="$3"
  shift 3
  "$NAUTILUS_BIN" live production-mutation-response-redaction \
    --run-id v160-production-mutation-response-redaction \
    --guarded-send "$guarded_send" \
    --response "$response" \
    --output "$output" \
    "$@"
}

run_response_redaction "$GUARDED_SEND" "$READY_RESPONSE" "$MISSING_FLAGS_REDACTION" >/dev/null

run_response_redaction "$GUARDED_SEND" "$READY_RESPONSE" "$READY_REDACTION" \
  --allow-production-mutation-response-redaction \
  --confirm-owner-approved-response-redaction \
  --confirm-no-raw-response-persistence \
  --confirm-no-headers-persistence \
  --confirm-no-secret-persistence \
  --confirm-order-metadata-only \
  --confirm-no-account-balances \
  --confirm-no-unrestricted-payload \
  --confirm-no-retry >/dev/null

run_response_redaction "$ACTUAL_GUARDED_SEND" "$READY_RESPONSE" "$ACTUAL_REDACTION" \
  --allow-production-mutation-response-redaction \
  --confirm-owner-approved-response-redaction \
  --confirm-no-raw-response-persistence \
  --confirm-no-headers-persistence \
  --confirm-no-secret-persistence \
  --confirm-order-metadata-only \
  --confirm-no-account-balances \
  --confirm-no-unrestricted-payload \
  --confirm-no-retry >/dev/null

run_response_redaction "$GUARDED_SEND" "$FORBIDDEN_RESPONSE" "$FORBIDDEN_REDACTION" \
  --allow-production-mutation-response-redaction \
  --confirm-owner-approved-response-redaction \
  --confirm-no-raw-response-persistence \
  --confirm-no-headers-persistence \
  --confirm-no-secret-persistence \
  --confirm-order-metadata-only \
  --confirm-no-account-balances \
  --confirm-no-unrestricted-payload \
  --confirm-no-retry >/dev/null

python3 - "$MISSING_FLAGS_REDACTION" "$READY_REDACTION" "$ACTUAL_REDACTION" "$FORBIDDEN_REDACTION" <<'PY'
import json
import sys
from pathlib import Path

missing_flags = json.loads(Path(sys.argv[1]).read_text())
ready = json.loads(Path(sys.argv[2]).read_text())
actual = json.loads(Path(sys.argv[3]).read_text())
forbidden = json.loads(Path(sys.argv[4]).read_text())

assert missing_flags["schema_version"] == "ntpro.v160_production_mutation_response_redaction.v1"
assert missing_flags["status"] == "blocked_missing_gate"
assert missing_flags["response_redaction_ready"] is False
assert "--allow-production-mutation-response-redaction" in missing_flags["missing_cli_flags"]
assert "--confirm-no-raw-response-persistence" in missing_flags["missing_cli_flags"]
assert missing_flags["raw_exchange_response_recorded"] is False
assert missing_flags["response_headers_recorded"] is False
assert missing_flags["unrestricted_payload_recorded"] is False
assert missing_flags["account_balances_recorded"] is False

assert ready["schema_version"] == "ntpro.v160_production_mutation_response_redaction.v1"
assert ready["status"] == "ready_response_redacted"
assert ready["response_redaction_ready"] is True
assert ready["response_redaction_source"] == "synthetic_fixture"
assert ready["source_guarded_send_run_id"] == "v160-production-mutation-guarded-send"
assert ready["source_guarded_send_hash"].startswith("fnv1a64:")
assert ready["redacted_response_derived_from_actual_http_result"] is False
assert ready["synthetic_fixture_redaction_only"] is True
assert ready["owner_run_mutation_closure_evidence"] is False
assert ready["source_guarded_send_status"] == "ready_guarded_send_path_offline_no_network"
assert ready["response_shape_validated"] is True
assert ready["response_type"] == "binance_order_response_redacted_metadata_v1"
assert ready["symbol"] == "BTCUSDT"
assert ready["side"] == "BUY"
assert ready["order_type"] == "LIMIT"
assert ready["time_in_force"] == "GTC"
assert ready["order_id"] == "123456789"
assert ready["client_order_id"] == "owner-approved-v160-single-shot"
assert ready["exchange_status"] == "NEW"
assert ready["transact_time_shape"] == "epoch_millis_present_redacted"
assert ready["working_time_shape"] == "epoch_millis_present_redacted"
assert ready["forbidden_response_markers"] == []
assert ready["source_artifact_issues"] == []
assert ready["missing_cli_flags"] == []
for field in [
    "api_key_value_recorded",
    "api_secret_value_recorded",
    "api_key_header_value_recorded",
    "signature_recorded",
    "signed_query_recorded",
    "signed_url_recorded",
    "request_body_recorded",
    "raw_request_body_recorded",
    "raw_exchange_response_recorded",
    "response_body_recorded",
    "response_headers_recorded",
    "unrestricted_payload_recorded",
    "account_balances_recorded",
    "fills_recorded",
    "retry_attempted",
    "cancel_attempted",
    "replace_attempted",
    "amend_attempted",
    "flatten_attempted",
    "dashboard_order_controls_enabled",
    "real_orders_submitted",
    "real_funds",
]:
    assert ready[field] is False, field
assert ready["response_redacted"] is True
assert ready["request_sent"] is False
assert ready["network_attempted"] is False
assert ready["production_orders_submitted"] == 0
assert ready["production_order_mutations_attempted"] == 0

assert actual["schema_version"] == "ntpro.v160_production_mutation_response_redaction.v1"
assert actual["status"] == "ready_response_redacted"
assert actual["response_redaction_ready"] is True
assert actual["response_redaction_source"] == "actual_guarded_send_http_result"
assert actual["source_guarded_send_status"] == "manual_online_send_attempt_recorded"
assert actual["source_guarded_send_hash"].startswith("fnv1a64:")
assert actual["redacted_response_derived_from_actual_http_result"] is True
assert actual["synthetic_fixture_redaction_only"] is False
assert actual["owner_run_mutation_closure_evidence"] is True
assert actual["request_sent"] is True
assert actual["network_attempted"] is True
assert actual["production_orders_submitted"] == 1
assert actual["production_order_mutations_attempted"] == 1
assert actual["raw_exchange_response_recorded"] is False
assert actual["response_headers_recorded"] is False
assert actual["unrestricted_payload_recorded"] is False
assert actual["account_balances_recorded"] is False

assert forbidden["status"] == "blocked_forbidden_response_marker"
assert forbidden["response_redaction_ready"] is False
assert forbidden["response_shape_validated"] is False
assert any("$.headers" in marker for marker in forbidden["forbidden_response_markers"])
assert any("$.signature" in marker for marker in forbidden["forbidden_response_markers"])
assert any("$.balances" in marker for marker in forbidden["forbidden_response_markers"])
assert any("$.payload" in marker for marker in forbidden["forbidden_response_markers"])
assert forbidden["raw_exchange_response_recorded"] is False
assert forbidden["response_headers_recorded"] is False
assert forbidden["unrestricted_payload_recorded"] is False
assert forbidden["account_balances_recorded"] is False
PY

if grep -R "X-MBX-APIKEY\\|signature=must_not_persist\\|must_not_persist\\|\\\"headers\\\"\\|\\\"payload\\\"\\|\\\"balances\\\"" "$READY_REDACTION" "$MISSING_FLAGS_REDACTION" >/dev/null; then
  echo "response redaction artifact persisted forbidden response material" >&2
  exit 1
fi

echo "v16_response_redaction status=ok root=$GATE_ROOT ready_response_redacted=true response_redaction_source=synthetic_fixture actual_http_result_source=ok owner_run_mutation_closure_evidence=true raw_exchange_response_recorded=false response_headers_recorded=false unrestricted_payload_recorded=false account_balances_recorded=false"
