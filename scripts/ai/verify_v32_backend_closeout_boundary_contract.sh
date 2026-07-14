#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V32_BOUNDARY_REPO:-atxinbao/NTPRO}"
CONTRACT_JSON="${NTPRO_V32_BOUNDARY_JSON:-docs/rust-cutover/release/v0_32_0_backend_closeout_boundary_contract.json}"
CONTRACT_MD="${NTPRO_V32_BOUNDARY_MD:-docs/rust-cutover/release/v0_32_0_backend_closeout_boundary_contract.md}"
TASK_PATH="${NTPRO_V32_BOUNDARY_TASK:-docs/rust-cutover/tasks/V320-001.md}"
EVIDENCE_PATH="${NTPRO_V32_BOUNDARY_EVIDENCE:-docs/rust-cutover/evidence/V320-001.md}"
INTAKE_JSON="${NTPRO_V32_BOUNDARY_INTAKE_JSON:-docs/rust-cutover/release/v0_32_0_intake_gate.json}"
INTAKE_MD="${NTPRO_V32_BOUNDARY_INTAKE_MD:-docs/rust-cutover/release/v0_32_0_intake_gate.md}"
RELEASE_INDEX="${NTPRO_V32_BOUNDARY_RELEASE_INDEX:-docs/rust-cutover/release/README.md}"
INTAKE_ISSUE="${NTPRO_V32_BOUNDARY_INTAKE_ISSUE:-1042}"
CURRENT_ISSUE="${NTPRO_V32_BOUNDARY_CURRENT_ISSUE:-1043}"
V320_MILESTONE_TITLE="${NTPRO_V32_BOUNDARY_MILESTONE_TITLE:-v0.32.0}"

