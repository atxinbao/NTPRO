#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

ARTIFACT_PATH="${NTPRO_V290_MONITORING_INCIDENT_ARTIFACT:-docs/rust-cutover/release/v0_29_0_monitoring_alert_incident_production_readiness_artifact.json}"
CONTRACT_PATH="${NTPRO_V290_MONITORING_INCIDENT_CONTRACT:-docs/rust-cutover/release/v0_29_0_monitoring_alert_incident_production_readiness.md}"
TASK_PATH="${NTPRO_V290_MONITORING_INCIDENT_TASK:-docs/rust-cutover/tasks/V290-007.md}"
EVIDENCE_PATH="${NTPRO_V290_MONITORING_INCIDENT_EVIDENCE:-docs/rust-cutover/evidence/V290-007.md}"
MATRIX_PATH="${NTPRO_V290_MONITORING_INCIDENT_MATRIX:-docs/rust-cutover/release/v0_29_0_backend_production_readiness_matrix.json}"
BOUNDARY_CONTRACT_PATH="${NTPRO_V290_MONITORING_INCIDENT_BOUNDARY_CONTRACT:-docs/rust-cutover/release/v0_29_0_backend_production_readiness_boundary_contract.md}"
V25_MONITORING_PATH="${NTPRO_V290_MONITORING_INCIDENT_V25_MONITORING:-docs/rust-cutover/release/v0_25_0_monitoring_observability_contract.md}"
V25_ALERT_PATH="${NTPRO_V290_MONITORING_INCIDENT_V25_ALERT:-docs/rust-cutover/release/v0_25_0_alert_taxonomy_routing.md}"
V25_INCIDENT_PATH="${NTPRO_V290_MONITORING_INCIDENT_V25_INCIDENT:-docs/rust-cutover/release/v0_25_0_incident_lifecycle_acknowledgement.md}"
TELEMETRY_ARTIFACT_PATH="${NTPRO_V290_MONITORING_INCIDENT_TELEMETRY_ARTIFACT:-docs/rust-cutover/release/v0_29_0_telemetry_slo_ingestion_production_readiness_artifact.json}"
AUDIT_ARTIFACT_PATH="${NTPRO_V290_MONITORING_INCIDENT_AUDIT_ARTIFACT:-docs/rust-cutover/release/v0_29_0_persistent_audit_storage_production_readiness_artifact.json}"
RUNBOOK_ARTIFACT_PATH="${NTPRO_V290_MONITORING_INCIDENT_RUNBOOK_ARTIFACT:-docs/rust-cutover/release/v0_29_0_deployment_config_runbook_production_readiness_artifact.json}"
INTAKE_PATH="${NTPRO_V290_MONITORING_INCIDENT_INTAKE:-docs/rust-cutover/release/v0_29_0_intake_gate.md}"
SELFTEST="${NTPRO_V290_MONITORING_INCIDENT_SELFTEST:-1}"

