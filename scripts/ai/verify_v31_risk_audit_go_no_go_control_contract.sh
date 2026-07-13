#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V31_GNG_REPO:-atxinbao/NTPRO}"
CURRENT_ISSUE="${NTPRO_V31_GNG_CURRENT_ISSUE:-1009}"
BOUNDARY_ISSUE="${NTPRO_V31_GNG_BOUNDARY_ISSUE:-1007}"
OPERATOR_ISSUE="${NTPRO_V31_GNG_OPERATOR_ISSUE:-1008}"
CONTRACT_JSON="${NTPRO_V31_GNG_JSON:-docs/rust-cutover/release/v0_31_0_risk_audit_go_no_go_control_contract.json}"
CONTRACT_MD="${NTPRO_V31_GNG_MD:-docs/rust-cutover/release/v0_31_0_risk_audit_go_no_go_control_contract.md}"
TASK_DOC="${NTPRO_V31_GNG_TASK:-docs/rust-cutover/tasks/V310-003.md}"
EVIDENCE_DOC="${NTPRO_V31_GNG_EVIDENCE:-docs/rust-cutover/evidence/V310-003.md}"

fail() {
  echo "v31 risk audit go/no-go contract failed: $*" >&2
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
  docs/rust-cutover/release/v0_31_0_operator_approval_freeze_change_window_lifecycle.json \
  docs/rust-cutover/tasks/V310-002.md \
  docs/rust-cutover/evidence/V310-002.md \
  docs/rust-cutover/release/README.md \
  scripts/ai/verify_v31_operator_approval_freeze_change_window_lifecycle.sh \
  scripts/ai/verify_v31_risk_audit_go_no_go_control_contract.sh; do
  require_file "$path"
done

scripts/ai/verify_v31_operator_approval_freeze_change_window_lifecycle.sh >/dev/null

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
    if case["risk_status"] == "stale":
        return "blocked_stale_risk_status"
    if case["audit_readiness"] == "missing":
        return "blocked_missing_audit_readiness"
    if case["audit_readiness"] == "stale":
        return "blocked_stale_audit_readiness"
    if case["release_identity"] == "missing":
        return "blocked_missing_release_identity"
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
    return "approved_candidate_no_execution_authority"


contract = json.loads(Path(os.environ["CONTRACT_JSON"]).read_text(encoding="utf-8"))
contract_md = Path(os.environ["CONTRACT_MD"]).read_text(encoding="utf-8")
task = Path(os.environ["TASK_DOC"]).read_text(encoding="utf-8")
evidence = Path(os.environ["EVIDENCE_DOC"]).read_text(encoding="utf-8")
release_index = Path("docs/rust-cutover/release/README.md").read_text(encoding="utf-8")

require(contract.get("schema_version") == "ntpro.v310_risk_audit_go_no_go.v1", "schema mismatch")
require(contract.get("task_id") == "V310-003", "task mismatch")
require(contract.get("github_issue") == 1009, "issue mismatch")
require(contract.get("control_status") == "risk_audit_go_no_go_required_no_execution_authority", "control status mismatch")
deps = {item.get("github_issue"): item for item in contract.get("depends_on") or []}
require(deps.get(1007, {}).get("required_status") == "closed", "missing V310-001 dependency")
require(deps.get(1008, {}).get("required_status") == "closed", "missing V310-002 dependency")

inputs = contract.get("required_inputs") or {}
for key in ["risk_status", "audit_readiness", "release_identity", "rollback_readiness", "operator_go_no_go"]:
    require((inputs.get(key) or {}).get("required") is True, f"{key} must be required")
require((inputs.get("operator_go_no_go") or {}).get("go_alone_authorizes_execution") is False, "go alone must not authorize execution")
for field in ["release_version", "release_tag", "build_commit", "workflow_run_id", "release_body_hash"]:
    require(field in set((inputs.get("release_identity") or {}).get("required_fields") or []), f"release identity missing {field}")

expected_reason_codes = {
    "missing_risk": "blocked_missing_risk_status",
    "stale_risk": "blocked_stale_risk_status",
    "degraded_risk": "candidate_degraded_risk_status",
    "missing_audit": "blocked_missing_audit_readiness",
    "stale_audit": "blocked_stale_audit_readiness",
    "release_mismatch": "blocked_release_identity_mismatch",
    "operator_no_go": "blocked_operator_no_go",
    "approved_candidate": "approved_candidate_no_execution_authority",
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
require(contract.get("runtime_behavior_changed") is False, "runtime behavior must not change")
require(contract.get("trading_behavior_changed") is False, "trading behavior must not change")

for marker in [
    "control_status = risk_audit_go_no_go_required_no_execution_authority",
    "risk status required = true",
    "audit readiness required = true",
    "release identity required = true",
    "rollback readiness required = true",
    "operator go/no-go required = true",
    "go alone authorizes execution = false",
    "missing risk -> blocked_missing_risk_status",
    "stale audit -> blocked_stale_audit_readiness",
    "release mismatch -> blocked_release_identity_mismatch",
    "operator no-go -> blocked_operator_no_go",
    "approved candidate -> approved_candidate_no_execution_authority",
    "go_no_go_authorizes_submit = false",
    "go_no_go_authorizes_mutation = false",
    "dashboard_trading_controls_enabled = false",
    "trader_terminal_order_ticket_enabled = false",
]:
    require_contains(contract_md, marker, "contract markdown")
    require_contains(evidence, marker, "evidence")

for label, text in {"task": task, "evidence": evidence, "README": release_index}.items():
    require_contains(text, "V310-003", label)
    require_contains(text, "v0.31.0", label)
PY

current_issue_json="$(gh_with_retry issue view "$CURRENT_ISSUE" --repo "$REPO" --json number,state,title)"
boundary_issue_json="$(gh_with_retry issue view "$BOUNDARY_ISSUE" --repo "$REPO" --json number,state,title)"
operator_issue_json="$(gh_with_retry issue view "$OPERATOR_ISSUE" --repo "$REPO" --json number,state,title)"

CURRENT_ISSUE_JSON="$current_issue_json" BOUNDARY_ISSUE_JSON="$boundary_issue_json" OPERATOR_ISSUE_JSON="$operator_issue_json" python3 <<'PY'
import json
import os


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


current = json.loads(os.environ["CURRENT_ISSUE_JSON"])
boundary = json.loads(os.environ["BOUNDARY_ISSUE_JSON"])
operator = json.loads(os.environ["OPERATOR_ISSUE_JSON"])
require(current.get("number") == 1009, "current issue mismatch")
require(boundary.get("number") == 1007 and boundary.get("state") == "CLOSED", "V310-001 must be closed")
require(operator.get("number") == 1008 and operator.get("state") == "CLOSED", "V310-002 must be closed")
print(
    "v31_risk_audit_go_no_go_contract_live "
    f"current_issue_state={current.get('state')} "
    "v310_001_state=CLOSED "
    "v310_002_state=CLOSED "
    "approved_candidate_no_execution=true"
)
PY

echo "v31_risk_audit_go_no_go_contract=pass task=V310-003 issue=1009 approved_candidate_no_execution=true controls_disabled=true"
