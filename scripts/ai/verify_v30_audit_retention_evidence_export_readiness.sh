#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

ARTIFACT_PATH="${NTPRO_V300_AUDIT_EXPORT_ARTIFACT:-docs/rust-cutover/release/v0_30_0_audit_retention_evidence_export_readiness.json}"
CONTRACT_PATH="${NTPRO_V300_AUDIT_EXPORT_CONTRACT:-docs/rust-cutover/release/v0_30_0_audit_retention_evidence_export_readiness.md}"
TASK_PATH="${NTPRO_V300_AUDIT_EXPORT_TASK:-docs/rust-cutover/tasks/V300-009.md}"
EVIDENCE_PATH="${NTPRO_V300_AUDIT_EXPORT_EVIDENCE:-docs/rust-cutover/evidence/V300-009.md}"
OPERATOR_LIFECYCLE="${NTPRO_V300_AUDIT_EXPORT_OPERATOR:-docs/rust-cutover/release/v0_30_0_operator_approval_freeze_change_window_lifecycle.json}"
CANARY_PREFLIGHT="${NTPRO_V300_AUDIT_EXPORT_CANARY:-docs/rust-cutover/release/v0_30_0_canary_execution_preflight_no_default_execution_gate.json}"
ROLLBACK_DR="${NTPRO_V300_AUDIT_EXPORT_ROLLBACK:-docs/rust-cutover/release/v0_30_0_rollback_disaster_recovery_execution_boundary.json}"
CONFIG_VENUE="${NTPRO_V300_AUDIT_EXPORT_CONFIG_VENUE:-docs/rust-cutover/release/v0_30_0_production_config_provenance_venue_connectivity_readiness.json}"
TELEMETRY_FREEZE="${NTPRO_V300_AUDIT_EXPORT_TELEMETRY:-docs/rust-cutover/release/v0_30_0_telemetry_slo_gate_incident_freeze_integration.json}"
V29_AUDIT="${NTPRO_V300_AUDIT_EXPORT_V29_AUDIT:-docs/rust-cutover/release/v0_29_0_persistent_audit_storage_production_readiness_artifact.json}"
RELEASE_INDEX="${NTPRO_V300_AUDIT_EXPORT_RELEASE_INDEX:-docs/rust-cutover/release/README.md}"
SELFTEST="${NTPRO_V300_AUDIT_EXPORT_SELFTEST:-1}"

fail() {
  echo "v30 audit retention evidence export readiness failed: $*" >&2
  exit 1
}

require_file() {
  local path="$1"
  [[ -f "$path" ]] || fail "missing required file: $path"
}

require_contains() {
  local path="$1"
  local marker="$2"
  if ! grep -F -- "$marker" "$path" >/dev/null; then
    fail "missing marker in $path: $marker"
  fi
}

for path in "$ARTIFACT_PATH" "$CONTRACT_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$OPERATOR_LIFECYCLE" "$CANARY_PREFLIGHT" "$ROLLBACK_DR" "$CONFIG_VENUE" "$TELEMETRY_FREEZE" "$V29_AUDIT" "$RELEASE_INDEX"; do
  require_file "$path"
done

require_contains "$TASK_PATH" "GitHub issue: \`#978\`"
require_contains "$EVIDENCE_PATH" "Task: \`V300-009\` / GitHub issue \`#978\`"
require_contains "$CONTRACT_PATH" "contract_version = ntpro.v300.audit_retention_evidence_export_readiness.v1"
require_contains "$CONTRACT_PATH" "audit_gate_mode = reconstructable_read_only_release_blocking"
require_contains "$CONTRACT_PATH" "evidence_export_mode = read_only_deterministic_readback"
require_contains "$CONTRACT_PATH" "missing_lineage => fail_closed_missing_lineage"
require_contains "$CONTRACT_PATH" "export_mutation_attempt => fail_closed_forbidden_export_mutation"
require_contains "$RELEASE_INDEX" "v0_30_0_audit_retention_evidence_export_readiness.md"
require_contains "$RELEASE_INDEX" "../evidence/V300-009.md"

ARTIFACT_PATH="$ARTIFACT_PATH" SELFTEST="$SELFTEST" python3 <<'PY'
from __future__ import annotations

