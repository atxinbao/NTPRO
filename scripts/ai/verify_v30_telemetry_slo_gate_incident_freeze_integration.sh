#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

ARTIFACT_PATH="${NTPRO_V300_TELEMETRY_FREEZE_ARTIFACT:-docs/rust-cutover/release/v0_30_0_telemetry_slo_gate_incident_freeze_integration.json}"
CONTRACT_PATH="${NTPRO_V300_TELEMETRY_FREEZE_CONTRACT:-docs/rust-cutover/release/v0_30_0_telemetry_slo_gate_incident_freeze_integration.md}"
TASK_PATH="${NTPRO_V300_TELEMETRY_FREEZE_TASK:-docs/rust-cutover/tasks/V300-008.md}"
EVIDENCE_PATH="${NTPRO_V300_TELEMETRY_FREEZE_EVIDENCE:-docs/rust-cutover/evidence/V300-008.md}"
DEPLOYMENT_READINESS="${NTPRO_V300_TELEMETRY_FREEZE_DEPLOYMENT:-docs/rust-cutover/release/v0_30_0_production_deployment_plan_environment_readiness.json}"
OPERATOR_LIFECYCLE="${NTPRO_V300_TELEMETRY_FREEZE_OPERATOR:-docs/rust-cutover/release/v0_30_0_operator_approval_freeze_change_window_lifecycle.json}"
CONFIG_VENUE="${NTPRO_V300_TELEMETRY_FREEZE_CONFIG_VENUE:-docs/rust-cutover/release/v0_30_0_production_config_provenance_venue_connectivity_readiness.json}"
V29_TELEMETRY="${NTPRO_V300_TELEMETRY_FREEZE_V29_TELEMETRY:-docs/rust-cutover/release/v0_29_0_telemetry_slo_ingestion_production_readiness_artifact.json}"
V29_INCIDENT="${NTPRO_V300_TELEMETRY_FREEZE_V29_INCIDENT:-docs/rust-cutover/release/v0_29_0_monitoring_alert_incident_production_readiness_artifact.json}"
RELEASE_INDEX="${NTPRO_V300_TELEMETRY_FREEZE_RELEASE_INDEX:-docs/rust-cutover/release/README.md}"
SELFTEST="${NTPRO_V300_TELEMETRY_FREEZE_SELFTEST:-1}"

