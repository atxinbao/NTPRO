#!/usr/bin/env bash
set -euo pipefail

# V170-001: v0.17 local production order ledger.
# This verifier stays local/offline. It links one v0.16 owner-approved mutation
# candidate evidence chain into a restart-readable local ledger and proves the
# ledger does not duplicate-submit, retry, cancel, remediate, enable Dashboard
# order controls, or persist secrets/signed material.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh
export NTPRO_SOURCE_COMMIT="${NTPRO_SOURCE_COMMIT:-$(git rev-parse HEAD)}"
export NTPRO_SOURCE_RELEASE_TAG="${NTPRO_SOURCE_RELEASE_TAG:-unreleased-v17-local-gate}"

if [[ "${NTPRO_V17_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V17_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V17_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi

GATE_ROOT="${NTPRO_V17_LOCAL_ORDER_LEDGER_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v17-local-ledger.XXXXXX")}"
FAILURE_ROOT="$GATE_ROOT/failure-semantics"
OUTPUT_DIR="$GATE_ROOT/command-output"
mkdir -p "$OUTPUT_DIR"

NTPRO_V16_SKIP_BUILD=1 \
NTPRO_V16_NAUTILUS_BIN="$NAUTILUS_BIN" \
NTPRO_V16_FAILURE_SEMANTICS_ROOT="$FAILURE_ROOT" \
  scripts/ai/verify_v16_failure_no_retry_semantics.sh >/dev/null

REQUEST_BUILDER="$FAILURE_ROOT/audit-trail/order-state-readback/response-redaction/guarded-send/request-builder/command-output/ready-request-builder.json"
GUARDED_SEND="$FAILURE_ROOT/audit-trail/order-state-readback/response-redaction/guarded-send/command-output/ready-offline-guarded-send.json"
RESPONSE_REDACTION="$FAILURE_ROOT/audit-trail/order-state-readback/response-redaction/command-output/ready-response-redaction.json"
ORDER_STATE_READBACK="$FAILURE_ROOT/audit-trail/order-state-readback/command-output/ready-offline-order-state-readback.json"
AUDIT_TRAIL="$FAILURE_ROOT/audit-trail/command-output/ready-redacted-audit-trail.json"
FAILURE_SEMANTICS="$FAILURE_ROOT/command-output/ready-readback-mismatch.json"
MISSING_FLAGS_LEDGER="$OUTPUT_DIR/missing-flags-local-order-ledger.json"
READY_LEDGER="$OUTPUT_DIR/ready-local-order-ledger.json"

for input in \
  "$REQUEST_BUILDER" \
  "$GUARDED_SEND" \
  "$RESPONSE_REDACTION" \
  "$ORDER_STATE_READBACK" \
  "$AUDIT_TRAIL" \
  "$FAILURE_SEMANTICS"; do
  if [[ ! -f "$input" ]]; then
    echo "local order ledger setup did not produce expected input: $input" >&2
    exit 1
  fi
done

run_local_order_ledger() {
  local output="$1"
  shift
  "$NAUTILUS_BIN" live production-mutation-local-order-ledger \
    --run-id v170-production-mutation-local-order-ledger \
    --order-lineage-id lineage-v160-single-shot \
    --request-builder "$REQUEST_BUILDER" \
    --guarded-send "$GUARDED_SEND" \
    --response-redaction "$RESPONSE_REDACTION" \
    --order-state-readback "$ORDER_STATE_READBACK" \
    --audit-trail "$AUDIT_TRAIL" \
    --failure-semantics "$FAILURE_SEMANTICS" \
    --output "$output" \
    "$@"
}

run_local_order_ledger "$MISSING_FLAGS_LEDGER" >/dev/null

run_local_order_ledger "$READY_LEDGER" \
  --allow-production-mutation-local-order-ledger \
  --confirm-single-v16-mutation-candidate-lineage \
  --confirm-read-only-reconciliation-scope \
  --confirm-no-network \
  --confirm-no-duplicate-submit \
  --confirm-no-retry \
  --confirm-no-cancel \
  --confirm-no-remediation \
  --confirm-dashboard-order-controls-disabled \
  --confirm-no-secret-persistence >/dev/null

python3 - "$MISSING_FLAGS_LEDGER" "$READY_LEDGER" <<'PY'
import json
import sys
from pathlib import Path

missing_flags = json.loads(Path(sys.argv[1]).read_text())
ready_path = Path(sys.argv[2])
ready = json.loads(ready_path.read_text())
reloaded = json.loads(ready_path.read_text())

assert missing_flags["schema_version"] == "ntpro.v170_production_mutation_local_order_ledger.v1"
assert missing_flags["status"] == "blocked_missing_gate"
assert missing_flags["local_ledger_ready"] is False
assert missing_flags["restart_readable"] is False
assert missing_flags["manual_review_required"] is True
assert missing_flags["new_orders_blocked"] is True
assert "--allow-production-mutation-local-order-ledger" in missing_flags["missing_cli_flags"]
assert "--confirm-no-duplicate-submit" in missing_flags["missing_cli_flags"]
assert missing_flags["duplicate_submit_attempted"] is False
assert missing_flags["retry_attempted"] is False
assert missing_flags["cancel_attempted"] is False
assert missing_flags["remediation_attempted"] is False

