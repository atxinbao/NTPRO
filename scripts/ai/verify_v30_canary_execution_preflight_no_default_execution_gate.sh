#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

ARTIFACT_PATH="${NTPRO_V300_CANARY_PREFLIGHT_ARTIFACT:-docs/rust-cutover/release/v0_30_0_canary_execution_preflight_no_default_execution_gate.json}"
CONTRACT_PATH="${NTPRO_V300_CANARY_PREFLIGHT_CONTRACT:-docs/rust-cutover/release/v0_30_0_canary_execution_preflight_no_default_execution_gate.md}"
TASK_PATH="${NTPRO_V300_CANARY_PREFLIGHT_TASK:-docs/rust-cutover/tasks/V300-005.md}"
EVIDENCE_PATH="${NTPRO_V300_CANARY_PREFLIGHT_EVIDENCE:-docs/rust-cutover/evidence/V300-005.md}"
DEPLOYMENT_READINESS="${NTPRO_V300_CANARY_PREFLIGHT_DEPLOYMENT:-docs/rust-cutover/release/v0_30_0_production_deployment_plan_environment_readiness.json}"
RUNTIME_FLAGS="${NTPRO_V300_CANARY_PREFLIGHT_RUNTIME_FLAGS:-docs/rust-cutover/release/v0_30_0_runtime_enablement_boundary_controlled_feature_flags.json}"
OPERATOR_LIFECYCLE="${NTPRO_V300_CANARY_PREFLIGHT_OPERATOR:-docs/rust-cutover/release/v0_30_0_operator_approval_freeze_change_window_lifecycle.json}"
V29_TELEMETRY="${NTPRO_V300_CANARY_PREFLIGHT_V29_TELEMETRY:-docs/rust-cutover/release/v0_29_0_telemetry_slo_ingestion_production_readiness_artifact.json}"
V29_INCIDENT="${NTPRO_V300_CANARY_PREFLIGHT_V29_INCIDENT:-docs/rust-cutover/release/v0_29_0_monitoring_alert_incident_production_readiness_artifact.json}"
V29_ROLLBACK="${NTPRO_V300_CANARY_PREFLIGHT_V29_ROLLBACK:-docs/rust-cutover/release/v0_29_0_canary_rollback_dr_preflight_readiness_artifact.json}"
RELEASE_INDEX="${NTPRO_V300_CANARY_PREFLIGHT_RELEASE_INDEX:-docs/rust-cutover/release/README.md}"
SELFTEST="${NTPRO_V300_CANARY_PREFLIGHT_SELFTEST:-1}"

