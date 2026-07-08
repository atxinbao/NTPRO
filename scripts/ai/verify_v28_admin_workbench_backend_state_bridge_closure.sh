#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

ARTIFACT_PATH="${NTPRO_V280_ADMIN_BRIDGE_ARTIFACT:-docs/rust-cutover/release/v0_28_0_admin_workbench_backend_state_bridge_artifact.json}"
CONTRACT_PATH="${NTPRO_V280_ADMIN_BRIDGE_CONTRACT:-docs/rust-cutover/release/v0_28_0_admin_workbench_backend_state_bridge_closure.md}"
TASK_PATH="${NTPRO_V280_ADMIN_BRIDGE_TASK:-docs/rust-cutover/tasks/V280-006.md}"
EVIDENCE_PATH="${NTPRO_V280_ADMIN_BRIDGE_EVIDENCE:-docs/rust-cutover/evidence/V280-006.md}"
MATRIX_PATH="${NTPRO_V280_ADMIN_BRIDGE_MATRIX:-docs/rust-cutover/release/v0_28_0_backend_closure_readiness_matrix.json}"
FOUNDATION_PATH="${NTPRO_V280_ADMIN_BRIDGE_FOUNDATION:-docs/rust-cutover/release/v0_27_0_admin_workbench_runtime_state_bridge.md}"
SELFTEST="${NTPRO_V280_ADMIN_BRIDGE_SELFTEST:-1}"

