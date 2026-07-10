#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

ARTIFACT_PATH="${NTPRO_V290_CANARY_ROLLBACK_DR_ARTIFACT:-docs/rust-cutover/release/v0_29_0_canary_rollback_dr_preflight_readiness_artifact.json}"
CONTRACT_PATH="${NTPRO_V290_CANARY_ROLLBACK_DR_CONTRACT:-docs/rust-cutover/release/v0_29_0_canary_rollback_dr_preflight_readiness.md}"
TASK_PATH="${NTPRO_V290_CANARY_ROLLBACK_DR_TASK:-docs/rust-cutover/tasks/V290-008.md}"
EVIDENCE_PATH="${NTPRO_V290_CANARY_ROLLBACK_DR_EVIDENCE:-docs/rust-cutover/evidence/V290-008.md}"
MATRIX_PATH="${NTPRO_V290_CANARY_ROLLBACK_DR_MATRIX:-docs/rust-cutover/release/v0_29_0_backend_production_readiness_matrix.json}"
BOUNDARY_CONTRACT_PATH="${NTPRO_V290_CANARY_ROLLBACK_DR_BOUNDARY_CONTRACT:-docs/rust-cutover/release/v0_29_0_backend_production_readiness_boundary_contract.md}"
V25_DR_PATH="${NTPRO_V290_CANARY_ROLLBACK_DR_V25_DR:-docs/rust-cutover/release/v0_25_0_dr_preview_drill_evidence.md}"
V26_RUNBOOK_PATH="${NTPRO_V290_CANARY_ROLLBACK_DR_V26_RUNBOOK:-docs/rust-cutover/release/v0_26_0_upgrade_rollback_runbook_evidence.md}"
V280_ARTIFACT_PATH="${NTPRO_V290_CANARY_ROLLBACK_DR_V280_ARTIFACT:-docs/rust-cutover/release/v0_28_0_deployment_orchestration_runtime_artifact.json}"
RUNBOOK_ARTIFACT_PATH="${NTPRO_V290_CANARY_ROLLBACK_DR_RUNBOOK_ARTIFACT:-docs/rust-cutover/release/v0_29_0_deployment_config_runbook_production_readiness_artifact.json}"
MONITORING_ARTIFACT_PATH="${NTPRO_V290_CANARY_ROLLBACK_DR_MONITORING_ARTIFACT:-docs/rust-cutover/release/v0_29_0_monitoring_alert_incident_production_readiness_artifact.json}"
INTAKE_PATH="${NTPRO_V290_CANARY_ROLLBACK_DR_INTAKE:-docs/rust-cutover/release/v0_29_0_intake_gate.md}"
SELFTEST="${NTPRO_V290_CANARY_ROLLBACK_DR_SELFTEST:-1}"

