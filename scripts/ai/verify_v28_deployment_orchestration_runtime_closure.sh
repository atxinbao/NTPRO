#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

ARTIFACT_PATH="${NTPRO_V280_DEPLOYMENT_ORCHESTRATION_ARTIFACT:-docs/rust-cutover/release/v0_28_0_deployment_orchestration_runtime_artifact.json}"
CONTRACT_PATH="${NTPRO_V280_DEPLOYMENT_ORCHESTRATION_CONTRACT:-docs/rust-cutover/release/v0_28_0_deployment_orchestration_runtime_closure.md}"
TASK_PATH="${NTPRO_V280_DEPLOYMENT_ORCHESTRATION_TASK:-docs/rust-cutover/tasks/V280-004.md}"
EVIDENCE_PATH="${NTPRO_V280_DEPLOYMENT_ORCHESTRATION_EVIDENCE:-docs/rust-cutover/evidence/V280-004.md}"
MATRIX_PATH="${NTPRO_V280_DEPLOYMENT_ORCHESTRATION_MATRIX:-docs/rust-cutover/release/v0_28_0_backend_closure_readiness_matrix.json}"
FOUNDATION_PATH="${NTPRO_V280_DEPLOYMENT_ORCHESTRATION_FOUNDATION:-docs/rust-cutover/release/v0_27_0_deployment_orchestration_foundation.md}"
RUNBOOK_PATH="${NTPRO_V280_DEPLOYMENT_ORCHESTRATION_RUNBOOK:-docs/rust-cutover/release/v0_26_0_upgrade_rollback_runbook_evidence.md}"
SELFTEST="${NTPRO_V280_DEPLOYMENT_ORCHESTRATION_SELFTEST:-1}"