fail() {
  echo "v29 monitoring alert incident production readiness failed: $*" >&2
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

for path in "$ARTIFACT_PATH" "$CONTRACT_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$MATRIX_PATH" "$BOUNDARY_CONTRACT_PATH" "$V25_MONITORING_PATH" "$V25_ALERT_PATH" "$V25_INCIDENT_PATH" "$TELEMETRY_ARTIFACT_PATH" "$AUDIT_ARTIFACT_PATH" "$RUNBOOK_ARTIFACT_PATH" "$INTAKE_PATH"; do
  require_file "$path"
done

require_contains "$TASK_PATH" "GitHub issue: \`#933\`"
require_contains "$EVIDENCE_PATH" "Task: \`V290-007\` / GitHub issue \`#933\`"
require_contains "$INTAKE_PATH" "start_gate_status = satisfied"
require_contains "$BOUNDARY_CONTRACT_PATH" "contract_version = ntpro.v290.backend_production_readiness_boundary.v1"
require_contains "$CONTRACT_PATH" "contract_version = ntpro.v290.monitoring_alert_incident_production_readiness.v1"
require_contains "$CONTRACT_PATH" "schema_version = ntpro.v290.monitoring_alert_incident_production_readiness_artifact.v1"
require_contains "$CONTRACT_PATH" "incident_mode = manual_read_only_handoff"
require_contains "$CONTRACT_PATH" "external_paging_service_connected = false"
require_contains "$CONTRACT_PATH" "external_ticket_mutation_allowed = false"
require_contains "$CONTRACT_PATH" "automatic_incident_generation_allowed = false"
require_contains "$CONTRACT_PATH" "release stage = scripts/ai/verify_release.sh v29-monitoring-alert-incident-production-readiness"

ARTIFACT_PATH="$ARTIFACT_PATH" MATRIX_PATH="$MATRIX_PATH" TELEMETRY_ARTIFACT_PATH="$TELEMETRY_ARTIFACT_PATH" AUDIT_ARTIFACT_PATH="$AUDIT_ARTIFACT_PATH" RUNBOOK_ARTIFACT_PATH="$RUNBOOK_ARTIFACT_PATH" SELFTEST="$SELFTEST" python3 <<'PY'
from __future__ import annotations

import copy
import json
import os
from pathlib import Path
from typing import Any

artifact_path = Path(os.environ["ARTIFACT_PATH"])
matrix_path = Path(os.environ["MATRIX_PATH"])
telemetry_artifact_path = Path(os.environ["TELEMETRY_ARTIFACT_PATH"])
audit_artifact_path = Path(os.environ["AUDIT_ARTIFACT_PATH"])
runbook_artifact_path = Path(os.environ["RUNBOOK_ARTIFACT_PATH"])
selftest = os.environ.get("SELFTEST", "1") != "0"

SCHEMA_VERSION = "ntpro.v290.monitoring_alert_incident_production_readiness_artifact.v1"
CONTRACT_VERSION = "ntpro.v290.monitoring_alert_incident_production_readiness.v1"
MODULE_ID = "monitoring_alert_incident_production_readiness"
DEPENDENCIES = {
    "V290-000",
    "V290-001",
    "V290-002",
    "V290-003",
    "V290-006",
    "V250-001",
    "V250-002",
    "V250-003",
    "v0.28.1-release-evidence",
}
EXPECTED_CASES = [
    "monitoring_alert_incident.production_readiness.ready.allowed.001",
    "monitoring_alert_incident.production_readiness.slo_breach.degraded.001",
    "monitoring_alert_incident.production_readiness.stale_alert_source.fail_closed.001",
    "monitoring_alert_incident.production_readiness.missing_owner_routing.fail_closed.001",
    "monitoring_alert_incident.production_readiness.missing_acknowledgement.fail_closed.001",
    "monitoring_alert_incident.production_readiness.missing_audit.fail_closed.001",
    "monitoring_alert_incident.production_readiness.unsafe_auto_remediation.fail_closed.001",
    "monitoring_alert_incident.production_readiness.forbidden_boundary.fail_closed.001",
]
EXPECTED_SEVERITY = ["info", "warning", "critical", "halt"]
EXPECTED_CATEGORIES = {
    "stale_data",
    "missing_provenance",
    "risk_fail_closed",
    "order_control_preview_blocked",
    "release_gate_drift",
    "production_readiness_slo_breach",
}
EXPECTED_STATES = ["opened", "triaged", "acknowledged", "mitigated", "resolved", "postmortem"]
EXPECTED_TRANSITIONS = [
    "none->opened",
    "opened->triaged",
    "triaged->acknowledged",
    "acknowledged->mitigated",
    "mitigated->resolved",
    "resolved->postmortem",
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
    "external_paging_service_connected",
    "automatic_paging_allowed",
    "external_ticket_mutation_allowed",
    "automatic_incident_generation_allowed",
    "automatic_strategy_stop_allowed",
    "incident_state_triggers_remediation",
    "slo_breach_triggers_trading_control",
]


def fail(message: str) -> None:
    raise SystemExit(f"v29 monitoring alert incident production readiness failed: {message}")


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
    for key in ("monitoring_source", "alert_taxonomy", "incident_lifecycle", "boundary_flags"):
        override = case.get(f"{key}_override")
        if override:
            result[key] = merge(result[key], override)
    for transition_id, override in (case.get("transition_overrides") or {}).items():
        for transition in result["incident_transitions"]:
            if transition.get("transition_id") == transition_id:
                transition.update(merge(transition, override))
                break
    return result


def classify_status(reasons: list[str]) -> str:
    if any(reason.startswith("stale_alert_source") for reason in reasons):
        return "fail_closed_stale_alert_source"
    if any(reason.startswith("missing_owner_routing") for reason in reasons):
        return "fail_closed_missing_owner_routing"
    if any(reason.startswith("missing_acknowledgement") for reason in reasons):
        return "fail_closed_missing_acknowledgement_semantics"
    if any(reason.startswith("missing_audit") for reason in reasons):
        return "fail_closed_missing_audit_requirements"
    if any(reason.startswith("unsafe_auto_remediation") for reason in reasons):
        return "fail_closed_unsafe_auto_remediation"
    if any(reason.startswith("forbidden_boundary") for reason in reasons):
        return "fail_closed_forbidden_operation_boundary"
    if any(reason.startswith("slo_breach") for reason in reasons):
        return "degraded_slo_breach_manual_handoff"
    if reasons:
        return "fail_closed_monitoring_alert_incident_readiness_violation"
    return "monitoring_alert_incident_readiness_ready"


def classify_artifact(artifact: dict[str, Any]) -> dict[str, Any]:
    reasons: list[str] = []
    if artifact.get("schema_version") != SCHEMA_VERSION or artifact.get("contract_version") != CONTRACT_VERSION:
        reasons.append("missing_audit:schema_contract")
    if artifact.get("task_id") != "V290-007" or artifact.get("github_issue") != 933:
        reasons.append("missing_audit:task_identity")
    if artifact.get("release_scope") != "backend_production_readiness_foundation_only":
        reasons.append("missing_audit:release_scope")
    if artifact.get("backend_module") != MODULE_ID or artifact.get("backend_module_status") != "production_ready_evidence":
        reasons.append("missing_audit:module")
    if artifact.get("readiness_mode") != "deterministic_readiness_replay":
        reasons.append("missing_audit:readiness_mode")
    if artifact.get("incident_mode") != "manual_read_only_handoff":
        reasons.append("forbidden_boundary:incident_mode")
    if set(artifact.get("dependency_contracts") or []) != DEPENDENCIES:
        reasons.append("missing_audit:dependencies")

    source = artifact.get("monitoring_source") or {}
    refs = source.get("source_refs")
    if not isinstance(refs, list) or not refs:
        reasons.append("stale_alert_source:missing_source_refs")
    for required in (
        "docs/rust-cutover/release/v0_25_0_monitoring_observability_contract.md",
        "docs/rust-cutover/release/v0_25_0_alert_taxonomy_routing.md",
        "docs/rust-cutover/release/v0_25_0_incident_lifecycle_acknowledgement.md",
        str(telemetry_artifact_path),
    ):
        if not isinstance(refs, list) or required not in refs:
            reasons.append(f"stale_alert_source:missing_ref:{required}")
    if source.get("operator_handoff_ref") != str(runbook_artifact_path):
        reasons.append("missing_owner_routing:operator_handoff_ref")
    if source.get("freshness_status") != "fresh" or source.get("source_provenance_status") != "linked":
        reasons.append("stale_alert_source:freshness_or_provenance")
    if source.get("redaction_status") != "redacted":
        reasons.append("missing_audit:redaction")
    if source.get("slo_status") == "breach":
        reasons.append("slo_breach:monitoring_source")
    elif source.get("slo_status") != "within_target":
        reasons.append("stale_alert_source:slo_status")
    if source.get("slo_breach_handling") != "manual_audit_only":
        reasons.append("unsafe_auto_remediation:slo_breach_handling")

    alert = artifact.get("alert_taxonomy") or {}
    if alert.get("severity_order") != EXPECTED_SEVERITY:
        reasons.append("missing_owner_routing:severity_order")
    if set(alert.get("categories") or []) != EXPECTED_CATEGORIES:
        reasons.append("missing_owner_routing:categories")
    if alert.get("owner_routing_status") != "routed":
        reasons.append("missing_owner_routing:owner_routing_status")
    if alert.get("acknowledgement_semantics") != "documented":
        reasons.append("missing_acknowledgement:alert_semantics")
    if alert.get("audit_requirement") != "required":
        reasons.append("missing_audit:alert_audit_requirement")
    if alert.get("escalation_paths_status") != "documented":
        reasons.append("missing_owner_routing:escalation_paths")
    if alert.get("stale_source_allowed") is not False:
        reasons.append("stale_alert_source:stale_source_allowed")
    if alert.get("external_paging_service_connected") is not False:
        reasons.append("forbidden_boundary:external_paging_service_connected")
    if alert.get("automatic_remediation_allowed") is not False:
        reasons.append("unsafe_auto_remediation:alert_taxonomy")

    incident = artifact.get("incident_lifecycle") or {}
    if incident.get("states") != EXPECTED_STATES:
        reasons.append("missing_acknowledgement:states")
    if incident.get("allowed_transitions") != EXPECTED_TRANSITIONS:
        reasons.append("missing_acknowledgement:allowed_transitions")
    for key in ("owner_required", "assignee_required", "source_alert_required"):
        if incident.get(key) is not True:
            reasons.append(f"missing_owner_routing:{key}")
    if incident.get("acknowledgement_required_before_resolved") is not True:
        reasons.append("missing_acknowledgement:before_resolved")
    if incident.get("audit_trace_required") is not True:
        reasons.append("missing_audit:incident_audit_trace_required")
    if incident.get("manual_handoff_only") is not True:
        reasons.append("unsafe_auto_remediation:manual_handoff_only")
    for key in ("external_ticket_mutation_allowed", "automatic_incident_generation_allowed", "automatic_strategy_stop_allowed"):
        if incident.get(key) is not False:
            reasons.append(f"forbidden_boundary:incident:{key}")

    transitions = artifact.get("incident_transitions")
    if not isinstance(transitions, list) or [item.get("transition_id") for item in transitions] != EXPECTED_TRANSITIONS:
        reasons.append("missing_acknowledgement:incident_transition_set")
        transitions = transitions if isinstance(transitions, list) else []
    for transition in transitions:
        transition_id = transition.get("transition_id")
        if transition.get("manual_operator_action") is not True:
            reasons.append(f"unsafe_auto_remediation:transition_manual:{transition_id}")
        if transition.get("owner_route_present") is not True:
            reasons.append(f"missing_owner_routing:transition:{transition_id}")
        if transition_id in {"triaged->acknowledged", "acknowledged->mitigated", "mitigated->resolved", "resolved->postmortem"} and transition.get("acknowledgement_present") is not True:
            reasons.append(f"missing_acknowledgement:transition:{transition_id}")
        if transition.get("audit_trace_status") != "linked":
            reasons.append(f"missing_audit:transition:{transition_id}")
        if transition.get("operation_effect") != "evidence_only" or transition.get("automatic_action_triggered") is not False:
            reasons.append(f"unsafe_auto_remediation:transition_effect:{transition_id}")

    boundary = artifact.get("boundary_flags") or {}
    for key in BOUNDARY_FALSE_FLAGS:
        if boundary.get(key) is not False:
            reasons.append(f"forbidden_boundary:{key}")

    status = classify_status(reasons)
    return {"status": status, "fail_closed": status.startswith("fail_closed"), "degraded": status.startswith("degraded"), "blocking_reasons": reasons}


artifact = json.loads(artifact_path.read_text(encoding="utf-8"))
telemetry_artifact = json.loads(telemetry_artifact_path.read_text(encoding="utf-8"))
if telemetry_artifact.get("schema_version") != "ntpro.v290.telemetry_slo_ingestion_production_readiness_artifact.v1":
    fail("telemetry artifact schema mismatch")
audit_artifact = json.loads(audit_artifact_path.read_text(encoding="utf-8"))
if audit_artifact.get("schema_version") != "ntpro.v290.persistent_audit_storage_production_readiness_artifact.v1":
    fail("audit artifact schema mismatch")
runbook_artifact = json.loads(runbook_artifact_path.read_text(encoding="utf-8"))
if runbook_artifact.get("schema_version") != "ntpro.v290.deployment_config_runbook_production_readiness_artifact.v1":
    fail("deployment config runbook artifact schema mismatch")

matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
module = next((item for item in matrix.get("module_readiness", []) if item.get("module_id") == MODULE_ID), None)
if not module:
    fail("matrix missing monitoring_alert_incident_production_readiness")
if module.get("classification") != "production-ready" or module.get("issue") != 933:
    fail("monitoring alert incident matrix entry mismatch")
if module.get("evidence_path") != "docs/rust-cutover/evidence/V290-007.md":
    fail("monitoring alert incident matrix evidence path mismatch")
if module.get("verification_command") != "scripts/ai/verify_release.sh v29-monitoring-alert-incident-production-readiness":
    fail("monitoring alert incident matrix verification command mismatch")

cases = artifact.get("readiness_cases")
if not isinstance(cases, list) or [case.get("case_id") for case in cases] != EXPECTED_CASES:
    fail("readiness cases mismatch")
allowed = degraded = fail_closed = 0
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
if allowed != 1 or degraded != 1 or fail_closed != 6:
    fail(f"unexpected case counts: allowed={allowed} degraded={degraded} fail_closed={fail_closed}")

if selftest:
    bad = copy.deepcopy(artifact)
    bad["boundary_flags"]["slo_breach_triggers_trading_control"] = True
    if classify_artifact(bad)["status"] != "fail_closed_forbidden_operation_boundary":
        fail("negative self-test unexpectedly allowed SLO breach trading control")

print(
    "v29_monitoring_alert_incident_production_readiness=pass "
    f"cases={len(cases)} "
    f"allowed={allowed} "
    f"degraded={degraded} "
    f"fail_closed={fail_closed} "
    f"incident_transitions={len(artifact['incident_transitions'])} "
    f"boundary_flags={len(BOUNDARY_FALSE_FLAGS)} "
    "negative_selftest=1"
)
PY