fail() {
  echo "v28 Admin Workbench backend state bridge closure failed: $*" >&2
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

for path in "$ARTIFACT_PATH" "$CONTRACT_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$MATRIX_PATH" "$FOUNDATION_PATH"; do
  require_file "$path"
done

require_contains "$TASK_PATH" "GitHub issue: \`#899\`"
require_contains "$EVIDENCE_PATH" "Task: \`V280-006\` / GitHub issue \`#899\`"
require_contains "$FOUNDATION_PATH" "admin_workbench_surface_scope = runtime_state_bridge_read_only_admin_surface"
require_contains "$CONTRACT_PATH" "contract_version = ntpro.v280.admin_workbench_backend_state_bridge_closure.v1"
require_contains "$CONTRACT_PATH" "schema_version = ntpro.v280.admin_workbench_backend_state_bridge_artifact.v1"
require_contains "$CONTRACT_PATH" "backend_module_status = runtime_closed"
require_contains "$CONTRACT_PATH" "release stage = scripts/ai/verify_release.sh v28-admin-workbench-backend-state-bridge-closure"

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

SCHEMA_VERSION = "ntpro.v280.admin_workbench_backend_state_bridge_artifact.v1"
CONTRACT_VERSION = "ntpro.v280.admin_workbench_backend_state_bridge_closure.v1"
DEPENDENCIES = {"V280-001", "V280-002", "V280-003", "V280-004", "V280-005", "V270-006", "V271-006"}
REQUIRED_COMPONENTS = [
    "identity_permission_runtime_closure",
    "persistent_audit_storage_runtime_closure",
    "deployment_upgrade_rollback_orchestration_runtime_closure",
    "telemetry_slo_ingestion_runtime_closure",
    "backend_closure_boundary_contract",
]
VALID_COMPONENT_STATUSES = {"ready", "degraded", "blocked", "stale"}
EXPECTED_CASES = [
    "admin_workbench_backend_bridge.ready.allowed.001",
    "admin_workbench_backend_bridge.degraded.readonly.001",
    "admin_workbench_backend_bridge.blocked.readonly.001",
    "admin_workbench_backend_bridge.stale.degraded.001",
    "admin_workbench_backend_bridge.missing_component.fail_closed.001",
    "admin_workbench_backend_bridge.malformed_component.fail_closed.001",
    "admin_workbench_backend_bridge.forbidden_controls.fail_closed.001",
]
BOUNDARY_FALSE_FLAGS = [
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
    "dashboard_operation_controls_enabled",
    "dashboard_trading_controls_enabled",
    "admin_workbench_operation_controls_enabled",
    "admin_workbench_trading_controls_enabled",
    "automatic_remediation_allowed",
    "retry_scheduler_enabled",
    "adapter_send_allowed",
    "live_exchange_request_allowed",
    "manual_operation_submit_allowed",
    "product_grade_trading_terminal_claim",
]


def fail(message: str) -> None:
    raise SystemExit(f"v28 Admin Workbench backend state bridge closure failed: {message}")


def non_empty(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


def merge(base: dict[str, Any], override: dict[str, Any] | None) -> dict[str, Any]:
    merged = copy.deepcopy(base)
    if override:
        for key, value in override.items():
            merged[key] = value
    return merged


def classify_status(reasons: list[str]) -> str:
    if any(reason.startswith("missing_component") for reason in reasons):
        return "fail_closed_missing_component"
    if any(reason.startswith("malformed_component") for reason in reasons):
        return "fail_closed_malformed_component_state"
    if any(reason.startswith("forbidden_controls") for reason in reasons):
        return "fail_closed_forbidden_controls"
    if any(reason.startswith("blocked_component") for reason in reasons):
        return "blocked_read_only_surface"
    if any(reason.startswith("stale_component") for reason in reasons):
        return "degraded_stale_component"
    if any(reason.startswith("degraded_component") for reason in reasons):
        return "degraded_read_only_surface"
    if reasons:
        return "fail_closed_admin_bridge_violation"
    return "admin_workbench_bridge_ready"


def apply_case(artifact: dict[str, Any], case: dict[str, Any]) -> dict[str, Any]:
    result = copy.deepcopy(artifact)
    for component_id, override in (case.get("component_overrides") or {}).items():
        result["component_states"][component_id] = merge(result["component_states"].get(component_id, {}), override)
    result["boundary_flags"] = merge(result["boundary_flags"], case.get("boundary_flags_override"))
    for surface_id, override in (case.get("surface_overrides") or {}).items():
        result["surfaces"][surface_id] = merge(result["surfaces"].get(surface_id, {}), override)
    return result


def require_existing_path(value: Any, reason: str, reasons: list[str]) -> None:
    if not non_empty(value):
        reasons.append(reason)
        return
    path = Path(str(value).split("#", 1)[0])
    if path.is_absolute() or not path.is_file():
        reasons.append(reason)


def classify_artifact(artifact: dict[str, Any]) -> dict[str, Any]:
    reasons: list[str] = []
    if artifact.get("schema_version") != SCHEMA_VERSION:
        reasons.append("malformed_component:schema_version_mismatch")
    if artifact.get("contract_version") != CONTRACT_VERSION:
        reasons.append("malformed_component:contract_version_mismatch")
    if artifact.get("task_id") != "V280-006" or artifact.get("github_issue") != 899:
        reasons.append("malformed_component:task_identity_mismatch")
    if artifact.get("backend_module") != "admin_workbench_backend_state_bridge_closure":
        reasons.append("malformed_component:backend_module_mismatch")
    if artifact.get("backend_module_status") != "runtime_closed":
        reasons.append("malformed_component:backend_module_not_runtime_closed")
    if set(artifact.get("dependency_contracts") or []) != DEPENDENCIES:
        reasons.append("malformed_component:dependency_contracts_mismatch")
    if artifact.get("bridge_mode") != "deterministic_backend_state_api_handoff":
        reasons.append(f"malformed_component:bridge_mode:{artifact.get('bridge_mode')}")

    api_surface = artifact.get("api_surface")
    if not isinstance(api_surface, dict):
        reasons.append("malformed_component:missing_api_surface")
        api_surface = {}
    if api_surface.get("read_only") is not True or api_surface.get("admin_only") is not True:
        reasons.append("forbidden_controls:api_surface_not_readonly_admin")
    if api_surface.get("ad_hoc_evidence_file_inspection_required") is not False:
        reasons.append("malformed_component:ad_hoc_evidence_required")
    if api_surface.get("product_grade_terminal_claim_allowed") is not False:
        reasons.append("forbidden_controls:api_surface_terminal_claim")

    boundary = artifact.get("boundary_flags")
    if not isinstance(boundary, dict):
        reasons.append("forbidden_controls:missing_boundary_flags")
        boundary = {}
    for key in BOUNDARY_FALSE_FLAGS:
        if key not in boundary:
            reasons.append(f"forbidden_controls:missing_required_false:{key}")
        elif boundary.get(key) is not False:
            reasons.append(f"forbidden_controls:boundary:{key}")

    surfaces = artifact.get("surfaces")
    if not isinstance(surfaces, dict):
        reasons.append("forbidden_controls:missing_surfaces")
        surfaces = {}
    for surface_id in ("admin_workbench", "dashboard"):
        surface = surfaces.get(surface_id)
        if not isinstance(surface, dict):
            reasons.append(f"forbidden_controls:missing_surface:{surface_id}")
            continue
        if surface.get("read_only") is not True or surface.get("display_only") is not True:
            reasons.append(f"forbidden_controls:surface_not_readonly:{surface_id}")
        for key in ("operation_controls_enabled", "trading_controls_enabled", "submit_controls_enabled", "product_grade_terminal_claim_allowed"):
            if surface.get(key) is not False:
                reasons.append(f"forbidden_controls:surface:{surface_id}:{key}")

    components = artifact.get("component_states")
    if not isinstance(components, dict):
        reasons.append("missing_component:component_states")
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
        if status not in VALID_COMPONENT_STATUSES:
            reasons.append(f"missing_component:{component_id}:{status}")
        elif status == "degraded":
            reasons.append(f"degraded_component:{component_id}")
        elif status == "blocked":
            reasons.append(f"blocked_component:{component_id}")
        elif status == "stale":
            reasons.append(f"stale_component:{component_id}")
        if status in {"degraded", "blocked", "stale"}:
            degradation_reasons = component.get("degradation_reasons")
            if not isinstance(degradation_reasons, list) or not degradation_reasons:
                reasons.append(f"malformed_component:missing_degradation_reasons:{component_id}")
        require_existing_path(component.get("source_ref"), f"malformed_component:source_ref:{component_id}", reasons)
        require_existing_path(component.get("evidence_path"), f"malformed_component:evidence_path:{component_id}", reasons)
        if not non_empty(component.get("verification_command")):
            reasons.append(f"malformed_component:verification_command:{component_id}")
        if component.get("provenance_status") != "verified":
            reasons.append(f"malformed_component:provenance:{component_id}")
        if component.get("freshness_status") != "fresh":
            reasons.append(f"stale_component:freshness:{component_id}")
        if component.get("redaction_status") != "redacted":
            reasons.append(f"malformed_component:redaction:{component_id}")
        if component.get("runtime_state_aligned") is not True:
            reasons.append(f"malformed_component:runtime_state_aligned:{component_id}")
        if component.get("read_only") is not True:
            reasons.append(f"forbidden_controls:component_not_readonly:{component_id}")
        if component.get("operation_controls_enabled") is not False or component.get("trading_controls_enabled") is not False:
            reasons.append(f"forbidden_controls:component_controls:{component_id}")

    status = classify_status(reasons)
    return {
        "status": status,
        "fail_closed": status.startswith("fail_closed"),
        "degraded": status.startswith("degraded"),
        "blocked": status.startswith("blocked"),
        "blocking_reasons": reasons,
    }


artifact = json.loads(artifact_path.read_text(encoding="utf-8"))
matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
module = next((item for item in matrix.get("module_readiness", []) if item.get("module_id") == "admin_workbench_backend_state_bridge_closure"), None)
if not module:
    fail("matrix missing admin_workbench_backend_state_bridge_closure")
if module.get("classification") != "runtime-closed":
    fail("Admin Workbench matrix entry must be runtime-closed")
if module.get("evidence_path") != "docs/rust-cutover/evidence/V280-006.md":
    fail("Admin Workbench matrix evidence path mismatch")
if module.get("verification_command") != "scripts/ai/verify_release.sh v28-admin-workbench-backend-state-bridge-closure":
    fail("Admin Workbench matrix verification command mismatch")

cases = artifact.get("admin_bridge_replay_cases")
if not isinstance(cases, list) or [case.get("case_id") for case in cases] != EXPECTED_CASES:
    fail("Admin Workbench bridge replay cases mismatch")
allowed = 0
degraded = 0
blocked = 0
fail_closed = 0
for case in cases:
    actual = classify_artifact(apply_case(artifact, case))
    if actual["status"] != case.get("expected_status"):
        fail(f"{case.get('case_id')}: expected {case.get('expected_status')} got {actual}")
    if actual["fail_closed"]:
        fail_closed += 1
    elif actual["blocked"]:
        blocked += 1
    elif actual["degraded"]:
        degraded += 1
    else:
        allowed += 1
if allowed != 1 or degraded != 2 or blocked != 1 or fail_closed != 3:
    fail(f"unexpected case counts: allowed={allowed} degraded={degraded} blocked={blocked} fail_closed={fail_closed}")

if selftest:
    opened = copy.deepcopy(artifact)
    opened["boundary_flags"]["admin_workbench_operation_controls_enabled"] = True
    if classify_artifact(opened)["status"] != "fail_closed_forbidden_controls":
        fail("negative self-test unexpectedly allowed admin_workbench_operation_controls_enabled")

print(
    "v28_admin_workbench_backend_state_bridge_closure=pass "
    f"cases={len(cases)} allowed={allowed} degraded={degraded} blocked={blocked} fail_closed={fail_closed} "
    f"components={len(REQUIRED_COMPONENTS)} boundary_flags={len(BOUNDARY_FALSE_FLAGS)} negative_selftest={int(selftest)}"
)
PY
