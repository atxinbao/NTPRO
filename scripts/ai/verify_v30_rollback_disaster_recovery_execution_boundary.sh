#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

ARTIFACT_PATH="${NTPRO_V300_ROLLBACK_DR_ARTIFACT:-docs/rust-cutover/release/v0_30_0_rollback_disaster_recovery_execution_boundary.json}"
CONTRACT_PATH="${NTPRO_V300_ROLLBACK_DR_CONTRACT:-docs/rust-cutover/release/v0_30_0_rollback_disaster_recovery_execution_boundary.md}"
TASK_PATH="${NTPRO_V300_ROLLBACK_DR_TASK:-docs/rust-cutover/tasks/V300-006.md}"
EVIDENCE_PATH="${NTPRO_V300_ROLLBACK_DR_EVIDENCE:-docs/rust-cutover/evidence/V300-006.md}"
DEPLOYMENT_READINESS="${NTPRO_V300_ROLLBACK_DR_DEPLOYMENT:-docs/rust-cutover/release/v0_30_0_production_deployment_plan_environment_readiness.json}"
OPERATOR_LIFECYCLE="${NTPRO_V300_ROLLBACK_DR_OPERATOR:-docs/rust-cutover/release/v0_30_0_operator_approval_freeze_change_window_lifecycle.json}"
CANARY_PREFLIGHT="${NTPRO_V300_ROLLBACK_DR_CANARY:-docs/rust-cutover/release/v0_30_0_canary_execution_preflight_no_default_execution_gate.json}"
V29_ROLLBACK="${NTPRO_V300_ROLLBACK_DR_V29_ROLLBACK:-docs/rust-cutover/release/v0_29_0_canary_rollback_dr_preflight_readiness_artifact.json}"
RELEASE_INDEX="${NTPRO_V300_ROLLBACK_DR_RELEASE_INDEX:-docs/rust-cutover/release/README.md}"
SELFTEST="${NTPRO_V300_ROLLBACK_DR_SELFTEST:-1}"