import copy
import json
import os
from pathlib import Path
from typing import Any

payload = json.loads(Path(os.environ["ARTIFACT_PATH"]).read_text(encoding="utf-8"))
selftest = os.environ.get("SELFTEST", "1") != "0"

SCHEMA_VERSION = "ntpro.v300.audit_retention_evidence_export_readiness.v1"
RELEASE_SCOPE = "backend_production_go_live_candidate_foundation_only"
READY_STATUS = "audit_retention_evidence_export_ready"
DEPENDENCIES = {
    "V300-004",
    "V300-005",
    "V300-006",
    "V300-007",
    "V300-008",
    "v0.29.1-release-evidence",
}
EXPECTED_RECORDS = {
    "operator_approval",
    "deployment_readiness",
    "canary_preflight",
    "rollback_dr_boundary",
    "config_venue_readiness",
    "telemetry_slo_gate",
    "incident_freeze",
    "go_no_go_decision",
}
EXPECTED_EXPORTS = {
    "candidate_audit_bundle",
    "operator_change_window_export",
    "telemetry_incident_export",
    "go_no_go_decision_export",
}
EXPECTED_CASES = {
    "audit_retention_export.preview.allowed.001",
    "audit_retention_export.missing_required_record.fail_closed.001",
    "audit_retention_export.missing_lineage.fail_closed.001",
    "audit_retention_export.redaction_failure.fail_closed.001",
    "audit_retention_export.unverifiable_reference.fail_closed.001",
    "audit_retention_export.retention_boundary.fail_closed.001",
    "audit_retention_export.export_readback_mismatch.fail_closed.001",
    "audit_retention_export.export_mutation.fail_closed.001",
    "audit_retention_export.operational_action.fail_closed.001",
    "audit_retention_export.forbidden_boundary.fail_closed.001",
}
REQUIRED_FALSE_FLAGS = [
    "new_submit_capability",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "cancel_order_allowed",
    "replace_order_allowed",
    "amend_order_allowed",
    "flatten_position_allowed",
    "execution_adapter_call_allowed",
    "adapter_send_allowed",
    "live_exchange_request_allowed",
    "network_attempted",
    "retry_scheduler_enabled",
    "automatic_remediation_allowed",
    "automatic_operation_action_allowed",
    "dashboard_operation_controls_enabled",
    "dashboard_trading_controls_enabled",
    "admin_workbench_operation_controls_enabled",
    "admin_workbench_trading_controls_enabled",
    "trader_terminal_order_ticket_enabled",
    "manual_operation_submit_allowed",
    "backend_go_live_claim",
    "ambiguous_backend_go_live_claim",
    "actual_backend_production_go_live_allowed",
    "production_runtime_enablement_allowed",
    "product_grade_trading_terminal_claim",
    "product_grade_live_trading_terminal_claim",
    "default_production_execution_allowed",
    "candidate_artifact_runtime_effect_allowed",
    "production_feature_flags_default_enabled",
    "shared_approval_consumption_allowed",
    "production_deployment_execution_allowed",
    "production_deployment_executed",
    "live_environment_mutation_allowed",
    "runtime_switch_enablement_allowed",
    "candidate_operation_execution_allowed",
    "approval_lifecycle_authorizes_trading_operations",
    "canary_execution_allowed",
    "default_canary_execution_allowed",
    "production_canary_action_executed",
    "live_exchange_side_effect_allowed",
    "rollback_execution_allowed",
    "production_rollback_execution_allowed",
    "dr_restore_execution_allowed",
    "data_restore_execution_allowed",
    "service_restart_execution_allowed",
    "ambiguous_rollback_execution_claim_allowed",
    "unredacted_sensitive_fields_present",
    "credential_material_present",
    "adapter_send_attempted",
    "live_exchange_request_attempted",
    "order_send_permission_allowed",
    "connectivity_probe_network_attempted",
    "telemetry_action_triggered",
    "automatic_remediation_attempted",
    "retry_scheduler_attempted",
    "incident_freeze_active",
    "critical_incident_active",
    "production_storage_mutation_allowed",
    "evidence_export_mutation_allowed",
    "evidence_export_runtime_effect_allowed",
    "audit_export_operation_action_allowed",
    "audit_export_network_attempted",
    "audit_export_trading_control_allowed",
    "go_no_go_decision_action_allowed",
    "evidence_export_adapter_send_allowed",
]


