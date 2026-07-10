#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

ARTIFACT_PATH="${NTPRO_V290_DEPLOYMENT_CONFIG_RUNBOOK_ARTIFACT:-docs/rust-cutover/release/v0_29_0_deployment_config_runbook_production_readiness_artifact.json}"
CONTRACT_PATH="${NTPRO_V290_DEPLOYMENT_CONFIG_RUNBOOK_CONTRACT:-docs/rust-cutover/release/v0_29_0_deployment_config_runbook_production_readiness.md}"
TASK_PATH="${NTPRO_V290_DEPLOYMENT_CONFIG_RUNBOOK_TASK:-docs/rust-cutover/tasks/V290-006.md}"
EVIDENCE_PATH="${NTPRO_V290_DEPLOYMENT_CONFIG_RUNBOOK_EVIDENCE:-docs/rust-cutover/evidence/V290-006.md}"
MATRIX_PATH="${NTPRO_V290_DEPLOYMENT_CONFIG_RUNBOOK_MATRIX:-docs/rust-cutover/release/v0_29_0_backend_production_readiness_matrix.json}"
BOUNDARY_CONTRACT_PATH="${NTPRO_V290_DEPLOYMENT_CONFIG_RUNBOOK_BOUNDARY_CONTRACT:-docs/rust-cutover/release/v0_29_0_backend_production_readiness_boundary_contract.md}"
V280_ARTIFACT_PATH="${NTPRO_V290_DEPLOYMENT_CONFIG_RUNBOOK_V280_ARTIFACT:-docs/rust-cutover/release/v0_28_0_deployment_orchestration_runtime_artifact.json}"
INTAKE_PATH="${NTPRO_V290_DEPLOYMENT_CONFIG_RUNBOOK_INTAKE:-docs/rust-cutover/release/v0_29_0_intake_gate.md}"
SELFTEST="${NTPRO_V290_DEPLOYMENT_CONFIG_RUNBOOK_SELFTEST:-1}"

fail() {
  echo "v29 deployment config runbook production readiness failed: $*" >&2
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

for path in "$ARTIFACT_PATH" "$CONTRACT_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$MATRIX_PATH" "$BOUNDARY_CONTRACT_PATH" "$V280_ARTIFACT_PATH" "$INTAKE_PATH"; do
  require_file "$path"
done

require_contains "$TASK_PATH" "GitHub issue: \`#932\`"
require_contains "$EVIDENCE_PATH" "Task: \`V290-006\` / GitHub issue \`#932\`"
require_contains "$INTAKE_PATH" "start_gate_status = satisfied"
require_contains "$BOUNDARY_CONTRACT_PATH" "contract_version = ntpro.v290.backend_production_readiness_boundary.v1"
require_contains "$CONTRACT_PATH" "contract_version = ntpro.v290.deployment_config_runbook_production_readiness.v1"
require_contains "$CONTRACT_PATH" "schema_version = ntpro.v290.deployment_config_runbook_production_readiness_artifact.v1"
require_contains "$CONTRACT_PATH" "deployment_mode = config_readiness_preview_only"
require_contains "$CONTRACT_PATH" "production_deployment_execution_allowed = false"
require_contains "$CONTRACT_PATH" "rollback_execution_allowed = false"
require_contains "$CONTRACT_PATH" "release stage = scripts/ai/verify_release.sh v29-deployment-config-runbook-production-readiness"

ARTIFACT_PATH="$ARTIFACT_PATH" MATRIX_PATH="$MATRIX_PATH" V280_ARTIFACT_PATH="$V280_ARTIFACT_PATH" SELFTEST="$SELFTEST" python3 <<'PY'
from __future__ import annotations

import copy
import json
import os
from pathlib import Path
from typing import Any

artifact_path = Path(os.environ["ARTIFACT_PATH"])
matrix_path = Path(os.environ["MATRIX_PATH"])
v280_artifact_path = Path(os.environ["V280_ARTIFACT_PATH"])
selftest = os.environ.get("SELFTEST", "1") != "0"

SCHEMA_VERSION = "ntpro.v290.deployment_config_runbook_production_readiness_artifact.v1"
CONTRACT_VERSION = "ntpro.v290.deployment_config_runbook_production_readiness.v1"
MODULE_ID = "deployment_config_runbook_production_readiness"
DEPENDENCIES = {"V290-000", "V290-001", "V290-005", "V280-004", "v0.28.1-release-evidence"}
EXPECTED_CASES = [
    "deployment_config_runbook.production_readiness.preview.allowed.001",
    "deployment_config_runbook.production_readiness.missing_config.fail_closed.001",
    "deployment_config_runbook.production_readiness.unsafe_defaults.fail_closed.001",
    "deployment_config_runbook.production_readiness.stale_runbook.fail_closed.001",
    "deployment_config_runbook.production_readiness.ambiguous_claim.fail_closed.001",
    "deployment_config_runbook.production_readiness.forbidden_execution.fail_closed.001",
    "deployment_config_runbook.production_readiness.forbidden_boundary.fail_closed.001",
]
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
    "production_deployment_execution_allowed",
    "rollback_execution_allowed",
]


