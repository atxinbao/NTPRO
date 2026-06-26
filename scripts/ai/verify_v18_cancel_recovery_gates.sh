#!/usr/bin/env bash
set -euo pipefail

# V180-003..V180-008: owner-approved cancel recovery preview artifacts.
# Default execution is local/offline and fail-closed. This script generates and
# validates preview-only cancel recovery artifacts without sending a cancel
# request, opening network access, or enabling Dashboard cancel controls.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

GATE_ROOT="${NTPRO_V18_CANCEL_GATE_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v18-cancel-recovery.XXXXXX")}"
ARTIFACT_ROOT="$GATE_ROOT/artifacts/v0_18"
mkdir -p "$ARTIFACT_ROOT"

python3 - "$ARTIFACT_ROOT" <<'PY'
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
common = {
    "run_id": "v180-cancel-recovery-preview",
    "order_lineage_id": "lineage-v160-single-shot",
    "cancel_candidate_id": "cancel-preview-v180-single-shot",
    "mode": "preview_only_offline",
    "manual_owner_approval_required": True,
    "owner_approved": False,
    "request_preview_ready": True,
    "risk_gate_passed": True,
    "response_redaction_ready": True,
    "post_cancel_readback_ready": True,
    "incident_closeout_ready": True,
    "actual_cancel_send_allowed": False,
    "cancel_attempted": False,
    "automatic_cancel_allowed": False,
    "automatic_remediation_allowed": False,
    "dashboard_cancel_controls_enabled": False,
    "dashboard_order_controls_enabled": False,
    "network_attempted": False,
    "request_sent": False,
    "production_order_mutations_attempted": 0,
    "production_order_state_reads_attempted": 0,
    "raw_exchange_response_recorded": False,
    "api_key_value_recorded": False,
    "api_secret_value_recorded": False,
    "signed_query_recorded": False,
    "signed_url_recorded": False,
}

artifacts = {
    "cancel_recovery_scope_decision.json": {
        "schema_version": "ntpro.v180_cancel_recovery_scope_decision.v1",
        "artifact_type": "cancel_recovery_scope_decision",
        "status": "scope_defined_preview_only",
        "scope_decision": "owner_approved_cancel_recovery_preview_only",
        "actual_cancel_send_allowed": False,
    },
    "cancel_recovery_artifact_contracts.json": {
        "schema_version": "ntpro.v180_cancel_recovery_artifact_contracts.v1",
        "artifact_type": "cancel_recovery_artifact_contracts",
        "status": "artifact_contracts_ready",
        "required_artifacts": [
            "cancel_request_preview",
            "cancel_risk_gate",
            "cancel_manual_approval_lifecycle",
            "cancel_response_redaction",
            "cancel_post_cancel_readback",
            "cancel_incident_audit_closeout",
        ],
    },
    "cancel_request_preview.json": {
        "schema_version": "ntpro.v180_cancel_request_preview.v1",
        "artifact_type": "cancel_request_preview",
        "status": "cancel_request_preview_ready",
        "http_method": "DELETE",
        "endpoint": "/api/v3/order",
        "redacted_request_only": True,
        "redacted_query_preview": "symbol=BTCUSDT&orderId=REDACTED&timestamp=REDACTED",
    },
    "cancel_risk_gate.json": {
        "schema_version": "ntpro.v180_cancel_risk_gate.v1",
        "artifact_type": "cancel_risk_gate",
        "status": "cancel_risk_gate_passed_for_preview_only",
        "risk_gate_decision": "allow_preview_only",
        "maximum_cancel_count": 0,
    },
    "cancel_manual_approval_lifecycle.json": {
        "schema_version": "ntpro.v180_cancel_manual_approval_lifecycle.v1",
        "artifact_type": "cancel_manual_approval_lifecycle",
        "status": "manual_owner_approval_required",
        "approval_state": "pending_owner_review",
    },
    "cancel_response_redaction.json": {
        "schema_version": "ntpro.v180_cancel_response_redaction.v1",
        "artifact_type": "cancel_response_redaction",
        "status": "cancel_response_redaction_ready",
        "redacted_response_preview": {"orderId": "REDACTED", "status": "CANCELED"},
    },
    "cancel_post_cancel_readback.json": {
        "schema_version": "ntpro.v180_cancel_post_cancel_readback.v1",
        "artifact_type": "cancel_post_cancel_readback",
        "status": "post_cancel_readback_contract_ready",
        "readback_fixture_only": True,
        "post_cancel_expected_state": "canceled_or_terminal",
    },
    "cancel_incident_audit_closeout.json": {
        "schema_version": "ntpro.v180_cancel_incident_audit_closeout.v1",
        "artifact_type": "cancel_incident_audit_closeout",
        "status": "incident_audit_closeout_ready",
        "audit_closeout_ready": True,
        "incident_severity": "review_required",
    },
}

