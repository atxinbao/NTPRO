#!/usr/bin/env bash
set -euo pipefail

# V160-009: v0.16 production mutation audit trail artifact.
# This verifier stays local/offline. It chains request-builder, guarded-send,
# response-redaction, and order-state readback evidence into one redacted audit
# trail and proves the artifact does not persist secrets, signatures, signed
# URLs, raw payloads, retries, follow-up mutations, or Dashboard order controls.

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

GATE_ROOT="${NTPRO_V16_AUDIT_TRAIL_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v16-audit-trail.XXXXXX")}"
ORDER_STATE_READBACK_ROOT="$GATE_ROOT/order-state-readback"
OUTPUT_DIR="$GATE_ROOT/command-output"
mkdir -p "$OUTPUT_DIR"

NTPRO_V16_SKIP_BUILD=1 \
NTPRO_V16_NAUTILUS_BIN="$NAUTILUS_BIN" \
NTPRO_V16_ORDER_STATE_READBACK_ROOT="$ORDER_STATE_READBACK_ROOT" \
  scripts/ai/verify_v16_order_state_readback.sh >/dev/null

REQUEST_BUILDER="$ORDER_STATE_READBACK_ROOT/response-redaction/guarded-send/request-builder/command-output/ready-request-builder.json"
GUARDED_SEND="$ORDER_STATE_READBACK_ROOT/response-redaction/guarded-send/command-output/ready-offline-guarded-send.json"
RESPONSE_REDACTION="$ORDER_STATE_READBACK_ROOT/response-redaction/command-output/ready-response-redaction.json"
ORDER_STATE_READBACK="$ORDER_STATE_READBACK_ROOT/command-output/ready-offline-order-state-readback.json"
MISSING_FLAGS_AUDIT="$OUTPUT_DIR/missing-flags-audit-trail.json"
READY_AUDIT="$OUTPUT_DIR/ready-redacted-audit-trail.json"

for input in "$REQUEST_BUILDER" "$GUARDED_SEND" "$RESPONSE_REDACTION" "$ORDER_STATE_READBACK"; do
  if [[ ! -f "$input" ]]; then
    echo "audit trail setup did not produce expected input: $input" >&2
    exit 1
  fi
done

run_audit_trail() {
  local output="$1"
  shift
  "$NAUTILUS_BIN" live production-mutation-audit-trail \
    --run-id v160-production-mutation-audit-trail \
    --request-builder "$REQUEST_BUILDER" \
    --guarded-send "$GUARDED_SEND" \
    --response-redaction "$RESPONSE_REDACTION" \
    --order-state-readback "$ORDER_STATE_READBACK" \
    --output "$output" \
    "$@"
}

run_audit_trail "$MISSING_FLAGS_AUDIT" >/dev/null

run_audit_trail "$READY_AUDIT" \
  --allow-production-mutation-audit-trail \
  --confirm-owner-approved-audit-trail \
  --confirm-redacted-artifacts-only \
  --confirm-no-secret-or-raw-payload-persistence \
  --confirm-no-retry-or-followup-mutation \
  --confirm-dashboard-order-controls-disabled \
  --confirm-no-listen-key-lifecycle >/dev/null

python3 - "$MISSING_FLAGS_AUDIT" "$READY_AUDIT" <<'PY'
import json
import sys
from pathlib import Path

missing_flags = json.loads(Path(sys.argv[1]).read_text())
ready = json.loads(Path(sys.argv[2]).read_text())

assert missing_flags["schema_version"] == "ntpro.v160_production_mutation_audit_trail.v1"
assert missing_flags["status"] == "blocked_missing_gate"
assert missing_flags["audit_trail_ready"] is False
assert missing_flags["request_sent"] is False
assert missing_flags["network_attempted"] is False
assert missing_flags["failure_state"] == "blocked_missing_gate"
assert "--allow-production-mutation-audit-trail" in missing_flags["missing_cli_flags"]
assert "--confirm-redacted-artifacts-only" in missing_flags["missing_cli_flags"]

assert ready["schema_version"] == "ntpro.v160_production_mutation_audit_trail.v1"
assert ready["status"] == "ready_redacted_audit_trail"
assert ready["audit_trail_ready"] is True
assert ready["preview_hash"].startswith("fnv1a64:")
assert ready["signing_approval_status"] == "ready_signing_material_approval"
assert ready["approval_state"] == "approved"
assert ready["manual_approval_recorded"] is True
assert ready["manual_approval_id"].startswith("owner-approval-v160-")
assert ready["approved_by"] == "owner"
assert ready["runtime_gate_status"] == "blocked_explicit_send_gate"
assert ready["runtime_gate_open"] is False
assert ready["send_consideration_allowed"] is False
assert ready["guarded_send_status"] == "ready_guarded_send_path_offline_no_network"
assert ready["request_sent"] is False
assert ready["network_attempted"] is False
assert ready["response_redaction_status"] == "ready_response_redacted"
assert ready["response_redaction_ready"] is True
assert ready["order_state_readback_status"] == "ready_offline_order_state_readback_contract"
assert ready["readback_contract_ready"] is True
assert ready["order_state_read_attempted"] is False
assert ready["kill_switch_checked_before_send"] is True
assert ready["kill_switch_checked_after_send"] is True
assert ready["pre_send_kill_switch_runtime_gate_open"] is True
assert ready["pre_send_kill_switch_active"] is False
assert ready["post_send_kill_switch_runtime_gate_open"] is True
assert ready["post_send_kill_switch_active"] is False
assert ready["kill_switch_blocked_send"] is False
assert ready["symbol"] == "BTCUSDT"
assert ready["side"] == "BUY"
assert ready["order_type"] == "LIMIT"
assert ready["time_in_force"] == "GTC"
assert ready["order_id"] == "123456789"
assert ready["client_order_id"] == "owner-approved-v160-single-shot"
assert ready["exchange_status"] == "NEW"
assert ready["source_artifact_issues"] == []
assert ready["missing_cli_flags"] == []
assert ready["failure_state"] == "none_recorded"
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
assert ready["redacted_artifacts_only_confirmed"] is True
assert ready["no_secret_or_raw_payload_persistence_confirmed"] is True
assert ready["no_retry_or_followup_mutation_confirmed"] is True
assert ready["dashboard_controls_disabled_confirmed"] is True
assert ready["no_listen_key_lifecycle_confirmed"] is True
PY

if grep -R "ntpro_v160005_production_like_api_key_value\\|ntpro_v160005_production_like_api_secret_value\\|ntpro_v160007_api_key_value\\|ntpro_v160007_api_secret_value\\|X-MBX-APIKEY\\|signature=" "$OUTPUT_DIR" >/dev/null; then
  echo "audit trail artifacts persisted forbidden secret or signed material" >&2
  exit 1
fi

echo "v16_mutation_audit_trail status=ok root=$GATE_ROOT audit_trail_ready=true request_sent=false network_attempted=false production_order_mutations_attempted=0 retry_attempted=false dashboard_order_controls_enabled=false"
