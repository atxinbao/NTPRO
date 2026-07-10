#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

ARTIFACT_PATH="${NTPRO_V290_FAIL_CLOSED_ARTIFACT:-docs/rust-cutover/release/v0_29_0_backend_production_readiness_fail_closed_hardening_artifact.json}"
CONTRACT_PATH="${NTPRO_V290_FAIL_CLOSED_CONTRACT:-docs/rust-cutover/release/v0_29_0_backend_production_readiness_fail_closed_hardening.md}"
TASK_PATH="${NTPRO_V290_FAIL_CLOSED_TASK:-docs/rust-cutover/tasks/V290-009.md}"
EVIDENCE_PATH="${NTPRO_V290_FAIL_CLOSED_EVIDENCE:-docs/rust-cutover/evidence/V290-009.md}"
MATRIX_PATH="${NTPRO_V290_FAIL_CLOSED_MATRIX:-docs/rust-cutover/release/v0_29_0_backend_production_readiness_matrix.json}"
BOUNDARY_CONTRACT_PATH="${NTPRO_V290_FAIL_CLOSED_BOUNDARY_CONTRACT:-docs/rust-cutover/release/v0_29_0_backend_production_readiness_boundary_contract.md}"
INTAKE_PATH="${NTPRO_V290_FAIL_CLOSED_INTAKE:-docs/rust-cutover/release/v0_29_0_intake_gate.md}"
SELFTEST="${NTPRO_V290_FAIL_CLOSED_SELFTEST:-1}"