fail() {
  echo "v29 canary rollback DR preflight readiness failed: $*" >&2
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

for path in "$ARTIFACT_PATH" "$CONTRACT_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$MATRIX_PATH" "$BOUNDARY_CONTRACT_PATH" "$V25_DR_PATH" "$V26_RUNBOOK_PATH" "$V280_ARTIFACT_PATH" "$RUNBOOK_ARTIFACT_PATH" "$MONITORING_ARTIFACT_PATH" "$INTAKE_PATH"; do
  require_file "$path"
done

require_contains "$TASK_PATH" "GitHub issue: \`#934\`"
require_contains "$EVIDENCE_PATH" "Task: \`V290-008\` / GitHub issue \`#934\`"
require_contains "$INTAKE_PATH" "start_gate_status = satisfied"
require_contains "$BOUNDARY_CONTRACT_PATH" "contract_version = ntpro.v290.backend_production_readiness_boundary.v1"
require_contains "$CONTRACT_PATH" "contract_version = ntpro.v290.canary_rollback_dr_preflight_readiness.v1"
require_contains "$CONTRACT_PATH" "schema_version = ntpro.v290.canary_rollback_dr_preflight_readiness_artifact.v1"
require_contains "$CONTRACT_PATH" "preflight_mode = source_controlled_preflight_only"
require_contains "$CONTRACT_PATH" "canary_execution_allowed = false"
require_contains "$CONTRACT_PATH" "rollback_execution_allowed = false"
require_contains "$CONTRACT_PATH" "dr_execution_allowed = false"
require_contains "$CONTRACT_PATH" "release stage = scripts/ai/verify_release.sh v29-canary-rollback-dr-preflight-readiness"

ARTIFACT_PATH="$ARTIFACT_PATH" MATRIX_PATH="$MATRIX_PATH" V280_ARTIFACT_PATH="$V280_ARTIFACT_PATH" RUNBOOK_ARTIFACT_PATH="$RUNBOOK_ARTIFACT_PATH" MONITORING_ARTIFACT_PATH="$MONITORING_ARTIFACT_PATH" SELFTEST="$SELFTEST" python3 <<'PY'
from __future__ import annotations

import copy
import json
import os
from pathlib import Path
from typing import Any

artifact_path = Path(os.environ["ARTIFACT_PATH"])
matrix_path = Path(os.environ["MATRIX_PATH"])
v280_artifact_path = Path(os.environ["V280_ARTIFACT_PATH"])
runbook_artifact_path = Path(os.environ["RUNBOOK_ARTIFACT_PATH"])
monitoring_artifact_path = Path(os.environ["MONITORING_ARTIFACT_PATH"])
selftest = os.environ.get("SELFTEST", "1") != "0"

SCHEMA_VERSION = "ntpro.v290.canary_rollback_dr_preflight_readiness_artifact.v1"
CONTRACT_VERSION = "ntpro.v290.canary_rollback_dr_preflight_readiness.v1"
MODULE_ID = "canary_rollback_dr_preflight_readiness"
DEPENDENCIES = {"V290-000", "V290-001", "V290-006", "V290-007", "V280-004", "V260-005", "V250-005", "v0.28.1-release-evidence"}
EXPECTED_CASES = [
    "canary_rollback_dr.preflight.ready.allowed.001",
    "canary_rollback_dr.preflight.missing_canary_eligibility.fail_closed.001",
    "canary_rollback_dr.preflight.missing_owner_approval.fail_closed.001",
    "canary_rollback_dr.preflight.stale_dr_evidence.fail_closed.001",
    "canary_rollback_dr.preflight.unsafe_rollback.fail_closed.001",
    "canary_rollback_dr.preflight.ambiguous_go_live.fail_closed.001",
    "canary_rollback_dr.preflight.forbidden_execution.fail_closed.001",
    "canary_rollback_dr.preflight.forbidden_boundary.fail_closed.001",
]
EXPECTED_CHECKS = ["canary-eligibility", "rollback-trigger-catalog", "dr-drill-evidence"]
BOUNDARY_FALSE_FLAGS = [
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
    "product_grade_trading_terminal_claim",
    "canary_execution_allowed",
    "production_canary_execution_allowed",
    "rollback_execution_allowed",
    "production_rollback_execution_allowed",
    "dr_execution_allowed",
    "data_restore_execution_allowed",
    "service_restart_execution_allowed",
    "ambiguous_go_live_claim_allowed",
]


def fail(message: str) -> None:
    raise SystemExit(f"v29 canary rollback DR preflight readiness failed: {message}")


def merge(base: Any, override: Any) -> Any:
    if isinstance(base, dict) and isinstance(override, dict):
        result = copy.deepcopy(base)
        for key, value in override.items():
            result[key] = merge(result.get(key), value)
        return result
    return copy.deepcopy(override)


def apply_case(artifact: dict[str, Any], case: dict[str, Any]) -> dict[str, Any]:
    result = copy.deepcopy(artifact)
    for key in ("canary_plan", "rollback_plan", "dr_drill", "boundary_flags"):
        override = case.get(f"{key}_override")
        if override:
            result[key] = merge(result[key], override)
    for check_id, override in (case.get("preflight_check_overrides") or {}).items():
        for check in result["preflight_checks"]:
            if check.get("check_id") == check_id:
                check.update(merge(check, override))
                break
    return result


def classify_status(reasons: list[str]) -> str:
    if any(reason.startswith("missing_canary_eligibility") for reason in reasons):
        return "fail_closed_missing_canary_eligibility"
    if any(reason.startswith("missing_owner_approval") for reason in reasons):
        return "fail_closed_missing_owner_approval"
    if any(reason.startswith("stale_dr_evidence") for reason in reasons):
        return "fail_closed_stale_dr_evidence"
    if any(reason.startswith("unsafe_rollback_plan") for reason in reasons):
        return "fail_closed_unsafe_rollback_plan"
    if any(reason.startswith("ambiguous_go_live_claim") for reason in reasons):
        return "fail_closed_ambiguous_go_live_claim"
    if any(reason.startswith("forbidden_execution") for reason in reasons):
        return "fail_closed_forbidden_execution"
    if any(reason.startswith("forbidden_boundary") for reason in reasons):
        return "fail_closed_forbidden_operation_boundary"
    if reasons:
        return "fail_closed_canary_rollback_dr_preflight_violation"
    return "canary_rollback_dr_preflight_ready"


def classify_artifact(artifact: dict[str, Any]) -> dict[str, Any]:
    reasons: list[str] = []
    if artifact.get("schema_version") != SCHEMA_VERSION or artifact.get("contract_version") != CONTRACT_VERSION:
        reasons.append("missing_owner_approval:schema_contract")
    if artifact.get("task_id") != "V290-008" or artifact.get("github_issue") != 934:
        reasons.append("missing_owner_approval:task_identity")
    if artifact.get("release_scope") != "backend_production_readiness_foundation_only":
        reasons.append("missing_owner_approval:release_scope")
    if artifact.get("backend_module") != MODULE_ID or artifact.get("backend_module_status") != "production_ready_evidence":
        reasons.append("missing_owner_approval:module")
    if artifact.get("readiness_mode") != "deterministic_readiness_replay":
        reasons.append("missing_owner_approval:readiness_mode")
    if artifact.get("preflight_mode") != "source_controlled_preflight_only":
        reasons.append("forbidden_execution:preflight_mode")
    if set(artifact.get("dependency_contracts") or []) != DEPENDENCIES:
        reasons.append("missing_owner_approval:dependencies")

    canary = artifact.get("canary_plan") or {}
    if canary.get("source_ref") != str(monitoring_artifact_path):
        reasons.append("missing_canary_eligibility:source_ref")
    if canary.get("eligibility_status") != "eligible":
        reasons.append("missing_canary_eligibility:eligibility_status")
    if not isinstance(canary.get("blast_radius_percent"), int) or not isinstance(canary.get("max_blast_radius_percent"), int) or canary["blast_radius_percent"] > canary["max_blast_radius_percent"]:
        reasons.append("missing_canary_eligibility:blast_radius")
    if canary.get("owner_approval_status") != "owner_approved" or not canary.get("approval_ref"):
        reasons.append("missing_owner_approval:canary")
    if canary.get("source_provenance_status") != "linked" or canary.get("freshness_status") != "fresh":
        reasons.append("missing_canary_eligibility:provenance_freshness")
    if canary.get("backend_go_live_claim_allowed") is not False:
        reasons.append("ambiguous_go_live_claim:canary")
    if canary.get("canary_execution_allowed") is not False:
        reasons.append("forbidden_execution:canary")

    rollback = artifact.get("rollback_plan") or {}
    if rollback.get("source_ref") != str(v280_artifact_path) or rollback.get("runbook_ref") != str(runbook_artifact_path):
        reasons.append("unsafe_rollback_plan:source_or_runbook")
    if rollback.get("trigger_catalog_status") != "documented":
        reasons.append("unsafe_rollback_plan:trigger_catalog")
    if rollback.get("owner_approval_status") != "owner_approved" or not rollback.get("approval_ref"):
        reasons.append("missing_owner_approval:rollback")
    if rollback.get("source_provenance_status") != "linked" or rollback.get("freshness_status") != "fresh":
        reasons.append("unsafe_rollback_plan:provenance_freshness")
    if rollback.get("unsafe_rollback_plan_allowed") is not False:
        reasons.append("unsafe_rollback_plan:allowed")
    if rollback.get("rollback_execution_allowed") is not False:
        reasons.append("forbidden_execution:rollback")
    if rollback.get("ambiguous_go_live_claim_allowed") is not False:
        reasons.append("ambiguous_go_live_claim:rollback")

    dr = artifact.get("dr_drill") or {}
    if dr.get("source_ref") != "docs/rust-cutover/release/v0_25_0_dr_preview_drill_evidence.md":
        reasons.append("stale_dr_evidence:source_ref")
    if dr.get("scenario_matrix_status") != "covered":
        reasons.append("stale_dr_evidence:scenario_matrix")
    for key in ("freshness_status", "recovery_point_status"):
        if dr.get(key) != "fresh":
            reasons.append(f"stale_dr_evidence:{key}")
    for key in ("source_provenance_status", "snapshot_lineage_status"):
        if dr.get(key) != "linked":
            reasons.append(f"stale_dr_evidence:{key}")
    if dr.get("owner_approval_status") != "owner_approved" or not dr.get("approval_ref"):
        reasons.append("missing_owner_approval:dr")
    for key in ("dr_execution_allowed", "data_restore_execution_allowed", "service_restart_execution_allowed"):
        if dr.get(key) is not False:
            reasons.append(f"forbidden_execution:dr:{key}")

    checks = artifact.get("preflight_checks")
    if not isinstance(checks, list) or [item.get("check_id") for item in checks] != EXPECTED_CHECKS:
        reasons.append("forbidden_execution:preflight_check_set")
        checks = checks if isinstance(checks, list) else []
    for check in checks:
        check_id = check.get("check_id")
        if check.get("status") != "pass":
            reasons.append(f"forbidden_execution:check_status:{check_id}")
        if check.get("preview_only") is not True or check.get("execution_triggered") is not False:
            reasons.append(f"forbidden_execution:check_execution:{check_id}")
        if check.get("operation_effect") != "validated_only":
            reasons.append(f"forbidden_execution:operation_effect:{check_id}")

    boundary = artifact.get("boundary_flags") or {}
    for key in BOUNDARY_FALSE_FLAGS:
        if boundary.get(key) is not False:
            reasons.append(f"forbidden_boundary:{key}")

    status = classify_status(reasons)
    return {"status": status, "fail_closed": status.startswith("fail_closed"), "blocking_reasons": reasons}


artifact = json.loads(artifact_path.read_text(encoding="utf-8"))
v280_artifact = json.loads(v280_artifact_path.read_text(encoding="utf-8"))
if v280_artifact.get("schema_version") != "ntpro.v280.deployment_orchestration_runtime_artifact.v1":
    fail("v28 deployment orchestration artifact schema mismatch")
runbook_artifact = json.loads(runbook_artifact_path.read_text(encoding="utf-8"))
if runbook_artifact.get("schema_version") != "ntpro.v290.deployment_config_runbook_production_readiness_artifact.v1":
    fail("deployment config runbook artifact schema mismatch")
monitoring_artifact = json.loads(monitoring_artifact_path.read_text(encoding="utf-8"))
if monitoring_artifact.get("schema_version") != "ntpro.v290.monitoring_alert_incident_production_readiness_artifact.v1":
    fail("monitoring alert incident artifact schema mismatch")

matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
module = next((item for item in matrix.get("module_readiness", []) if item.get("module_id") == MODULE_ID), None)
if not module:
    fail("matrix missing canary_rollback_dr_preflight_readiness")
if module.get("classification") != "production-ready" or module.get("issue") != 934:
    fail("canary rollback DR matrix entry mismatch")
if module.get("evidence_path") != "docs/rust-cutover/evidence/V290-008.md":
    fail("canary rollback DR matrix evidence path mismatch")
if module.get("verification_command") != "scripts/ai/verify_release.sh v29-canary-rollback-dr-preflight-readiness":
    fail("canary rollback DR matrix verification command mismatch")

cases = artifact.get("readiness_cases")
if not isinstance(cases, list) or [case.get("case_id") for case in cases] != EXPECTED_CASES:
    fail("readiness cases mismatch")
allowed = fail_closed = 0
for case in cases:
    actual = classify_artifact(apply_case(artifact, case))
    if actual["status"] != case.get("expected_status"):
        fail(f"{case.get('case_id')}: expected {case.get('expected_status')} got {actual}")
    if actual["fail_closed"]:
        fail_closed += 1
    else:
        allowed += 1
if allowed != 1 or fail_closed != 7:
    fail(f"unexpected case counts: allowed={allowed} fail_closed={fail_closed}")

if selftest:
    bad = copy.deepcopy(artifact)
    bad["boundary_flags"]["production_canary_execution_allowed"] = True
    if classify_artifact(bad)["status"] != "fail_closed_forbidden_operation_boundary":
        fail("negative self-test unexpectedly allowed production canary execution")

print(
    "v29_canary_rollback_dr_preflight_readiness=pass "
    f"cases={len(cases)} "
    f"allowed={allowed} "
    f"fail_closed={fail_closed} "
    f"preflight_checks={len(artifact['preflight_checks'])} "
    f"boundary_flags={len(BOUNDARY_FALSE_FLAGS)} "
    "negative_selftest=1"
)
PY