def fail(message: str) -> None:
    raise SystemExit(f"v29 deployment config runbook production readiness failed: {message}")


def merge(base: Any, override: Any) -> Any:
    if isinstance(base, dict) and isinstance(override, dict):
        result = copy.deepcopy(base)
        for key, value in override.items():
            result[key] = merge(result.get(key), value)
        return result
    return copy.deepcopy(override)


def apply_case(artifact: dict[str, Any], case: dict[str, Any]) -> dict[str, Any]:
    result = copy.deepcopy(artifact)
    if case.get("deployment_config_override"):
        result["deployment_config"] = merge(result["deployment_config"], case["deployment_config_override"])
    if case.get("runbook_override"):
        result["runbook"] = merge(result["runbook"], case["runbook_override"])
    if case.get("boundary_flags_override"):
        result["boundary_flags"] = merge(result["boundary_flags"], case["boundary_flags_override"])
    for transition_id, override in (case.get("transition_overrides") or {}).items():
        for transition in result["preview_transitions"]:
            if transition.get("transition_id") == transition_id:
                transition.update(merge(transition, override))
                break
        else:
            result["preview_transitions"].append({"transition_id": transition_id, **copy.deepcopy(override)})
    return result


def classify_status(reasons: list[str]) -> str:
    if any(reason.startswith("missing_config") for reason in reasons):
        return "fail_closed_missing_config"
    if any(reason.startswith("unsafe_defaults") for reason in reasons):
        return "fail_closed_unsafe_defaults"
    if any(reason.startswith("stale_runbook") for reason in reasons):
        return "fail_closed_stale_runbook"
    if any(reason.startswith("ambiguous_claim") for reason in reasons):
        return "fail_closed_ambiguous_production_claim"
    if any(reason.startswith("forbidden_execution") for reason in reasons):
        return "fail_closed_forbidden_execution"
    if any(reason.startswith("forbidden_boundary") for reason in reasons):
        return "fail_closed_forbidden_operation_boundary"
    if reasons:
        return "fail_closed_missing_config"
    return "deployment_config_runbook_readiness_ready"


