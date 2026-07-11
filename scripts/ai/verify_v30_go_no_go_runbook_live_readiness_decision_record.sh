#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

ARTIFACT_PATH="${NTPRO_V300_GO_NO_GO_ARTIFACT:-docs/rust-cutover/release/v0_30_0_go_no_go_runbook_live_readiness_decision_record.json}"
CONTRACT_PATH="${NTPRO_V300_GO_NO_GO_CONTRACT:-docs/rust-cutover/release/v0_30_0_go_no_go_runbook_live_readiness_decision_record.md}"
TASK_PATH="${NTPRO_V300_GO_NO_GO_TASK:-docs/rust-cutover/tasks/V300-010.md}"
EVIDENCE_PATH="${NTPRO_V300_GO_NO_GO_EVIDENCE:-docs/rust-cutover/evidence/V300-010.md}"
BOUNDARY_CONTRACT="${NTPRO_V300_GO_NO_GO_BOUNDARY:-docs/rust-cutover/release/v0_30_0_backend_go_live_candidate_boundary_contract.json}"
DEPLOYMENT_READINESS="${NTPRO_V300_GO_NO_GO_DEPLOYMENT:-docs/rust-cutover/release/v0_30_0_production_deployment_plan_environment_readiness.json}"
RUNTIME_FLAGS="${NTPRO_V300_GO_NO_GO_RUNTIME_FLAGS:-docs/rust-cutover/release/v0_30_0_runtime_enablement_boundary_controlled_feature_flags.json}"
OPERATOR_LIFECYCLE="${NTPRO_V300_GO_NO_GO_OPERATOR:-docs/rust-cutover/release/v0_30_0_operator_approval_freeze_change_window_lifecycle.json}"
CANARY_PREFLIGHT="${NTPRO_V300_GO_NO_GO_CANARY:-docs/rust-cutover/release/v0_30_0_canary_execution_preflight_no_default_execution_gate.json}"
ROLLBACK_DR="${NTPRO_V300_GO_NO_GO_ROLLBACK:-docs/rust-cutover/release/v0_30_0_rollback_disaster_recovery_execution_boundary.json}"
CONFIG_VENUE="${NTPRO_V300_GO_NO_GO_CONFIG_VENUE:-docs/rust-cutover/release/v0_30_0_production_config_provenance_venue_connectivity_readiness.json}"
TELEMETRY_FREEZE="${NTPRO_V300_GO_NO_GO_TELEMETRY:-docs/rust-cutover/release/v0_30_0_telemetry_slo_gate_incident_freeze_integration.json}"
AUDIT_EXPORT="${NTPRO_V300_GO_NO_GO_AUDIT_EXPORT:-docs/rust-cutover/release/v0_30_0_audit_retention_evidence_export_readiness.json}"
V291_RELEASE="${NTPRO_V300_GO_NO_GO_V291_RELEASE:-docs/rust-cutover/release/v0_29_1_release_closeout_evidence.md}"
RELEASE_INDEX="${NTPRO_V300_GO_NO_GO_RELEASE_INDEX:-docs/rust-cutover/release/README.md}"
SELFTEST="${NTPRO_V300_GO_NO_GO_SELFTEST:-1}"

