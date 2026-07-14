#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V32_NEGATIVE_REPO:-atxinbao/NTPRO}"
CURRENT_ISSUE="${NTPRO_V32_NEGATIVE_CURRENT_ISSUE:-1050}"
PREV_ISSUE="${NTPRO_V32_NEGATIVE_PREV_ISSUE:-1049}"
V320_MILESTONE_TITLE="${NTPRO_V32_NEGATIVE_MILESTONE_TITLE:-v0.32.0}"
CONTRACT_JSON="${NTPRO_V32_NEGATIVE_JSON:-docs/rust-cutover/release/v0_32_0_fail_closed_negative_tests.json}"
CONTRACT_MD="${NTPRO_V32_NEGATIVE_MD:-docs/rust-cutover/release/v0_32_0_fail_closed_negative_tests.md}"
TASK_DOC="${NTPRO_V32_NEGATIVE_TASK:-docs/rust-cutover/tasks/V320-008.md}"
EVIDENCE_DOC="${NTPRO_V32_NEGATIVE_EVIDENCE:-docs/rust-cutover/evidence/V320-008.md}"

fail() { echo "v32 fail-closed negative tests failed: $*" >&2; exit 1; }
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
  docs/rust-cutover/release/v0_32_0_backend_enablement_read_model_admin_bridge_closeout.json \
  docs/rust-cutover/tasks/V320-007.md docs/rust-cutover/evidence/V320-007.md \
  docs/rust-cutover/release/README.md scripts/ai/verify_v32_backend_enablement_read_model_admin_bridge_closeout.sh \
  scripts/ai/verify_v32_fail_closed_negative_tests.sh; do
  require_file "$path"
done

scripts/ai/verify_v32_backend_enablement_read_model_admin_bridge_closeout.sh >/dev/null

CONTRACT_JSON="$CONTRACT_JSON" CONTRACT_MD="$CONTRACT_MD" TASK_DOC="$TASK_DOC" EVIDENCE_DOC="$EVIDENCE_DOC" python3 <<'PY'
import json, os
from pathlib import Path

def require(condition, message):
    if not condition:
        raise SystemExit(message)

def require_contains(text, marker, label):
    require(marker in text, f"{label} missing marker: {marker}")

def classify(case):
    missing = case.get("missing_proof", "none")
    evidence = case.get("evidence_state", "fresh")
    control = case.get("forbidden_control", "none")
    missing_map = {
        "approval": "fail_closed_missing_approval",
        "risk_audit_go_no_go": "fail_closed_missing_risk_audit_go_no_go",
        "rollback_dr": "fail_closed_missing_rollback_dr",
        "telemetry_slo_incident": "fail_closed_missing_telemetry_slo_incident",
        "forbidden_control_boundary": "fail_closed_missing_forbidden_control_boundary",
    }
    evidence_map = {
        "stale_config": "fail_closed_stale_config",
        "wrong_venue": "fail_closed_wrong_venue",
        "unresolved_incident": "fail_closed_unresolved_incident",
        "stale_release_evidence": "fail_closed_stale_release_evidence",
    }
    control_map = {
        "submit": "fail_closed_unscoped_submit",
        "mutation": "fail_closed_unscoped_mutation",
        "adapter_send": "fail_closed_adapter_send",
        "live_exchange_request": "fail_closed_live_exchange_request",
        "retry_scheduler": "fail_closed_retry_scheduler",
        "dashboard_forbidden_controls": "fail_closed_dashboard_forbidden_controls",
        "admin_bridge_mutation": "fail_closed_admin_mutation_control",
        "trader_terminal_order_ticket": "fail_closed_trader_terminal_order_ticket",
    }
    if missing != "none":
        return missing_map[missing]
    if evidence != "fresh":
        return evidence_map[evidence]
    if control != "none":
        return control_map[control]
    return "negative_matrix_ready_no_positive_execution"

contract = json.loads(Path(os.environ["CONTRACT_JSON"]).read_text())
md = Path(os.environ["CONTRACT_MD"]).read_text()
task = Path(os.environ["TASK_DOC"]).read_text()
evidence = Path(os.environ["EVIDENCE_DOC"]).read_text()
readme = Path("docs/rust-cutover/release/README.md").read_text()

