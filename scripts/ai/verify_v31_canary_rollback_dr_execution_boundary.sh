#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V31_CANARY_REPO:-atxinbao/NTPRO}"
CURRENT_ISSUE="${NTPRO_V31_CANARY_CURRENT_ISSUE:-1010}"
PREV_ISSUE="${NTPRO_V31_CANARY_PREV_ISSUE:-1009}"
CONTRACT_JSON="${NTPRO_V31_CANARY_JSON:-docs/rust-cutover/release/v0_31_0_canary_rollback_dr_execution_boundary.json}"
CONTRACT_MD="${NTPRO_V31_CANARY_MD:-docs/rust-cutover/release/v0_31_0_canary_rollback_dr_execution_boundary.md}"
TASK_DOC="${NTPRO_V31_CANARY_TASK:-docs/rust-cutover/tasks/V310-004.md}"
EVIDENCE_DOC="${NTPRO_V31_CANARY_EVIDENCE:-docs/rust-cutover/evidence/V310-004.md}"

fail() { echo "v31 canary rollback DR boundary failed: $*" >&2; exit 1; }
require_file() { [[ -f "$1" ]] || fail "missing required file: $1"; }
gh_with_retry() {
  local attempt=1
  while true; do
    if GODEBUG=http2client=0 gh "$@"; then return 0; fi
    (( attempt >= 4 )) && return 1
    sleep "$((attempt * 2))"
    attempt=$((attempt + 1))
  done
}

for path in "$CONTRACT_JSON" "$CONTRACT_MD" "$TASK_DOC" "$EVIDENCE_DOC" \
  docs/rust-cutover/release/v0_31_0_risk_audit_go_no_go_control_contract.json \
  docs/rust-cutover/tasks/V310-003.md docs/rust-cutover/evidence/V310-003.md \
  docs/rust-cutover/release/README.md scripts/ai/verify_v31_risk_audit_go_no_go_control_contract.sh \
  scripts/ai/verify_v31_canary_rollback_dr_execution_boundary.sh; do
  require_file "$path"
done

scripts/ai/verify_v31_risk_audit_go_no_go_control_contract.sh >/dev/null

CONTRACT_JSON="$CONTRACT_JSON" CONTRACT_MD="$CONTRACT_MD" TASK_DOC="$TASK_DOC" EVIDENCE_DOC="$EVIDENCE_DOC" python3 <<'PY'
import json, os
from pathlib import Path

def require(condition, message):
    if not condition:
        raise SystemExit(message)

def require_contains(text, marker, label):
    require(marker in text, f"{label} missing marker: {marker}")

def classify(case):
    if case["canary_scope"] == "widened":
        return "fail_closed_widened_canary_scope"
    if case["rollback_path"] == "missing":
        return "fail_closed_missing_rollback_path"
    if case["dr_evidence"] == "stale":
        return "fail_closed_stale_dr_evidence"
    return "canary_rollback_dr_ready_no_automatic_recovery"

contract = json.loads(Path(os.environ["CONTRACT_JSON"]).read_text())
md = Path(os.environ["CONTRACT_MD"]).read_text()
task = Path(os.environ["TASK_DOC"]).read_text()
evidence = Path(os.environ["EVIDENCE_DOC"]).read_text()
readme = Path("docs/rust-cutover/release/README.md").read_text()

require(contract["schema_version"] == "ntpro.v310_canary_rollback_dr_boundary.v1", "schema mismatch")
require(contract["task_id"] == "V310-004", "task mismatch")
require(contract["github_issue"] == 1010, "issue mismatch")
require(contract["boundary_status"] == "canary_rollback_dr_required_no_automatic_recovery", "boundary status mismatch")
require(contract["depends_on"][0]["github_issue"] == 1009 and contract["depends_on"][0]["required_status"] == "closed", "dependency mismatch")
canary = contract["canary_scope"]
require(canary["required"] is True, "canary required")
require(canary["blast_radius_limit_required"] is True, "blast radius required")
require(canary["canary_bypasses_rollback"] is False, "canary must not bypass rollback")
require(canary["canary_bypasses_dr"] is False, "canary must not bypass DR")
rollback = contract["rollback_checkpoints"]
dr = contract["dr_readiness"]
for obj, label in [(rollback, "rollback"), (dr, "dr")]:
    require(obj["required"] is True, f"{label} required")
    require(obj["source_provenance_required"] is True, f"{label} source provenance required")
    require(obj["release_bound_required"] is True, f"{label} release bound required")
for case in contract["decision_cases"]:
    require(classify(case) == case["expected_status"], f"case mismatch {case['case_id']}")
for key, value in contract["runtime_boundary_flags"].items():
    require(value is False, f"runtime flag must be false: {key}")
require(contract["runtime_behavior_changed"] is False, "runtime behavior changed")
require(contract["trading_behavior_changed"] is False, "trading behavior changed")
for marker in [
    "boundary_status = canary_rollback_dr_required_no_automatic_recovery",
    "canary scope required = true",
    "blast radius limit required = true",
    "canary bypasses rollback = false",
    "canary bypasses DR = false",
    "missing rollback path -> fail_closed_missing_rollback_path",
    "stale DR evidence -> fail_closed_stale_dr_evidence",
    "widened canary scope -> fail_closed_widened_canary_scope",
    "automatic_remediation_allowed = false",
    "automatic_recovery_allowed = false",
    "backend_go_live_claim = false",
]:
    require_contains(md, marker, "contract markdown")
    require_contains(evidence, marker, "evidence")
for label, text in {"task": task, "evidence": evidence, "README": readme}.items():
    require_contains(text, "V310-004", label)
    require_contains(text, "v0.31.0", label)
PY

current_issue_json="$(gh_with_retry issue view "$CURRENT_ISSUE" --repo "$REPO" --json number,state,title)"
prev_issue_json="$(gh_with_retry issue view "$PREV_ISSUE" --repo "$REPO" --json number,state,title)"
CURRENT_ISSUE_JSON="$current_issue_json" PREV_ISSUE_JSON="$prev_issue_json" python3 <<'PY'
import json, os
current = json.loads(os.environ["CURRENT_ISSUE_JSON"])
prev = json.loads(os.environ["PREV_ISSUE_JSON"])
if current.get("number") != 1010:
    raise SystemExit("current issue mismatch")
if prev.get("number") != 1009 or prev.get("state") != "CLOSED":
    raise SystemExit("V310-003 must be closed")
print("v31_canary_rollback_dr_boundary_live current_issue_state=%s v310_003_state=CLOSED rollback_required=true dr_required=true" % current.get("state"))
PY
echo "v31_canary_rollback_dr_boundary=pass task=V310-004 issue=1010 rollback_required=true dr_required=true automatic_recovery=false"
