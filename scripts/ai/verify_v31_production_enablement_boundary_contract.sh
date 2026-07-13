#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V31_BOUNDARY_REPO:-atxinbao/NTPRO}"
CURRENT_ISSUE="${NTPRO_V31_BOUNDARY_CURRENT_ISSUE:-1007}"
INTAKE_ISSUE="${NTPRO_V31_BOUNDARY_INTAKE_ISSUE:-1006}"
CONTRACT_JSON="${NTPRO_V31_BOUNDARY_JSON:-docs/rust-cutover/release/v0_31_0_production_enablement_boundary_contract.json}"
CONTRACT_MD="${NTPRO_V31_BOUNDARY_MD:-docs/rust-cutover/release/v0_31_0_production_enablement_boundary_contract.md}"
TASK_DOC="${NTPRO_V31_BOUNDARY_TASK:-docs/rust-cutover/tasks/V310-001.md}"
EVIDENCE_DOC="${NTPRO_V31_BOUNDARY_EVIDENCE:-docs/rust-cutover/evidence/V310-001.md}"

fail() {
  echo "v31 production enablement boundary contract failed: $*" >&2
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
  docs/rust-cutover/release/v0_31_0_intake_gate.json \
  docs/rust-cutover/release/v0_31_0_intake_gate.md \
  docs/rust-cutover/tasks/V310-000.md \
  docs/rust-cutover/evidence/V310-000.md \
  docs/rust-cutover/release/README.md \
  scripts/ai/verify_v31_intake_gate.sh \
  scripts/ai/verify_v31_production_enablement_boundary_contract.sh; do
  require_file "$path"
done

scripts/ai/verify_v31_intake_gate.sh >/dev/null

CONTRACT_JSON="$CONTRACT_JSON" \
CONTRACT_MD="$CONTRACT_MD" \
TASK_DOC="$TASK_DOC" \
EVIDENCE_DOC="$EVIDENCE_DOC" \
python3 <<'PY'
import copy
import json
import os
from pathlib import Path


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def require_contains(text: str, marker: str, label: str) -> None:
    require(marker in text, f"{label} missing marker: {marker}")


def classify(case: dict, boundary_flags: dict) -> str:
    scenario_flags = copy.deepcopy(boundary_flags)
    scenario_flags.update(case.get("boundary_flags_override") or {})
    if case.get("scoped_approval_present") is not True:
        return "fail_closed_missing_scoped_approval"
    if case.get("inherited_execution_authority") is True:
        return "fail_closed_inherited_execution_authority"
    if any(value is not False for value in scenario_flags.values()):
        return "fail_closed_inherited_execution_authority"
    if case.get("downstream_gates_complete") is not True:
        return "approval_recorded_execution_still_blocked_by_downstream_gates"
    return "boundary_contract_satisfied_no_runtime_execution"


contract = json.loads(Path(os.environ["CONTRACT_JSON"]).read_text(encoding="utf-8"))
contract_md = Path(os.environ["CONTRACT_MD"]).read_text(encoding="utf-8")
task = Path(os.environ["TASK_DOC"]).read_text(encoding="utf-8")
evidence = Path(os.environ["EVIDENCE_DOC"]).read_text(encoding="utf-8")
release_index = Path("docs/rust-cutover/release/README.md").read_text(encoding="utf-8")

require(contract.get("schema_version") == "ntpro.v310_production_enablement_boundary.v1", "schema mismatch")
require(contract.get("task_id") == "V310-001", "task mismatch")
require(contract.get("github_issue") == 1007, "issue mismatch")
require(contract.get("milestone") == "v0.31.0", "milestone mismatch")
require(contract.get("capability_track") == "controlled_backend_production_enablement_candidate", "capability track mismatch")
require(contract.get("boundary_status") == "scoped_approval_required_no_execution_authority", "boundary status mismatch")

depends_on = contract.get("depends_on") or {}
require(depends_on.get("task_id") == "V310-000", "dependency task mismatch")
require(depends_on.get("github_issue") == 1006, "dependency issue mismatch")
require(depends_on.get("required_status") == "closed", "dependency status mismatch")

definition = contract.get("definition") or {}
require(definition.get("default_runtime_authority") == "none", "default runtime authority mismatch")
require(definition.get("approval_source_of_truth") == "source_controlled_artifact", "approval source mismatch")
require(definition.get("chat_approval_allowed") is False, "chat approval must be false")
require(definition.get("external_notes_approval_allowed") is False, "external notes approval must be false")
require(definition.get("runtime_execution_authorized_by_this_contract") is False, "contract must not authorize runtime execution")

approval = contract.get("explicit_scoped_approval_contract") or {}
require(approval.get("required") is True, "scoped approval must be required")
require(approval.get("missing_approval_status") == "fail_closed_missing_scoped_approval", "missing approval status mismatch")
require(approval.get("approval_alone_authorizes_execution") is False, "approval alone must not authorize execution")
required_fields = set(approval.get("approval_scope_must_include") or [])
for field in [
    "approval_id",
    "approver",
    "operator",
    "github_issue",
    "release_version",
    "environment",
    "venue_scope",
    "account_scope",
    "change_window_id",
    "requested_capability",
    "request_digest",
    "boundary_digest",
    "issued_at",
    "expires_at",
]:
    require(field in required_fields, f"approval scope missing {field}")
require(
    approval.get("allowed_requested_capabilities") == [
        "backend_production_enablement_candidate_readiness_evaluation"
    ],
    "allowed requested capabilities mismatch",
)
for forbidden in [
    "submit_order",
    "cancel_order",
    "replace_order",
    "amend_order",
    "flatten_position",
    "adapter_send",
    "live_exchange_request",
    "automatic_remediation",
    "dashboard_trading_control",
    "admin_workbench_trading_control",
    "trader_terminal_order_ticket",
]:
    require(forbidden in set(approval.get("forbidden_requested_capabilities") or []), f"missing forbidden capability {forbidden}")

non_inheritance = contract.get("non_inheritance_boundary") or {}
runtime_flags = contract.get("runtime_boundary_flags") or {}
for key, value in non_inheritance.items():
    require(value is False, f"non-inheritance flag must be false: {key}")
for key, value in runtime_flags.items():
    require(value is False, f"runtime boundary flag must be false: {key}")

for case in contract.get("decision_cases") or []:
    expected = case.get("expected_status")
    got = classify(case, runtime_flags)
    require(got == expected, f"case {case.get('case_id')} expected {expected} got {got}")

auditability = contract.get("auditability") or {}
require(auditability.get("requires_source_controlled_contract") is True, "source-controlled contract required")
require(auditability.get("requires_source_controlled_evidence") is True, "source-controlled evidence required")
require(auditability.get("chat_or_external_notes_sufficient") is False, "chat/external notes must not be sufficient")
require(auditability.get("deterministic_negative_cases_required") is True, "negative cases required")
require(contract.get("runtime_behavior_changed") is False, "runtime behavior must not change")
require(contract.get("trading_behavior_changed") is False, "trading behavior must not change")

for marker in [
    "boundary_status = scoped_approval_required_no_execution_authority",
    "explicit scoped approval required = true",
    "missing scoped approval status = fail_closed_missing_scoped_approval",
    "approval alone authorizes execution = false",
    "inherits_submit = false",
    "inherits_adapter_send = false",
    "production_order_submission_allowed = false",
    "adapter_send_allowed = false",
    "dashboard_trading_controls_enabled = false",
    "backend_go_live_claim = false",
    "product_grade_trading_terminal_claim = false",
    "missing scoped approval -> fail_closed_missing_scoped_approval",
    "inherited adapter send -> fail_closed_inherited_execution_authority",
]:
    require_contains(contract_md, marker, "contract markdown")
    require_contains(evidence, marker, "evidence")

for label, text in {"task": task, "evidence": evidence, "README": release_index}.items():
    require_contains(text, "V310-001", label)
    require_contains(text, "v0.31.0", label)
PY

current_issue_json="$(gh_with_retry issue view "$CURRENT_ISSUE" --repo "$REPO" --json number,state,title)"
intake_issue_json="$(gh_with_retry issue view "$INTAKE_ISSUE" --repo "$REPO" --json number,state,title)"

CURRENT_ISSUE_JSON="$current_issue_json" \
INTAKE_ISSUE_JSON="$intake_issue_json" \
python3 <<'PY'
import json
import os


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


current = json.loads(os.environ["CURRENT_ISSUE_JSON"])
intake = json.loads(os.environ["INTAKE_ISSUE_JSON"])
require(current.get("number") == 1007, "current issue mismatch")
require(intake.get("number") == 1006, "intake issue mismatch")
require(intake.get("state") == "CLOSED", "V310-000 must be closed before V310-001")

print(
    "v31_production_enablement_boundary_contract_live "
    f"current_issue_state={current.get('state')} "
    "v310_000_state=CLOSED "
    "approval_required=true "
    "no_inherited_execution=true"
)
PY

echo "v31_production_enablement_boundary_contract=pass task=V310-001 issue=1007 approval_required=true no_inherited_execution=true runtime_authority=none"