fail() {
  echo "v29 backend production readiness fail-closed hardening failed: $*" >&2
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

for path in "$ARTIFACT_PATH" "$CONTRACT_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$MATRIX_PATH" "$BOUNDARY_CONTRACT_PATH" "$INTAKE_PATH"; do
  require_file "$path"
done

require_contains "$TASK_PATH" "GitHub issue: \`#935\`"
require_contains "$EVIDENCE_PATH" "Task: \`V290-009\` / GitHub issue \`#935\`"
require_contains "$INTAKE_PATH" "start_gate_status = satisfied"
require_contains "$BOUNDARY_CONTRACT_PATH" "contract_version = ntpro.v290.backend_production_readiness_boundary.v1"
require_contains "$CONTRACT_PATH" "contract_version = ntpro.v290.backend_production_readiness_fail_closed_hardening.v1"
require_contains "$CONTRACT_PATH" "schema_version = ntpro.v290.backend_production_readiness_fail_closed_hardening_artifact.v1"
require_contains "$CONTRACT_PATH" "hardening_mode = deterministic_backend_production_readiness_fail_closed_replay"
require_contains "$CONTRACT_PATH" "production_readiness_health_separate_from_go_live = true"
require_contains "$CONTRACT_PATH" "live_trading_readiness_claim_allowed = false"
require_contains "$CONTRACT_PATH" "release stage = scripts/ai/verify_release.sh v29-backend-production-readiness-fail-closed-hardening"

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

SCHEMA_VERSION = "ntpro.v290.backend_production_readiness_fail_closed_hardening_artifact.v1"
CONTRACT_VERSION = "ntpro.v290.backend_production_readiness_fail_closed_hardening.v1"
MODULE_ID = "backend_production_readiness_fail_closed_hardening"
DEPENDENCIES = {
    "V290-000",
    "V290-001",
    "V290-002",
    "V290-003",
    "V290-004",
    "V290-005",
    "V290-006",
    "V290-007",
    "V290-008",
    "v0.28.1-release-evidence",
}
EXPECTED_COMPONENTS = [
    "persistent_audit_storage",
    "telemetry_slo_ingestion",
    "permission_source",
    "read_only_backend_api",
    "deployment_config",
    "operator_runbook",
    "monitoring_alert_incident",
    "canary_preflight",
    "rollback_dr_preflight",
]
EXPECTED_CASES = [
    "backend_production_readiness.fail_closed.ready.allowed.001",
    "backend_production_readiness.fail_closed.partial.degraded.001",
    "backend_production_readiness.fail_closed.stale.blocked.001",
    "backend_production_readiness.fail_closed.missing_component.fail_closed.001",
    "backend_production_readiness.fail_closed.source_drift.fail_closed.001",
    "backend_production_readiness.fail_closed.go_live_claim.fail_closed.001",
    "backend_production_readiness.fail_closed.live_trading_claim.fail_closed.001",
    "backend_production_readiness.fail_closed.forbidden_control.fail_closed.001",
    "backend_production_readiness.fail_closed.missing_boundary.fail_closed.001",
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
    "backend_go_live_ready_claim",
    "live_trading_ready_claim",
    "production_readiness_health_equals_go_live",
    "production_readiness_health_equals_trading",
    "ambiguous_live_trading_claim_allowed",
    "readiness_report_product_ready_badge",
]


def fail(message: str) -> None:
    raise SystemExit(f"v29 backend production readiness fail-closed hardening failed: {message}")


def merge(base: Any, override: Any) -> Any:
    if isinstance(base, dict) and isinstance(override, dict):
        result = copy.deepcopy(base)
        for key, value in override.items():
            result[key] = merge(result.get(key), value)
        return result
    return copy.deepcopy(override)


def apply_case(artifact: dict[str, Any], case: dict[str, Any]) -> dict[str, Any]:
    result = copy.deepcopy(artifact)
    if case.get("readiness_semantics_override"):
        result["readiness_semantics"] = merge(result["readiness_semantics"], case["readiness_semantics_override"])
    if case.get("boundary_flags_override"):
        result["required_false_boundary_flags"] = merge(result["required_false_boundary_flags"], case["boundary_flags_override"])
    for key in case.get("boundary_flag_removals") or []:
        result["required_false_boundary_flags"].pop(key, None)
    removals = set(case.get("component_removals") or [])
    if removals:
        result["components"] = [component for component in result["components"] if component.get("component_id") not in removals]
    overrides = case.get("component_overrides") or {}
    for component in result["components"]:
        component_id = component.get("component_id")
        if component_id in overrides:
            component.update(merge(component, overrides[component_id]))
    return result


def classify_status(reasons: list[str]) -> str:
    if any(reason.startswith("missing_boundary") for reason in reasons):
        return "fail_closed_missing_required_false_boundary"
    if any(reason.startswith("missing_component") for reason in reasons):
        return "fail_closed_missing_required_component"
    if any(reason.startswith("source_drift") for reason in reasons):
        return "fail_closed_source_drift"
    if any(reason.startswith("forbidden_control") for reason in reasons):
        return "fail_closed_forbidden_control"
    if any(reason.startswith("live_trading_claim") for reason in reasons):
        return "fail_closed_live_trading_claim"
    if any(reason.startswith("backend_go_live_claim") for reason in reasons):
        return "fail_closed_backend_go_live_claim"
    if any(reason.startswith("stale_component") for reason in reasons):
        return "blocked_stale_backend_readiness"
    if any(reason.startswith("partial_component") for reason in reasons):
        return "degraded_partial_backend_readiness"
    if reasons:
        return "fail_closed_backend_production_readiness_hardening_violation"
    return "backend_production_readiness_evidence_ready_go_live_not_ready"


def classify_artifact(artifact: dict[str, Any]) -> dict[str, Any]:
    reasons: list[str] = []
    if artifact.get("schema_version") != SCHEMA_VERSION or artifact.get("contract_version") != CONTRACT_VERSION:
        reasons.append("missing_component:schema_contract")
    if artifact.get("task_id") != "V290-009" or artifact.get("github_issue") != 935:
        reasons.append("missing_component:task_identity")
    if artifact.get("release_scope") != "backend_production_readiness_foundation_only":
        reasons.append("missing_component:release_scope")
    if artifact.get("backend_module") != MODULE_ID or artifact.get("backend_module_status") != "production_ready_evidence":
        reasons.append("missing_component:module")
    if artifact.get("readiness_mode") != "deterministic_readiness_replay":
        reasons.append("missing_component:readiness_mode")
    if artifact.get("hardening_mode") != "deterministic_backend_production_readiness_fail_closed_replay":
        reasons.append("missing_component:hardening_mode")
    if set(artifact.get("dependency_contracts") or []) != DEPENDENCIES:
        reasons.append("missing_component:dependencies")

    semantics = artifact.get("readiness_semantics") or {}
    if semantics.get("production_readiness_health") != "backend_readiness_evidence_only":
        reasons.append("backend_go_live_claim:production_readiness_health")
    if semantics.get("go_live_readiness_status") != "not_ready":
        reasons.append("backend_go_live_claim:go_live_status")
    if semantics.get("trading_readiness_status") != "not_ready":
        reasons.append("live_trading_claim:trading_status")
    if semantics.get("production_readiness_health_separate_from_go_live") is not True:
        reasons.append("backend_go_live_claim:separation")
    if semantics.get("production_readiness_health_separate_from_trading_readiness") is not True:
        reasons.append("live_trading_claim:separation")
    for key in ("partial_readiness_go_live_allowed", "backend_go_live_claim_allowed"):
        if semantics.get(key) is not False:
            reasons.append(f"backend_go_live_claim:{key}")
    for key in ("partial_readiness_trading_allowed", "live_trading_readiness_claim_allowed", "product_grade_trading_terminal_claim_allowed"):
        if semantics.get(key) is not False:
            reasons.append(f"live_trading_claim:{key}")

    report = artifact.get("readiness_report") or {}
    for key in ("report_id", "source_ref", "evidence_path"):
        if not isinstance(report.get(key), str) or not report[key].strip():
            reasons.append(f"missing_component:report:{key}")
    if report.get("auditable") is not True:
        reasons.append("missing_component:report_auditable")
    for key in ("source_linkage_status", "provenance_status", "freshness_status", "redaction_status"):
        expected = "linked" if key == "source_linkage_status" else ("verified" if key == "provenance_status" else ("fresh" if key == "freshness_status" else "redacted"))
        if report.get(key) != expected:
            reasons.append(f"missing_component:report:{key}")

    components = artifact.get("components")
    if not isinstance(components, list):
        reasons.append("missing_component:components_not_list")
        components = []
    by_component = {component.get("component_id"): component for component in components if isinstance(component, dict)}
    if list(by_component) != EXPECTED_COMPONENTS:
        reasons.append("missing_component:component_set")
    for component_id in EXPECTED_COMPONENTS:
        component = by_component.get(component_id)
        if component is None:
            reasons.append(f"missing_component:{component_id}")
            continue
        if component.get("component_status") == "partial" or component.get("component_status") == "degraded":
            if not component.get("degradation_reasons"):
                reasons.append(f"missing_component:degradation_reasons:{component_id}")
            reasons.append(f"partial_component:{component_id}")
        elif component.get("component_status") in {"stale", "blocked"}:
            if not component.get("degradation_reasons"):
                reasons.append(f"missing_component:stale_reasons:{component_id}")
            reasons.append(f"stale_component:{component_id}")
        elif component.get("component_status") != "ready":
            reasons.append(f"missing_component:status:{component_id}")
        for key in ("source_ref", "evidence_path", "verification_command"):
            if not isinstance(component.get(key), str) or not component[key].strip():
                reasons.append(f"missing_component:{component_id}:{key}")
        if component.get("provenance_status") != "verified" or component.get("freshness_status") not in {"fresh", "stale"} or component.get("redaction_status") != "redacted":
            reasons.append(f"missing_component:{component_id}:provenance_freshness_redaction")
        if component.get("freshness_status") == "stale":
            reasons.append(f"stale_component:{component_id}:freshness")
        if component.get("source_drift_status") != "aligned":
            reasons.append(f"source_drift:{component_id}")
        if component.get("read_only") is not True:
            reasons.append(f"forbidden_control:{component_id}:not_read_only")
        if component.get("backend_go_live_claim_allowed") is not False:
            reasons.append(f"backend_go_live_claim:{component_id}")
        if component.get("trading_readiness_claim_allowed") is not False:
            reasons.append(f"live_trading_claim:{component_id}")

    boundary = artifact.get("required_false_boundary_flags")
    if not isinstance(boundary, dict):
        reasons.append("missing_boundary:boundary_not_object")
        boundary = {}
    for key in BOUNDARY_FALSE_FLAGS:
        if key not in boundary:
            reasons.append(f"missing_boundary:{key}")
        elif boundary.get(key) is not False:
            reasons.append(f"forbidden_control:{key}")

    status = classify_status(reasons)
    return {
        "status": status,
        "degraded": status.startswith("degraded"),
        "blocked": status.startswith("blocked"),
        "fail_closed": status.startswith("fail_closed"),
        "blocking_reasons": reasons,
    }


artifact = json.loads(artifact_path.read_text(encoding="utf-8"))
matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
module = next((item for item in matrix.get("module_readiness", []) if item.get("module_id") == MODULE_ID), None)
if not module:
    fail("matrix missing backend_production_readiness_fail_closed_hardening")
if module.get("classification") != "production-ready" or module.get("issue") != 935:
    fail("backend production readiness hardening matrix entry mismatch")
if module.get("evidence_path") != "docs/rust-cutover/evidence/V290-009.md":
    fail("backend production readiness hardening matrix evidence path mismatch")
if module.get("verification_command") != "scripts/ai/verify_release.sh v29-backend-production-readiness-fail-closed-hardening":
    fail("backend production readiness hardening matrix verification command mismatch")

cases = artifact.get("readiness_cases")
if not isinstance(cases, list) or [case.get("case_id") for case in cases] != EXPECTED_CASES:
    fail("readiness cases mismatch")
allowed = degraded = blocked = fail_closed = 0
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
if allowed != 1 or degraded != 1 or blocked != 1 or fail_closed != 6:
    fail(f"unexpected case counts: allowed={allowed} degraded={degraded} blocked={blocked} fail_closed={fail_closed}")

if selftest:
    bad = copy.deepcopy(artifact)
    bad["required_false_boundary_flags"]["production_order_submission_allowed"] = True
    if classify_artifact(bad)["status"] != "fail_closed_forbidden_control":
        fail("negative self-test unexpectedly allowed production_order_submission_allowed")

print(
    "v29_backend_production_readiness_fail_closed_hardening=pass "
    f"cases={len(cases)} "
    f"allowed={allowed} "
    f"degraded={degraded} "
    f"blocked={blocked} "
    f"fail_closed={fail_closed} "
    f"components={len(EXPECTED_COMPONENTS)} "
    f"boundary_flags={len(BOUNDARY_FALSE_FLAGS)} "
    "negative_selftest=1"
)
PY
