#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V32_TELEMETRY_REPO:-atxinbao/NTPRO}"
CURRENT_ISSUE="${NTPRO_V32_TELEMETRY_CURRENT_ISSUE:-1048}"
PREV_ISSUE="${NTPRO_V32_TELEMETRY_PREV_ISSUE:-1047}"
V320_MILESTONE_TITLE="${NTPRO_V32_TELEMETRY_MILESTONE_TITLE:-v0.32.0}"
CONTRACT_JSON="${NTPRO_V32_TELEMETRY_JSON:-docs/rust-cutover/release/v0_32_0_telemetry_slo_incident_closeout.json}"
CONTRACT_MD="${NTPRO_V32_TELEMETRY_MD:-docs/rust-cutover/release/v0_32_0_telemetry_slo_incident_closeout.md}"
TASK_DOC="${NTPRO_V32_TELEMETRY_TASK:-docs/rust-cutover/tasks/V320-006.md}"
EVIDENCE_DOC="${NTPRO_V32_TELEMETRY_EVIDENCE:-docs/rust-cutover/evidence/V320-006.md}"

fail() { echo "v32 telemetry SLO incident closeout failed: $*" >&2; exit 1; }
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
  docs/rust-cutover/release/v0_32_0_canary_rollback_dr_closeout.json \
  docs/rust-cutover/tasks/V320-005.md docs/rust-cutover/evidence/V320-005.md \
  docs/rust-cutover/release/README.md scripts/ai/verify_v32_canary_rollback_dr_closeout.sh \
  scripts/ai/verify_v32_telemetry_slo_incident_closeout.sh; do
  require_file "$path"
done

scripts/ai/verify_v32_canary_rollback_dr_closeout.sh >/dev/null

CONTRACT_JSON="$CONTRACT_JSON" CONTRACT_MD="$CONTRACT_MD" TASK_DOC="$TASK_DOC" EVIDENCE_DOC="$EVIDENCE_DOC" python3 <<'PY'
import json, os
from pathlib import Path

def require(condition, message):
    if not condition:
        raise SystemExit(message)

def require_contains(text, marker, label):
    require(marker in text, f"{label} missing marker: {marker}")

def classify(case):
    if case["telemetry"] == "missing":
        return "fail_closed_missing_telemetry"
    if case["telemetry"] == "stale":
        return "fail_closed_stale_telemetry"
    if case["slo"] == "breach":
        return "fail_closed_slo_breach"
    if case["alert_route"] == "missing":
        return "fail_closed_missing_alert_route"
    if case["alert_ack"] == "missing":
        return "fail_closed_unacknowledged_alert"
    if case["incident"] == "active":
        return "fail_closed_unresolved_incident"
    if case["rollback_ref"] == "missing":
        return "fail_closed_missing_rollback_reference"
    return "telemetry_slo_alert_incident_ready_no_automatic_action"

contract = json.loads(Path(os.environ["CONTRACT_JSON"]).read_text())
md = Path(os.environ["CONTRACT_MD"]).read_text()
task = Path(os.environ["TASK_DOC"]).read_text()
evidence = Path(os.environ["EVIDENCE_DOC"]).read_text()
readme = Path("docs/rust-cutover/release/README.md").read_text()

require(contract["schema_version"] == "ntpro.v320_telemetry_slo_incident_closeout.v1", "schema mismatch")
require(contract["task_id"] == "V320-006", "task mismatch")
require(contract["github_issue"] == 1048, "issue mismatch")
require(contract["milestone"] == "v0.32.0", "milestone mismatch")
require(contract["gate_status"] == "telemetry_slo_alert_incident_ready_required_no_automatic_action", "gate status mismatch")
require(contract["depends_on"][0]["github_issue"] == 1047 and contract["depends_on"][0]["required_status"] == "closed", "dependency mismatch")
telemetry = contract["telemetry_evidence"]
slo = contract["slo_evidence"]
incident = contract["incident_response_evidence"]
for key in ["required", "source_provenance_required", "release_bound_required", "environment_bound_required", "rollback_plan_reference_required", "freshness_required"]:
    require(telemetry[key] is True, f"telemetry flag must be true: {key}")
