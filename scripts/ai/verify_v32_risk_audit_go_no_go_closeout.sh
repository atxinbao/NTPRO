#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V32_GNG_REPO:-atxinbao/NTPRO}"
CURRENT_ISSUE="${NTPRO_V32_GNG_CURRENT_ISSUE:-1045}"
OPERATOR_ISSUE="${NTPRO_V32_GNG_OPERATOR_ISSUE:-1044}"
BOUNDARY_ISSUE="${NTPRO_V32_GNG_BOUNDARY_ISSUE:-1043}"
V320_MILESTONE_TITLE="${NTPRO_V32_GNG_MILESTONE_TITLE:-v0.32.0}"
CONTRACT_JSON="${NTPRO_V32_GNG_JSON:-docs/rust-cutover/release/v0_32_0_risk_audit_go_no_go_closeout.json}"
CONTRACT_MD="${NTPRO_V32_GNG_MD:-docs/rust-cutover/release/v0_32_0_risk_audit_go_no_go_closeout.md}"
TASK_DOC="${NTPRO_V32_GNG_TASK:-docs/rust-cutover/tasks/V320-003.md}"
EVIDENCE_DOC="${NTPRO_V32_GNG_EVIDENCE:-docs/rust-cutover/evidence/V320-003.md}"

fail() {
  echo "v32 risk audit go/no-go closeout failed: $*" >&2
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
  docs/rust-cutover/release/v0_32_0_owner_operator_change_window_closeout.json \
  docs/rust-cutover/tasks/V320-002.md \
  docs/rust-cutover/evidence/V320-002.md \
  docs/rust-cutover/release/README.md \
  scripts/ai/verify_v32_owner_operator_change_window_closeout.sh \
  scripts/ai/verify_v32_risk_audit_go_no_go_closeout.sh; do
  require_file "$path"
done

scripts/ai/verify_v32_owner_operator_change_window_closeout.sh >/dev/null

CONTRACT_JSON="$CONTRACT_JSON" CONTRACT_MD="$CONTRACT_MD" TASK_DOC="$TASK_DOC" EVIDENCE_DOC="$EVIDENCE_DOC" python3 <<'PY'
import json
import os
from pathlib import Path


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def require_contains(text: str, marker: str, label: str) -> None:
    require(marker in text, f"{label} missing marker: {marker}")


def classify(case: dict) -> str:
    if case["risk_status"] == "missing":
        return "blocked_missing_risk_status"
    if case["risk_status"] == "failed":
        return "blocked_failed_risk_status"
    if case["risk_status"] == "stale":
        return "blocked_stale_risk_status"
    if case["audit_readiness"] == "missing":
        return "blocked_missing_audit_readiness"
    if case["audit_readiness"] == "failed":
        return "blocked_failed_audit_readiness"
    if case["audit_readiness"] == "stale":
        return "blocked_stale_audit_readiness"
    if case["go_no_go_freshness"] == "stale":
        return "blocked_stale_operator_go_no_go"
    if case.get("contradictory_decision_state") is True:
        return "blocked_contradictory_decision_state"
    if case["release_identity"] == "mismatch":
        return "blocked_release_identity_mismatch"
    if case["rollback_readiness"] == "missing":
        return "blocked_missing_rollback_readiness"
    if case["rollback_readiness"] == "stale":
        return "blocked_stale_rollback_readiness"
    if case["operator_go_no_go"] == "missing":
        return "blocked_missing_operator_go_no_go"
    if case["operator_go_no_go"] == "no_go":
        return "blocked_operator_no_go"
    if case["risk_status"] == "degraded":
        return "candidate_degraded_risk_status"
    return "approved_closeout_no_execution_authority"


contract = json.loads(Path(os.environ["CONTRACT_JSON"]).read_text(encoding="utf-8"))
contract_md = Path(os.environ["CONTRACT_MD"]).read_text(encoding="utf-8")
task = Path(os.environ["TASK_DOC"]).read_text(encoding="utf-8")
evidence = Path(os.environ["EVIDENCE_DOC"]).read_text(encoding="utf-8")
release_index = Path("docs/rust-cutover/release/README.md").read_text(encoding="utf-8")

require(contract.get("schema_version") == "ntpro.v320_risk_audit_go_no_go_closeout.v1", "schema mismatch")
require(contract.get("task_id") == "V320-003", "task mismatch")
require(contract.get("github_issue") == 1045, "issue mismatch")
require(contract.get("milestone") == "v0.32.0", "milestone mismatch")
require(contract.get("control_status") == "risk_audit_go_no_go_required_no_execution_authority", "control status mismatch")
deps = {item.get("github_issue"): item for item in contract.get("depends_on") or []}
require(deps.get(1043, {}).get("required_status") == "closed", "missing V320-001 dependency")
require(deps.get(1044, {}).get("required_status") == "closed", "missing V320-002 dependency")

inputs = contract.get("required_inputs") or {}
for key in ["risk_status", "audit_readiness", "release_identity", "rollback_readiness", "operator_go_no_go"]:
    require((inputs.get(key) or {}).get("required") is True, f"{key} must be required")
require((inputs.get("risk_status") or {}).get("stable_identifier_required") is True, "risk stable id required")
require((inputs.get("audit_readiness") or {}).get("immutable_evidence_required") is True, "audit immutable evidence required")
require((inputs.get("operator_go_no_go") or {}).get("stable_decision_digest_required") is True, "go/no-go digest required")
require((inputs.get("operator_go_no_go") or {}).get("go_alone_authorizes_execution") is False, "go alone must not authorize execution")
require((inputs.get("operator_go_no_go") or {}).get("contradictory_decision_state_allowed") is False, "contradictory decision must be false")
require((inputs.get("evidence_reconstruction") or {}).get("source_controlled_or_remote_reconstructable_required") is True, "reconstructable evidence required")
require((inputs.get("evidence_reconstruction") or {}).get("stable_identifiers_required") is True, "stable identifiers required")

for field in ["risk_decision_id", "risk_model_version", "risk_policy_digest", "risk_evidence_digest"]:
    require(field in set((inputs.get("risk_status") or {}).get("required_fields") or []), f"risk field missing {field}")
for field in ["audit_record_id", "immutable_storage_ref", "audit_evidence_digest", "remote_reconstruction_ref"]:
    require(field in set((inputs.get("audit_readiness") or {}).get("required_fields") or []), f"audit field missing {field}")
for field in ["release_version", "release_tag", "build_commit", "workflow_run_id", "release_body_hash", "gate_run_id"]:
    require(field in set((inputs.get("release_identity") or {}).get("required_fields") or []), f"release identity missing {field}")
for field in ["decision_id", "approver", "operator", "decision", "rollback_ref", "risk_decision_ref", "audit_record_ref", "decision_digest", "contradictory_state"]:
    require(field in set((inputs.get("operator_go_no_go") or {}).get("required_fields") or []), f"go/no-go field missing {field}")

expected_reason_codes = {
    "missing_risk": "blocked_missing_risk_status",
    "failed_risk": "blocked_failed_risk_status",
    "stale_risk": "blocked_stale_risk_status",
    "degraded_risk": "candidate_degraded_risk_status",
    "missing_audit": "blocked_missing_audit_readiness",
    "failed_audit": "blocked_failed_audit_readiness",
    "stale_audit": "blocked_stale_audit_readiness",
    "missing_go_no_go": "blocked_missing_operator_go_no_go",
    "stale_go_no_go": "blocked_stale_operator_go_no_go",
    "contradictory_decision_state": "blocked_contradictory_decision_state",
    "release_mismatch": "blocked_release_identity_mismatch",
    "operator_no_go": "blocked_operator_no_go",
    "approved_closeout": "approved_closeout_no_execution_authority",
}
reason_codes = contract.get("reason_codes") or {}
for key, value in expected_reason_codes.items():
    require(reason_codes.get(key) == value, f"reason code mismatch for {key}")
for key, value in (contract.get("runtime_boundary_flags") or {}).items():
    require(value is False, f"runtime boundary must stay false: {key}")
for case in contract.get("decision_cases") or []:
    expected = case.get("expected_status")
    got = classify(case)
    require(got == expected, f"case {case.get('case_id')} expected {expected} got {got}")
require(len(contract.get("decision_cases") or []) == 10, "decision case count mismatch")
require(contract.get("runtime_behavior_changed") is False, "runtime behavior must not change")
require(contract.get("trading_behavior_changed") is False, "trading behavior must not change")

for marker in [
    "control_status = risk_audit_go_no_go_required_no_execution_authority",
    "depends_on_issue_1043 = closed",
    "depends_on_issue_1044 = closed",
    "risk status required = true",
    "risk stable identifier required = true",
    "audit readiness required = true",
    "audit immutable evidence required = true",
    "release identity required = true",
    "rollback readiness required = true",
    "operator go/no-go required = true",
    "go/no-go stable decision digest required = true",
    "go alone authorizes execution = false",
    "contradictory decision state allowed = false",
    "source controlled or remote reconstructable evidence required = true",
    "missing risk -> blocked_missing_risk_status",
    "failed risk -> blocked_failed_risk_status",
    "missing audit -> blocked_missing_audit_readiness",
    "stale go/no-go -> blocked_stale_operator_go_no_go",
    "contradictory decision state -> blocked_contradictory_decision_state",
    "operator no-go -> blocked_operator_no_go",
    "approved closeout -> approved_closeout_no_execution_authority",
    "go_no_go_authorizes_submit = false",
    "go_no_go_authorizes_mutation = false",
    "go_no_go_authorizes_adapter_send = false",
    "frontend_completion_claim = false",
    "backend_go_live_claim = false",
]:
    require_contains(contract_md, marker, "contract markdown")
    require_contains(evidence, marker, "evidence")

for label, text in {"task": task, "evidence": evidence, "README": release_index}.items():
    require_contains(text, "V320-003", label)
    require_contains(text, "v0.32.0", label)
PY

current_issue_json="$(gh_with_retry issue view "$CURRENT_ISSUE" --repo "$REPO" --json number,state,title,milestone)"
operator_issue_json="$(gh_with_retry issue view "$OPERATOR_ISSUE" --repo "$REPO" --json number,state,title,milestone)"
boundary_issue_json="$(gh_with_retry issue view "$BOUNDARY_ISSUE" --repo "$REPO" --json number,state,title,milestone)"
v320_issues_json="$(gh_with_retry issue list --repo "$REPO" --state all --milestone "$V320_MILESTONE_TITLE" --limit 100 --json number,state,title)"

CURRENT_ISSUE_JSON="$current_issue_json" OPERATOR_ISSUE_JSON="$operator_issue_json" BOUNDARY_ISSUE_JSON="$boundary_issue_json" V320_ISSUES_JSON="$v320_issues_json" python3 <<'PY'
import json
import os


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


current = json.loads(os.environ["CURRENT_ISSUE_JSON"])
operator = json.loads(os.environ["OPERATOR_ISSUE_JSON"])
boundary = json.loads(os.environ["BOUNDARY_ISSUE_JSON"])
v320_issues = json.loads(os.environ["V320_ISSUES_JSON"])
require(current.get("number") == 1045, "current issue mismatch")
require((current.get("milestone") or {}).get("title") == "v0.32.0", "current milestone mismatch")
require(operator.get("number") == 1044 and operator.get("state") == "CLOSED", "V320-002 must be closed")
require(boundary.get("number") == 1043 and boundary.get("state") == "CLOSED", "V320-001 must be closed")
v320_map = {item["number"]: item for item in v320_issues}
missing = [number for number in range(1042, 1052) if number not in v320_map]
require(not missing, f"missing V320 issues: {missing}")
print(
    "v32_risk_audit_go_no_go_live "
    "operator_issue=1044:CLOSED "
    f"current_issue=1045:{current.get('state')} "
    "v320_issues=10 "
    "approved_closeout_no_execution=true"
)
PY

echo "v32_risk_audit_go_no_go_closeout=pass task=V320-003 issue=1045 decision_cases=10 stable_ids_required=true controls_disabled=true"