def fail(message: str) -> None:
    raise SystemExit(f"v30 audit retention evidence export readiness failed: {message}")


def merge(base: Any, override: Any) -> Any:
    if isinstance(base, dict) and isinstance(override, dict):
        result = copy.deepcopy(base)
        for key, value in override.items():
            result[key] = merge(result.get(key), value)
        return result
    return copy.deepcopy(override)


def apply_indexed_overrides(items: list[dict[str, Any]], id_key: str, overrides: dict[str, Any]) -> list[dict[str, Any]]:
    result = copy.deepcopy(items)
    for item_id, override in overrides.items():
        for index, item in enumerate(result):
            if item.get(id_key) == item_id:
                result[index] = merge(item, override)
                break
        else:
            result.append({id_key: item_id, **copy.deepcopy(override)})
    return result


def apply_case(artifact: dict[str, Any], case: dict[str, Any]) -> dict[str, Any]:
    result = copy.deepcopy(artifact)
    if case.get("retention_policy_override"):
        result["retention_policy"] = merge(result.get("retention_policy"), case["retention_policy_override"])
    if case.get("redaction_policy_override"):
        result["redaction_policy"] = merge(result.get("redaction_policy"), case["redaction_policy_override"])
    if case.get("lineage_policy_override"):
        result["lineage_policy"] = merge(result.get("lineage_policy"), case["lineage_policy_override"])
    if case.get("record_overrides"):
        result["required_audit_records"] = apply_indexed_overrides(
            result["required_audit_records"],
            "record_id",
            case["record_overrides"],
        )
    if case.get("export_overrides"):
        result["evidence_exports"] = apply_indexed_overrides(
            result["evidence_exports"],
            "export_id",
            case["export_overrides"],
        )
    if case.get("boundary_flags_override"):
        result["boundary_flags"] = merge(result["boundary_flags"], case["boundary_flags_override"])
    return result


def classify_status(reasons: list[str]) -> str:
    if any(reason.startswith("forbidden_boundary") for reason in reasons):
        return "fail_closed_forbidden_boundary"
    if any(reason.startswith("forbidden_action") for reason in reasons):
        return "fail_closed_forbidden_action"
    if any(reason.startswith("export_mutation") for reason in reasons):
        return "fail_closed_forbidden_export_mutation"
    if any(reason.startswith("export_readback_mismatch") for reason in reasons):
        return "fail_closed_export_readback_mismatch"
    if any(reason.startswith("missing_required_record") for reason in reasons):
        return "fail_closed_missing_required_audit_record"
    if any(reason.startswith("unverifiable_reference") for reason in reasons):
        return "fail_closed_unverifiable_audit_reference"
    if any(reason.startswith("missing_lineage") for reason in reasons):
        return "fail_closed_missing_lineage"
    if any(reason.startswith("redaction_failure") for reason in reasons):
        return "fail_closed_redaction_failure"
    if any(reason.startswith("retention_boundary") for reason in reasons):
        return "fail_closed_retention_boundary"
    if reasons:
        return "fail_closed_forbidden_boundary"
    return READY_STATUS