for name, payload in artifacts.items():
    merged = {**common, **payload}
    (root / name).write_text(json.dumps(merged, indent=2, sort_keys=True) + "\n")
PY

python3 - "$ARTIFACT_ROOT" <<'PY'
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
expected = {
    "cancel_recovery_scope_decision.json": "ntpro.v180_cancel_recovery_scope_decision.v1",
    "cancel_recovery_artifact_contracts.json": "ntpro.v180_cancel_recovery_artifact_contracts.v1",
    "cancel_request_preview.json": "ntpro.v180_cancel_request_preview.v1",
    "cancel_risk_gate.json": "ntpro.v180_cancel_risk_gate.v1",
    "cancel_manual_approval_lifecycle.json": "ntpro.v180_cancel_manual_approval_lifecycle.v1",
    "cancel_response_redaction.json": "ntpro.v180_cancel_response_redaction.v1",
    "cancel_post_cancel_readback.json": "ntpro.v180_cancel_post_cancel_readback.v1",
    "cancel_incident_audit_closeout.json": "ntpro.v180_cancel_incident_audit_closeout.v1",
}
for name, schema in expected.items():
    path = root / name
    if not path.exists():
        raise SystemExit(f"missing artifact {name}")
    artifact = json.loads(path.read_text())
    if artifact.get("schema_version") != schema:
        raise SystemExit(f"schema mismatch for {name}: {artifact.get('schema_version')}")
    for field in (
        "actual_cancel_send_allowed",
        "cancel_attempted",
        "automatic_cancel_allowed",
        "dashboard_cancel_controls_enabled",
        "network_attempted",
        "request_sent",
        "api_key_value_recorded",
        "api_secret_value_recorded",
        "signed_query_recorded",
        "signed_url_recorded",
        "raw_exchange_response_recorded",
    ):
        if artifact.get(field) is not False:
            raise SystemExit(f"{name} violates {field}=false")
    for field in ("production_order_mutations_attempted", "production_order_state_reads_attempted"):
        if artifact.get(field) != 0:
            raise SystemExit(f"{name} violates {field}=0")
    if artifact.get("owner_approved") is not False:
        raise SystemExit(f"{name} must remain pending owner approval")
    if artifact.get("manual_owner_approval_required") is not True:
        raise SystemExit(f"{name} must require manual owner approval")
PY

if grep -RE \
  "actual_cancel_send_allowed[\"=: ]+true|cancel_attempted[\"=: ]+true|automatic_cancel_allowed[\"=: ]+true|dashboard_cancel_controls_enabled[\"=: ]+true|network_attempted[\"=: ]+true|request_sent[\"=: ]+true|production_order_mutations_attempted[\"=: ]+[1-9]" \
  "$ARTIFACT_ROOT" >/tmp/ntpro-v18-cancel-forbidden.txt; then
  echo "v18 cancel recovery artifacts contain forbidden mutation markers:" >&2
  cat /tmp/ntpro-v18-cancel-forbidden.txt >&2
  exit 1
fi

echo "v18_cancel_recovery_gates status=ok root=$GATE_ROOT actual_cancel_send_allowed=false cancel_attempted=false automatic_cancel_allowed=false dashboard_cancel_controls_enabled=false network_attempted=false production_order_mutations_attempted=0 manual_owner_approval_required=true owner_approved=false"