fail() {
  echo "v30 canary execution preflight no-default-execution gate failed: $*" >&2
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

for path in "$ARTIFACT_PATH" "$CONTRACT_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$DEPLOYMENT_READINESS" "$RUNTIME_FLAGS" "$OPERATOR_LIFECYCLE" "$V29_TELEMETRY" "$V29_INCIDENT" "$V29_ROLLBACK" "$RELEASE_INDEX"; do
  require_file "$path"
done

require_contains "$TASK_PATH" "GitHub issue: \`#974\`"
require_contains "$EVIDENCE_PATH" "Task: \`V300-005\` / GitHub issue \`#974\`"
require_contains "$CONTRACT_PATH" "contract_version = ntpro.v300.canary_execution_preflight_no_default_execution_gate.v1"
require_contains "$CONTRACT_PATH" "no_default_execution_gate = closed"
require_contains "$CONTRACT_PATH" "canary_execution_allowed = false"
require_contains "$CONTRACT_PATH" "live_exchange_side_effect_allowed = false"
require_contains "$CONTRACT_PATH" "execution_true => fail_closed_forbidden_execution"
require_contains "$CONTRACT_PATH" "live_exchange_side_effect => fail_closed_live_exchange_side_effect"
require_contains "$RELEASE_INDEX" "v0_30_0_canary_execution_preflight_no_default_execution_gate.md"
require_contains "$RELEASE_INDEX" "../evidence/V300-005.md"

ARTIFACT_PATH="$ARTIFACT_PATH" SELFTEST="$SELFTEST" python3 <<'PY'
from __future__ import annotations

import copy
import json
import os
from pathlib import Path
from typing import Any

payload = json.loads(Path(os.environ["ARTIFACT_PATH"]).read_text(encoding="utf-8"))
selftest = os.environ.get("SELFTEST", "1") != "0"

SCHEMA_VERSION = "ntpro.v300.canary_execution_preflight_no_default_execution_gate.v1"
RELEASE_SCOPE = "backend_production_go_live_candidate_foundation_only"
READY_STATUS = "canary_execution_preflight_no_default_execution_ready"
DEPENDENCIES = {"V300-002", "V300-003", "V300-004", "v0.29.1-release-evidence"}
EXPECTED_LINKS = {
    "deployment_readiness",
    "runtime_flag_boundary",
    "operator_lifecycle",
    "telemetry_slo_evidence",
    "incident_freeze_evidence",
    "rollback_evidence",
}
EXPECTED_CHECKS = {
    "canary-eligibility",
    "deployment-readiness-link",
    "runtime-flags-default-disabled",
    "operator-acknowledgement",
    "no-default-execution",
}
EXPECTED_ABORTS = {
    "stale_source_detected",
    "operator_ack_missing",
    "telemetry_slo_breach",
    "incident_freeze_active",
    "rollback_readiness_missing",
    "live_exchange_side_effect_detected",
}
EXPECTED_CASES = {
    "canary_execution_preflight.preview.allowed.001",
    "canary_execution_preflight.missing_eligibility.fail_closed.001",
    "canary_execution_preflight.stale_source.fail_closed.001",
    "canary_execution_preflight.missing_operator_ack.fail_closed.001",
    "canary_execution_preflight.missing_linked_evidence.fail_closed.001",
    "canary_execution_preflight.missing_abort_criteria.fail_closed.001",
    "canary_execution_preflight.default_execution_open.fail_closed.001",
    "canary_execution_preflight.execution_true.fail_closed.001",
    "canary_execution_preflight.live_exchange_side_effect.fail_closed.001",
    "canary_execution_preflight.forbidden_boundary.fail_closed.001",
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
]


def fail(message: str) -> None:
    raise SystemExit(f"v30 canary execution preflight no-default-execution gate failed: {message}")


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
    if case.get("canary_eligibility_override"):
        result["canary_eligibility"] = merge(result["canary_eligibility"], case["canary_eligibility_override"])
    if case.get("evidence_link_overrides"):
        result["evidence_links"] = apply_indexed_overrides(
            result["evidence_links"],
            "link_id",
            case["evidence_link_overrides"],
        )
    if case.get("preflight_check_overrides"):
        result["preflight_checks"] = apply_indexed_overrides(
            result["preflight_checks"],
            "check_id",
            case["preflight_check_overrides"],
        )
    if case.get("abort_criteria_overrides"):
        result["abort_criteria"] = apply_indexed_overrides(
            result["abort_criteria"],
            "criterion_id",
            case["abort_criteria_overrides"],
        )
    if case.get("boundary_flags_override"):
        result["boundary_flags"] = merge(result["boundary_flags"], case["boundary_flags_override"])
    return result


def classify_status(reasons: list[str]) -> str:
    if any(reason.startswith("forbidden_boundary") for reason in reasons):
        return "fail_closed_forbidden_boundary"
    if any(reason.startswith("live_exchange_side_effect") for reason in reasons):
        return "fail_closed_live_exchange_side_effect"
    if any(reason.startswith("forbidden_execution") for reason in reasons):
        return "fail_closed_forbidden_execution"
    if any(reason.startswith("default_execution_open") for reason in reasons):
        return "fail_closed_default_execution_open"
    if any(reason.startswith("missing_canary_eligibility") for reason in reasons):
        return "fail_closed_missing_canary_eligibility"
    if any(reason.startswith("stale_source") for reason in reasons):
        return "fail_closed_stale_source"
    if any(reason.startswith("missing_operator_acknowledgement") for reason in reasons):
        return "fail_closed_missing_operator_acknowledgement"
    if any(reason.startswith("missing_linked_evidence") for reason in reasons):
        return "fail_closed_missing_linked_evidence"
    if any(reason.startswith("missing_abort_criteria") for reason in reasons):
        return "fail_closed_missing_abort_criteria"
    if reasons:
        return "fail_closed_forbidden_boundary"
    return READY_STATUS


def collect_reasons(artifact: dict[str, Any]) -> list[str]:
    reasons: list[str] = []
    if artifact.get("schema_version") != SCHEMA_VERSION:
        reasons.append("forbidden_boundary:schema_version")
    if artifact.get("contract_version") != SCHEMA_VERSION:
        reasons.append("forbidden_boundary:contract_version")
    if artifact.get("task_id") != "V300-005" or artifact.get("github_issue") != 974:
        reasons.append("forbidden_boundary:task_identity")
    if artifact.get("milestone") != "v0.30.0" or artifact.get("release_scope") != RELEASE_SCOPE:
        reasons.append("forbidden_boundary:release_scope")
    if artifact.get("candidate_claim") != "canary_execution_preflight_no_default_execution_gate":
        reasons.append("forbidden_boundary:candidate_claim")
    if artifact.get("preflight_mode") != "source_controlled_preflight_only":
        reasons.append("forbidden_boundary:preflight_mode")
    if artifact.get("no_default_execution_gate") != "closed":
        reasons.append("default_execution_open:gate")
    if artifact.get("canary_execution_allowed") is not False:
        reasons.append("forbidden_execution:top_level_canary")
    if artifact.get("default_canary_execution_allowed") is not False:
        reasons.append("default_execution_open:top_level_default")
    if artifact.get("production_canary_action_executed") is not False:
        reasons.append("forbidden_execution:production_canary_action")
    if artifact.get("live_exchange_side_effect_allowed") is not False:
        reasons.append("live_exchange_side_effect:allowed")
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

    canary = artifact.get("canary_eligibility") or {}
    if canary.get("plan_id") != "v300-canary-preflight":
        reasons.append("missing_canary_eligibility:plan_id")
    if canary.get("environment_id") != "prod-candidate-canary":
        reasons.append("missing_canary_eligibility:environment")
    if canary.get("eligibility_status") != "eligible_for_preview":
        reasons.append("missing_canary_eligibility:status")
    if canary.get("source_provenance_status") != "linked":
        reasons.append("missing_linked_evidence:canary_source")
    if canary.get("freshness_status") != "fresh":
        reasons.append("stale_source:canary")
    if canary.get("operator_acknowledgement_status") != "acknowledged_for_preview_only":
        reasons.append("missing_operator_acknowledgement:status")
    if not canary.get("operator_ack_ref"):
        reasons.append("missing_operator_acknowledgement:ref")
    if canary.get("blast_radius_percent") != 0 or canary.get("max_blast_radius_percent") != 0:
        reasons.append("forbidden_execution:blast_radius")
    for key in ("canary_execution_allowed", "production_canary_action_executed"):
        if canary.get(key) is not False:
            reasons.append(f"forbidden_execution:canary:{key}")
    if canary.get("live_exchange_side_effect_allowed") is not False:
        reasons.append("live_exchange_side_effect:canary")

    links_raw = artifact.get("evidence_links")
    if not isinstance(links_raw, list):
        reasons.append("missing_linked_evidence:list")
        links_raw = []
    links: dict[str, dict[str, Any]] = {}
    for link in links_raw:
        if not isinstance(link, dict):
            reasons.append("missing_linked_evidence:entry_type")
            continue
        link_id = link.get("link_id")
        if not isinstance(link_id, str):
            reasons.append("missing_linked_evidence:id")
            continue
        links[link_id] = link
        if link_id not in EXPECTED_LINKS:
            reasons.append(f"missing_linked_evidence:unexpected:{link_id}")
        if not str(link.get("link_status", "")).startswith("linked"):
            reasons.append(f"missing_linked_evidence:status:{link_id}")
        if link.get("freshness_status") != "fresh":
            reasons.append(f"stale_source:link:{link_id}")
        if not link.get("source_ref"):
            reasons.append(f"missing_linked_evidence:source:{link_id}")
        if link.get("execution_authorized") is not False:
            reasons.append(f"forbidden_execution:link:{link_id}")
    if set(links) != EXPECTED_LINKS:
        reasons.append("missing_linked_evidence:set")

    checks_raw = artifact.get("preflight_checks")
    if not isinstance(checks_raw, list):
        reasons.append("missing_canary_eligibility:checks")
        checks_raw = []
    checks: dict[str, dict[str, Any]] = {}
    for check in checks_raw:
        if not isinstance(check, dict):
            reasons.append("missing_canary_eligibility:check_type")
            continue
        check_id = check.get("check_id")
        if not isinstance(check_id, str):
            reasons.append("missing_canary_eligibility:check_id")
            continue
        checks[check_id] = check
        if check_id not in EXPECTED_CHECKS:
            reasons.append(f"missing_canary_eligibility:unexpected_check:{check_id}")
        if check.get("status") != "pass":
            reasons.append(f"missing_canary_eligibility:check_status:{check_id}")
        if check.get("preview_only") is not True:
            reasons.append(f"forbidden_execution:check_preview:{check_id}")
        if check.get("execution_triggered") is not False:
            reasons.append(f"forbidden_execution:check_execution:{check_id}")
        if check.get("live_exchange_side_effect_detected") is not False:
            reasons.append(f"live_exchange_side_effect:check:{check_id}")
        if check.get("operation_effect") not in {"validated_only", "blocked_execution"}:
            reasons.append(f"forbidden_execution:operation_effect:{check_id}")
    if set(checks) != EXPECTED_CHECKS:
        reasons.append("missing_canary_eligibility:check_set")

    aborts_raw = artifact.get("abort_criteria")
    if not isinstance(aborts_raw, list):
        reasons.append("missing_abort_criteria:list")
        aborts_raw = []
    aborts: dict[str, dict[str, Any]] = {}
    for criterion in aborts_raw:
        if not isinstance(criterion, dict):
            reasons.append("missing_abort_criteria:entry_type")
            continue
        criterion_id = criterion.get("criterion_id")
        if not isinstance(criterion_id, str):
            reasons.append("missing_abort_criteria:id")
            continue
        aborts[criterion_id] = criterion
        if criterion_id not in EXPECTED_ABORTS:
            reasons.append(f"missing_abort_criteria:unexpected:{criterion_id}")
        if criterion.get("status") != "documented":
            reasons.append(f"missing_abort_criteria:status:{criterion_id}")
        if criterion.get("abort_required") is not True:
            reasons.append(f"missing_abort_criteria:required:{criterion_id}")
        if criterion.get("automatic_remediation_allowed") is not False:
            reasons.append(f"forbidden_execution:auto_remediation:{criterion_id}")
        if criterion.get("execution_allowed") is not False:
            reasons.append(f"forbidden_execution:abort_execution:{criterion_id}")
    if set(aborts) != EXPECTED_ABORTS:
        reasons.append("missing_abort_criteria:set")

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
    execution = copy.deepcopy(payload)
    execution["preflight_checks"][4]["execution_triggered"] = True
    if classify_status(collect_reasons(execution)) == "fail_closed_forbidden_execution":
        negative_selftests += 1
    else:
        fail("negative self-test unexpectedly allowed canary execution")

    side_effect = copy.deepcopy(payload)
    side_effect["preflight_checks"][0]["live_exchange_side_effect_detected"] = True
    if classify_status(collect_reasons(side_effect)) == "fail_closed_live_exchange_side_effect":
        negative_selftests += 1
    else:
        fail("negative self-test unexpectedly allowed live exchange side effect")

    stale = copy.deepcopy(payload)
    stale["canary_eligibility"]["freshness_status"] = "stale"
    if classify_status(collect_reasons(stale)) == "fail_closed_stale_source":
        negative_selftests += 1
    else:
        fail("negative self-test unexpectedly allowed stale source")

    default_open = copy.deepcopy(payload)
    default_open["no_default_execution_gate"] = "open"
    default_open["default_canary_execution_allowed"] = True
    if classify_status(collect_reasons(default_open)) == "fail_closed_default_execution_open":
        negative_selftests += 1
    else:
        fail("negative self-test unexpectedly allowed default execution")

print(
    "v30_canary_execution_preflight_no_default_execution_gate=pass "
    f"evidence_links={len(EXPECTED_LINKS)} "
    f"preflight_checks={len(EXPECTED_CHECKS)} "
    f"abort_criteria={len(EXPECTED_ABORTS)} "
    f"readiness_cases={len(EXPECTED_CASES)} "
    f"required_false_flags={len(REQUIRED_FALSE_FLAGS)} "
    f"negative_selftest={negative_selftests}"
)
PY
