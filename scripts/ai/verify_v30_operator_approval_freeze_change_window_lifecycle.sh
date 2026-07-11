#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

ARTIFACT_PATH="${NTPRO_V300_OPERATOR_LIFECYCLE_ARTIFACT:-docs/rust-cutover/release/v0_30_0_operator_approval_freeze_change_window_lifecycle.json}"
CONTRACT_PATH="${NTPRO_V300_OPERATOR_LIFECYCLE_CONTRACT:-docs/rust-cutover/release/v0_30_0_operator_approval_freeze_change_window_lifecycle.md}"
TASK_PATH="${NTPRO_V300_OPERATOR_LIFECYCLE_TASK:-docs/rust-cutover/tasks/V300-004.md}"
EVIDENCE_PATH="${NTPRO_V300_OPERATOR_LIFECYCLE_EVIDENCE:-docs/rust-cutover/evidence/V300-004.md}"
BOUNDARY_PATH="${NTPRO_V300_OPERATOR_LIFECYCLE_BOUNDARY:-docs/rust-cutover/release/v0_30_0_backend_go_live_candidate_boundary_contract.md}"
RUNTIME_FLAGS="${NTPRO_V300_OPERATOR_LIFECYCLE_RUNTIME_FLAGS:-docs/rust-cutover/release/v0_30_0_runtime_enablement_boundary_controlled_feature_flags.json}"
RUNTIME_FLAGS_CONTRACT="${NTPRO_V300_OPERATOR_LIFECYCLE_RUNTIME_FLAGS_CONTRACT:-docs/rust-cutover/release/v0_30_0_runtime_enablement_boundary_controlled_feature_flags.md}"
RELEASE_INDEX="${NTPRO_V300_OPERATOR_LIFECYCLE_RELEASE_INDEX:-docs/rust-cutover/release/README.md}"
SELFTEST="${NTPRO_V300_OPERATOR_LIFECYCLE_SELFTEST:-1}"