require(contract["schema_version"] == "ntpro.v320_fail_closed_negative_tests.v1", "schema mismatch")
require(contract["task_id"] == "V320-008", "task mismatch")
require(contract["github_issue"] == 1050, "issue mismatch")
require(contract["milestone"] == "v0.32.0", "milestone mismatch")
require(contract["gate_status"] == "v32_fail_closed_negative_tests_required_no_positive_execution_path", "gate status mismatch")
require(contract["depends_on"][0]["github_issue"] == 1049 and contract["depends_on"][0]["required_status"] == "closed", "dependency mismatch")
matrix = contract["negative_matrix"]
for key in ["required", "control_boundary_required_explicit_false", "local_verifier_path_required", "pr_smoke_path_documented", "release_gate_path_required"]:
    require(matrix[key] is True, f"negative matrix flag must be true: {key}")
require(matrix["positive_production_execution_authorized"] is False, "positive production execution must be false")
require(len(contract["negative_cases"]) >= 17, "negative cases incomplete")
for case in contract["negative_cases"]:
    require(classify(case) == case["expected_status"], f"case mismatch {case['case_id']}")
boundaries = contract["control_boundaries"]
require(boundaries["required_explicit_false"] is True, "control boundaries must be explicit false")
require(boundaries["missing_status"] == "fail_closed_missing_forbidden_control_boundary", "missing boundary status mismatch")
require(len(boundaries["fields"]) >= 13, "control boundary fields incomplete")
for key, value in contract["surface_boundary_flags"].items():
    require(value is False, f"surface boundary flag must be false: {key}")
require(contract["runtime_behavior_changed"] is False, "runtime behavior changed")
require(contract["trading_behavior_changed"] is False, "trading behavior changed")
for marker in [
    "gate_status = v32_fail_closed_negative_tests_required_no_positive_execution_path",
    "depends_on_issue_1049 = closed",
    "negative matrix required = true",
    "missing approval -> fail_closed_missing_approval",
    "missing risk audit go no go -> fail_closed_missing_risk_audit_go_no_go",
    "missing rollback dr -> fail_closed_missing_rollback_dr",
    "missing telemetry slo incident -> fail_closed_missing_telemetry_slo_incident",
    "stale config -> fail_closed_stale_config",
    "wrong venue -> fail_closed_wrong_venue",
    "unresolved incident -> fail_closed_unresolved_incident",
    "stale release evidence -> fail_closed_stale_release_evidence",
    "unscoped submit -> fail_closed_unscoped_submit",
    "unscoped mutation -> fail_closed_unscoped_mutation",
    "adapter send -> fail_closed_adapter_send",
    "live exchange request -> fail_closed_live_exchange_request",
    "retry scheduler -> fail_closed_retry_scheduler",
    "dashboard forbidden controls -> fail_closed_dashboard_forbidden_controls",
    "admin bridge mutation -> fail_closed_admin_mutation_control",
    "trader terminal order ticket -> fail_closed_trader_terminal_order_ticket",
    "missing forbidden control boundary -> fail_closed_missing_forbidden_control_boundary",
    "control boundary required explicit false = true",
    "release gate path required = true",
    "positive production execution authorized = false",
    "submit_control_enabled = false",
    "cancel_control_enabled = false",
    "replace_control_enabled = false",
    "amend_control_enabled = false",
    "flatten_control_enabled = false",
    "adapter_send_allowed = false",
    "live_exchange_request_allowed = false",
    "retry_scheduler_enabled = false",
    "automatic_remediation_allowed = false",
    "trader_terminal_order_ticket_enabled = false",
    "backend_go_live_claim = false",
]:
    require_contains(md, marker, "contract markdown")
    require_contains(evidence, marker, "evidence")
for label, text in {"task": task, "evidence": evidence, "README": readme}.items():
    require_contains(text, "V320-008", label)
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
if current.get("number") != 1050:
    raise SystemExit("current issue mismatch")
if (current.get("milestone") or {}).get("title") != "v0.32.0":
    raise SystemExit("current milestone mismatch")
if prev.get("number") != 1049 or prev.get("state") != "CLOSED":
    raise SystemExit("V320-007 must be closed")
issue_map = {item["number"]: item for item in issues}
missing = [number for number in range(1042, 1052) if number not in issue_map]
if missing:
    raise SystemExit(f"missing V320 issues: {missing}")
print("v32_fail_closed_negative_live previous_issue=1049:CLOSED current_issue=1050:%s v320_issues=10 negative_cases=17" % current.get("state"))
PY
echo "v32_fail_closed_negative_tests=pass task=V320-008 issue=1050 positive_execution=false forbidden_controls_fail_closed=true"