def collect_reasons(artifact: dict[str, Any]) -> list[str]:
    reasons: list[str] = []
    if artifact.get("schema_version") != SCHEMA_VERSION:
        reasons.append("forbidden_boundary:schema_version")
    if artifact.get("contract_version") != SCHEMA_VERSION:
        reasons.append("forbidden_boundary:contract_version")
    if artifact.get("task_id") != "V300-009" or artifact.get("github_issue") != 978:
        reasons.append("forbidden_boundary:task_identity")
    if artifact.get("milestone") != "v0.30.0" or artifact.get("release_scope") != RELEASE_SCOPE:
        reasons.append("forbidden_boundary:release_scope")
    if artifact.get("candidate_claim") != "audit_retention_evidence_export_readiness":
        reasons.append("forbidden_boundary:candidate_claim")
    if artifact.get("audit_gate_mode") != "reconstructable_read_only_release_blocking":
        reasons.append("forbidden_boundary:audit_gate_mode")
    if artifact.get("evidence_export_mode") != "read_only_deterministic_readback":
        reasons.append("forbidden_boundary:evidence_export_mode")
    if artifact.get("retention_mode") != "immutable_until_expiry":
        reasons.append("retention_boundary:top_level_mode")
    for key in (
        "production_storage_mutation_allowed",
        "evidence_export_mutation_allowed",
        "evidence_export_runtime_effect_allowed",
        "audit_export_operation_action_allowed",
    ):
        if artifact.get(key) is not False:
            reasons.append(f"forbidden_action:top_level:{key}")
    if set(artifact.get("dependency_contracts") or []) != DEPENDENCIES:
        reasons.append("forbidden_boundary:dependency_contracts")

    retention = artifact.get("retention_policy")
    if not isinstance(retention, dict):
        reasons.append("retention_boundary:missing_policy")
    else:
        if retention.get("retention_policy_id") != "audit-retention-v300-go-live-candidate":
            reasons.append("retention_boundary:policy_id")
        if retention.get("mode") != "immutable_until_expiry":
            reasons.append("retention_boundary:mode")
        if retention.get("min_days", 0) < 730:
            reasons.append("retention_boundary:min_days")
        if retention.get("delete_before_retention_allowed") is not False:
            reasons.append("retention_boundary:delete_before_retention")
        if retention.get("retention_boundary_status") != "active":
            reasons.append("retention_boundary:status")

    redaction = artifact.get("redaction_policy")
    if not isinstance(redaction, dict):
        reasons.append("redaction_failure:missing_policy")
    else:
        if redaction.get("redaction_status") != "redacted":
            reasons.append("redaction_failure:policy_status")
        if redaction.get("unredacted_payload_allowed") is not False:
            reasons.append("redaction_failure:unredacted_allowed")
        if redaction.get("raw_secret_material_allowed") is not False:
            reasons.append("redaction_failure:raw_secret_allowed")

    lineage = artifact.get("lineage_policy")
    if not isinstance(lineage, dict):
        reasons.append("missing_lineage:missing_policy")
    else:
        if lineage.get("source_chain_required") is not True:
            reasons.append("missing_lineage:source_chain")
        if lineage.get("all_records_linked") is not True:
            reasons.append("missing_lineage:all_records_linked")
        if lineage.get("missing_lineage_fail_closed") is not True:
            reasons.append("missing_lineage:fail_closed")
        if lineage.get("unverifiable_reference_fail_closed") is not True:
            reasons.append("unverifiable_reference:fail_closed")
        if lineage.get("source_drift_allowed") is not False:
            reasons.append("unverifiable_reference:source_drift")

    records = artifact.get("required_audit_records")
    if not isinstance(records, list):
        reasons.append("missing_required_record:missing_records")
        records = []
    record_ids = {record.get("record_id") for record in records if isinstance(record, dict)}
    if record_ids != EXPECTED_RECORDS:
        reasons.append("missing_required_record:record_set")
    for record in records:
        if not isinstance(record, dict):
            reasons.append("missing_required_record:invalid_record")
            continue
        record_id = record.get("record_id")
        if record.get("presence_status") != "present":
            reasons.append(f"missing_required_record:{record_id}")
        if not record.get("source_ref") or record.get("reference_status") != "verified":
            reasons.append(f"unverifiable_reference:{record_id}")
        if record.get("lineage_status") != "linked" or record.get("immutable_audit_ref_present") is not True:
            reasons.append(f"missing_lineage:{record_id}")
        if record.get("redaction_status") != "redacted":
            reasons.append(f"redaction_failure:{record_id}")
        if record.get("retention_status") != "retained":
            reasons.append(f"retention_boundary:{record_id}")
        if record.get("readback_status") != "verified":
            reasons.append(f"export_readback_mismatch:record:{record_id}")
        if not record.get("export_ref") or record.get("export_ref") not in EXPECTED_EXPORTS:
            reasons.append(f"unverifiable_reference:export_ref:{record_id}")
        if record.get("reconstructable") is not True:
            reasons.append(f"missing_lineage:reconstructable:{record_id}")
        if record.get("operation_effect") != "none":
            reasons.append(f"forbidden_action:record:{record_id}")

    exports = artifact.get("evidence_exports")
    if not isinstance(exports, list):
        reasons.append("unverifiable_reference:missing_exports")
        exports = []
    export_ids = {export.get("export_id") for export in exports if isinstance(export, dict)}
    if export_ids != EXPECTED_EXPORTS:
        reasons.append("unverifiable_reference:export_set")
    for export in exports:
        if not isinstance(export, dict):
            reasons.append("unverifiable_reference:invalid_export")
            continue
        export_id = export.get("export_id")
        source_record_ids = set(export.get("source_record_ids") or [])
        if not source_record_ids or not source_record_ids.issubset(EXPECTED_RECORDS):
            reasons.append(f"unverifiable_reference:export_sources:{export_id}")
        if export.get("read_only") is not True:
            reasons.append(f"export_mutation:{export_id}")
        if export.get("readback_status") != "verified":
            reasons.append(f"export_readback_mismatch:{export_id}")
        if not str(export.get("export_digest", "")).startswith("sha256:"):
            reasons.append(f"unverifiable_reference:export_digest:{export_id}")
        if export.get("redaction_status") != "redacted":
            reasons.append(f"redaction_failure:export:{export_id}")
        if export.get("network_attempted") is not False:
            reasons.append(f"forbidden_action:network:{export_id}")
        if export.get("action_trigger_allowed") is not False:
            reasons.append(f"forbidden_action:trigger:{export_id}")
        if export.get("operation_effect") != "none":
            reasons.append(f"forbidden_action:export:{export_id}")

    flags = artifact.get("boundary_flags")
    if not isinstance(flags, dict):
        reasons.append("forbidden_boundary:missing_boundary_flags")
    else:
        for flag in REQUIRED_FALSE_FLAGS:
            if flag not in flags:
                reasons.append(f"forbidden_boundary:missing_flag:{flag}")
            elif flags[flag] is not False:
                reasons.append(f"forbidden_boundary:flag_open:{flag}")
    return reasons


