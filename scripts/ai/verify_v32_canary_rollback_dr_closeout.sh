#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V32_CANARY_REPO:-atxinbao/NTPRO}"
CURRENT_ISSUE="${NTPRO_V32_CANARY_CURRENT_ISSUE:-1047}"
PREV_ISSUE="${NTPRO_V32_CANARY_PREV_ISSUE:-1046}"
V320_MILESTONE_TITLE="${NTPRO_V32_CANARY_MILESTONE_TITLE:-v0.32.0}"
CONTRACT_JSON="${NTPRO_V32_CANARY_JSON:-docs/rust-cutover/release/v0_32_0_canary_rollback_dr_closeout.json}"
CONTRACT_MD="${NTPRO_V32_CANARY_MD:-docs/rust-cutover/release/v0_32_0_canary_rollback_dr_closeout.md}"
TASK_DOC="${NTPRO_V32_CANARY_TASK:-docs/rust-cutover/tasks/V320-005.md}"
EVIDENCE_DOC="${NTPRO_V32_CANARY_EVIDENCE:-docs/rust-cutover/evidence/V320-005.md}"

fail() { echo "v32 canary rollback DR closeout failed: $*" >&2; exit 1; }
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
  docs/rust-cutover/release/v0_32_0_config_venue_credential_provenance.json \
  docs/rust-cutover/tasks/V320-004.md docs/rust-cutover/evidence/V320-004.md \
  docs/rust-cutover/release/README.md scripts/ai/verify_v32_config_venue_credential_provenance.sh \
  scripts/ai/verify_v32_canary_rollback_dr_closeout.sh; do
  require_file "$path"
done

scripts/ai/verify_v32_config_venue_credential_provenance.sh >/dev/null

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
    if case["canary_status"] == "failed":
        return "fail_closed_failed_canary"
    if case["incident_freeze"] == "active":
        return "fail_closed_unresolved_incident_freeze"
    if case["restoration_evidence"] == "missing":
        return "fail_closed_missing_restoration_evidence"
    if case["abort_state"] == "uncleared":
        return "fail_closed_uncleared_abort_state"
    return "canary_rollback_dr_ready_no_automatic_recovery"

contract = json.loads(Path(os.environ["CONTRACT_JSON"]).read_text())
md = Path(os.environ["CONTRACT_MD"]).read_text()
task = Path(os.environ["TASK_DOC"]).read_text()
evidence = Path(os.environ["EVIDENCE_DOC"]).read_text()
readme = Path("docs/rust-cutover/release/README.md").read_text()

require(contract["schema_version"] == "ntpro.v320_canary_rollback_dr_closeout.v1", "schema mismatch")
require(contract["task_id"] == "V320-005", "task mismatch")
require(contract["github_issue"] == 1047, "issue mismatch")
require(contract["milestone"] == "v0.32.0", "milestone mismatch")
require(contract["boundary_status"] == "canary_rollback_dr_required_abortable_no_automatic_recovery", "boundary status mismatch")
require(contract["depends_on"][0]["github_issue"] == 1046 and contract["depends_on"][0]["required_status"] == "closed", "dependency mismatch")
canary = contract["canary_scope"]
for key in ["required", "entry_criteria_required", "exit_criteria_required", "abort_triggers_required", "blast_radius_limit_required"]:
    require(canary[key] is True, f"canary flag must be true: {key}")
for key in ["canary_bypasses_rollback", "canary_bypasses_restore", "canary_bypasses_dr"]:
    require(canary[key] is False, f"canary bypass must be false: {key}")
for section_name in ["rollback_checkpoints", "restoration_evidence", "dr_readiness"]:
    section = contract[section_name]
    require(section["required"] is True, f"{section_name} required")
    require(section["source_provenance_required"] is True, f"{section_name} source provenance required")
dr = contract["dr_readiness"]
require(dr["failover_boundary_required"] is True, "DR failover required")
require(dr["failback_boundary_required"] is True, "DR failback required")
abort = contract["abort_control"]
require(abort["abort_state_requires_scoped_clear_decision"] is True, "abort clear decision required")
require(abort["automatic_retry_allowed"] is False, "automatic retry false")
require(abort["automatic_remediation_allowed"] is False, "automatic remediation false")
require(abort["automatic_recovery_allowed"] is False, "automatic recovery false")
for case in contract["decision_cases"]:
    require(classify(case) == case["expected_status"], f"case mismatch {case['case_id']}")
for key, value in contract["runtime_boundary_flags"].items():
    require(value is False, f"runtime flag must be false: {key}")
require(contract["runtime_behavior_changed"] is False, "runtime behavior changed")
require(contract["trading_behavior_changed"] is False, "trading behavior changed")
for marker in [
    "boundary_status = canary_rollback_dr_required_abortable_no_automatic_recovery",
    "depends_on_issue_1046 = closed",
    "canary entry criteria required = true",
    "canary exit criteria required = true",
    "canary abort triggers required = true",
    "canary bypasses rollback = false",
    "canary bypasses restore = false",
    "canary bypasses DR = false",
    "restoration evidence required = true",
    "DR failover boundary required = true",
    "DR failback boundary required = true",
    "abort state requires scoped clear decision = true",
    "failed canary -> fail_closed_failed_canary",
    "unresolved incident freeze -> fail_closed_unresolved_incident_freeze",
    "uncleared abort state -> fail_closed_uncleared_abort_state",
    "automatic_remediation_allowed = false",
    "automatic_recovery_allowed = false",
    "backend_go_live_claim = false",
]:
    require_contains(md, marker, "contract markdown")
    require_contains(evidence, marker, "evidence")
for label, text in {"task": task, "evidence": evidence, "README": readme}.items():
    require_contains(text, "V320-005", label)
    require_contains(text, "v0.32.0", label)
PY

current_issue_json="$(gh_with_retry issue view "$CURRENT_ISSUE" --repo "$REPO" --json number,state,title,milestone)"
prev_issue_json="$(gh_with_retry issue view "$PREV_ISSUE" --repo "$REPO" --json number,state,title,milestone)"
v320_issues_json="$(gh_with_retry issue list --repo "$REPO" --state all --milestone "$V320_MILESTONE_TITLE" --limit 100 --json number,state,title)"
CURRENT_ISSUE_JSON="$current_issue_json" PREV_ISSUE_JSON="$prev_issue_json" V320_ISSUES_JSON="$v320_issues_json" python3 <<'PY'
import json, os
current = json.loads(os.environ["CURRENT_ISSUE_JSON"])
prev = json.loads(os.environ["PREV_ISSUE_JSON"])
issues = json.loads(os.environ["V320_ISSUES_JSON"])
if current.get("number") != 1047:
    raise SystemExit("current issue mismatch")
if (current.get("milestone") or {}).get("title") != "v0.32.0":
    raise SystemExit("current milestone mismatch")
if prev.get("number") != 1046 or prev.get("state") != "CLOSED":
    raise SystemExit("V320-004 must be closed")
issue_map = {item["number"]: item for item in issues}
missing = [number for number in range(1042, 1052) if number not in issue_map]
if missing:
    raise SystemExit(f"missing V320 issues: {missing}")
print("v32_canary_rollback_dr_live previous_issue=1046:CLOSED current_issue=1047:%s v320_issues=10 rollback_required=true dr_required=true" % current.get("state"))
PY
echo "v32_canary_rollback_dr_closeout=pass task=V320-005 issue=1047 rollback_required=true dr_required=true automatic_recovery=false abort_requires_scoped_clear=true"
