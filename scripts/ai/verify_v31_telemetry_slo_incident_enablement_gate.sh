#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V31_TELEMETRY_REPO:-atxinbao/NTPRO}"
CURRENT_ISSUE="${NTPRO_V31_TELEMETRY_CURRENT_ISSUE:-1012}"
PREV_ISSUE="${NTPRO_V31_TELEMETRY_PREV_ISSUE:-1011}"
CONTRACT_JSON="${NTPRO_V31_TELEMETRY_JSON:-docs/rust-cutover/release/v0_31_0_telemetry_slo_incident_enablement_gate.json}"
CONTRACT_MD="${NTPRO_V31_TELEMETRY_MD:-docs/rust-cutover/release/v0_31_0_telemetry_slo_incident_enablement_gate.md}"
TASK_DOC="${NTPRO_V31_TELEMETRY_TASK:-docs/rust-cutover/tasks/V310-006.md}"
EVIDENCE_DOC="${NTPRO_V31_TELEMETRY_EVIDENCE:-docs/rust-cutover/evidence/V310-006.md}"

fail() { echo "v31 telemetry SLO incident enablement gate failed: $*" >&2; exit 1; }
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
  docs/rust-cutover/release/v0_31_0_production_config_venue_readiness_provenance_gate.json \
  docs/rust-cutover/tasks/V310-005.md docs/rust-cutover/evidence/V310-005.md \
  docs/rust-cutover/release/README.md scripts/ai/verify_v31_production_config_venue_readiness_gate.sh \
  scripts/ai/verify_v31_telemetry_slo_incident_enablement_gate.sh; do
  require_file "$path"
done

scripts/ai/verify_v31_production_config_venue_readiness_gate.sh >/dev/null

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
    if case["slo"] == "degraded":
        return "fail_closed_degraded_slo"
    if case["incident"] == "active":
        return "fail_closed_incident_active"
    return "telemetry_slo_incident_ready_no_automatic_action"

contract = json.loads(Path(os.environ["CONTRACT_JSON"]).read_text())
md = Path(os.environ["CONTRACT_MD"]).read_text()
task = Path(os.environ["TASK_DOC"]).read_text()
evidence = Path(os.environ["EVIDENCE_DOC"]).read_text()
readme = Path("docs/rust-cutover/release/README.md").read_text()

require(contract["schema_version"] == "ntpro.v310_telemetry_slo_incident_enablement_gate.v1", "schema mismatch")
require(contract["task_id"] == "V310-006", "task mismatch")
require(contract["github_issue"] == 1012, "issue mismatch")
require(contract["gate_status"] == "telemetry_slo_incident_ready_required_no_automatic_action", "gate status mismatch")
deps = {(item["task_id"], item["github_issue"], item["required_status"]) for item in contract["depends_on"]}
for expected in [("V310-003", 1009, "closed"), ("V310-004", 1010, "closed"), ("V310-005", 1011, "closed")]:
    require(expected in deps, f"dependency missing: {expected}")
telemetry = contract["telemetry_evidence"]
slo = contract["slo_evidence"]
incident = contract["incident_response_evidence"]
for key in [
    "required",
    "source_provenance_required",
    "release_bound_required",
    "runtime_boundary_bound_required",
    "freshness_required",
]:
    require(telemetry[key] is True, f"telemetry flag must be true: {key}")
require(telemetry["max_age_seconds"] == 300, "telemetry max age mismatch")
require(len(telemetry["required_sources"]) >= 4, "telemetry sources incomplete")
for key in ["required", "release_bound_required", "runtime_boundary_bound_required"]:
    require(slo[key] is True, f"slo flag must be true: {key}")
require(len(slo["required_thresholds"]) >= 5, "slo thresholds incomplete")
for key in ["required", "incident_owner_required", "escalation_route_required", "alert_routing_required"]:
    require(incident[key] is True, f"incident flag must be true: {key}")
require(incident["automatic_incident_action_allowed"] is False, "automatic incident action must be false")
states = {state["state"]: state for state in contract["candidate_states"]}
require(states["healthy"]["candidate_readiness_allowed"] is True, "healthy candidate readiness")
for state_name in ["degraded", "stale", "missing", "incident_active"]:
    require(states[state_name]["candidate_readiness_allowed"] is False, f"{state_name} candidate readiness must be false")
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
    "gate_status = telemetry_slo_incident_ready_required_no_automatic_action",
    "telemetry evidence required = true",
    "telemetry source provenance required = true",
    "telemetry release-bound required = true",
    "telemetry runtime-bound required = true",
    "telemetry freshness required = true",
    "slo evidence required = true",
    "incident response evidence required = true",
    "incident owner required = true",
    "missing telemetry -> fail_closed_missing_telemetry",
    "stale telemetry -> fail_closed_stale_telemetry",
    "degraded SLO -> fail_closed_degraded_slo",
    "incident active -> fail_closed_incident_active",
    "telemetry_action_effect_allowed = false",
    "automatic_remediation_allowed = false",
    "backend_go_live_claim = false",
]:
    require_contains(md, marker, "contract markdown")
    require_contains(evidence, marker, "evidence")
for label, text in {"task": task, "evidence": evidence, "README": readme}.items():
    require_contains(text, "V310-006", label)
    require_contains(text, "v0.31.0", label)
PY

current_issue_json="$(gh_with_retry issue view "$CURRENT_ISSUE" --repo "$REPO" --json number,state,title)"
prev_issue_json="$(gh_with_retry issue view "$PREV_ISSUE" --repo "$REPO" --json number,state,title)"
CURRENT_ISSUE_JSON="$current_issue_json" PREV_ISSUE_JSON="$prev_issue_json" python3 <<'PY'
import json, os
current = json.loads(os.environ["CURRENT_ISSUE_JSON"])
prev = json.loads(os.environ["PREV_ISSUE_JSON"])
if current.get("number") != 1012:
    raise SystemExit("current issue mismatch")
if prev.get("number") != 1011 or prev.get("state") != "CLOSED":
    raise SystemExit("V310-005 must be closed")
print("v31_telemetry_slo_incident_gate_live current_issue_state=%s v310_005_state=CLOSED telemetry_required=true incident_gate=true" % current.get("state"))
PY
echo "v31_telemetry_slo_incident_gate=pass task=V310-006 issue=1012 automatic_action=false backend_go_live=false"