assert ready["schema_version"] == "ntpro.v170_production_mutation_local_order_ledger.v1"
assert ready["status"] == "ready_local_order_ledger"
assert ready["run_id"] == "v170-production-mutation-local-order-ledger"
assert ready["order_lineage_id"] == "lineage-v160-single-shot"
assert ready["capability"] == "Production Reconciliation And Orphan Recovery Evidence"
assert ready["capability_expansion_from_v16"] == "reconciliation_evidence_only"
assert ready["lineage_scope"] == "single_v16_mutation_candidate"
assert ready["current_local_state"] == "local_ledger_pending_exchange_reconciliation"
assert ready["default_fail_closed"] is True
assert ready["owner_gated_readback_required"] is True
assert ready["local_ledger_ready"] is True
assert ready["restart_readable"] is True
assert ready["source_artifact_issues"] == []
assert ready["missing_cli_flags"] == []

expected_refs = {
    "request_builder_ref": "ntpro.v160_production_mutation_request_builder.v1",
    "guarded_send_ref": "ntpro.v160_production_mutation_guarded_send.v1",
    "response_redaction_ref": "ntpro.v160_production_mutation_response_redaction.v1",
    "readback_ref": "ntpro.v160_production_mutation_order_state_readback.v1",
    "audit_ref": "ntpro.v160_production_mutation_audit_trail.v1",
    "failure_ref": "ntpro.v160_production_mutation_failure_semantics.v1",
}
for field, schema in expected_refs.items():
    assert ready[field]["schema_version"] == schema, field
    assert ready[field]["hash"].startswith("fnv1a64:"), field
    assert ready[field]["sha256"].startswith("sha256:"), field
    assert len(ready[field]["sha256"]) == 71, field
    assert ready[field]["bytes"] > 0, field
    assert ready[field]["source_command"] != "unknown", field
    assert ready[field]["source_commit"] != "unknown", field
    assert ready[field]["source_release_tag"], field

assert ready["symbol"] == "BTCUSDT"
assert ready["side"] == "BUY"
assert ready["order_type"] == "LIMIT"
assert ready["time_in_force"] == "GTC"
assert ready["order_id"] == "123456789"
assert ready["client_order_id"] == "owner-approved-v160-single-shot"
assert ready["exchange_status"] == "NEW"
assert ready["exchange_readback_mapped"] is False
assert ready["reconciliation_classified"] is False
assert ready["orphan_risk_detected"] is False
assert ready["manual_review_required"] is False
assert ready["new_orders_blocked"] is False

for field in [
    "request_sent",
    "network_attempted",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "production_order_state_reads_allowed",
    "listen_key_lifecycle_allowed",
    "duplicate_submit_attempted",
    "retry_attempted",
    "cancel_attempted",
    "replace_attempted",
    "amend_attempted",
    "flatten_attempted",
    "remediation_attempted",
    "automatic_cancel_allowed",
    "automatic_remediation_allowed",
    "dashboard_order_controls_enabled",
    "dashboard_cancel_controls_enabled",
    "api_key_value_recorded",
    "api_secret_value_recorded",
    "api_key_header_value_recorded",
    "signature_recorded",
    "signed_query_recorded",
    "signed_url_recorded",
    "raw_exchange_response_recorded",
    "response_body_recorded",
    "response_headers_recorded",
]:
    assert ready[field] is False, field

assert ready["production_order_submissions_attempted"] == 0
assert ready["production_orders_submitted"] == 0
assert ready["production_order_mutations_attempted"] == 0
assert ready["production_order_state_reads_attempted"] == 0
assert ready["listen_key_lifecycle_attempted"] == 0
assert ready["no_network_confirmed"] is True
assert ready["no_duplicate_submit_confirmed"] is True
assert ready["no_retry_confirmed"] is True
assert ready["no_cancel_confirmed"] is True
assert ready["no_remediation_confirmed"] is True
assert ready["dashboard_controls_disabled_confirmed"] is True
assert ready["no_secret_persistence_confirmed"] is True

assert reloaded["order_lineage_id"] == ready["order_lineage_id"]
assert reloaded["request_builder_ref"]["hash"] == ready["request_builder_ref"]["hash"]
assert reloaded["local_ledger_ready"] is True
assert reloaded["restart_readable"] is True
PY

if grep -R "ntpro_v160005_production_like_api_key_value\\|ntpro_v160005_production_like_api_secret_value\\|ntpro_v160007_api_key_value\\|ntpro_v160007_api_secret_value\\|X-MBX-APIKEY\\|signature=" "$OUTPUT_DIR" >/dev/null; then
  echo "local order ledger artifacts persisted forbidden secret or signed material" >&2
  exit 1
fi

echo "v17_local_order_ledger status=ok root=$GATE_ROOT local_ledger_ready=true restart_readable=true duplicate_submit_attempted=false retry_attempted=false cancel_attempted=false remediation_attempted=false dashboard_order_controls_enabled=false"
