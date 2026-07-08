#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

ARTIFACT_PATH="${NTPRO_V280_FAIL_CLOSED_ARTIFACT:-docs/rust-cutover/release/v0_28_0_backend_closure_fail_closed_hardening_artifact.json}"
CONTRACT_PATH="${NTPRO_V280_FAIL_CLOSED_CONTRACT:-docs/rust-cutover/release/v0_28_0_backend_closure_fail_closed_hardening.md}"
TASK_PATH="${NTPRO_V280_FAIL_CLOSED_TASK:-docs/rust-cutover/tasks/V280-008.md}"
EVIDENCE_PATH="${NTPRO_V280_FAIL_CLOSED_EVIDENCE:-docs/rust-cutover/evidence/V280-008.md}"
MATRIX_PATH="${NTPRO_V280_FAIL_CLOSED_MATRIX:-docs/rust-cutover/release/v0_28_0_backend_closure_readiness_matrix.json}"
SELFTEST="${NTPRO_V280_FAIL_CLOSED_SELFTEST:-1}"

fail() {
  echo "v28 backend closure fail-closed hardening failed: $*" >&2
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

for path in "$ARTIFACT_PATH" "$CONTRACT_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$MATRIX_PATH"; do
  require_file "$path"
done

require_contains "$TASK_PATH" "GitHub issue: \`#901\`"
require_contains "$EVIDENCE_PATH" "Task: \`V280-008\` / GitHub issue \`#901\`"
require_contains "$CONTRACT_PATH" "contract_version = ntpro.v280.backend_closure_fail_closed_hardening.v1"
require_contains "$CONTRACT_PATH" "schema_version = ntpro.v280.backend_closure_fail_closed_hardening_artifact.v1"
require_contains "$CONTRACT_PATH" "backend_module_status = runtime_closed"
require_contains "$CONTRACT_PATH" "runtime_closure_health_separate_from_trading_readiness = true"
require_contains "$CONTRACT_PATH" "partial_backend_closure_product_ready_allowed = false"
require_contains "$CONTRACT_PATH" "release stage = scripts/ai/verify_release.sh v28-backend-closure-fail-closed-hardening"

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

SCHEMA_VERSION = "ntpro.v280.backend_closure_fail_closed_hardening_artifact.v1"
CONTRACT_VERSION = "ntpro.v280.backend_closure_fail_closed_hardening.v1"
DEPENDENCIES = {
    "V280-001",
    "V280-002",
    "V280-003",
    "V280-004",
    "V280-005",
    "V280-006",
    "V280-007",
    "V270-007",
    "V271-006",
}
REQUIRED_COMPONENTS = [
    "backend_closure_boundary_contract",
    "identity_permission_runtime_closure",
    "persistent_audit_storage_runtime_closure",
    "deployment_upgrade_rollback_orchestration_runtime_closure",
    "telemetry_slo_ingestion_runtime_closure",
    "admin_workbench_backend_state_bridge_closure",
    "trader_terminal_backend_api_contract_handoff",
]
VALID_COMPONENT_STATUSES = {"ready", "degraded", "blocked", "stale"}
EXPECTED_CASES = [
    "backend_closure_fail_closed_hardening.ready.allowed.001",
    "backend_closure_fail_closed_hardening.partial.degraded.001",
    "backend_closure_fail_closed_hardening.stale.degraded.001",
    "backend_closure_fail_closed_hardening.missing_component.fail_closed.001",
    "backend_closure_fail_closed_hardening.malformed_evidence.fail_closed.001",
    "backend_closure_fail_closed_hardening.source_drift.fail_closed.001",
    "backend_closure_fail_closed_hardening.product_ready_claim.fail_closed.001",
    "backend_closure_fail_closed_hardening.forbidden_control.fail_closed.001",
    "backend_closure_fail_closed_hardening.missing_required_false.fail_closed.001",
]
BOUNDARY_FALSE_FLAGS = [
    "new_submit_capability",
    "default_submit_allowed",
    "submit_order_allowed",
    "cancel_order_allowed",
    "retry_order_allowed",
    "replace_order_allowed",
    "amend_order_allowed",
    "flatten_position_allowed",
    "order_ticket_enabled",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
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
    "trader_terminal_submit_controls_enabled",
    "manual_operation_entry_enabled",
    "manual_operation_submit_allowed",
    "backend_complete_claim",
    "frontend_product_work_complete_claim",
    "product_grade_trading_terminal_claim",
    "product_grade_trading_ready",
    "display_product_ready_badge",
]
PRODUCT_READY_FLAGS = {
    "backend_complete_claim",
    "frontend_product_work_complete_claim",
    "product_grade_trading_terminal_claim",
    "product_grade_trading_ready",
    "display_product_ready_badge",
}


def fail(message: str) -> None:
    raise SystemExit(f"v28 backend closure fail-closed hardening failed: {message}")


def non_empty(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


def merge(base: Any, override: Any) -> Any:
    if isinstance(base, dict) and isinstance(override, dict):
        result = copy.deepcopy(base)
        for key, value in override.items():
            result[key] = merge(result.get(key), value)
        return result
    return copy.deepcopy(override)


def apply_case(artifact: dict[str, Any], case: dict[str, Any]) -> dict[str, Any]:
    result = copy.deepcopy(artifact)
    for component_id, override in (case.get("component_overrides") or {}).items():
        result["component_integrations"][component_id] = merge(result["component_integrations"].get(component_id, {}), override)
    if case.get("claim_boundary_override"):
        result["claim_boundary"] = merge(result["claim_boundary"], case["claim_boundary_override"])
    if case.get("boundary_flags_override"):
        result["boundary_flags"] = merge(result["boundary_flags"], case["boundary_flags_override"])
    for key in case.get("drop_boundary_flags") or []:
        result["boundary_flags"].pop(key, None)
    return result


def require_existing_path(value: Any, reason: str, reasons: list[str]) -> None:
    if not non_empty(value):
        reasons.append(reason)
        return
    path = Path(str(value).split("#", 1)[0])
    if path.is_absolute() or not path.is_file():
        reasons.append(reason)


def classify_status(reasons: list[str]) -> str:
    if any(reason.startswith("missing_required_false") for reason in reasons):
        return "fail_closed_missing_required_false_boundary"
    if any(reason.startswith("missing_component") for reason in reasons):
        return "fail_closed_missing_required_component"
    if any(reason.startswith("malformed_component") for reason in reasons):
        return "fail_closed_malformed_component_evidence"
    if any(reason.startswith("source_drift") for reason in reasons):
        return "fail_closed_source_drift"
    if any(reason.startswith("product_ready_claim") for reason in reasons):
        return "fail_closed_product_ready_claim"
    if any(reason.startswith("forbidden_control") for reason in reasons):
        return "fail_closed_forbidden_control"
    if any(reason.startswith("stale_component") for reason in reasons):
        return "degraded_stale_backend_closure"
    if any(reason.startswith(("degraded_component", "blocked_component")) for reason in reasons):
        return "degraded_partial_backend_closure"
    if reasons:
        return "fail_closed_malformed_component_evidence"
    return "backend_closure_runtime_ready_trading_not_ready"


def classify_artifact(artifact: dict[str, Any]) -> dict[str, Any]:
    reasons: list[str] = []
    if artifact.get("schema_version") != SCHEMA_VERSION:
        reasons.append("malformed_component:schema_version_mismatch")
    if artifact.get("contract_version") != CONTRACT_VERSION:
        reasons.append("malformed_component:contract_version_mismatch")
    if artifact.get("task_id") != "V280-008" or artifact.get("github_issue") != 901:
        reasons.append("malformed_component:task_identity_mismatch")
    if artifact.get("backend_module") != "backend_closure_fail_closed_hardening":
        reasons.append("malformed_component:backend_module_mismatch")
    if artifact.get("backend_module_status") != "runtime_closed":
        reasons.append("malformed_component:backend_module_not_runtime_closed")
    if set(artifact.get("dependency_contracts") or []) != DEPENDENCIES:
        reasons.append("malformed_component:dependency_contracts_mismatch")
    if artifact.get("hardening_mode") != "deterministic_backend_closure_fail_closed_replay":
        reasons.append(f"malformed_component:hardening_mode:{artifact.get('hardening_mode')}")

    claim_boundary = artifact.get("claim_boundary")
    if not isinstance(claim_boundary, dict):
        reasons.append("malformed_component:missing_claim_boundary")
        claim_boundary = {}
    if claim_boundary.get("runtime_closure_health_separate_from_trading_readiness") is not True:
        reasons.append("product_ready_claim:runtime_health_not_separate")
    for key in (
        "backend_complete_claim_allowed",
        "frontend_product_claim_allowed",
        "production_execution_runtime_claim_allowed",
        "partial_backend_closure_product_ready_allowed",
        "product_grade_trading_ready_allowed",
        "product_grade_terminal_claim_allowed",
    ):
        if claim_boundary.get(key) is not False:
            reasons.append(f"product_ready_claim:claim_boundary:{key}")
    if claim_boundary.get("trading_readiness_status") != "not_ready":
        reasons.append("product_ready_claim:trading_readiness_status")
    if claim_boundary.get("backend_closure_health_values") != ["ready", "degraded", "blocked", "stale", "fail_closed"]:
        reasons.append("malformed_component:backend_closure_health_values")

    boundary = artifact.get("boundary_flags")
    if not isinstance(boundary, dict):
        reasons.append("missing_required_false:boundary_flags")
        boundary = {}
    for key in BOUNDARY_FALSE_FLAGS:
        if key not in boundary:
            reasons.append(f"missing_required_false:{key}")
        elif boundary.get(key) is not False:
            if key in PRODUCT_READY_FLAGS:
                reasons.append(f"product_ready_claim:boundary:{key}")
            else:
                reasons.append(f"forbidden_control:boundary:{key}")

    components = artifact.get("component_integrations")
    if not isinstance(components, dict):
        reasons.append("missing_component:component_integrations")
        components = {}
    if set(components) != set(REQUIRED_COMPONENTS):
        missing = sorted(set(REQUIRED_COMPONENTS) - set(components))
        extra = sorted(set(components) - set(REQUIRED_COMPONENTS))
        if missing:
            reasons.append(f"missing_component:set:{missing}")
        if extra:
            reasons.append(f"malformed_component:extra:{extra}")

    for component_id in REQUIRED_COMPONENTS:
        component = components.get(component_id)
        if not isinstance(component, dict):
            reasons.append(f"missing_component:{component_id}")
            continue
        status = component.get("component_status")
        if status == "missing":
            reasons.append(f"missing_component:{component_id}")
        elif status not in VALID_COMPONENT_STATUSES:
            reasons.append(f"malformed_component:status:{component_id}:{status}")
        elif status == "degraded":
            reasons.append(f"degraded_component:{component_id}")
        elif status == "blocked":
            reasons.append(f"blocked_component:{component_id}")
        elif status == "stale":
            reasons.append(f"stale_component:{component_id}")
        if status in {"degraded", "blocked", "stale", "missing"}:
            degradation_reasons = component.get("degradation_reasons")
            if not isinstance(degradation_reasons, list) or not degradation_reasons:
                reasons.append(f"malformed_component:missing_degradation_reasons:{component_id}")
        require_existing_path(component.get("source_ref"), f"missing_component:source_ref:{component_id}", reasons)
        require_existing_path(component.get("evidence_path"), f"missing_component:evidence_path:{component_id}", reasons)
        if not non_empty(component.get("verification_command")):
            reasons.append(f"missing_component:verification_command:{component_id}")
        if component.get("provenance_status") != "verified":
            reasons.append(f"malformed_component:provenance:{component_id}")
        if component.get("redaction_status") != "redacted":
            reasons.append(f"malformed_component:redaction:{component_id}")
        if component.get("freshness_status") == "stale":
            reasons.append(f"stale_component:freshness:{component_id}")
        elif component.get("freshness_status") != "fresh":
            reasons.append(f"malformed_component:freshness:{component_id}")
        if component.get("source_drift_status") != "aligned":
            reasons.append(f"source_drift:{component_id}")
        if component.get("runtime_state_aligned") is not True:
            reasons.append(f"source_drift:runtime_state_aligned:{component_id}")
        if component.get("read_only") is not True:
            reasons.append(f"forbidden_control:component_not_readonly:{component_id}")
        if component.get("operation_controls_enabled") is not False or component.get("trading_controls_enabled") is not False:
            reasons.append(f"forbidden_control:component_controls:{component_id}")
        if component.get("product_ready_claim_allowed") is not False or component.get("trading_readiness_claim_allowed") is not False:
            reasons.append(f"product_ready_claim:component:{component_id}")

    status = classify_status(reasons)
    return {
        "status": status,
        "fail_closed": status.startswith("fail_closed"),
        "degraded": status.startswith("degraded"),
        "blocking_reasons": reasons,
    }


artifact = json.loads(artifact_path.read_text(encoding="utf-8"))
matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
module = next(
    (item for item in matrix.get("module_readiness", []) if item.get("module_id") == "backend_closure_fail_closed_hardening"),
    None,
)
if not module:
    fail("matrix missing backend_closure_fail_closed_hardening")
if module.get("classification") != "runtime-closed":
    fail("backend_closure_fail_closed_hardening matrix entry must be runtime-closed")
if module.get("evidence_path") != "docs/rust-cutover/evidence/V280-008.md":
    fail("backend_closure_fail_closed_hardening matrix evidence path mismatch")
if module.get("verification_command") != "scripts/ai/verify_release.sh v28-backend-closure-fail-closed-hardening":
    fail("backend_closure_fail_closed_hardening matrix verification command mismatch")

cases = artifact.get("hardening_replay_cases")
if not isinstance(cases, list) or [case.get("case_id") for case in cases] != EXPECTED_CASES:
    fail("backend closure fail-closed replay cases mismatch")
allowed = 0
degraded = 0
fail_closed = 0
for case in cases:
    actual = classify_artifact(apply_case(artifact, case))
    if actual["status"] != case.get("expected_status"):
        fail(f"{case.get('case_id')}: expected {case.get('expected_status')} got {actual}")
    if actual["fail_closed"]:
        fail_closed += 1
    elif actual["degraded"]:
        degraded += 1
    else:
        allowed += 1
if allowed != 1 or degraded != 2 or fail_closed != 6:
    fail(f"unexpected case counts: allowed={allowed} degraded={degraded} fail_closed={fail_closed}")

if selftest:
    product_ready = copy.deepcopy(artifact)
    product_ready["claim_boundary"]["product_grade_terminal_claim_allowed"] = True
    if classify_artifact(product_ready)["status"] != "fail_closed_product_ready_claim":
        fail("negative self-test unexpectedly allowed product-grade terminal claim")

    missing_boundary = copy.deepcopy(artifact)
    missing_boundary["boundary_flags"].pop("adapter_send_allowed", None)
    if classify_artifact(missing_boundary)["status"] != "fail_closed_missing_required_false_boundary":
        fail("negative self-test unexpectedly allowed missing adapter_send_allowed boundary")

print(
    "v28_backend_closure_fail_closed_hardening=pass "
    f"cases={len(cases)} allowed={allowed} degraded={degraded} fail_closed={fail_closed} "
    f"components={len(REQUIRED_COMPONENTS)} boundary_flags={len(BOUNDARY_FALSE_FLAGS)} negative_selftest={int(selftest)}"
)
PY