fail() {
  echo "v30 telemetry SLO gate incident freeze integration failed: $*" >&2
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

for path in "$ARTIFACT_PATH" "$CONTRACT_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$DEPLOYMENT_READINESS" "$OPERATOR_LIFECYCLE" "$CONFIG_VENUE" "$V29_TELEMETRY" "$V29_INCIDENT" "$RELEASE_INDEX"; do
  require_file "$path"
done

require_contains "$TASK_PATH" "GitHub issue: \`#977\`"
require_contains "$EVIDENCE_PATH" "Task: \`V300-008\` / GitHub issue \`#977\`"
require_contains "$CONTRACT_PATH" "contract_version = ntpro.v300.telemetry_slo_gate_incident_freeze_integration.v1"
require_contains "$CONTRACT_PATH" "telemetry_gate_mode = observability_only_release_blocking"
require_contains "$CONTRACT_PATH" "telemetry_action_effect_allowed = false"
require_contains "$CONTRACT_PATH" "stale_telemetry => fail_closed_stale_telemetry"
require_contains "$CONTRACT_PATH" "active_freeze_criteria => fail_closed_active_freeze_criteria"
require_contains "$RELEASE_INDEX" "v0_30_0_telemetry_slo_gate_incident_freeze_integration.md"
require_contains "$RELEASE_INDEX" "../evidence/V300-008.md"

ARTIFACT_PATH="$ARTIFACT_PATH" SELFTEST="$SELFTEST" python3 <<'PY'
from __future__ import annotations

import copy
import json
import os
from pathlib import Path
from typing import Any

payload = json.loads(Path(os.environ["ARTIFACT_PATH"]).read_text(encoding="utf-8"))
selftest = os.environ.get("SELFTEST", "1") != "0"

SCHEMA_VERSION = "ntpro.v300.telemetry_slo_gate_incident_freeze_integration.v1"
RELEASE_SCOPE = "backend_production_go_live_candidate_foundation_only"
READY_STATUS = "telemetry_slo_gate_incident_freeze_ready"
DEPENDENCIES = {"V300-002", "V300-004", "V300-007", "v0.29.1-release-evidence"}
EXPECTED_SOURCES = {"telemetry_ingestion", "slo_snapshot", "incident_snapshot"}
EXPECTED_SLOS = {
    "telemetry_ingestion_freshness",
    "read_api_availability",
    "audit_export_latency",
    "config_venue_readiness",
}
EXPECTED_ROUTES = {
    "slo-breach-to-operator-freeze",
    "critical-incident-to-release-block",
    "config-venue-readiness-to-release-block",
}
EXPECTED_FREEZE = {
    "critical_incident_active",
    "incident_freeze_active",
    "stale_telemetry_freeze",
    "degraded_slo_freeze",
}
EXPECTED_STATES = {"healthy_preview", "degraded", "stale", "critical_incident"}
EXPECTED_CASES = {
    "telemetry_slo_freeze.preview.allowed.001",
    "telemetry_slo_freeze.stale_telemetry.fail_closed.001",
    "telemetry_slo_freeze.missing_telemetry.fail_closed.001",
    "telemetry_slo_freeze.degraded_slo.fail_closed.001",
    "telemetry_slo_freeze.critical_incident.fail_closed.001",
    "telemetry_slo_freeze.active_freeze.fail_closed.001",
    "telemetry_slo_freeze.automatic_remediation.fail_closed.001",
    "telemetry_slo_freeze.telemetry_action.fail_closed.001",
    "telemetry_slo_freeze.retry_attempt.fail_closed.001",
    "telemetry_slo_freeze.forbidden_boundary.fail_closed.001",
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
]


def fail(message: str) -> None:
    raise SystemExit(f"v30 telemetry SLO gate incident freeze integration failed: {message}")


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
    if case.get("telemetry_source_overrides"):
        result["telemetry_sources"] = apply_indexed_overrides(
            result["telemetry_sources"],
            "source_id",
            case["telemetry_source_overrides"],
        )
    if case.get("slo_threshold_overrides"):
        result["slo_thresholds"] = apply_indexed_overrides(
            result["slo_thresholds"],
            "slo_id",
            case["slo_threshold_overrides"],
        )
    if case.get("alert_route_overrides"):
        result["alert_routes"] = apply_indexed_overrides(
            result["alert_routes"],
            "route_id",
            case["alert_route_overrides"],
        )
    if case.get("incident_freeze_overrides"):
        result["incident_freeze_criteria"] = apply_indexed_overrides(
            result["incident_freeze_criteria"],
            "criterion_id",
            case["incident_freeze_overrides"],
        )
    if case.get("boundary_flags_override"):
        result["boundary_flags"] = merge(result["boundary_flags"], case["boundary_flags_override"])
    return result


def classify_status(reasons: list[str]) -> str:
    if any(reason.startswith("forbidden_boundary") for reason in reasons):
        return "fail_closed_forbidden_boundary"
    if any(reason.startswith("forbidden_action") for reason in reasons):
        return "fail_closed_forbidden_action"
    if any(reason.startswith("critical_incident") for reason in reasons):
        return "fail_closed_critical_incident_freeze"
    if any(reason.startswith("active_freeze") for reason in reasons):
        return "fail_closed_active_freeze_criteria"
    if any(reason.startswith("stale_telemetry") for reason in reasons):
        return "fail_closed_stale_telemetry"
    if any(reason.startswith("missing_telemetry") for reason in reasons):
        return "fail_closed_missing_telemetry"
    if any(reason.startswith("degraded_slo") for reason in reasons):
        return "fail_closed_degraded_slo"
    if reasons:
        return "fail_closed_forbidden_boundary"
    return READY_STATUS


def collect_reasons(artifact: dict[str, Any]) -> list[str]:
    reasons: list[str] = []
    if artifact.get("schema_version") != SCHEMA_VERSION:
        reasons.append("forbidden_boundary:schema_version")
    if artifact.get("contract_version") != SCHEMA_VERSION:
        reasons.append("forbidden_boundary:contract_version")
    if artifact.get("task_id") != "V300-008" or artifact.get("github_issue") != 977:
        reasons.append("forbidden_boundary:task_identity")
    if artifact.get("milestone") != "v0.30.0" or artifact.get("release_scope") != RELEASE_SCOPE:
        reasons.append("forbidden_boundary:release_scope")
    if artifact.get("candidate_claim") != "telemetry_slo_gate_incident_freeze_integration":
        reasons.append("forbidden_boundary:candidate_claim")
    if artifact.get("telemetry_gate_mode") != "observability_only_release_blocking":
        reasons.append("forbidden_boundary:telemetry_gate_mode")
    if artifact.get("incident_freeze_mode") != "manual_review_gate_only":
        reasons.append("forbidden_boundary:incident_freeze_mode")
    for key in ("telemetry_action_effect_allowed", "automatic_remediation_allowed", "retry_scheduler_enabled", "adapter_send_allowed"):
        if artifact.get(key) is not False:
            reasons.append(f"forbidden_action:top_level:{key}")
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

    sources_raw = artifact.get("telemetry_sources")
    if not isinstance(sources_raw, list):
        reasons.append("missing_telemetry:sources")
        sources_raw = []
    sources: dict[str, dict[str, Any]] = {}
    for source in sources_raw:
        if not isinstance(source, dict):
            reasons.append("missing_telemetry:source_type")
            continue
        source_id = source.get("source_id")
        if not isinstance(source_id, str):
            reasons.append("missing_telemetry:source_id")
            continue
        sources[source_id] = source
        if source_id not in EXPECTED_SOURCES:
            reasons.append(f"missing_telemetry:unexpected_source:{source_id}")
        if source.get("presence_status") != "present":
            reasons.append(f"missing_telemetry:presence:{source_id}")
        if source.get("freshness_status") != "fresh":
            reasons.append(f"stale_telemetry:{source_id}")
        if not isinstance(source.get("max_age_seconds"), int) or source["max_age_seconds"] > 300:
            reasons.append(f"stale_telemetry:max_age:{source_id}")
        if source.get("release_blocking_on_stale") is not True:
            reasons.append(f"stale_telemetry:blocking:{source_id}")
        if source.get("action_effect_allowed") is not False:
            reasons.append(f"forbidden_action:source:{source_id}")
    if set(sources) != EXPECTED_SOURCES:
        reasons.append("missing_telemetry:source_set")

    slos_raw = artifact.get("slo_thresholds")
    if not isinstance(slos_raw, list):
        reasons.append("degraded_slo:list")
        slos_raw = []
    slos: dict[str, dict[str, Any]] = {}
    for slo in slos_raw:
        if not isinstance(slo, dict):
            reasons.append("degraded_slo:type")
            continue
        slo_id = slo.get("slo_id")
        if not isinstance(slo_id, str):
            reasons.append("degraded_slo:id")
            continue
        slos[slo_id] = slo
        if slo_id not in EXPECTED_SLOS:
            reasons.append(f"degraded_slo:unexpected:{slo_id}")
        if slo.get("status") != "pass" or slo.get("threshold_status") != "within_threshold":
            reasons.append(f"degraded_slo:status:{slo_id}")
        if slo.get("release_blocking_on_breach") is not True:
            reasons.append(f"degraded_slo:blocking:{slo_id}")
        if slo.get("automatic_action_allowed") is not False:
            reasons.append(f"forbidden_action:slo:{slo_id}")
    if set(slos) != EXPECTED_SLOS:
        reasons.append("degraded_slo:set")

    routes_raw = artifact.get("alert_routes")
    if not isinstance(routes_raw, list):
        reasons.append("missing_telemetry:routes")
        routes_raw = []
    routes: dict[str, dict[str, Any]] = {}
    for route in routes_raw:
        if not isinstance(route, dict):
            reasons.append("missing_telemetry:route_type")
            continue
        route_id = route.get("route_id")
        if not isinstance(route_id, str):
            reasons.append("missing_telemetry:route_id")
            continue
        routes[route_id] = route
        if route_id not in EXPECTED_ROUTES:
            reasons.append(f"missing_telemetry:unexpected_route:{route_id}")
        if route.get("route_status") != "linked":
            reasons.append(f"missing_telemetry:route_status:{route_id}")
        if not route.get("owner_ref"):
            reasons.append(f"missing_telemetry:route_owner:{route_id}")
        if route.get("operator_acknowledgement_required") is not True:
            reasons.append(f"active_freeze:route_ack:{route_id}")
        if route.get("automatic_action_allowed") is not False:
            reasons.append(f"forbidden_action:route:{route_id}")
    if set(routes) != EXPECTED_ROUTES:
        reasons.append("missing_telemetry:route_set")

    freeze_raw = artifact.get("incident_freeze_criteria")
    if not isinstance(freeze_raw, list):
        reasons.append("active_freeze:list")
        freeze_raw = []
    freeze: dict[str, dict[str, Any]] = {}
    for criterion in freeze_raw:
        if not isinstance(criterion, dict):
            reasons.append("active_freeze:type")
            continue
        criterion_id = criterion.get("criterion_id")
        if not isinstance(criterion_id, str):
            reasons.append("active_freeze:id")
            continue
        freeze[criterion_id] = criterion
        if criterion_id not in EXPECTED_FREEZE:
            reasons.append(f"active_freeze:unexpected:{criterion_id}")
        if criterion.get("criterion_status") != "armed_inactive":
            reasons.append(f"active_freeze:status:{criterion_id}")
        if criterion.get("active") is not False:
            if criterion_id == "critical_incident_active":
                reasons.append(f"critical_incident:{criterion_id}")
            else:
                reasons.append(f"active_freeze:{criterion_id}")
        if criterion.get("release_blocking_when_active") is not True:
            reasons.append(f"active_freeze:blocking:{criterion_id}")
        if criterion.get("automatic_remediation_allowed") is not False:
            reasons.append(f"forbidden_action:freeze:{criterion_id}")
    if set(freeze) != EXPECTED_FREEZE:
        reasons.append("active_freeze:set")

    states_raw = artifact.get("release_blocking_health_states")
    if not isinstance(states_raw, list):
        reasons.append("degraded_slo:states")
        states_raw = []
    states = {state.get("state_id"): state for state in states_raw if isinstance(state, dict)}
    if set(states) != EXPECTED_STATES:
        reasons.append("degraded_slo:state_set")
    healthy = states.get("healthy_preview") or {}
    if healthy.get("go_no_go_candidate_advancement_allowed") is not True or healthy.get("runtime_execution_allowed") is not False:
        reasons.append("degraded_slo:healthy_state")
    for state_id in ("degraded", "stale", "critical_incident"):
        state = states.get(state_id) or {}
        if state.get("go_no_go_candidate_advancement_allowed") is not False:
            reasons.append(f"degraded_slo:state_advancement:{state_id}")
        if state.get("runtime_execution_allowed") is not False:
            reasons.append(f"forbidden_action:state_execution:{state_id}")

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
    stale = copy.deepcopy(payload)
    stale["telemetry_sources"][0]["freshness_status"] = "stale"
    if classify_status(collect_reasons(stale)) == "fail_closed_stale_telemetry":
        negative_selftests += 1
    else:
        fail("negative self-test unexpectedly allowed stale telemetry")

    degraded = copy.deepcopy(payload)
    degraded["slo_thresholds"][1]["status"] = "degraded"
    if classify_status(collect_reasons(degraded)) == "fail_closed_degraded_slo":
        negative_selftests += 1
    else:
        fail("negative self-test unexpectedly allowed degraded SLO")

    freeze = copy.deepcopy(payload)
    freeze["incident_freeze_criteria"][1]["active"] = True
    if classify_status(collect_reasons(freeze)) == "fail_closed_active_freeze_criteria":
        negative_selftests += 1
    else:
        fail("negative self-test unexpectedly allowed active freeze")

    action = copy.deepcopy(payload)
    action["alert_routes"][0]["automatic_action_allowed"] = True
    if classify_status(collect_reasons(action)) == "fail_closed_forbidden_action":
        negative_selftests += 1
    else:
        fail("negative self-test unexpectedly allowed telemetry action")

print(
    "v30_telemetry_slo_gate_incident_freeze_integration=pass "
    f"telemetry_sources={len(EXPECTED_SOURCES)} "
    f"slo_thresholds={len(EXPECTED_SLOS)} "
    f"alert_routes={len(EXPECTED_ROUTES)} "
    f"incident_freeze_criteria={len(EXPECTED_FREEZE)} "
    f"readiness_cases={len(EXPECTED_CASES)} "
    f"required_false_flags={len(REQUIRED_FALSE_FLAGS)} "
    f"negative_selftest={negative_selftests}"
)
PY