def classify_artifact(artifact: dict[str, Any]) -> dict[str, Any]:
    reasons: list[str] = []
    if artifact.get("schema_version") != SCHEMA_VERSION or artifact.get("contract_version") != CONTRACT_VERSION:
        reasons.append("missing_config:schema_contract")
    if artifact.get("task_id") != "V290-006" or artifact.get("github_issue") != 932:
        reasons.append("missing_config:task_identity")
    if artifact.get("backend_module") != MODULE_ID or artifact.get("backend_module_status") != "production_ready_evidence":
        reasons.append("missing_config:module")
    if artifact.get("readiness_mode") != "deterministic_readiness_replay":
        reasons.append("missing_config:readiness_mode")
    if artifact.get("deployment_mode") != "config_readiness_preview_only":
        reasons.append("forbidden_execution:deployment_mode")
    if set(artifact.get("dependency_contracts") or []) != DEPENDENCIES:
        reasons.append("missing_config:dependencies")

    config = artifact.get("deployment_config") or {}
    for key in ("config_id", "source_ref", "environment", "config_digest", "secret_rotation_expectation"):
        if not config.get(key):
            reasons.append(f"missing_config:{key}")
    if config.get("source_ref") != "docs/rust-cutover/release/v0_28_0_deployment_orchestration_runtime_artifact.json":
        reasons.append("missing_config:source_ref")
    if config.get("environment") != "production_readiness_sandbox":
        reasons.append("missing_config:environment")
    if config.get("required_inputs_present") is not True:
        reasons.append("missing_config:required_inputs")
    if config.get("environment_provenance_status") != "linked":
        reasons.append("missing_config:environment_provenance")
    if config.get("redaction_status") != "redacted":
        reasons.append("missing_config:redaction")
    if config.get("secret_rotation_expectation") != "documented":
        reasons.append("missing_config:rotation")
    if config.get("unsafe_defaults_allowed") is not False:
        reasons.append("unsafe_defaults:allowed")
    if config.get("production_execution_claim_allowed") is not False:
        reasons.append("ambiguous_claim:config")
    commands = config.get("validation_commands")
    if not isinstance(commands, list) or "scripts/ai/verify_release.sh v29-deployment-config-runbook-production-readiness" not in commands:
        reasons.append("missing_config:validation_commands")

    runbook = artifact.get("runbook") or {}
    for key in ("runbook_id", "source_ref"):
        if not runbook.get(key):
            reasons.append(f"stale_runbook:{key}")
    if runbook.get("source_ref") != "docs/rust-cutover/evidence/V290-006.md":
        reasons.append("stale_runbook:source_ref")
    if runbook.get("freshness_status") != "fresh":
        reasons.append("stale_runbook:freshness")
    if runbook.get("provenance_status") != "linked":
        reasons.append("stale_runbook:provenance")
    for key in ("operator_handoff_status", "rollback_criteria_status", "escalation_paths_status"):
        if runbook.get(key) != "documented":
            reasons.append(f"stale_runbook:{key}")
    if runbook.get("owner_approval_required") is not True:
        reasons.append("stale_runbook:owner_approval_required")
    if runbook.get("ambiguous_production_claim_allowed") is not False:
        reasons.append("ambiguous_claim:runbook")

    transitions = artifact.get("preview_transitions")
    if not isinstance(transitions, list) or len(transitions) != 3:
        reasons.append("forbidden_execution:transition_count")
        transitions = transitions if isinstance(transitions, list) else []
    transition_ids = {transition.get("transition_id") for transition in transitions}
    if transition_ids != {"deploy-preview", "upgrade-preview", "rollback-preview"}:
        reasons.append("forbidden_execution:transition_ids")
    for transition in transitions:
        if transition.get("preview_only") is not True:
            reasons.append(f"forbidden_execution:preview_only:{transition.get('transition_id')}")
        if transition.get("owner_approval_present") is not True:
            reasons.append(f"forbidden_execution:owner_approval:{transition.get('transition_id')}")
        if transition.get("execution_triggered") is not False:
            reasons.append(f"forbidden_execution:triggered:{transition.get('transition_id')}")
        if transition.get("operation_effect") != "validated_only":
            reasons.append(f"forbidden_execution:effect:{transition.get('transition_id')}")

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

matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
module = next((item for item in matrix.get("module_readiness", []) if item.get("module_id") == MODULE_ID), None)
if not module:
    fail("matrix missing deployment_config_runbook_production_readiness")
if module.get("classification") != "production-ready" or module.get("issue") != 932:
    fail("deployment config runbook matrix entry mismatch")
if module.get("evidence_path") != "docs/rust-cutover/evidence/V290-006.md":
    fail("deployment config runbook matrix evidence path mismatch")
if module.get("verification_command") != "scripts/ai/verify_release.sh v29-deployment-config-runbook-production-readiness":
    fail("deployment config runbook matrix verification command mismatch")

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
if allowed != 1 or fail_closed != 6:
    fail(f"unexpected case counts: allowed={allowed} fail_closed={fail_closed}")

if selftest:
    bad = copy.deepcopy(artifact)
    bad["boundary_flags"]["production_deployment_execution_allowed"] = True
    if classify_artifact(bad)["status"] != "fail_closed_forbidden_operation_boundary":
        fail("negative self-test unexpectedly allowed production deployment execution")

print(
    "v29_deployment_config_runbook_production_readiness=pass "
    f"cases={len(cases)} "
    f"allowed={allowed} "
    f"fail_closed={fail_closed} "
    f"transitions={len(artifact['preview_transitions'])} "
    f"boundary_flags={len(BOUNDARY_FALSE_FLAGS)} "
    "negative_selftest=1"
)
PY
