#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V32_ADMIN_BRIDGE_REPO:-atxinbao/NTPRO}"
CURRENT_ISSUE="${NTPRO_V32_ADMIN_BRIDGE_CURRENT_ISSUE:-1049}"
PREV_ISSUE="${NTPRO_V32_ADMIN_BRIDGE_PREV_ISSUE:-1048}"
V320_MILESTONE_TITLE="${NTPRO_V32_ADMIN_BRIDGE_MILESTONE_TITLE:-v0.32.0}"
CONTRACT_JSON="${NTPRO_V32_ADMIN_BRIDGE_JSON:-docs/rust-cutover/release/v0_32_0_backend_enablement_read_model_admin_bridge_closeout.json}"
CONTRACT_MD="${NTPRO_V32_ADMIN_BRIDGE_MD:-docs/rust-cutover/release/v0_32_0_backend_enablement_read_model_admin_bridge_closeout.md}"
TASK_DOC="${NTPRO_V32_ADMIN_BRIDGE_TASK:-docs/rust-cutover/tasks/V320-007.md}"
EVIDENCE_DOC="${NTPRO_V32_ADMIN_BRIDGE_EVIDENCE:-docs/rust-cutover/evidence/V320-007.md}"

fail() { echo "v32 backend read model/admin bridge closeout failed: $*" >&2; exit 1; }
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
  docs/rust-cutover/release/v0_32_0_telemetry_slo_incident_closeout.json \
  docs/rust-cutover/tasks/V320-006.md docs/rust-cutover/evidence/V320-006.md \
  docs/rust-cutover/release/README.md scripts/ai/verify_v32_telemetry_slo_incident_closeout.sh \
  scripts/ai/verify_v32_backend_enablement_read_model_admin_bridge_closeout.sh; do
  require_file "$path"
done

scripts/ai/verify_v32_telemetry_slo_incident_closeout.sh >/dev/null

CONTRACT_JSON="$CONTRACT_JSON" CONTRACT_MD="$CONTRACT_MD" TASK_DOC="$TASK_DOC" EVIDENCE_DOC="$EVIDENCE_DOC" python3 <<'PY'
import json, os
from pathlib import Path

def require(condition, message):
    if not condition:
        raise SystemExit(message)

def require_contains(text, marker, label):
    require(marker in text, f"{label} missing marker: {marker}")

def classify(case):
    if case["forbidden_control_present"]:
        return "fail_closed_forbidden_control"
    if case["projection"] == "missing":
        return "fail_closed_missing_projection"
    if case["projection"] == "stale":
        return "fail_closed_stale_projection"
    if case["backend_gate"] == "blocked":
        return "fail_closed_backend_gate_blocked"
    if case["change_window"] == "frozen":
        return "fail_closed_change_window_frozen"
    if case["rollback"] == "active":
        return "fail_closed_rollback_active"
    if case["incident"] == "active":
        return "fail_closed_incident_active"
    return "backend_closeout_state_visible_read_only_no_controls"

contract = json.loads(Path(os.environ["CONTRACT_JSON"]).read_text())
md = Path(os.environ["CONTRACT_MD"]).read_text()
task = Path(os.environ["TASK_DOC"]).read_text()
evidence = Path(os.environ["EVIDENCE_DOC"]).read_text()
readme = Path("docs/rust-cutover/release/README.md").read_text()

require(contract["schema_version"] == "ntpro.v320_backend_enablement_read_model_admin_bridge_closeout.v1", "schema mismatch")
require(contract["task_id"] == "V320-007", "task mismatch")
require(contract["github_issue"] == 1049, "issue mismatch")
require(contract["milestone"] == "v0.32.0", "milestone mismatch")
require(contract["gate_status"] == "backend_enablement_state_read_model_admin_bridge_ready_read_only_no_controls", "gate status mismatch")
require(contract["depends_on"][0]["github_issue"] == 1048 and contract["depends_on"][0]["required_status"] == "closed", "dependency mismatch")
projection = contract["read_model_projection"]
bridge = contract["admin_bridge"]
for key in ["required", "provenance_required", "release_bound_required", "environment_bound_required", "freshness_required"]:
    require(projection[key] is True, f"projection flag must be true: {key}")