fail() {
  echo "v32 backend closeout boundary contract failed: $*" >&2
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

gh_with_retry() {
  local attempt=1
  local max_attempts=4
  while true; do
    if GODEBUG=http2client=0 gh "$@"; then
      return 0
    fi
    if (( attempt >= max_attempts )); then
      return 1
    fi
    sleep "$((attempt * 2))"
    attempt=$((attempt + 1))
  done
}

for path in \
  "$CONTRACT_JSON" \
  "$CONTRACT_MD" \
  "$TASK_PATH" \
  "$EVIDENCE_PATH" \
  "$INTAKE_JSON" \
  "$INTAKE_MD" \
  "$RELEASE_INDEX" \
  docs/rust-cutover/tasks/V320-000.md \
  docs/rust-cutover/evidence/V320-000.md \
  scripts/ai/verify_v32_intake_gate.sh \
  scripts/ai/verify_v32_backend_closeout_boundary_contract.sh; do
  require_file "$path"
done

scripts/ai/verify_v32_intake_gate.sh >/dev/null

for marker in \
  "contract_version = ntpro.v320.backend_closeout_boundary.v1" \
  "release_scope = backend_production_closeout_scoped_authorization_contract" \
  "dependency_issue_1042 = closed" \
  "boundary_status = scoped_authorization_required_no_execution_authority" \
  "runtime_execution_authorized_by_this_contract = false" \
  "explicit scoped authorization required = true" \
  "missing scoped authorization status = fail_closed_missing_scoped_authorization" \
  "approval alone authorizes execution = false" \
  "authorization source of truth = source_controlled_artifact" \
  "chat approval allowed = false" \
  "external notes approval allowed = false" \
  "forbidden requested capability = submit_order" \
  "forbidden requested capability = adapter_send" \
  "forbidden requested capability = live_exchange_request" \
  "forbidden requested capability = product_grade_live_trading_terminal" \
  "missing V320-000 intake proof -> fail_closed_missing_v32_intake_dependency" \
  "scoped authorization without downstream gates -> scoped_authorization_recorded_execution_still_blocked_by_downstream_gates" \
  "release stage = scripts/ai/verify_release.sh v32-backend-closeout-boundary-contract"; do
  require_contains "$CONTRACT_MD" "$marker"
done

for marker in \
  "boundary_contract = docs/rust-cutover/release/v0_32_0_backend_closeout_boundary_contract.md" \
  "release stage = scripts/ai/verify_release.sh v32-backend-closeout-boundary-contract" \
  "depends_on_issue = #1042 closed" \
  "boundary_status = scoped_authorization_required_no_execution_authority" \
  "explicit scoped authorization required = true" \
  "missing scoped authorization status = fail_closed_missing_scoped_authorization"; do
  require_contains "$EVIDENCE_PATH" "$marker"
done

require_contains "$TASK_PATH" "GitHub issue: #1043"
require_contains "$TASK_PATH" "Depends on \`V320-000\` intake gate"
require_contains "$RELEASE_INDEX" "v0_32_0_backend_closeout_boundary_contract.md"
require_contains "$RELEASE_INDEX" "../evidence/V320-001.md"

for marker in \
  "new_submit_capability = false" \
  "production_order_submission_allowed = false" \
  "production_order_mutation_allowed = false" \
  "cancel_order_allowed = false" \
  "replace_order_allowed = false" \
  "amend_order_allowed = false" \
  "flatten_position_allowed = false" \
  "execution_adapter_call_allowed = false" \
  "adapter_send_allowed = false" \
  "live_exchange_request_allowed = false" \
  "network_attempted = false" \
  "retry_scheduler_enabled = false" \
  "automatic_remediation_allowed = false" \
  "automatic_operation_action_allowed = false" \
  "dashboard_operation_controls_enabled = false" \
  "dashboard_trading_controls_enabled = false" \
  "admin_workbench_operation_controls_enabled = false" \
  "admin_workbench_trading_controls_enabled = false" \
  "trader_terminal_order_ticket_enabled = false" \
  "manual_operation_submit_allowed = false" \
  "frontend_completion_claim = false" \
  "backend_go_live_claim = false" \
  "actual_backend_production_go_live_allowed = false" \
  "product_grade_trading_terminal_claim = false" \
  "product_grade_live_trading_terminal_claim = false" \
  "default_production_execution_allowed = false" \
  "unscoped_production_execution_allowed = false" \
  "scoped_authorization_alone_executes = false"; do
  require_contains "$CONTRACT_MD" "$marker"
  require_contains "$EVIDENCE_PATH" "$marker"
done

CONTRACT_JSON="$CONTRACT_JSON" INTAKE_JSON="$INTAKE_JSON" python3 <<'PY'
from __future__ import annotations

import copy
import json
import os
from pathlib import Path
from typing import Any

SCHEMA_VERSION = "ntpro.v320.backend_closeout_boundary.v1"
INTAKE_STATUS = "dependency_proof_satisfied_backend_closeout_scoped_intake_only"
RELEASE_SCOPE = "backend_production_closeout_scoped_authorization_contract"
EXPECTED_SCOPE_FIELDS = [
    "approval_id",
    "owner",
    "operator",
    "reviewer",
    "github_issue",
    "release_version",
    "environment",
    "venue_scope",
    "account_scope",
    "strategy_scope",
    "change_window_id",
    "requested_capability",
    "risk_decision_ref",
    "audit_evidence_ref",
    "rollback_plan_ref",
    "telemetry_slo_ref",
    "request_digest",
    "boundary_digest",
    "issued_at",
    "expires_at",
    "revocation_conditions",
]
EXPECTED_ALLOWED_CAPABILITIES = {
    "backend_production_closeout_readiness_evaluation",
    "backend_production_closeout_scoped_authorization_recording",
    "backend_enablement_state_read_model_evidence",
}
EXPECTED_FORBIDDEN_CAPABILITIES = {
    "submit_order",
    "cancel_order",
    "replace_order",
    "amend_order",
    "flatten_position",
    "adapter_send",
    "live_exchange_request",
    "retry_scheduler",
    "automatic_remediation",
    "dashboard_trading_control",
    "admin_workbench_trading_control",
    "trader_terminal_order_ticket",
    "broad_live_exchange_access",
    "unbounded_production_execution",
    "product_grade_live_trading_terminal",
    "frontend_completion",
}
EXPECTED_DOWNSTREAM = {
    "owner_operator_approval_freeze_change_window": 1044,
    "risk_audit_go_no_go": 1045,
    "production_config_venue_credential_environment_provenance": 1046,
    "canary_rollback_disaster_recovery": 1047,
    "telemetry_slo_alerting_incident": 1048,
    "backend_enablement_state_read_model_admin_bridge": 1049,
    "fail_closed_negative_tests": 1050,
    "v32_release_gates_strict_provenance_publication": 1051,
}
EXPECTED_REVOCATION = {
    "authorization_expired",
    "authorization_explicitly_revoked",
    "boundary_digest_mismatch",
    "request_digest_mismatch",
    "environment_scope_drift",
    "venue_scope_drift",
    "account_scope_drift",
    "strategy_scope_drift",
    "incident_freeze_active",
    "risk_gate_not_green",
    "audit_gate_not_green",
    "rollback_dr_unavailable",
    "telemetry_slo_gate_not_green",
    "release_gate_not_successful",
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
    "frontend_completion_claim",
    "backend_go_live_claim",
    "actual_backend_production_go_live_allowed",
    "product_grade_trading_terminal_claim",
    "product_grade_live_trading_terminal_claim",
    "default_production_execution_allowed",
    "unscoped_production_execution_allowed",
    "scoped_authorization_alone_executes",
]


def fail(message: str) -> None:
    raise SystemExit(f"v32 backend closeout boundary contract failed: {message}")


def require(condition: bool, message: str) -> None:
    if not condition:
        fail(message)


def classify(case: dict[str, Any], payload: dict[str, Any]) -> str:
    if case.get("v320_intake_proof_present") is not True:
        return "fail_closed_missing_v32_intake_dependency"
    if case.get("scoped_authorization_present") is not True:
        return "fail_closed_missing_scoped_authorization"
    if case.get("authorization_revoked_or_expired") is True:
        return "fail_closed_revoked_or_expired_authorization"
    requested = case.get("requested_capability")
    forbidden = set(payload["explicit_scoped_authorization_contract"]["forbidden_requested_capabilities"])
    if requested in forbidden:
        return "fail_closed_forbidden_requested_capability"
    boundary = copy.deepcopy(payload["runtime_boundary_flags"])
    boundary.update(case.get("boundary_flags_override") or {})
    inheritance = copy.deepcopy(payload["non_inheritance_boundary"])
    inheritance.update(case.get("non_inheritance_override") or {})
    if case.get("unscoped_production_execution") is True:
        return "fail_closed_unscoped_or_inherited_execution_authority"
    if any(value is not False for value in boundary.values()):
        return "fail_closed_unscoped_or_inherited_execution_authority"
    if any(value is not False for value in inheritance.values()):
        return "fail_closed_unscoped_or_inherited_execution_authority"
    if case.get("downstream_gates_complete") is not True:
        return "scoped_authorization_recorded_execution_still_blocked_by_downstream_gates"
    return "boundary_contract_satisfied_no_runtime_execution"


payload = json.loads(Path(os.environ["CONTRACT_JSON"]).read_text(encoding="utf-8"))
intake = json.loads(Path(os.environ["INTAKE_JSON"]).read_text(encoding="utf-8"))

require(intake.get("intake_status") == INTAKE_STATUS, "V320 intake status mismatch")
require(intake.get("github_issue") == 1042, "V320 intake issue mismatch")
require(intake.get("backend_closeout_scope", {}).get("v0_32_backend_closeout_version") is True, "V320 intake backend closeout marker missing")

require(payload.get("schema_version") == SCHEMA_VERSION, "schema version mismatch")
require(payload.get("contract_version") == SCHEMA_VERSION, "contract version mismatch")
require(payload.get("task_id") == "V320-001", "task mismatch")
require(payload.get("github_issue") == 1043, "issue mismatch")
require(payload.get("milestone") == "v0.32.0", "milestone mismatch")
require(payload.get("capability_track") == "backend_production_closeout", "capability track mismatch")
require(payload.get("release_scope") == RELEASE_SCOPE, "release scope mismatch")
require(payload.get("boundary_status") == "scoped_authorization_required_no_execution_authority", "boundary status mismatch")

dependency = payload.get("depends_on") or {}
require(dependency.get("task_id") == "V320-000", "dependency task mismatch")
require(dependency.get("github_issue") == 1042, "dependency issue mismatch")
require(dependency.get("required_issue_state") == "closed", "dependency issue state mismatch")
require(dependency.get("required_intake_status") == INTAKE_STATUS, "dependency intake status mismatch")
require(dependency.get("required_predecessor_release") == "ntpro-rust-only-v0.31.1", "dependency release mismatch")

definition = payload.get("definition") or {}
for key in [
    "backend_service_closeout",
    "scoped_backend_production_authorization",
    "actual_backend_production_execution",
    "product_grade_live_trading_terminal",
    "authorization_source_of_truth",
]:
    require(isinstance(definition.get(key), str) and definition[key], f"definition missing: {key}")
require(definition.get("chat_approval_allowed") is False, "chat approval must be false")
require(definition.get("external_notes_approval_allowed") is False, "external notes approval must be false")
require(definition.get("runtime_execution_authorized_by_this_contract") is False, "runtime execution must be false")
does_not_mean = set(definition.get("backend_closeout_does_not_mean") or [])
for value in [
    "frontend_completion",
    "product_grade_live_trading_terminal",
    "default_production_execution",
    "production_order_submission",
    "production_order_mutation",
    "adapter_send",
    "live_exchange_request",
    "automatic_remediation",
    "dashboard_admin_trader_terminal_trading_controls",
]:
    require(value in does_not_mean, f"does_not_mean missing: {value}")

authorization = payload.get("explicit_scoped_authorization_contract") or {}
require(authorization.get("required") is True, "scoped authorization must be required")
require(authorization.get("missing_authorization_status") == "fail_closed_missing_scoped_authorization", "missing authorization status mismatch")
require(authorization.get("approval_alone_authorizes_execution") is False, "approval alone must not authorize execution")
require(authorization.get("runtime_execution_authorized_by_this_contract") is False, "contract must not authorize runtime execution")
require(authorization.get("authorization_scope_must_include") == EXPECTED_SCOPE_FIELDS, "authorization scope fields mismatch")
require(set(authorization.get("allowed_requested_capabilities") or []) == EXPECTED_ALLOWED_CAPABILITIES, "allowed capabilities mismatch")
require(set(authorization.get("forbidden_requested_capabilities") or []) == EXPECTED_FORBIDDEN_CAPABILITIES, "forbidden capabilities mismatch")
require(set(authorization.get("revocation_conditions") or []) == EXPECTED_REVOCATION, "revocation conditions mismatch")
downstream = authorization.get("downstream_gates_still_required") or []
require(len(downstream) == len(EXPECTED_DOWNSTREAM), "downstream gate count mismatch")
for item in downstream:
    gate_id = item.get("id")
    require(EXPECTED_DOWNSTREAM.get(gate_id) == item.get("issue"), f"downstream issue mismatch: {gate_id}")
    require(item.get("status") == "required_later", f"downstream status mismatch: {gate_id}")
    require(item.get("current_satisfied") is False, f"downstream must be unsatisfied now: {gate_id}")
    require(item.get("bypass_allowed") is False, f"downstream bypass must be false: {gate_id}")

for section_name in ["non_inheritance_boundary", "runtime_boundary_flags"]:
    section = payload.get(section_name) or {}
    require(section, f"missing section: {section_name}")
    for key, value in section.items():
        require(value is False, f"{section_name} must remain false: {key}")
for key in REQUIRED_FALSE_FLAGS:
    require(key in payload["runtime_boundary_flags"], f"required false flag missing: {key}")

cases = payload.get("decision_cases") or []
require(len(cases) == 7, "decision case count mismatch")
for case in cases:
    expected = case.get("expected_status")
    got = classify(case, payload)
    require(got == expected, f"case {case.get('case_id')} expected {expected} got {got}")

auditability = payload.get("auditability") or {}
for key in [
    "requires_source_controlled_contract",
    "requires_source_controlled_evidence",
    "github_issue_required",
    "deterministic_negative_cases_required",
    "live_dependency_check_required",
]:
    require(auditability.get(key) is True, f"auditability missing: {key}")
require(auditability.get("chat_or_external_notes_sufficient") is False, "chat/external notes must not be sufficient")
require(payload.get("runtime_behavior_changed") is False, "runtime behavior must not change")
require(payload.get("trading_behavior_changed") is False, "trading behavior must not change")

print(
    "v32_backend_closeout_boundary_source "
    f"required_scope_fields={len(EXPECTED_SCOPE_FIELDS)} "
    f"forbidden_capabilities={len(EXPECTED_FORBIDDEN_CAPABILITIES)} "
    f"required_false_flags={len(REQUIRED_FALSE_FLAGS)} "
    f"decision_cases={len(cases)}"
)
PY

intake_issue_json="$(gh_with_retry issue view "$INTAKE_ISSUE" --repo "$REPO" --json number,state,title,milestone)"
current_issue_json="$(gh_with_retry issue view "$CURRENT_ISSUE" --repo "$REPO" --json number,state,title,milestone)"
v320_issues_json="$(gh_with_retry issue list --repo "$REPO" --state all --milestone "$V320_MILESTONE_TITLE" --limit 100 --json number,state,title)"

LIVE_JSON_DIR="$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v32-boundary-live-json.XXXXXX")"
trap 'rm -rf "$LIVE_JSON_DIR"' EXIT
printf '%s' "$intake_issue_json" >"$LIVE_JSON_DIR/intake_issue.json"
printf '%s' "$current_issue_json" >"$LIVE_JSON_DIR/current_issue.json"
printf '%s' "$v320_issues_json" >"$LIVE_JSON_DIR/v320_issues.json"