fail() {
  echo "v30 operator approval freeze change-window lifecycle failed: $*" >&2
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

for path in "$ARTIFACT_PATH" "$CONTRACT_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$BOUNDARY_PATH" "$RUNTIME_FLAGS" "$RUNTIME_FLAGS_CONTRACT" "$RELEASE_INDEX"; do
  require_file "$path"
done

require_contains "$TASK_PATH" "GitHub issue: \`#973\`"
require_contains "$EVIDENCE_PATH" "Task: \`V300-004\` / GitHub issue \`#973\`"
require_contains "$BOUNDARY_PATH" "operator_approval_freeze_record = #973"
require_contains "$BOUNDARY_PATH" "change_window_lifecycle_record = #973"
require_contains "$RUNTIME_FLAGS_CONTRACT" "Task: \`V300-003\` / GitHub issue \`#972\`"
require_contains "$CONTRACT_PATH" "contract_version = ntpro.v300.operator_approval_freeze_change_window_lifecycle.v1"
require_contains "$CONTRACT_PATH" "candidate_operations_require_approval = true"
require_contains "$CONTRACT_PATH" "candidate_operations_require_active_change_window = true"
require_contains "$CONTRACT_PATH" "approval_lifecycle_authorizes_trading_operations = false"
require_contains "$CONTRACT_PATH" "missing_approval => fail_closed_missing_approval"
require_contains "$CONTRACT_PATH" "missing_active_change_window => fail_closed_missing_active_change_window"
require_contains "$CONTRACT_PATH" "approval_authorizes_trading_operation => fail_closed_trading_authorization_violation"
require_contains "$RELEASE_INDEX" "v0_30_0_operator_approval_freeze_change_window_lifecycle.md"
require_contains "$RELEASE_INDEX" "../evidence/V300-004.md"

ARTIFACT_PATH="$ARTIFACT_PATH" SELFTEST="$SELFTEST" python3 <<'PY'
from __future__ import annotations

import copy
import json
import os
from pathlib import Path
from typing import Any

payload = json.loads(Path(os.environ["ARTIFACT_PATH"]).read_text(encoding="utf-8"))
selftest = os.environ.get("SELFTEST", "1") != "0"

SCHEMA_VERSION = "ntpro.v300.operator_approval_freeze_change_window_lifecycle.v1"
RELEASE_SCOPE = "backend_production_go_live_candidate_foundation_only"
READY_STATUS = "operator_approval_freeze_change_window_lifecycle_ready"
DEPENDENCIES = {"V300-001", "V300-003", "v0.29.1-release-evidence"}
EXPECTED_APPROVALS = {
    "go_no_go_candidate_approval": "approved_for_candidate_review",
    "freeze_operator_approval": "approved_for_candidate_freeze",
    "change_window_operator_approval": "approved_for_preview_window",
}
EXPECTED_EVENTS = {
    "go_no_go_review_opened",
    "candidate_freeze_entered",
    "preview_change_window_opened",
    "emergency_stop_bound",
    "unfreeze_plan_recorded",
}
EXPECTED_CASES = {
    "operator_approval_freeze_window.preview.allowed.001",
    "operator_approval_freeze_window.missing_approval.fail_closed.001",
    "operator_approval_freeze_window.inactive_window.fail_closed.001",
    "operator_approval_freeze_window.missing_identity.fail_closed.001",
    "operator_approval_freeze_window.missing_audit.fail_closed.001",
    "operator_approval_freeze_window.freeze_bypass.fail_closed.001",
    "operator_approval_freeze_window.missing_emergency_stop.fail_closed.001",
    "operator_approval_freeze_window.trading_authorization.fail_closed.001",
    "operator_approval_freeze_window.forbidden_boundary.fail_closed.001",
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
]
TRADING_AUTH_FIELDS = [
    "authorizes_trading_operations",
    "authorizes_submit",
    "authorizes_cancel",
    "authorizes_replace",
    "authorizes_amend",
    "authorizes_flatten",
    "authorizes_automatic_remediation",
]
OPERATION_AUTH_FIELDS = [
    "candidate_operation_execution_allowed",
    "approval_evidence_authorizes_submit",
    "approval_evidence_authorizes_cancel",
    "approval_evidence_authorizes_replace",
    "approval_evidence_authorizes_amend",
    "approval_evidence_authorizes_flatten",
    "approval_evidence_authorizes_automatic_remediation",
    "approval_evidence_authorizes_adapter_send",
    "approval_evidence_authorizes_live_exchange_request",
]


def fail(message: str) -> None:
    raise SystemExit(f"v30 operator approval freeze change-window lifecycle failed: {message}")


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
    if case.get("approval_record_overrides"):
        result["approval_records"] = apply_indexed_overrides(
            result["approval_records"],
            "approval_id",
            case["approval_record_overrides"],
        )
    if case.get("lifecycle_event_overrides"):
        result["lifecycle_events"] = apply_indexed_overrides(
            result["lifecycle_events"],
            "event_id",
            case["lifecycle_event_overrides"],
        )
    if case.get("freeze_state_override"):
        result["freeze_state"] = merge(result["freeze_state"], case["freeze_state_override"])
    if case.get("change_window_override"):
        result["change_window"] = merge(result["change_window"], case["change_window_override"])
    if case.get("operation_authorization_boundary_override"):
        result["operation_authorization_boundary"] = merge(
            result["operation_authorization_boundary"],
            case["operation_authorization_boundary_override"],
        )
    if case.get("boundary_flags_override"):
        result["boundary_flags"] = merge(result["boundary_flags"], case["boundary_flags_override"])
    return result


def classify_status(reasons: list[str]) -> str:
    if any(reason.startswith("forbidden_boundary") for reason in reasons):
        return "fail_closed_forbidden_boundary"
    if any(reason.startswith("trading_authorization") for reason in reasons):
        return "fail_closed_trading_authorization_violation"
    if any(reason.startswith("missing_approval") for reason in reasons):
        return "fail_closed_missing_approval"
    if any(reason.startswith("missing_active_change_window") for reason in reasons):
        return "fail_closed_missing_active_change_window"
    if any(reason.startswith("missing_identity_provenance") for reason in reasons):
        return "fail_closed_missing_identity_provenance"
    if any(reason.startswith("missing_immutable_audit_trail") for reason in reasons):
        return "fail_closed_missing_immutable_audit_trail"
    if any(reason.startswith("freeze_lifecycle_violation") for reason in reasons):
        return "fail_closed_freeze_lifecycle_violation"
    if any(reason.startswith("missing_emergency_stop") for reason in reasons):
        return "fail_closed_missing_emergency_stop"
    if reasons:
        return "fail_closed_forbidden_boundary"
    return READY_STATUS


def collect_reasons(artifact: dict[str, Any]) -> list[str]:
    reasons: list[str] = []
    if artifact.get("schema_version") != SCHEMA_VERSION:
        reasons.append("forbidden_boundary:schema_version")
    if artifact.get("contract_version") != SCHEMA_VERSION:
        reasons.append("forbidden_boundary:contract_version")
    if artifact.get("task_id") != "V300-004" or artifact.get("github_issue") != 973:
        reasons.append("forbidden_boundary:task_identity")
    if artifact.get("milestone") != "v0.30.0" or artifact.get("release_scope") != RELEASE_SCOPE:
        reasons.append("forbidden_boundary:release_scope")
    if artifact.get("candidate_claim") != "operator_approval_freeze_change_window_lifecycle":
        reasons.append("forbidden_boundary:candidate_claim")
    if artifact.get("lifecycle_mode") != "audited_candidate_evidence_only":
        reasons.append("forbidden_boundary:lifecycle_mode")
    if artifact.get("candidate_operations_require_approval") is not True:
        reasons.append("missing_approval:top_level")
    if artifact.get("candidate_operations_require_active_change_window") is not True:
        reasons.append("missing_active_change_window:top_level")
    if artifact.get("candidate_operation_execution_allowed") is not False:
        reasons.append("forbidden_boundary:candidate_operation_execution_allowed")
    if artifact.get("approval_lifecycle_authorizes_trading_operations") is not False:
        reasons.append("trading_authorization:top_level")
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

    approvals_raw = artifact.get("approval_records")
    if not isinstance(approvals_raw, list):
        reasons.append("missing_approval:list")
        approvals_raw = []
    approvals: dict[str, dict[str, Any]] = {}
    for approval in approvals_raw:
        if not isinstance(approval, dict):
            reasons.append("missing_approval:entry_type")
            continue
        approval_id = approval.get("approval_id")
        if not isinstance(approval_id, str):
            reasons.append("missing_approval:id")
            continue
        approvals[approval_id] = approval
        expected_status = EXPECTED_APPROVALS.get(approval_id)
        if expected_status is None:
            reasons.append(f"missing_approval:unexpected:{approval_id}")
        elif approval.get("approval_status") != expected_status:
            reasons.append(f"missing_approval:status:{approval_id}")
        if not approval.get("owner_operator_id"):
            reasons.append(f"missing_identity_provenance:owner:{approval_id}")
        if approval.get("identity_provenance_status") != "linked":
            reasons.append(f"missing_identity_provenance:status:{approval_id}")
        if not approval.get("identity_source_ref"):
            reasons.append(f"missing_identity_provenance:source:{approval_id}")
        if approval.get("immutable_audit_trail_ref_present") is not True:
            reasons.append(f"missing_immutable_audit_trail:approval:{approval_id}")
        if approval.get("audit_freshness_status") != "fresh":
            reasons.append(f"missing_immutable_audit_trail:freshness:{approval_id}")
        for key in TRADING_AUTH_FIELDS:
            if approval.get(key) is not False:
                reasons.append(f"trading_authorization:approval:{approval_id}:{key}")
    if set(approvals) != set(EXPECTED_APPROVALS):
        reasons.append("missing_approval:set")

    freeze = artifact.get("freeze_state") or {}
    if freeze.get("state_id") != "candidate_freeze_active":
        reasons.append("freeze_lifecycle_violation:state_id")
    if freeze.get("freeze_status") != "active":
        reasons.append("freeze_lifecycle_violation:status")
    if freeze.get("freeze_bypass_allowed") is not False:
        reasons.append("freeze_lifecycle_violation:bypass")
    if freeze.get("automatic_unfreeze_allowed") is not False:
        reasons.append("freeze_lifecycle_violation:auto_unfreeze")
    if freeze.get("unfreeze_requires_later_approval") is not True:
        reasons.append("freeze_lifecycle_violation:unfreeze_approval")
    if freeze.get("emergency_stop_available") is not True:
        reasons.append("missing_emergency_stop:available")
    if freeze.get("emergency_stop_audit_ref_present") is not True:
        reasons.append("missing_emergency_stop:audit")
    if freeze.get("authorizes_trading_operations") is not False:
        reasons.append("trading_authorization:freeze")

    window = artifact.get("change_window") or {}
    if window.get("window_status") != "active_preview_window" or window.get("active") is not True:
        reasons.append("missing_active_change_window:status")
    if window.get("owner_operator_identity_provenance_status") != "linked":
        reasons.append("missing_identity_provenance:window")
    if window.get("immutable_audit_trail_ref_present") is not True:
        reasons.append("missing_immutable_audit_trail:window")
    if window.get("window_freshness_status") != "fresh":
        reasons.append("missing_active_change_window:freshness")
    if window.get("execution_allowed") is not False:
        reasons.append("forbidden_boundary:window_execution")
    if window.get("authorizes_trading_operations") is not False:
        reasons.append("trading_authorization:window")

    events_raw = artifact.get("lifecycle_events")
    if not isinstance(events_raw, list):
        reasons.append("missing_immutable_audit_trail:event_list")
        events_raw = []
    events: dict[str, dict[str, Any]] = {}
    for event in events_raw:
        if not isinstance(event, dict):
            reasons.append("missing_immutable_audit_trail:event_type")
            continue
        event_id = event.get("event_id")
        if not isinstance(event_id, str):
            reasons.append("missing_immutable_audit_trail:event_id")
            continue
        events[event_id] = event
        if event_id not in EXPECTED_EVENTS:
            reasons.append(f"missing_immutable_audit_trail:unexpected_event:{event_id}")
        if event.get("event_status") != "recorded":
            reasons.append(f"missing_immutable_audit_trail:event_status:{event_id}")
        if event.get("identity_provenance_status") != "linked":
            reasons.append(f"missing_identity_provenance:event:{event_id}")
        if event.get("immutable_audit_trail_ref_present") is not True:
            reasons.append(f"missing_immutable_audit_trail:event:{event_id}")
        if event.get("audit_freshness_status") != "fresh":
            reasons.append(f"missing_immutable_audit_trail:event_freshness:{event_id}")
        if event.get("trading_authorization_granted") is not False:
            reasons.append(f"trading_authorization:event:{event_id}")
    if set(events) != EXPECTED_EVENTS:
        reasons.append("missing_immutable_audit_trail:event_set")

    operation_boundary = artifact.get("operation_authorization_boundary") or {}
    for key in OPERATION_AUTH_FIELDS:
        if operation_boundary.get(key) is not False:
            reasons.append(f"trading_authorization:operation_boundary:{key}")

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
    missing_approval = copy.deepcopy(payload)
    missing_approval["approval_records"][0]["approval_status"] = "missing"
    if classify_status(collect_reasons(missing_approval)) == "fail_closed_missing_approval":
        negative_selftests += 1
    else:
        fail("negative self-test unexpectedly allowed missing approval")

    inactive_window = copy.deepcopy(payload)
    inactive_window["change_window"]["active"] = False
    inactive_window["change_window"]["window_status"] = "inactive"
    if classify_status(collect_reasons(inactive_window)) == "fail_closed_missing_active_change_window":
        negative_selftests += 1
    else:
        fail("negative self-test unexpectedly allowed inactive change window")

    missing_audit = copy.deepcopy(payload)
    missing_audit["lifecycle_events"][2]["immutable_audit_trail_ref_present"] = False
    if classify_status(collect_reasons(missing_audit)) == "fail_closed_missing_immutable_audit_trail":
        negative_selftests += 1
    else:
        fail("negative self-test unexpectedly allowed missing immutable audit trail")

    trading_auth = copy.deepcopy(payload)
    trading_auth["approval_records"][2]["authorizes_submit"] = True
    if classify_status(collect_reasons(trading_auth)) == "fail_closed_trading_authorization_violation":
        negative_selftests += 1
    else:
        fail("negative self-test unexpectedly allowed trading authorization")

print(
    "v30_operator_approval_freeze_change_window_lifecycle=pass "
    f"approval_records={len(EXPECTED_APPROVALS)} "
    f"lifecycle_events={len(EXPECTED_EVENTS)} "
    f"readiness_cases={len(EXPECTED_CASES)} "
    f"required_false_flags={len(REQUIRED_FALSE_FLAGS)} "
    f"negative_selftest={negative_selftests}"
)
PY
