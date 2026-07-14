#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V32_OWNER_OPERATOR_REPO:-atxinbao/NTPRO}"
CURRENT_ISSUE="${NTPRO_V32_OWNER_OPERATOR_CURRENT_ISSUE:-1044}"
BOUNDARY_ISSUE="${NTPRO_V32_OWNER_OPERATOR_BOUNDARY_ISSUE:-1043}"
INTAKE_ISSUE="${NTPRO_V32_OWNER_OPERATOR_INTAKE_ISSUE:-1042}"
V320_MILESTONE_TITLE="${NTPRO_V32_OWNER_OPERATOR_MILESTONE_TITLE:-v0.32.0}"
CONTRACT_JSON="${NTPRO_V32_OWNER_OPERATOR_JSON:-docs/rust-cutover/release/v0_32_0_owner_operator_change_window_closeout.json}"
CONTRACT_MD="${NTPRO_V32_OWNER_OPERATOR_MD:-docs/rust-cutover/release/v0_32_0_owner_operator_change_window_closeout.md}"
TASK_DOC="${NTPRO_V32_OWNER_OPERATOR_TASK:-docs/rust-cutover/tasks/V320-002.md}"
EVIDENCE_DOC="${NTPRO_V32_OWNER_OPERATOR_EVIDENCE:-docs/rust-cutover/evidence/V320-002.md}"

fail() {
  echo "v32 owner/operator change-window closeout failed: $*" >&2
  exit 1
}