def validate_cases(artifact: dict[str, Any]) -> None:
    cases = artifact.get("readiness_cases")
    if not isinstance(cases, list):
        fail("missing readiness_cases")
    case_ids = {case.get("case_id") for case in cases if isinstance(case, dict)}
    if case_ids != EXPECTED_CASES:
        fail(f"unexpected readiness cases: {sorted(case_ids)}")
    for case in cases:
        if not isinstance(case, dict):
            fail("invalid readiness case")
        scenario = apply_case(artifact, case)
        actual = classify_status(collect_reasons(scenario))
        expected = case.get("expected_status")
        if actual != expected:
            fail(f"case {case.get('case_id')} expected {expected}, got {actual}")


def run_selftests(artifact: dict[str, Any]) -> int:
    mutations = [
        ({"schema_version": "broken"}, "fail_closed_forbidden_boundary"),
        ({"boundary_flags": {"adapter_send_allowed": True}}, "fail_closed_forbidden_boundary"),
        (
            {
                "evidence_exports": apply_indexed_overrides(
                    artifact["evidence_exports"],
                    "export_id",
                    {"candidate_audit_bundle": {"read_only": False}},
                )
            },
            "fail_closed_forbidden_export_mutation",
        ),
        (
            {
                "required_audit_records": apply_indexed_overrides(
                    artifact["required_audit_records"],
                    "record_id",
                    {"operator_approval": {"redaction_status": "unredacted"}},
                )
            },
            "fail_closed_redaction_failure",
        ),
    ]
    for override, expected in mutations:
        actual = classify_status(collect_reasons(merge(artifact, override)))
        if actual != expected:
            fail(f"negative selftest expected {expected}, got {actual}")
    return len(mutations)


reasons = collect_reasons(payload)
status = classify_status(reasons)
if status != READY_STATUS:
    fail(f"baseline expected {READY_STATUS}, got {status}: {reasons}")
validate_cases(payload)
negative_selftests = run_selftests(payload) if selftest else 0

print(
    "v30_audit_retention_evidence_export_readiness=pass "
    f"audit_records={len(payload['required_audit_records'])} "
    f"evidence_exports={len(payload['evidence_exports'])} "
    f"readiness_cases={len(payload['readiness_cases'])} "
    f"required_false_flags={len(REQUIRED_FALSE_FLAGS)} "
    f"negative_selftest={negative_selftests}"
)
PY