fail() {
  echo "v30 rollback disaster recovery execution boundary failed: $*" >&2
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

for path in "$ARTIFACT_PATH" "$CONTRACT_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$DEPLOYMENT_READINESS" "$OPERATOR_LIFECYCLE" "$CANARY_PREFLIGHT" "$V29_ROLLBACK" "$RELEASE_INDEX"; do
  require_file "$path"
done

require_contains "$TASK_PATH" "GitHub issue: \`#975\`"
require_contains "$EVIDENCE_PATH" "Task: \`V300-006\` / GitHub issue \`#975\`"
require_contains "$CONTRACT_PATH" "contract_version = ntpro.v300.rollback_disaster_recovery_execution_boundary.v1"
require_contains "$CONTRACT_PATH" "rollback_execution_allowed = false"
require_contains "$CONTRACT_PATH" "dr_restore_execution_allowed = false"
require_contains "$CONTRACT_PATH" "ambiguous_rollback_execution => fail_closed_ambiguous_rollback_execution"
require_contains "$CONTRACT_PATH" "stale_restore_point => fail_closed_stale_restore_point"
require_contains "$CONTRACT_PATH" "inconsistent_deployment_provenance => fail_closed_inconsistent_deployment_provenance"
require_contains "$RELEASE_INDEX" "v0_30_0_rollback_disaster_recovery_execution_boundary.md"
require_contains "$RELEASE_INDEX" "../evidence/V300-006.md"

ARTIFACT_PATH="$ARTIFACT_PATH" SELFTEST="$SELFTEST" python3 <<'PY'
from __future__ import annotations

import copy
import json
import os
from pathlib import Path
from typing import Any

payload = json.loads(Path(os.environ["ARTIFACT_PATH"]).read_text(encoding="utf-8"))
selftest = os.environ.get("SELFTEST", "1") != "0"

SCHEMA_VERSION = "ntpro.v300.rollback_disaster_recovery_execution_boundary.v1"
RELEASE_SCOPE = "backend_production_go_live_candidate_foundation_only"
READY_STATUS = "rollback_disaster_recovery_execution_boundary_ready"
DEPENDENCIES = {"V300-002", "V300-004", "V300-005", "v0.29.1-release-evidence"}
EXPECTED_ROLLBACK_PLANS = {
    "artifact_revert_plan",
    "config_revert_plan",
    "schema_rollback_preview_plan",
    "traffic_revert_plan",
}
EXPECTED_DR_BOUNDARIES = {
    "snapshot_lineage_checkpoint",
    "recovery_point_checkpoint",
    "restore_sandbox_preview",
    "service_restart_preview",
}
EXPECTED_SAFETY = {
    "pre_deploy_snapshot",
    "audit_retention_snapshot",
    "config_digest_checkpoint",
    "idempotency_replay_checkpoint",
}
EXPECTED_TRIGGERS = {
    "telemetry_slo_breach",
    "operator_emergency_stop",
    "rollback_plan_mismatch",
    "restore_point_stale",
}
EXPECTED_CASES = {
    "rollback_dr_boundary.preview.allowed.001",
    "rollback_dr_boundary.ambiguous_execution.fail_closed.001",
    "rollback_dr_boundary.missing_operator_approval.fail_closed.001",
    "rollback_dr_boundary.stale_restore_point.fail_closed.001",
    "rollback_dr_boundary.inconsistent_deployment_provenance.fail_closed.001",
    "rollback_dr_boundary.missing_data_safety.fail_closed.001",
    "rollback_dr_boundary.missing_incident_trigger.fail_closed.001",
    "rollback_dr_boundary.execution_true.fail_closed.001",
    "rollback_dr_boundary.restore_execution_true.fail_closed.001",
    "rollback_dr_boundary.forbidden_boundary.fail_closed.001",
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
]


def fail(message: str) -> None:
    raise SystemExit(f"v30 rollback disaster recovery execution boundary failed: {message}")


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
    if case.get("manual_approval_override"):
        result["manual_approval"] = merge(result["manual_approval"], case["manual_approval_override"])
    if case.get("rollback_plan_overrides"):
        result["rollback_plans"] = apply_indexed_overrides(
            result["rollback_plans"],
            "plan_id",
            case["rollback_plan_overrides"],
        )
    if case.get("dr_restore_boundary_overrides"):
        result["dr_restore_boundaries"] = apply_indexed_overrides(
            result["dr_restore_boundaries"],
            "boundary_id",
            case["dr_restore_boundary_overrides"],
        )
    if case.get("data_safety_checkpoint_overrides"):
        result["data_safety_checkpoints"] = apply_indexed_overrides(
            result["data_safety_checkpoints"],
            "checkpoint_id",
            case["data_safety_checkpoint_overrides"],
        )
    if case.get("incident_freeze_trigger_overrides"):
        result["incident_freeze_triggers"] = apply_indexed_overrides(
            result["incident_freeze_triggers"],
            "trigger_id",
            case["incident_freeze_trigger_overrides"],
        )
    if case.get("boundary_flags_override"):
        result["boundary_flags"] = merge(result["boundary_flags"], case["boundary_flags_override"])
    return result


def classify_status(reasons: list[str]) -> str:
    if any(reason.startswith("forbidden_boundary") for reason in reasons):
        return "fail_closed_forbidden_boundary"
    if any(reason.startswith("forbidden_execution") for reason in reasons):
        return "fail_closed_forbidden_execution"
    if any(reason.startswith("ambiguous_rollback_execution") for reason in reasons):
        return "fail_closed_ambiguous_rollback_execution"
    if any(reason.startswith("missing_operator_approval") for reason in reasons):
        return "fail_closed_missing_operator_approval"
    if any(reason.startswith("stale_restore_point") for reason in reasons):
        return "fail_closed_stale_restore_point"
    if any(reason.startswith("inconsistent_deployment_provenance") for reason in reasons):
        return "fail_closed_inconsistent_deployment_provenance"
    if any(reason.startswith("missing_data_safety_checkpoint") for reason in reasons):
        return "fail_closed_missing_data_safety_checkpoint"
    if any(reason.startswith("missing_incident_freeze_trigger") for reason in reasons):
        return "fail_closed_missing_incident_freeze_trigger"
    if reasons:
        return "fail_closed_forbidden_boundary"
    return READY_STATUS


def collect_reasons(artifact: dict[str, Any]) -> list[str]:
    reasons: list[str] = []
    if artifact.get("schema_version") != SCHEMA_VERSION:
        reasons.append("forbidden_boundary:schema_version")
    if artifact.get("contract_version") != SCHEMA_VERSION:
        reasons.append("forbidden_boundary:contract_version")
    if artifact.get("task_id") != "V300-006" or artifact.get("github_issue") != 975:
        reasons.append("forbidden_boundary:task_identity")
    if artifact.get("milestone") != "v0.30.0" or artifact.get("release_scope") != RELEASE_SCOPE:
        reasons.append("forbidden_boundary:release_scope")
    if artifact.get("candidate_claim") != "rollback_disaster_recovery_execution_boundary":
        reasons.append("forbidden_boundary:candidate_claim")
    if artifact.get("rollback_readiness_mode") != "source_controlled_preview_only":
        reasons.append("forbidden_boundary:rollback_readiness_mode")
    if artifact.get("execution_evidence_present") is not False:
        reasons.append("forbidden_execution:execution_evidence")
    for key in ("rollback_execution_allowed", "dr_restore_execution_allowed", "data_restore_execution_allowed", "service_restart_execution_allowed"):
        if artifact.get(key) is not False:
            reasons.append(f"forbidden_execution:top_level:{key}")
    if set(artifact.get("dependency_contracts") or []) != DEPENDENCIES:
        reasons.append("forbidden_boundary:dependency_contracts")

    flags = artifact.get("boundary_flags")
    if not isinstance(flags, dict):
        reasons.append("forbidden_boundary:missing_boundary_flags")
    else:
        for key in REQUIRED_FALSE_FLAGS:
            if key not in flags:
                reasons.append(f"forbidden_boundary:missing:{key}")
            elif flags.get(key) is not False:
                reasons.append(f"forbidden_boundary:opened:{key}")

    approval = artifact.get("manual_approval") or {}
    if approval.get("approval_status") != "approved_for_preview_only":
        reasons.append("missing_operator_approval:status")
    if not approval.get("operator_lifecycle_ref"):
        reasons.append("missing_operator_approval:ref")
    if approval.get("identity_provenance_status") != "linked":
        reasons.append("missing_operator_approval:identity")
    if approval.get("immutable_audit_trail_ref_present") is not True:
        reasons.append("missing_operator_approval:audit")
    if approval.get("authorizes_execution") is not False:
        reasons.append("forbidden_execution:approval_execution")
    if approval.get("authorizes_trading_operations") is not False:
        reasons.append("forbidden_execution:approval_trading")

    plans_raw = artifact.get("rollback_plans")
    if not isinstance(plans_raw, list):
        reasons.append("ambiguous_rollback_execution:plans")
        plans_raw = []
    plans: dict[str, dict[str, Any]] = {}
    for plan in plans_raw:
        if not isinstance(plan, dict):
            reasons.append("ambiguous_rollback_execution:plan_type")
            continue
        plan_id = plan.get("plan_id")
        if not isinstance(plan_id, str):
            reasons.append("ambiguous_rollback_execution:plan_id")
            continue
        plans[plan_id] = plan
        if plan_id not in EXPECTED_ROLLBACK_PLANS:
            reasons.append(f"ambiguous_rollback_execution:unexpected_plan:{plan_id}")
        if plan.get("status") != "preview_ready":
            reasons.append(f"ambiguous_rollback_execution:status:{plan_id}")
        if not plan.get("source_ref"):
            reasons.append(f"inconsistent_deployment_provenance:source:{plan_id}")
        if plan.get("deployment_provenance_status") != "matched":
            reasons.append(f"inconsistent_deployment_provenance:plan:{plan_id}")
        if plan.get("freshness_status") != "fresh":
            reasons.append(f"stale_restore_point:plan:{plan_id}")
        if plan.get("operator_approval_required") is not True:
            reasons.append(f"missing_operator_approval:plan:{plan_id}")
        if plan.get("execution_allowed") is not False:
            reasons.append(f"forbidden_execution:plan:{plan_id}")
        if plan.get("ambiguous_execution_claim_allowed") is not False:
            reasons.append(f"ambiguous_rollback_execution:plan:{plan_id}")
    if set(plans) != EXPECTED_ROLLBACK_PLANS:
        reasons.append("ambiguous_rollback_execution:plan_set")

    dr_raw = artifact.get("dr_restore_boundaries")
    if not isinstance(dr_raw, list):
        reasons.append("stale_restore_point:dr_list")
        dr_raw = []
    dr_boundaries: dict[str, dict[str, Any]] = {}
    for boundary in dr_raw:
        if not isinstance(boundary, dict):
            reasons.append("stale_restore_point:dr_type")
            continue
        boundary_id = boundary.get("boundary_id")
        if not isinstance(boundary_id, str):
            reasons.append("stale_restore_point:dr_id")
            continue
        dr_boundaries[boundary_id] = boundary
        if boundary_id not in EXPECTED_DR_BOUNDARIES:
            reasons.append(f"stale_restore_point:unexpected_dr:{boundary_id}")
        if boundary.get("status") not in {"linked", "preview_ready"}:
            reasons.append(f"stale_restore_point:status:{boundary_id}")
        if boundary.get("restore_point_freshness_status") != "fresh":
            reasons.append(f"stale_restore_point:freshness:{boundary_id}")
        if not boundary.get("source_ref"):
            reasons.append(f"inconsistent_deployment_provenance:dr_source:{boundary_id}")
        for key in ("restore_execution_allowed", "data_restore_execution_allowed", "service_restart_execution_allowed"):
            if boundary.get(key) is not False:
                reasons.append(f"forbidden_execution:dr:{boundary_id}:{key}")
    if set(dr_boundaries) != EXPECTED_DR_BOUNDARIES:
        reasons.append("stale_restore_point:dr_set")

    safety_raw = artifact.get("data_safety_checkpoints")
    if not isinstance(safety_raw, list):
        reasons.append("missing_data_safety_checkpoint:list")
        safety_raw = []
    safety: dict[str, dict[str, Any]] = {}
    for checkpoint in safety_raw:
        if not isinstance(checkpoint, dict):
            reasons.append("missing_data_safety_checkpoint:entry_type")
            continue
        checkpoint_id = checkpoint.get("checkpoint_id")
        if not isinstance(checkpoint_id, str):
            reasons.append("missing_data_safety_checkpoint:id")
            continue
        safety[checkpoint_id] = checkpoint
        if checkpoint_id not in EXPECTED_SAFETY:
            reasons.append(f"missing_data_safety_checkpoint:unexpected:{checkpoint_id}")
        if checkpoint.get("status") not in {"fresh", "matched"}:
            reasons.append(f"missing_data_safety_checkpoint:status:{checkpoint_id}")
        if checkpoint.get("deployment_provenance_status") != "matched":
            reasons.append(f"inconsistent_deployment_provenance:safety:{checkpoint_id}")
        if checkpoint.get("execution_allowed") is not False:
            reasons.append(f"forbidden_execution:safety:{checkpoint_id}")
    if set(safety) != EXPECTED_SAFETY:
        reasons.append("missing_data_safety_checkpoint:set")

    triggers_raw = artifact.get("incident_freeze_triggers")
    if not isinstance(triggers_raw, list):
        reasons.append("missing_incident_freeze_trigger:list")
        triggers_raw = []
    triggers: dict[str, dict[str, Any]] = {}
    for trigger in triggers_raw:
        if not isinstance(trigger, dict):
            reasons.append("missing_incident_freeze_trigger:entry_type")
            continue
        trigger_id = trigger.get("trigger_id")
        if not isinstance(trigger_id, str):
            reasons.append("missing_incident_freeze_trigger:id")
            continue
        triggers[trigger_id] = trigger
        if trigger_id not in EXPECTED_TRIGGERS:
            reasons.append(f"missing_incident_freeze_trigger:unexpected:{trigger_id}")
        if trigger.get("status") != "documented":
            reasons.append(f"missing_incident_freeze_trigger:status:{trigger_id}")
        if trigger.get("manual_approval_required") is not True:
            reasons.append(f"missing_operator_approval:trigger:{trigger_id}")
        if trigger.get("automatic_remediation_allowed") is not False:
            reasons.append(f"forbidden_execution:trigger_auto:{trigger_id}")
        if trigger.get("execution_allowed") is not False:
            reasons.append(f"forbidden_execution:trigger_execution:{trigger_id}")
    if set(triggers) != EXPECTED_TRIGGERS:
        reasons.append("missing_incident_freeze_trigger:set")

    return reasons


base_status = classify_status(collect_reasons(payload))
if base_status != READY_STATUS:
    fail(f"base artifact status mismatch: {base_status}")

cases = payload.get("readiness_cases")
if not isinstance(cases, list):
    fail("readiness_cases must be a list")
seen_cases: set[str] = set()
for case in cases:
    if not isinstance(case, dict):
        fail("readiness case entries must be objects")
    case_id = case.get("case_id")
    expected = case.get("expected_status")
    if not isinstance(case_id, str) or not isinstance(expected, str):
        fail("readiness case missing id/status")
    if case_id in seen_cases:
        fail(f"duplicate readiness case: {case_id}")
    seen_cases.add(case_id)
    if case_id not in EXPECTED_CASES:
        fail(f"unexpected readiness case: {case_id}")
    snapshot = apply_case(payload, case)
    actual = classify_status(collect_reasons(snapshot))
    if actual != expected:
        fail(f"case {case_id} expected {expected}, got {actual}")
if seen_cases != EXPECTED_CASES:
    fail(f"readiness case set mismatch: {sorted(seen_cases)}")

negative_selftests = 0
if selftest:
    ambiguous = copy.deepcopy(payload)
    ambiguous["rollback_plans"][0]["ambiguous_execution_claim_allowed"] = True
    if classify_status(collect_reasons(ambiguous)) == "fail_closed_ambiguous_rollback_execution":
        negative_selftests += 1
    else:
        fail("negative self-test unexpectedly allowed ambiguous rollback execution")

    stale = copy.deepcopy(payload)
    stale["dr_restore_boundaries"][1]["restore_point_freshness_status"] = "stale"
    if classify_status(collect_reasons(stale)) == "fail_closed_stale_restore_point":
        negative_selftests += 1
    else:
        fail("negative self-test unexpectedly allowed stale restore point")

    provenance = copy.deepcopy(payload)
    provenance["rollback_plans"][1]["deployment_provenance_status"] = "mismatched"
    if classify_status(collect_reasons(provenance)) == "fail_closed_inconsistent_deployment_provenance":
        negative_selftests += 1
    else:
        fail("negative self-test unexpectedly allowed inconsistent deployment provenance")

    execution = copy.deepcopy(payload)
    execution["dr_restore_boundaries"][2]["restore_execution_allowed"] = True
    if classify_status(collect_reasons(execution)) == "fail_closed_forbidden_execution":
        negative_selftests += 1
    else:
        fail("negative self-test unexpectedly allowed restore execution")

print(
    "v30_rollback_disaster_recovery_execution_boundary=pass "
    f"rollback_plans={len(EXPECTED_ROLLBACK_PLANS)} "
    f"dr_restore_boundaries={len(EXPECTED_DR_BOUNDARIES)} "
    f"data_safety_checkpoints={len(EXPECTED_SAFETY)} "
    f"incident_freeze_triggers={len(EXPECTED_TRIGGERS)} "
    f"readiness_cases={len(EXPECTED_CASES)} "
    f"required_false_flags={len(REQUIRED_FALSE_FLAGS)} "
    f"negative_selftest={negative_selftests}"
)
PY