require_file() {
  local path="$1"
  [[ -f "$path" ]] || fail "missing required file: $path"
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
  "$TASK_DOC" \
  "$EVIDENCE_DOC" \
  docs/rust-cutover/release/v0_32_0_backend_closeout_boundary_contract.json \
  docs/rust-cutover/release/v0_32_0_backend_closeout_boundary_contract.md \
  docs/rust-cutover/tasks/V320-001.md \
  docs/rust-cutover/evidence/V320-001.md \
  docs/rust-cutover/release/README.md \
  scripts/ai/verify_v32_backend_closeout_boundary_contract.sh \
  scripts/ai/verify_v32_owner_operator_change_window_closeout.sh; do
  require_file "$path"
done

scripts/ai/verify_v32_backend_closeout_boundary_contract.sh >/dev/null

CONTRACT_JSON="$CONTRACT_JSON" \
CONTRACT_MD="$CONTRACT_MD" \
TASK_DOC="$TASK_DOC" \
EVIDENCE_DOC="$EVIDENCE_DOC" \
python3 <<'PY'
import json
import os
from pathlib import Path


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def require_contains(text: str, marker: str, label: str) -> None:
    require(marker in text, f"{label} missing marker: {marker}")


def classify(case: dict) -> str:
    approval = case.get("approval_state")
    freeze = case.get("freeze_state")
    if approval == "missing":
        return "fail_closed_missing_scoped_approval"
    if approval == "expired":
        return "fail_closed_expired_approval"
    if approval == "revoked":
        return "fail_closed_revoked_approval"
    if approval == "wrong_owner":
        return "fail_closed_wrong_owner"
    if approval == "wrong_operator":
        return "fail_closed_wrong_operator"
    if case.get("release_matches") is not True:
        return "fail_closed_release_mismatch"
    if freeze == "active":
        return "fail_closed_active_production_freeze"
    if case.get("within_change_window") is not True:
        return "fail_closed_outside_approved_change_window"
    if case.get("scope_reuse_attempted") is True:
        return "fail_closed_scope_reuse_or_drift"
    return "approval_window_valid_execution_still_blocked_by_downstream_gates"


contract = json.loads(Path(os.environ["CONTRACT_JSON"]).read_text(encoding="utf-8"))
contract_md = Path(os.environ["CONTRACT_MD"]).read_text(encoding="utf-8")
task = Path(os.environ["TASK_DOC"]).read_text(encoding="utf-8")
evidence = Path(os.environ["EVIDENCE_DOC"]).read_text(encoding="utf-8")
release_index = Path("docs/rust-cutover/release/README.md").read_text(encoding="utf-8")

require(contract.get("schema_version") == "ntpro.v320_owner_operator_change_window_closeout.v1", "schema mismatch")
require(contract.get("task_id") == "V320-002", "task mismatch")
require(contract.get("github_issue") == 1044, "issue mismatch")
require(contract.get("milestone") == "v0.32.0", "milestone mismatch")
require(contract.get("capability_track") == "backend_production_closeout", "capability track mismatch")
require(contract.get("lifecycle_status") == "owner_operator_approval_change_window_required_no_execution_authority", "lifecycle status mismatch")

deps = {item.get("github_issue"): item for item in contract.get("depends_on") or []}
require(deps.get(1042, {}).get("required_status") == "closed", "missing V320-000 dependency")
require(deps.get(1043, {}).get("required_status") == "closed", "missing V320-001 dependency")

for state in ["missing", "approved", "expired", "revoked", "wrong_owner", "wrong_operator", "scope_mismatch", "release_mismatch", "window_active"]:
    require(state in set(contract.get("approval_states") or []), f"missing approval state {state}")
for state in ["none", "scheduled", "active", "lifted", "expired"]:
    require(state in set(contract.get("freeze_states") or []), f"missing freeze state {state}")

required_approval = set(contract.get("required_approval_evidence") or [])
for field in [
    "approval_id",
    "owner",
    "operator",
    "reviewer",
    "release_version",
    "release_tag",
    "build_commit",
    "github_issue",
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
    "approval_digest",
    "boundary_digest",
    "request_digest",
    "redaction_profile",
    "source_provenance",
    "approval_scope_digest",
]:
    require(field in required_approval, f"approval evidence missing {field}")

window = contract.get("change_window_evidence") or {}
require(window.get("required") is True, "change window evidence must be required")
required_window = set(window.get("required_fields") or [])
for field in [
    "change_window_id",
    "environment",
    "venue_scope",
    "account_scope",
    "strategy_scope",
    "window_start",
    "window_end",
    "freeze_state",
    "approval_id",
    "release_version",
    "release_tag",
    "build_commit",
    "owner",
    "operator",
    "rollback_plan_ref",
    "incident_freeze_ref",
    "source_provenance",
    "redaction_profile",
]:
    require(field in required_window, f"change window evidence missing {field}")
for key, value in {
    "outside_window_status": "fail_closed_outside_approved_change_window",
    "active_freeze_status": "fail_closed_active_production_freeze",
    "expired_approval_status": "fail_closed_expired_approval",
    "revoked_approval_status": "fail_closed_revoked_approval",
    "release_mismatch_status": "fail_closed_release_mismatch",
    "wrong_owner_status": "fail_closed_wrong_owner",
    "wrong_operator_status": "fail_closed_wrong_operator",
    "scope_drift_status": "fail_closed_scope_reuse_or_drift",
}.items():
    require(window.get(key) == value, f"{key} mismatch")

scope = contract.get("scope_reuse_boundary") or {}
for key, value in scope.items():
    require(value is False, f"scope reuse boundary must be false: {key}")

provenance = contract.get("provenance_and_redaction") or {}
for key in [
    "immutable_evidence_required",
    "source_provenance_required",
    "redaction_required",
    "rollback_plan_reference_required",
    "risk_decision_reference_required",
    "audit_evidence_reference_required",
    "telemetry_gate_reference_required",
]:
    require(provenance.get(key) is True, f"provenance required missing: {key}")
for key in [
    "raw_secret_allowed",
    "raw_account_identifier_allowed",
    "raw_operator_token_allowed",
    "chat_or_external_notes_sufficient",
]:
    require(provenance.get(key) is False, f"provenance false field opened: {key}")

for key, value in (contract.get("runtime_boundary_flags") or {}).items():
    require(value is False, f"runtime flag must be false: {key}")
for case in contract.get("decision_cases") or []:
    expected = case.get("expected_status")
    got = classify(case)
    require(got == expected, f"case {case.get('case_id')} expected {expected} got {got}")
require(len(contract.get("decision_cases") or []) == 10, "decision case count mismatch")
require(contract.get("runtime_behavior_changed") is False, "runtime behavior must not change")
require(contract.get("trading_behavior_changed") is False, "trading behavior must not change")

for marker in [
    "lifecycle_status = owner_operator_approval_change_window_required_no_execution_authority",
    "depends_on_issue_1042 = closed",
    "depends_on_issue_1043 = closed",
    "change window evidence required = true",
    "immutable evidence required = true",
    "source provenance required = true",
    "redaction required = true",
    "rollback plan reference required = true",
    "risk decision reference required = true",
    "audit evidence reference required = true",
    "telemetry gate reference required = true",
    "approval scope reuse allowed = false",
    "broader scope approval consumption allowed = false",
    "shared approval consumption allowed = false",
    "missing approval -> fail_closed_missing_scoped_approval",
    "expired approval -> fail_closed_expired_approval",
    "wrong owner -> fail_closed_wrong_owner",
    "wrong operator -> fail_closed_wrong_operator",
    "active freeze -> fail_closed_active_production_freeze",
    "outside approved change window -> fail_closed_outside_approved_change_window",
    "approval scope reuse -> fail_closed_scope_reuse_or_drift",
    "approval evidence is redacted and provenance-bound = true",
    "adapter_send_allowed = false",
    "frontend_completion_claim = false",
    "backend_go_live_claim = false",
    "default_production_execution_allowed = false",
]:
    require_contains(contract_md, marker, "contract markdown")
    require_contains(evidence, marker, "evidence")

for label, text in {"task": task, "evidence": evidence, "README": release_index}.items():
    require_contains(text, "V320-002", label)
    require_contains(text, "v0.32.0", label)

print(
    "v32_owner_operator_change_window_source "
    f"required_approval_fields={len(contract.get('required_approval_evidence') or [])} "
    f"change_window_fields={len(required_window)} "
    f"decision_cases={len(contract.get('decision_cases') or [])}"
)
PY

current_issue_json="$(gh_with_retry issue view "$CURRENT_ISSUE" --repo "$REPO" --json number,state,title,milestone)"
boundary_issue_json="$(gh_with_retry issue view "$BOUNDARY_ISSUE" --repo "$REPO" --json number,state,title,milestone)"
intake_issue_json="$(gh_with_retry issue view "$INTAKE_ISSUE" --repo "$REPO" --json number,state,title,milestone)"
v320_issues_json="$(gh_with_retry issue list --repo "$REPO" --state all --milestone "$V320_MILESTONE_TITLE" --limit 100 --json number,state,title)"

CURRENT_ISSUE_JSON="$current_issue_json" \
BOUNDARY_ISSUE_JSON="$boundary_issue_json" \
INTAKE_ISSUE_JSON="$intake_issue_json" \
V320_ISSUES_JSON="$v320_issues_json" \
python3 <<'PY'
import json
import os


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


current = json.loads(os.environ["CURRENT_ISSUE_JSON"])
boundary = json.loads(os.environ["BOUNDARY_ISSUE_JSON"])
intake = json.loads(os.environ["INTAKE_ISSUE_JSON"])
v320_issues = json.loads(os.environ["V320_ISSUES_JSON"])
require(current.get("number") == 1044, "current issue mismatch")
require((current.get("milestone") or {}).get("title") == "v0.32.0", "current milestone mismatch")
require(boundary.get("number") == 1043 and boundary.get("state") == "CLOSED", "V320-001 must be closed")
require(intake.get("number") == 1042 and intake.get("state") == "CLOSED", "V320-000 must be closed")
v320_map = {item["number"]: item for item in v320_issues}
missing = [number for number in range(1042, 1052) if number not in v320_map]
require(not missing, f"missing V320 issues: {missing}")

print(
    "v32_owner_operator_change_window_live "
    "boundary_issue=1043:CLOSED "
    f"current_issue=1044:{current.get('state')} "
    "v320_issues=10 "
    "change_window_required=true "
    "active_freeze_fail_closed=true"
)
PY

echo "v32_owner_operator_change_window_closeout=pass task=V320-002 issue=1044 required_approval_fields=28 change_window_fields=20 decision_cases=10 approval_scope_reuse_allowed=false"