require(projection["max_age_seconds"] == 300, "projection max age mismatch")
require(len(projection["required_inputs"]) >= 7, "projection inputs incomplete")
require(len(projection["required_fields"]) >= 14, "projection fields incomplete")
for key in ["required", "read_only", "operator_visibility_required", "audit_visibility_required"]:
    require(bridge[key] is True, f"admin bridge flag must be true: {key}")
require(bridge["mutation_route_status"] == "fail_closed_admin_mutation_control", "mutation route status mismatch")
required_allowed = {"read_backend_closeout_state", "render_backend_closeout_state", "read_provenance_refs", "read_audit_refs"}
require(required_allowed.issubset(set(bridge["allowed_actions"])), "allowed read-only actions incomplete")
required_forbidden = {"submit", "cancel", "replace", "amend", "flatten", "remediate", "adapter_send", "live_exchange_request", "retry_schedule", "production_state_mutation"}
require(required_forbidden.issubset(set(bridge["forbidden_actions"])), "forbidden actions incomplete")
cases = {case["state"]: case for case in contract["render_replay_cases"] if case["state"] in {"ready", "blocked", "frozen", "rollback_active", "incident_active"}}
require(set(cases) == {"ready", "blocked", "frozen", "rollback_active", "incident_active"}, "required replay states incomplete")
for case in contract["render_replay_cases"]:
    require(classify(case) == case["expected_status"], f"case mismatch {case['case_id']}")
for key, value in contract["surface_boundary_flags"].items():
    require(value is False, f"surface boundary flag must be false: {key}")
require(contract["runtime_behavior_changed"] is False, "runtime behavior changed")
require(contract["trading_behavior_changed"] is False, "trading behavior changed")
for marker in [
    "gate_status = backend_enablement_state_read_model_admin_bridge_ready_read_only_no_controls",
    "depends_on_issue_1048 = closed",
    "read model projection required = true",
    "read model provenance required = true",
    "admin bridge required = true",
    "admin bridge read only = true",
    "operator visibility required = true",
    "audit visibility required = true",
    "render replay required = true",
    "ready replay case = required",
    "blocked replay case = required",
    "frozen replay case = required",
    "rollback_active replay case = required",
    "incident_active replay case = required",
    "forbidden submit control -> fail_closed_forbidden_control",
    "forbidden cancel control -> fail_closed_forbidden_control",
    "forbidden replace control -> fail_closed_forbidden_control",
    "forbidden amend control -> fail_closed_forbidden_control",
    "forbidden flatten control -> fail_closed_forbidden_control",
    "forbidden remediation control -> fail_closed_forbidden_control",
    "missing projection -> fail_closed_missing_projection",
    "stale projection -> fail_closed_stale_projection",
    "admin mutation route -> fail_closed_admin_mutation_control",
    "blocked backend gate -> fail_closed_backend_gate_blocked",
    "frozen change window -> fail_closed_change_window_frozen",
    "rollback active -> fail_closed_rollback_active",
    "incident active -> fail_closed_incident_active",
    "admin_bridge_mutation_allowed = false",
    "dashboard_trading_controls_enabled = false",
    "trader_terminal_order_ticket_enabled = false",
    "adapter_send_allowed = false",
    "live_exchange_request_allowed = false",
    "retry_scheduler_enabled = false",
    "automatic_remediation_allowed = false",
    "backend_go_live_claim = false",
]:
    require_contains(md, marker, "contract markdown")
    require_contains(evidence, marker, "evidence")
for label, text in {"task": task, "evidence": evidence, "README": readme}.items():
    require_contains(text, "V320-007", label)
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
if current.get("number") != 1049:
    raise SystemExit("current issue mismatch")
if (current.get("milestone") or {}).get("title") != "v0.32.0":
    raise SystemExit("current milestone mismatch")
if prev.get("number") != 1048 or prev.get("state") != "CLOSED":
    raise SystemExit("V320-006 must be closed")
issue_map = {item["number"]: item for item in issues}
missing = [number for number in range(1042, 1052) if number not in issue_map]
if missing:
    raise SystemExit(f"missing V320 issues: {missing}")
print("v32_backend_admin_bridge_live previous_issue=1048:CLOSED current_issue=1049:%s v320_issues=10 read_only_bridge=true" % current.get("state"))
PY
echo "v32_backend_enablement_admin_bridge_closeout=pass task=V320-007 issue=1049 read_only=true forbidden_controls_fail_closed=true backend_go_live=false"