require(telemetry["max_age_seconds"] == 300, "telemetry max age mismatch")
require(len(telemetry["required_sources"]) >= 6, "telemetry sources incomplete")
for key in ["required", "release_bound_required", "threshold_pass_required"]:
    require(slo[key] is True, f"slo flag must be true: {key}")
for key in ["required", "incident_owner_required", "escalation_route_required", "alert_routing_required", "alert_acknowledgement_required", "incident_freeze_criteria_required"]:
    require(incident[key] is True, f"incident flag must be true: {key}")
require(incident["automatic_incident_action_allowed"] is False, "automatic incident action must be false")
states = {state["state"]: state for state in contract["candidate_states"]}
require(states["healthy"]["candidate_readiness_allowed"] is True, "healthy readiness")
for state_name in ["degraded", "stale", "missing", "incident_active", "alert_unacknowledged"]:
    require(states[state_name]["candidate_readiness_allowed"] is False, f"{state_name} readiness must be false")
for state in states.values():
    require(state["runtime_execution_allowed"] is False, "runtime execution must be false")
    require(state["automatic_action_allowed"] is False, "automatic action must be false")
for case in contract["decision_cases"]:
    require(classify(case) == case["expected_status"], f"case mismatch {case['case_id']}")
for key, value in contract["runtime_boundary_flags"].items():
    require(value is False, f"runtime flag must be false: {key}")
require(contract["runtime_behavior_changed"] is False, "runtime behavior changed")
require(contract["trading_behavior_changed"] is False, "trading behavior changed")
for marker in [
    "gate_status = telemetry_slo_alert_incident_ready_required_no_automatic_action",
    "depends_on_issue_1047 = closed",
    "telemetry evidence required = true",
    "telemetry source provenance required = true",
    "telemetry rollback plan reference required = true",
    "slo threshold pass required = true",
    "incident response evidence required = true",
    "alert routing required = true",
    "alert acknowledgement required = true",
    "missing telemetry -> fail_closed_missing_telemetry",
    "stale telemetry -> fail_closed_stale_telemetry",
    "SLO breach -> fail_closed_slo_breach",
    "missing alert route -> fail_closed_missing_alert_route",
    "unacknowledged alert -> fail_closed_unacknowledged_alert",
    "unresolved incident -> fail_closed_unresolved_incident",
    "telemetry_action_effect_allowed = false",
    "autonomous_production_operation_allowed = false",
    "automatic_remediation_allowed = false",
    "backend_go_live_claim = false",
]:
    require_contains(md, marker, "contract markdown")
    require_contains(evidence, marker, "evidence")
for label, text in {"task": task, "evidence": evidence, "README": readme}.items():
    require_contains(text, "V320-006", label)
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
if current.get("number") != 1048:
    raise SystemExit("current issue mismatch")
if (current.get("milestone") or {}).get("title") != "v0.32.0":
    raise SystemExit("current milestone mismatch")
if prev.get("number") != 1047 or prev.get("state") != "CLOSED":
    raise SystemExit("V320-005 must be closed")
issue_map = {item["number"]: item for item in issues}
missing = [number for number in range(1042, 1052) if number not in issue_map]
if missing:
    raise SystemExit(f"missing V320 issues: {missing}")
print("v32_telemetry_slo_incident_live previous_issue=1047:CLOSED current_issue=1048:%s v320_issues=10 telemetry_required=true incident_gate=true" % current.get("state"))
PY
echo "v32_telemetry_slo_incident_closeout=pass task=V320-006 issue=1048 automatic_action=false backend_go_live=false alert_ack_required=true"