INTAKE_ISSUE_JSON_PATH="$LIVE_JSON_DIR/intake_issue.json" \
CURRENT_ISSUE_JSON_PATH="$LIVE_JSON_DIR/current_issue.json" \
V320_ISSUES_JSON_PATH="$LIVE_JSON_DIR/v320_issues.json" \
python3 <<'PY'
import json
import os
from pathlib import Path


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"v32 backend closeout boundary contract failed: {message}")


intake_issue = json.loads(Path(os.environ["INTAKE_ISSUE_JSON_PATH"]).read_text(encoding="utf-8"))
current_issue = json.loads(Path(os.environ["CURRENT_ISSUE_JSON_PATH"]).read_text(encoding="utf-8"))
v320_issues = json.loads(Path(os.environ["V320_ISSUES_JSON_PATH"]).read_text(encoding="utf-8"))

require(intake_issue.get("number") == 1042, "intake issue number mismatch")
require(intake_issue.get("state") == "CLOSED", "intake issue must be closed")
require((intake_issue.get("milestone") or {}).get("title") == "v0.32.0", "intake milestone mismatch")
require(current_issue.get("number") == 1043, "current issue number mismatch")
require(current_issue.get("state") in {"OPEN", "CLOSED"}, "current issue state invalid")
require((current_issue.get("milestone") or {}).get("title") == "v0.32.0", "current milestone mismatch")

v320_map = {item["number"]: item for item in v320_issues}
expected_v320 = list(range(1042, 1052))
missing_v320 = [number for number in expected_v320 if number not in v320_map]
require(not missing_v320, f"missing V320 issues: {missing_v320}")

print(
    "v32_backend_closeout_boundary_live "
    f"intake_issue=1042:{intake_issue.get('state')} "
    f"current_issue=1043:{current_issue.get('state')} "
    "v320_issues=10 "
    "boundary_status=scoped_authorization_required_no_execution_authority"
)
PY

echo "v32_backend_closeout_boundary_contract=pass required_scope_fields=21 forbidden_capabilities=16 required_false_flags=28 decision_cases=7 no_runtime_execution=true"
