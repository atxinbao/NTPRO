#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V31_OPERATOR_REPO:-atxinbao/NTPRO}"
CURRENT_ISSUE="${NTPRO_V31_OPERATOR_CURRENT_ISSUE:-1008}"
BOUNDARY_ISSUE="${NTPRO_V31_OPERATOR_BOUNDARY_ISSUE:-1007}"
INTAKE_ISSUE="${NTPRO_V31_OPERATOR_INTAKE_ISSUE:-1006}"
CONTRACT_JSON="${NTPRO_V31_OPERATOR_JSON:-docs/rust-cutover/release/v0_31_0_operator_approval_freeze_change_window_lifecycle.json}"
CONTRACT_MD="${NTPRO_V31_OPERATOR_MD:-docs/rust-cutover/release/v0_31_0_operator_approval_freeze_change_window_lifecycle.md}"
TASK_DOC="${NTPRO_V31_OPERATOR_TASK:-docs/rust-cutover/tasks/V310-002.md}"
EVIDENCE_DOC="${NTPRO_V31_OPERATOR_EVIDENCE:-docs/rust-cutover/evidence/V310-002.md}"

fail() {
  echo "v31 operator approval lifecycle failed: $*" >&2
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
  docs/rust-cutover/release/v0_31_0_production_enablement_boundary_contract.json \
  docs/rust-cutover/release/v0_31_0_production_enablement_boundary_contract.md \
  docs/rust-cutover/tasks/V310-001.md \
  docs/rust-cutover/evidence/V310-001.md \
  docs/rust-cutover/release/README.md \
  scripts/ai/verify_v31_production_enablement_boundary_contract.sh \
  scripts/ai/verify_v31_operator_approval_freeze_change_window_lifecycle.sh; do
  require_file "$path"
done

scripts/ai/verify_v31_production_enablement_boundary_contract.sh >/dev/null

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
    if case.get("release_matches") is not True:
        return "fail_closed_release_mismatch"
    if freeze == "active":
        return "fail_closed_active_production_freeze"
    if case.get("within_change_window") is not True:
        return "fail_closed_outside_approved_change_window"
    return "approval_window_valid_execution_still_blocked_by_downstream_gates"


contract = json.loads(Path(os.environ["CONTRACT_JSON"]).read_text(encoding="utf-8"))
contract_md = Path(os.environ["CONTRACT_MD"]).read_text(encoding="utf-8")
task = Path(os.environ["TASK_DOC"]).read_text(encoding="utf-8")
evidence = Path(os.environ["EVIDENCE_DOC"]).read_text(encoding="utf-8")
release_index = Path("docs/rust-cutover/release/README.md").read_text(encoding="utf-8")

require(contract.get("schema_version") == "ntpro.v310_operator_approval_lifecycle.v1", "schema mismatch")
require(contract.get("task_id") == "V310-002", "task mismatch")
require(contract.get("github_issue") == 1008, "issue mismatch")
require(contract.get("lifecycle_status") == "operator_approval_change_window_required_no_execution_authority", "lifecycle status mismatch")

deps = {item.get("github_issue"): item for item in contract.get("depends_on") or []}
require(deps.get(1006, {}).get("required_status") == "closed", "missing V310-000 dependency")
require(deps.get(1007, {}).get("required_status") == "closed", "missing V310-001 dependency")

for state in ["missing", "approved", "expired", "revoked", "release_mismatch", "window_active"]:
    require(state in set(contract.get("approval_states") or []), f"missing approval state {state}")
for state in ["none", "scheduled", "active", "lifted", "expired"]:
    require(state in set(contract.get("freeze_states") or []), f"missing freeze state {state}")

required_approval = set(contract.get("required_approval_evidence") or [])
for field in [
    "approval_id",
    "approver",
    "operator",
    "release_version",
    "release_tag",
    "build_commit",
    "github_issue",
    "change_window_id",
    "approval_digest",
    "boundary_digest",
    "redaction_profile",
    "source_provenance",
]:
    require(field in required_approval, f"approval evidence missing {field}")

window = contract.get("change_window_evidence") or {}
require(window.get("required") is True, "change window evidence must be required")
for key, value in {
    "outside_window_status": "fail_closed_outside_approved_change_window",
    "active_freeze_status": "fail_closed_active_production_freeze",
    "expired_approval_status": "fail_closed_expired_approval",
    "revoked_approval_status": "fail_closed_revoked_approval",
    "release_mismatch_status": "fail_closed_release_mismatch",
}.items():
    require(window.get(key) == value, f"{key} mismatch")

provenance = contract.get("provenance_and_redaction") or {}
require(provenance.get("immutable_evidence_required") is True, "immutable evidence required")
require(provenance.get("source_provenance_required") is True, "source provenance required")
require(provenance.get("redaction_required") is True, "redaction required")
require(provenance.get("raw_secret_allowed") is False, "raw secret must be false")
require(provenance.get("raw_account_identifier_allowed") is False, "raw account id must be false")
require(provenance.get("raw_operator_token_allowed") is False, "raw operator token must be false")
require(provenance.get("chat_or_external_notes_sufficient") is False, "chat/external notes must be false")

for key, value in (contract.get("runtime_boundary_flags") or {}).items():
    require(value is False, f"runtime flag must be false: {key}")
for case in contract.get("decision_cases") or []:
    expected = case.get("expected_status")
    got = classify(case)
    require(got == expected, f"case {case.get('case_id')} expected {expected} got {got}")
require(contract.get("runtime_behavior_changed") is False, "runtime behavior must not change")
require(contract.get("trading_behavior_changed") is False, "trading behavior must not change")

for marker in [
    "lifecycle_status = operator_approval_change_window_required_no_execution_authority",
    "change window evidence required = true",
    "immutable evidence required = true",
    "source provenance required = true",
    "redaction required = true",
    "missing approval -> fail_closed_missing_scoped_approval",
    "expired approval -> fail_closed_expired_approval",
    "release mismatch -> fail_closed_release_mismatch",
    "active freeze -> fail_closed_active_production_freeze",
    "outside approved change window -> fail_closed_outside_approved_change_window",
    "approval evidence is redacted and provenance-bound = true",
    "adapter_send_allowed = false",
    "backend_go_live_claim = false",
]:
    require_contains(contract_md, marker, "contract markdown")
    require_contains(evidence, marker, "evidence")

for label, text in {"task": task, "evidence": evidence, "README": release_index}.items():
    require_contains(text, "V310-002", label)
    require_contains(text, "v0.31.0", label)
PY

current_issue_json="$(gh_with_retry issue view "$CURRENT_ISSUE" --repo "$REPO" --json number,state,title)"
boundary_issue_json="$(gh_with_retry issue view "$BOUNDARY_ISSUE" --repo "$REPO" --json number,state,title)"
intake_issue_json="$(gh_with_retry issue view "$INTAKE_ISSUE" --repo "$REPO" --json number,state,title)"

CURRENT_ISSUE_JSON="$current_issue_json" \
BOUNDARY_ISSUE_JSON="$boundary_issue_json" \
INTAKE_ISSUE_JSON="$intake_issue_json" \
python3 <<'PY'
import json
import os


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


current = json.loads(os.environ["CURRENT_ISSUE_JSON"])
boundary = json.loads(os.environ["BOUNDARY_ISSUE_JSON"])
intake = json.loads(os.environ["INTAKE_ISSUE_JSON"])
require(current.get("number") == 1008, "current issue mismatch")
require(boundary.get("number") == 1007 and boundary.get("state") == "CLOSED", "V310-001 must be closed")
require(intake.get("number") == 1006 and intake.get("state") == "CLOSED", "V310-000 must be closed")

print(
    "v31_operator_approval_lifecycle_live "
    f"current_issue_state={current.get('state')} "
    "v310_000_state=CLOSED "
    "v310_001_state=CLOSED "
    "change_window_required=true "
    "active_freeze_fail_closed=true"
)
PY

echo "v31_operator_approval_lifecycle=pass task=V310-002 issue=1008 change_window_required=true active_freeze_fail_closed=true approval_evidence_redacted=true"
