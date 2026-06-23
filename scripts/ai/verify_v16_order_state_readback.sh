#!/usr/bin/env bash
set -euo pipefail

# V160-007: v0.16 post-submit order-state readback proof.
# Default validation is local/offline. It proves GET /api/v3/order readback is
# derived only from known order identifiers and does not mutate, retry, infer
# strategy success, persist secrets, or open network unless manual-online gates
# are explicitly opened.

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

GATE_ROOT="${NTPRO_V16_ORDER_STATE_READBACK_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v16-order-state-readback.XXXXXX")}"
RESPONSE_REDACTION_ROOT="$GATE_ROOT/response-redaction"
OUTPUT_DIR="$GATE_ROOT/command-output"
mkdir -p "$OUTPUT_DIR"

NTPRO_V16_SKIP_BUILD=1 \
NTPRO_V16_NAUTILUS_BIN="$NAUTILUS_BIN" \
NTPRO_V16_RESPONSE_REDACTION_ROOT="$RESPONSE_REDACTION_ROOT" \
  scripts/ai/verify_v16_response_redaction.sh >/dev/null

RESPONSE_REDACTION="$RESPONSE_REDACTION_ROOT/command-output/ready-response-redaction.json"
MISSING_FLAGS_READBACK="$OUTPUT_DIR/missing-flags-order-state-readback.json"
READY_OFFLINE_READBACK="$OUTPUT_DIR/ready-offline-order-state-readback.json"
MANUAL_MISSING_ENV_READBACK="$OUTPUT_DIR/manual-missing-env-order-state-readback.json"

if [[ ! -f "$RESPONSE_REDACTION" ]]; then
  echo "response-redaction setup did not produce expected input" >&2
  exit 1
fi

run_readback() {
  local output="$1"
  shift
  "$NAUTILUS_BIN" live production-mutation-order-state-readback \
    --run-id v160-production-mutation-order-state-readback \
    --response-redaction "$RESPONSE_REDACTION" \
    --output "$output" \
    --api-key-env NTPRO_V160007_API_KEY \
    --api-secret-env NTPRO_V160007_API_SECRET \
    --recv-window-ms 5000 \
    "$@"
}

run_readback "$MISSING_FLAGS_READBACK" >/dev/null

run_readback "$READY_OFFLINE_READBACK" \
  --allow-production-mutation-order-state-readback \
  --confirm-owner-approved-order-state-readback \
  --confirm-known-order-identifier-only \
  --confirm-read-only-get-order \
  --confirm-response-redacted \
  --confirm-no-production-order-mutation \
  --confirm-no-secret-persistence \
  --confirm-no-retry \
  --confirm-dashboard-order-controls-disabled \
  --confirm-no-listen-key-lifecycle >/dev/null

run_readback "$MANUAL_MISSING_ENV_READBACK" \
  --manual-online \
  --allow-production-mutation-order-state-readback \
  --confirm-owner-approved-order-state-readback \
  --confirm-known-order-identifier-only \
  --confirm-read-only-get-order \
  --confirm-response-redacted \
  --confirm-no-production-order-mutation \
  --confirm-no-secret-persistence \
  --confirm-no-retry \
  --confirm-dashboard-order-controls-disabled \
  --confirm-no-listen-key-lifecycle >/dev/null

python3 - "$MISSING_FLAGS_READBACK" "$READY_OFFLINE_READBACK" "$MANUAL_MISSING_ENV_READBACK" <<'PY'
import json
import sys
from pathlib import Path

missing_flags = json.loads(Path(sys.argv[1]).read_text())
ready = json.loads(Path(sys.argv[2]).read_text())
manual_missing_env = json.loads(Path(sys.argv[3]).read_text())

assert missing_flags["schema_version"] == "ntpro.v160_production_mutation_order_state_readback.v1"
assert missing_flags["status"] == "blocked_missing_gate"
assert missing_flags["readback_contract_ready"] is False
assert missing_flags["order_state_read_attempted"] is False
assert missing_flags["network_attempted"] is False
assert missing_flags["production_order_state_reads_attempted"] == 0
assert "--allow-production-mutation-order-state-readback" in missing_flags["missing_cli_flags"]
assert "--confirm-known-order-identifier-only" in missing_flags["missing_cli_flags"]

assert ready["schema_version"] == "ntpro.v160_production_mutation_order_state_readback.v1"
assert ready["status"] == "ready_offline_order_state_readback_contract"
assert ready["readback_contract_ready"] is True
assert ready["source_response_redaction_status"] == "ready_response_redacted"
assert ready["known_order_identifier_source"] == "production_mutation_response_redaction"
assert ready["known_order_id"] == "123456789"
assert ready["known_client_order_id"] == "owner-approved-v160-single-shot"
assert ready["symbol"] == "BTCUSDT"
assert ready["endpoint"] == "order"
assert ready["method"] == "GET"
assert ready["path"] == "/api/v3/order"
assert ready["manual_online_requested"] is False
assert ready["order_state_read_allowed"] is False
assert ready["order_state_read_attempted"] is False
assert ready["response_shape"] == "binance_order_state_v1"
assert ready["response_shape_validated"] is False
assert ready["strategy_success_inferred"] is False
assert ready["strategy_success_proof"] == "not_inferred_readback_is_observability_only"
assert ready["source_artifact_issues"] == []
assert ready["missing_cli_flags"] == []
assert ready["missing_env_vars"] == []
for field in [
    "api_key_value_recorded",
    "api_secret_value_recorded",
    "api_key_header_value_recorded",
    "signature_recorded",
    "signed_query_recorded",
    "signed_url_recorded",
    "raw_exchange_response_recorded",
    "response_body_recorded",
    "response_headers_recorded",
    "network_attempted",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "production_order_state_reads_allowed",
    "listen_key_lifecycle_allowed",
    "retry_attempted",
    "cancel_attempted",
    "replace_attempted",
    "amend_attempted",
    "flatten_attempted",
    "dashboard_order_controls_enabled",
    "real_orders_submitted",
    "real_funds",
    "production_trading_enabled",
]:
    assert ready[field] is False, field
assert ready["response_redacted"] is True
assert ready["production_order_submissions_attempted"] == 0
assert ready["production_orders_submitted"] == 0
assert ready["production_order_mutations_attempted"] == 0
assert ready["production_order_state_reads_attempted"] == 0
assert ready["listen_key_lifecycle_attempted"] == 0

assert manual_missing_env["status"] == "blocked_missing_manual_online_gate"
assert manual_missing_env["manual_online_requested"] is True
assert manual_missing_env["readback_contract_ready"] is False
assert manual_missing_env["order_state_read_allowed"] is False
assert manual_missing_env["order_state_read_attempted"] is False
assert manual_missing_env["network_attempted"] is False
assert manual_missing_env["production_order_state_reads_attempted"] == 0
assert "NTPRO_ALLOW_PRODUCTION_ORDER_STATE_READ" in manual_missing_env["missing_env_vars"]
assert "NTPRO_OWNER_APPROVED_PRODUCTION_ORDER_STATE_READ_ONLY" in manual_missing_env["missing_env_vars"]
PY

if grep -R "ntpro_v160007_api_key_value\\|ntpro_v160007_api_secret_value\\|X-MBX-APIKEY" "$OUTPUT_DIR" >/dev/null; then
  echo "order-state readback artifacts persisted secret material" >&2
  exit 1
fi

echo "v16_order_state_readback status=ok root=$GATE_ROOT readback_contract_ready=true order_state_read_attempted=false network_attempted=false production_order_state_reads_attempted=0 production_order_mutations_attempted=0 strategy_success_inferred=false"