fail() {
  echo "v28 deployment orchestration runtime closure failed: $*" >&2
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

for path in "$ARTIFACT_PATH" "$CONTRACT_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$MATRIX_PATH" "$FOUNDATION_PATH" "$RUNBOOK_PATH"; do
  require_file "$path"
done

require_contains "$TASK_PATH" "GitHub issue: \`#897\`"
require_contains "$EVIDENCE_PATH" "Task: \`V280-004\` / GitHub issue \`#897\`"
require_contains "$FOUNDATION_PATH" "orchestration_scope = deployment_upgrade_rollback_orchestration_foundation_only"
require_contains "$RUNBOOK_PATH" "runbook_artifact_scope = upgrade_rollback_runbook_preview_only"
require_contains "$CONTRACT_PATH" "contract_version = ntpro.v280.deployment_orchestration_runtime_closure.v1"
require_contains "$CONTRACT_PATH" "schema_version = ntpro.v280.deployment_orchestration_runtime_artifact.v1"
require_contains "$CONTRACT_PATH" "backend_module_status = runtime_closed"
require_contains "$CONTRACT_PATH" "release stage = scripts/ai/verify_release.sh v28-deployment-orchestration-runtime-closure"

ARTIFACT_PATH="$ARTIFACT_PATH" MATRIX_PATH="$MATRIX_PATH" SELFTEST="$SELFTEST" python3 <<'PY'
from __future__ import annotations

import copy
import json
import os
from pathlib import Path
from typing import Any

artifact_path = Path(os.environ["ARTIFACT_PATH"])
matrix_path = Path(os.environ["MATRIX_PATH"])
selftest = os.environ.get("SELFTEST", "1") != "0"

SCHEMA_VERSION = "ntpro.v280.deployment_orchestration_runtime_artifact.v1"
CONTRACT_VERSION = "ntpro.v280.deployment_orchestration_runtime_closure.v1"
DEPENDENCIES = {
    "V280-001",
    "V280-002",
    "V280-003",
    "V270-004",
    "V260-004",
    "V260-005",
    "V271-006",
}
EXPECTED_OPERATIONS = ["deploy", "upgrade", "rollback", "blocked", "degraded", "closed"]
EXPECTED_CASES = [
    "deployment_orchestration.plan.allowed.001",
    "deployment_orchestration.missing_approval.fail_closed.001",
    "deployment_orchestration.stale_runbook.fail_closed.001",
    "deployment_orchestration.source_drift.fail_closed.001",
    "deployment_orchestration.automatic_remediation.fail_closed.001",
    "deployment_orchestration.production_execution.fail_closed.001",
]
BOUNDARY_FALSE_FLAGS = [
    "production_deployment_execution_allowed",
    "deployment_execution_allowed",
    "production_rollback_execution_allowed",
    "rollback_execution_allowed",
    "production_order_mutation_allowed",
    "default_submit_allowed",
    "adapter_send_allowed",
    "live_exchange_request_allowed",
    "retry_scheduler_enabled",
    "automatic_remediation_allowed",
    "dashboard_operation_controls_enabled",
    "dashboard_trading_controls_enabled",
    "admin_workbench_operation_controls_enabled",
    "admin_workbench_trading_controls_enabled",
    "trading_operation_allowed",
    "manual_operation_submit_allowed",
    "product_grade_trading_terminal_claim",
]


def fail(message: str) -> None:
    raise SystemExit(f"v28 deployment orchestration runtime closure failed: {message}")


def non_empty(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


def merge(base: dict[str, Any], override: dict[str, Any] | None) -> dict[str, Any]:
    merged = copy.deepcopy(base)
    if override:
        for key, value in override.items():
            merged[key] = value
    return merged


def classify_status(reasons: list[str]) -> str:
    if any(reason.startswith("missing_approval") for reason in reasons):
        return "fail_closed_missing_approval"
    if any(reason.startswith("stale_runbook") for reason in reasons):
        return "fail_closed_stale_runbook"
    if any(reason.startswith("source_drift") for reason in reasons):
        return "fail_closed_source_drift"
    if any(reason.startswith("forbidden_automatic_remediation") for reason in reasons):
        return "fail_closed_forbidden_automatic_remediation"
    if any(reason.startswith("forbidden_production_execution") for reason in reasons):
        return "fail_closed_forbidden_production_execution"
    if any(reason.startswith("forbidden") for reason in reasons):
        return "fail_closed_forbidden_operation_boundary"
    if reasons:
        return "fail_closed_orchestration_violation"
    return "orchestration_plan_replay_ready"


def apply_case(artifact: dict[str, Any], case: dict[str, Any]) -> dict[str, Any]:
    result = copy.deepcopy(artifact)
    result["owner_approval"] = merge(result["owner_approval"], case.get("owner_approval_override"))
    result["runbook_provenance"] = merge(result["runbook_provenance"], case.get("runbook_provenance_override"))
    result["source_provenance"] = merge(result["source_provenance"], case.get("source_provenance_override"))
    result["boundary_flags"] = merge(result["boundary_flags"], case.get("boundary_flags_override"))
    transition_overrides = case.get("transition_overrides") or {}
    for transition in result["orchestration_plan"]["state_transitions"]:
        override = transition_overrides.get(transition["transition_id"])
        if override:
            transition.update(override)
    return result


def classify_artifact(artifact: dict[str, Any]) -> dict[str, Any]:
    reasons: list[str] = []
    if artifact.get("schema_version") != SCHEMA_VERSION:
        reasons.append("schema_version_mismatch")
    if artifact.get("contract_version") != CONTRACT_VERSION:
        reasons.append("contract_version_mismatch")
    if artifact.get("task_id") != "V280-004" or artifact.get("github_issue") != 897:
        reasons.append("task_identity_mismatch")
    if artifact.get("backend_module") != "deployment_upgrade_rollback_orchestration_runtime_closure":
        reasons.append("backend_module_mismatch")
    if artifact.get("backend_module_status") != "runtime_closed":
        reasons.append("backend_module_not_runtime_closed")
    if set(artifact.get("dependency_contracts") or []) != DEPENDENCIES:
        reasons.append("dependency_contracts_mismatch")

    policy = artifact.get("operation_preview_policy")
    if not isinstance(policy, dict):
        reasons.append("missing_policy")
        policy = {}
    for key in (
        "owner_approval_required",
        "runbook_provenance_required",
        "source_provenance_required",
        "deterministic_replay_required",
    ):
        if policy.get(key) is not True:
            reasons.append(f"missing_policy:{key}")
    if policy.get("production_execution_allowed") is not False:
        reasons.append("forbidden_production_execution:policy")
    if policy.get("orchestration_mode") != "deterministic_preview_replay":
        reasons.append(f"orchestration_mode_mismatch:{policy.get('orchestration_mode')}")

    approval = artifact.get("owner_approval")
    if not isinstance(approval, dict):
        reasons.append("missing_approval:object")
        approval = {}
    if approval.get("required") is not True:
        reasons.append("missing_approval:not_required")
    if approval.get("status") != "approved":
        reasons.append(f"missing_approval:status:{approval.get('status')}")
    if not non_empty(approval.get("approval_ref")):
        reasons.append("missing_approval:approval_ref")

    runbook = artifact.get("runbook_provenance")
    if not isinstance(runbook, dict):
        reasons.append("stale_runbook:missing_object")
        runbook = {}
    if runbook.get("required") is not True:
        reasons.append("stale_runbook:not_required")
    if runbook.get("freshness_status") != "fresh":
        reasons.append(f"stale_runbook:freshness:{runbook.get('freshness_status')}")
    if runbook.get("lineage_status") != "linked":
        reasons.append(f"stale_runbook:lineage:{runbook.get('lineage_status')}")
    for key in ("runbook_ref", "lineage_ref", "runbook_digest", "last_verified_at", "release_tag"):
        if not non_empty(runbook.get(key)):
            reasons.append(f"stale_runbook:missing:{key}")
    if runbook.get("release_tag") != artifact.get("dependency_release_tag"):
        reasons.append("stale_runbook:release_tag_mismatch")

    source = artifact.get("source_provenance")
    if not isinstance(source, dict):
        reasons.append("source_drift:missing_object")
        source = {}
    if source.get("required") is not True:
        reasons.append("source_drift:not_required")
    if source.get("drift_status") != "no_drift":
        reasons.append(f"source_drift:status:{source.get('drift_status')}")
    if source.get("source_ref") != artifact.get("source_ref"):
        reasons.append("source_drift:source_ref_mismatch")
    for key in ("source_ref", "source_digest", "v27_foundation_ref", "v27_foundation_digest"):
        if not non_empty(source.get(key)):
            reasons.append(f"source_drift:missing:{key}")

    gate = artifact.get("release_gate_evidence")
    if not isinstance(gate, dict):
        reasons.append("source_drift:missing_release_gate")
        gate = {}
    if gate.get("required") is not True or gate.get("status") != "passed":
        reasons.append(f"source_drift:release_gate:{gate.get('status')}")
    if gate.get("release_tag") != artifact.get("dependency_release_tag"):
        reasons.append("source_drift:release_gate_tag_mismatch")

    boundary = artifact.get("boundary_flags")
    if not isinstance(boundary, dict):
        reasons.append("forbidden:missing_boundary_flags")
        boundary = {}
    for key in BOUNDARY_FALSE_FLAGS:
        if key not in boundary:
            reasons.append(f"forbidden:missing_required_false:{key}")
        elif boundary.get(key) is not False:
            if key in {
                "production_deployment_execution_allowed",
                "deployment_execution_allowed",
                "production_rollback_execution_allowed",
                "rollback_execution_allowed",
            }:
                reasons.append(f"forbidden_production_execution:boundary:{key}")
            elif key == "automatic_remediation_allowed":
                reasons.append(f"forbidden_automatic_remediation:boundary:{key}")
            else:
                reasons.append(f"forbidden:boundary:{key}")

    plan = artifact.get("orchestration_plan")
    if not isinstance(plan, dict):
        fail("orchestration_plan must be an object")
    if plan.get("mode") != "deterministic_preview_replay":
        reasons.append(f"orchestration_mode_mismatch:{plan.get('mode')}")
    if plan.get("approval_ref") != approval.get("approval_ref"):
        reasons.append("missing_approval:plan_ref_mismatch")
    if plan.get("runbook_ref") != runbook.get("runbook_ref"):
        reasons.append("stale_runbook:plan_ref_mismatch")
    if plan.get("source_ref") != source.get("source_ref"):
        reasons.append("source_drift:plan_ref_mismatch")

    transitions = plan.get("state_transitions")
    if not isinstance(transitions, list) or not transitions:
        fail("state_transitions must be a non-empty list")
    operations = [transition.get("operation") for transition in transitions if isinstance(transition, dict)]
    if operations != EXPECTED_OPERATIONS:
        reasons.append(f"state_transition_sequence_mismatch:{operations}")
    previous_to_status: str | None = None
    for transition in transitions:
        if not isinstance(transition, dict):
            fail("state transition must be an object")
        operation = str(transition.get("operation") or "unknown")
        if not non_empty(transition.get("from_status")) or not non_empty(transition.get("to_status")):
            reasons.append(f"state_transition_missing_status:{operation}")
        if previous_to_status is not None and transition.get("from_status") != previous_to_status:
            reasons.append(f"state_transition_chain_mismatch:{operation}")
        previous_to_status = str(transition.get("to_status"))
        if transition.get("preview_only") is not True:
            reasons.append(f"forbidden_production_execution:not_preview:{operation}")
        for key in ("requires_owner_approval", "requires_runbook_provenance", "requires_source_provenance"):
            if transition.get(key) is not True:
                reasons.append(f"state_transition_missing_requirement:{operation}:{key}")
        if transition.get("execution_triggered") is not False:
            reasons.append(f"forbidden_production_execution:execution_triggered:{operation}")
        if transition.get("automatic_remediation_triggered") is not False:
            reasons.append(f"forbidden_automatic_remediation:transition:{operation}")
        for key in (
            "trading_operation_triggered",
            "adapter_send_requested",
            "live_exchange_request_requested",
            "retry_scheduled",
        ):
            if transition.get(key) is not False:
                reasons.append(f"forbidden:transition:{operation}:{key}")
        if transition.get("operation_effect") != "validated_only":
            effect = transition.get("operation_effect")
            if effect == "automatic_remediation_triggered":
                reasons.append(f"forbidden_automatic_remediation:effect:{operation}")
            elif effect == "executed":
                reasons.append(f"forbidden_production_execution:effect:{operation}")
            else:
                reasons.append(f"forbidden:effect:{operation}:{effect}")

    return {
        "status": classify_status(reasons),
        "fail_closed": bool(reasons),
        "blocking_reasons": reasons,
    }


artifact = json.loads(artifact_path.read_text(encoding="utf-8"))
matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
module = next((item for item in matrix.get("module_readiness", []) if item.get("module_id") == "deployment_upgrade_rollback_orchestration_runtime_closure"), None)
if not module:
    fail("matrix missing deployment_upgrade_rollback_orchestration_runtime_closure")
if module.get("classification") != "runtime-closed":
    fail("deployment orchestration matrix entry must be runtime-closed")
if module.get("evidence_path") != "docs/rust-cutover/evidence/V280-004.md":
    fail("deployment orchestration matrix evidence path mismatch")
if module.get("verification_command") != "scripts/ai/verify_release.sh v28-deployment-orchestration-runtime-closure":
    fail("deployment orchestration matrix verification command mismatch")

cases = artifact.get("orchestration_replay_cases")
if not isinstance(cases, list) or [case.get("case_id") for case in cases] != EXPECTED_CASES:
    fail("orchestration replay cases mismatch")
allowed = 0
fail_closed = 0
for case in cases:
    actual = classify_artifact(apply_case(artifact, case))
    if actual["status"] != case.get("expected_status"):
        fail(f"{case.get('case_id')}: expected {case.get('expected_status')} got {actual}")
    if actual["fail_closed"]:
        fail_closed += 1
    else:
        allowed += 1
if allowed != 1 or fail_closed != 5:
    fail(f"unexpected case counts: allowed={allowed} fail_closed={fail_closed}")

if selftest:
    opened = copy.deepcopy(artifact)
    opened["boundary_flags"]["automatic_remediation_allowed"] = True
    if classify_artifact(opened)["status"] != "fail_closed_forbidden_automatic_remediation":
        fail("negative self-test unexpectedly allowed automatic_remediation_allowed")

print(
    "v28_deployment_orchestration_runtime_closure=pass "
    f"cases={len(cases)} allowed={allowed} fail_closed={fail_closed} "
    f"transitions={len(artifact['orchestration_plan']['state_transitions'])} "
    f"boundary_flags={len(BOUNDARY_FALSE_FLAGS)} negative_selftest={int(selftest)}"
)
PY