fail() {
  echo "v30 go/no-go runbook live readiness decision record failed: $*" >&2
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

for path in "$ARTIFACT_PATH" "$CONTRACT_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$BOUNDARY_CONTRACT" "$DEPLOYMENT_READINESS" "$RUNTIME_FLAGS" "$OPERATOR_LIFECYCLE" "$CANARY_PREFLIGHT" "$ROLLBACK_DR" "$CONFIG_VENUE" "$TELEMETRY_FREEZE" "$AUDIT_EXPORT" "$V291_RELEASE" "$RELEASE_INDEX"; do
  require_file "$path"
done

require_contains "$TASK_PATH" "GitHub issue: \`#979\`"
require_contains "$EVIDENCE_PATH" "Task: \`V300-010\` / GitHub issue \`#979\`"
require_contains "$CONTRACT_PATH" "contract_version = ntpro.v300.go_no_go_runbook_live_readiness_decision_record.v1"
require_contains "$CONTRACT_PATH" "decision_record_mode = candidate_ready_only_no_production_enablement"
require_contains "$CONTRACT_PATH" "ready_outcome_meaning = candidate_ready_only"
require_contains "$CONTRACT_PATH" "actual_backend_go_live_claim => fail_closed_actual_backend_go_live_claim"
require_contains "$RELEASE_INDEX" "v0_30_0_go_no_go_runbook_live_readiness_decision_record.md"
require_contains "$RELEASE_INDEX" "../evidence/V300-010.md"

ARTIFACT_PATH="$ARTIFACT_PATH" SELFTEST="$SELFTEST" python3 <<'PY'
from __future__ import annotations

import copy
import json
import os
from pathlib import Path
from typing import Any

payload = json.loads(Path(os.environ["ARTIFACT_PATH"]).read_text(encoding="utf-8"))
selftest = os.environ.get("SELFTEST", "1") != "0"

SCHEMA_VERSION = "ntpro.v300.go_no_go_runbook_live_readiness_decision_record.v1"
RELEASE_SCOPE = "backend_production_go_live_candidate_foundation_only"
READY_STATUS = "go_no_go_candidate_ready_only"
DEPENDENCIES = {
    "V300-001",
    "V300-002",
    "V300-003",
    "V300-004",
    "V300-005",
    "V300-006",
    "V300-007",
    "V300-008",
    "V300-009",
    "v0.29.1-release-evidence",
}
EXPECTED_INPUTS = {
    "backend_go_live_candidate_boundary",
    "deployment_readiness",
    "runtime_feature_flags",
    "operator_approval_freeze_window",
    "canary_preflight",
    "rollback_dr_boundary",
    "config_venue_readiness",
    "telemetry_slo_incident_freeze",
    "audit_retention_export",
    "v291_release_evidence",
}
EXPECTED_OWNERS = {"release_gatekeeper", "owner_operator", "control_scope", "runtime_owner"}
EXPECTED_GATES = {
    "boundary_contract",
    "deployment_environment_readiness",
    "runtime_flags_default_disabled",
    "operator_freeze_window",
    "canary_no_default_execution",
    "rollback_dr_ready",
    "config_venue_ready",
    "telemetry_incident_freeze_ready",
    "audit_retention_export_ready",
    "v291_release_evidence_ready",
}
EXPECTED_FREEZE = {
    "missing_required_gate",
    "stale_required_gate",
    "blocked_required_gate",
    "active_operator_freeze",
    "telemetry_slo_breach",
    "rollback_not_ready",
}
EXPECTED_ROLLBACK_REFS = {"rollback_dr_boundary", "canary_abort_reference", "incident_freeze_reference"}
EXPECTED_DECISIONS = {"ready_candidate", "degraded_candidate", "blocked_candidate", "aborted_candidate"}
EXPECTED_OUTCOMES = {"ready", "degraded", "blocked", "aborted"}
EXPECTED_CASES = {
    "go_no_go_runbook.ready_candidate.allowed.001",
    "go_no_go_runbook.missing_gate.fail_closed.001",
    "go_no_go_runbook.stale_gate.fail_closed.001",
    "go_no_go_runbook.blocked_gate.fail_closed.001",
    "go_no_go_runbook.active_freeze.fail_closed.001",
    "go_no_go_runbook.missing_owner.fail_closed.001",
    "go_no_go_runbook.missing_rollback_reference.fail_closed.001",
    "go_no_go_runbook.actual_go_live_claim.fail_closed.001",
    "go_no_go_runbook.operational_action.fail_closed.001",
    "go_no_go_runbook.forbidden_boundary.fail_closed.001",
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
    "go_no_go_record_enables_execution",
    "decision_record_backend_go_live_allowed",
    "runtime_enablement_from_go_no_go_allowed",
    "operator_approval_reused_for_execution_allowed",
    "production_enablement_handoff_allowed",
]


def fail(message: str) -> None:
    raise SystemExit(f"v30 go/no-go runbook live readiness decision record failed: {message}")


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
    if case.get("top_level_override"):
        result = merge(result, case["top_level_override"])
    if case.get("input_overrides"):
        result["required_inputs"] = apply_indexed_overrides(result["required_inputs"], "input_id", case["input_overrides"])
    if case.get("owner_overrides"):
        result["decision_owners"] = apply_indexed_overrides(result["decision_owners"], "owner_id", case["owner_overrides"])
    if case.get("gate_overrides"):
        result["checklist_gates"] = apply_indexed_overrides(result["checklist_gates"], "gate_id", case["gate_overrides"])
    if case.get("freeze_abort_overrides"):
        result["freeze_abort_criteria"] = apply_indexed_overrides(
            result["freeze_abort_criteria"],
            "criterion_id",
            case["freeze_abort_overrides"],
        )
    if case.get("rollback_overrides"):
        result["rollback_references"] = apply_indexed_overrides(
            result["rollback_references"],
            "reference_id",
            case["rollback_overrides"],
        )
    if case.get("decision_record_overrides"):
        result["decision_records"] = apply_indexed_overrides(
            result["decision_records"],
            "decision_id",
            case["decision_record_overrides"],
        )
    if case.get("boundary_flags_override"):
        result["boundary_flags"] = merge(result["boundary_flags"], case["boundary_flags_override"])
    return result


def classify_status(reasons: list[str]) -> str:
    if any(reason.startswith("forbidden_boundary") for reason in reasons):
        return "fail_closed_forbidden_boundary"
    if any(reason.startswith("forbidden_action") for reason in reasons):
        return "fail_closed_forbidden_action"
    if any(reason.startswith("actual_go_live") for reason in reasons):
        return "fail_closed_actual_backend_go_live_claim"
    if any(reason.startswith("missing_required_gate") for reason in reasons):
        return "fail_closed_missing_required_gate"
    if any(reason.startswith("stale_required_gate") for reason in reasons):
        return "fail_closed_stale_required_gate"
    if any(reason.startswith("blocked_required_gate") for reason in reasons):
        return "fail_closed_blocked_required_gate"
    if any(reason.startswith("active_freeze") for reason in reasons):
        return "fail_closed_active_freeze_or_abort"
    if any(reason.startswith("missing_decision_owner") for reason in reasons):
        return "fail_closed_missing_decision_owner"
    if any(reason.startswith("missing_rollback_reference") for reason in reasons):
        return "fail_closed_missing_rollback_reference"
    if reasons:
        return "fail_closed_forbidden_boundary"
    return READY_STATUS


def collect_reasons(artifact: dict[str, Any]) -> list[str]:
    reasons: list[str] = []
    if artifact.get("schema_version") != SCHEMA_VERSION:
        reasons.append("forbidden_boundary:schema_version")
    if artifact.get("contract_version") != SCHEMA_VERSION:
        reasons.append("forbidden_boundary:contract_version")
    if artifact.get("task_id") != "V300-010" or artifact.get("github_issue") != 979:
        reasons.append("forbidden_boundary:task_identity")
    if artifact.get("milestone") != "v0.30.0" or artifact.get("release_scope") != RELEASE_SCOPE:
        reasons.append("forbidden_boundary:release_scope")
    if artifact.get("candidate_claim") != "go_no_go_runbook_live_readiness_decision_record":
        reasons.append("forbidden_boundary:candidate_claim")
    if artifact.get("decision_record_mode") != "candidate_ready_only_no_production_enablement":
        reasons.append("forbidden_boundary:decision_record_mode")
    if artifact.get("runbook_mode") != "manual_owner_operator_release_review":
        reasons.append("forbidden_boundary:runbook_mode")
    if artifact.get("ready_outcome_meaning") != "candidate_ready_only":
        reasons.append("forbidden_boundary:ready_outcome_meaning")
    if artifact.get("actual_backend_production_go_live_allowed") is not False:
        reasons.append("actual_go_live:top_level")
    if artifact.get("decision_record_runtime_effect_allowed") is not False:
        reasons.append("forbidden_action:decision_record_runtime_effect")
    if artifact.get("production_execution_enabled_by_decision") is not False:
        reasons.append("forbidden_action:production_execution_enabled_by_decision")
    if set(artifact.get("dependency_contracts") or []) != DEPENDENCIES:
        reasons.append("forbidden_boundary:dependency_contracts")

    inputs = artifact.get("required_inputs")
    if not isinstance(inputs, list):
        reasons.append("missing_required_gate:missing_inputs")
        inputs = []
    input_ids = {item.get("input_id") for item in inputs if isinstance(item, dict)}
    if input_ids != EXPECTED_INPUTS:
        reasons.append("missing_required_gate:input_set")
    for item in inputs:
        if not isinstance(item, dict):
            reasons.append("missing_required_gate:invalid_input")
            continue
        item_id = item.get("input_id")
        if item.get("presence_status") != "present":
            reasons.append(f"missing_required_gate:{item_id}")
        if item.get("freshness_status") != "fresh":
            reasons.append(f"stale_required_gate:{item_id}")
        if item.get("gate_status") != "pass":
            reasons.append(f"blocked_required_gate:{item_id}")
        if item.get("reference_status") != "verified" or not item.get("source_ref"):
            reasons.append(f"missing_required_gate:reference:{item_id}")
        if item.get("release_blocking") is not True:
            reasons.append(f"forbidden_boundary:release_blocking:{item_id}")
        if item.get("runtime_effect_allowed") is not False:
            reasons.append(f"forbidden_action:input:{item_id}")

    owners = artifact.get("decision_owners")
    if not isinstance(owners, list):
        reasons.append("missing_decision_owner:missing_owners")
        owners = []
    owner_ids = {owner.get("owner_id") for owner in owners if isinstance(owner, dict)}
    if owner_ids != EXPECTED_OWNERS:
        reasons.append("missing_decision_owner:owner_set")
    for owner in owners:
        if not isinstance(owner, dict):
            reasons.append("missing_decision_owner:invalid_owner")
            continue
        owner_id = owner.get("owner_id")
        if owner.get("acknowledgement_status") != "acknowledged":
            reasons.append(f"missing_decision_owner:{owner_id}")
        if owner.get("approval_scope") != "candidate_ready_only":
            reasons.append(f"actual_go_live:owner_scope:{owner_id}")
        if owner.get("production_go_live_authorized") is not False:
            reasons.append(f"actual_go_live:owner_authorized:{owner_id}")
        if owner.get("bypass_allowed") is not False:
            reasons.append(f"forbidden_boundary:owner_bypass:{owner_id}")

    gates = artifact.get("checklist_gates")
    if not isinstance(gates, list):
        reasons.append("missing_required_gate:missing_checklist")
        gates = []
    gate_ids = {gate.get("gate_id") for gate in gates if isinstance(gate, dict)}
    if gate_ids != EXPECTED_GATES:
        reasons.append("missing_required_gate:gate_set")
    for gate in gates:
        if not isinstance(gate, dict):
            reasons.append("missing_required_gate:invalid_gate")
            continue
        gate_id = gate.get("gate_id")
        if gate.get("input_id") not in EXPECTED_INPUTS:
            reasons.append(f"missing_required_gate:gate_input:{gate_id}")
        if gate.get("gate_status") != "pass":
            reasons.append(f"blocked_required_gate:{gate_id}")
        if gate.get("required") is not True or gate.get("release_blocking") is not True:
            reasons.append(f"forbidden_boundary:gate_required:{gate_id}")
        if gate.get("bypass_allowed") is not False:
            reasons.append(f"forbidden_boundary:gate_bypass:{gate_id}")

    freeze = artifact.get("freeze_abort_criteria")
    if not isinstance(freeze, list):
        reasons.append("active_freeze:missing_criteria")
        freeze = []
    freeze_ids = {criterion.get("criterion_id") for criterion in freeze if isinstance(criterion, dict)}
    if freeze_ids != EXPECTED_FREEZE:
        reasons.append("active_freeze:criterion_set")
    for criterion in freeze:
        if not isinstance(criterion, dict):
            reasons.append("active_freeze:invalid_criterion")
            continue
        criterion_id = criterion.get("criterion_id")
        if criterion.get("active") is not False:
            reasons.append(f"active_freeze:{criterion_id}")
        if criterion.get("abort_required_when_active") is not True:
            reasons.append(f"forbidden_boundary:abort_required:{criterion_id}")
        if criterion.get("automatic_remediation_allowed") is not False or criterion.get("execution_allowed") is not False:
            reasons.append(f"forbidden_action:freeze:{criterion_id}")

    rollback_refs = artifact.get("rollback_references")
    if not isinstance(rollback_refs, list):
        reasons.append("missing_rollback_reference:missing_refs")
        rollback_refs = []
    rollback_ids = {ref.get("reference_id") for ref in rollback_refs if isinstance(ref, dict)}
    if rollback_ids != EXPECTED_ROLLBACK_REFS:
        reasons.append("missing_rollback_reference:ref_set")
    for ref in rollback_refs:
        if not isinstance(ref, dict):
            reasons.append("missing_rollback_reference:invalid_ref")
            continue
        ref_id = ref.get("reference_id")
        if ref.get("reference_status") != "linked" or not ref.get("source_ref"):
            reasons.append(f"missing_rollback_reference:{ref_id}")
        if ref.get("execution_allowed") is not False or ref.get("live_mutation_allowed") is not False:
            reasons.append(f"forbidden_action:rollback_ref:{ref_id}")

    decisions = artifact.get("decision_records")
    if not isinstance(decisions, list):
        reasons.append("forbidden_boundary:missing_decisions")
        decisions = []
    decision_ids = {record.get("decision_id") for record in decisions if isinstance(record, dict)}
    outcomes = {record.get("outcome") for record in decisions if isinstance(record, dict)}
    if decision_ids != EXPECTED_DECISIONS or outcomes != EXPECTED_OUTCOMES:
        reasons.append("forbidden_boundary:decision_set")
    for record in decisions:
        if not isinstance(record, dict):
            reasons.append("forbidden_boundary:invalid_decision")
            continue
        decision_id = record.get("decision_id")
        if record.get("record_status") != "deterministic":
            reasons.append(f"forbidden_boundary:decision_status:{decision_id}")
        expected_advancement = decision_id == "ready_candidate"
        if record.get("candidate_advancement_allowed") is not expected_advancement:
            reasons.append(f"blocked_required_gate:decision_advancement:{decision_id}")
        if record.get("actual_backend_production_go_live_allowed") is not False:
            reasons.append(f"actual_go_live:decision:{decision_id}")
        if record.get("production_execution_enabled") is not False:
            reasons.append(f"forbidden_action:decision_execution:{decision_id}")
        if record.get("requires_v31_scoped_enablement") is not True:
            reasons.append(f"forbidden_boundary:v31_enablement:{decision_id}")
        if record.get("operation_effect") != "none":
            reasons.append(f"forbidden_action:decision_effect:{decision_id}")

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
                "required_inputs": apply_indexed_overrides(
                    artifact["required_inputs"],
                    "input_id",
                    {"telemetry_slo_incident_freeze": {"freshness_status": "stale"}},
                )
            },
            "fail_closed_stale_required_gate",
        ),
        (
            {
                "decision_records": apply_indexed_overrides(
                    artifact["decision_records"],
                    "decision_id",
                    {"ready_candidate": {"actual_backend_production_go_live_allowed": True}},
                )
            },
            "fail_closed_actual_backend_go_live_claim",
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
    "v30_go_no_go_runbook_live_readiness_decision_record=pass "
    f"required_inputs={len(payload['required_inputs'])} "
    f"decision_owners={len(payload['decision_owners'])} "
    f"checklist_gates={len(payload['checklist_gates'])} "
    f"freeze_abort_criteria={len(payload['freeze_abort_criteria'])} "
    f"rollback_references={len(payload['rollback_references'])} "
    f"decision_records={len(payload['decision_records'])} "
    f"readiness_cases={len(payload['readiness_cases'])} "
    f"required_false_flags={len(REQUIRED_FALSE_FLAGS)} "
    f"negative_selftest={negative_selftests}"
)
PY
