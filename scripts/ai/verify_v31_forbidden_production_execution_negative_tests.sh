#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V31_NEGATIVE_REPO:-atxinbao/NTPRO}"
CURRENT_ISSUE="${NTPRO_V31_NEGATIVE_CURRENT_ISSUE:-1014}"
PREV_ISSUE="${NTPRO_V31_NEGATIVE_PREV_ISSUE:-1013}"
CONTRACT_JSON="${NTPRO_V31_NEGATIVE_JSON:-docs/rust-cutover/release/v0_31_0_forbidden_production_execution_negative_tests.json}"
CONTRACT_MD="${NTPRO_V31_NEGATIVE_MD:-docs/rust-cutover/release/v0_31_0_forbidden_production_execution_negative_tests.md}"
TASK_DOC="${NTPRO_V31_NEGATIVE_TASK:-docs/rust-cutover/tasks/V310-008.md}"
EVIDENCE_DOC="${NTPRO_V31_NEGATIVE_EVIDENCE:-docs/rust-cutover/evidence/V310-008.md}"

fail() { echo "v31 forbidden production execution negative tests failed: $*" >&2; exit 1; }
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
  docs/rust-cutover/release/v0_31_0_backend_enablement_state_read_model_admin_bridge.json \
  docs/rust-cutover/tasks/V310-007.md docs/rust-cutover/evidence/V310-007.md \
  docs/rust-cutover/release/README.md scripts/ai/verify_v31_backend_enablement_state_read_model_admin_bridge.sh \
  scripts/ai/verify_v31_forbidden_production_execution_negative_tests.sh; do
  require_file "$path"
done

scripts/ai/verify_v31_backend_enablement_state_read_model_admin_bridge.sh >/dev/null

CONTRACT_JSON="$CONTRACT_JSON" CONTRACT_MD="$CONTRACT_MD" TASK_DOC="$TASK_DOC" EVIDENCE_DOC="$EVIDENCE_DOC" python3 <<'PY'
import json, os
from pathlib import Path

def require(condition, message):
    if not condition:
        raise SystemExit(message)

def require_contains(text, marker, label):
    require(marker in text, f"{label} missing marker: {marker}")

contract = json.loads(Path(os.environ["CONTRACT_JSON"]).read_text())
md = Path(os.environ["CONTRACT_MD"]).read_text()
task = Path(os.environ["TASK_DOC"]).read_text()
evidence = Path(os.environ["EVIDENCE_DOC"]).read_text()
readme = Path("docs/rust-cutover/release/README.md").read_text()

require(contract["schema_version"] == "ntpro.v310_forbidden_production_execution_negative_tests.v1", "schema mismatch")
require(contract["task_id"] == "V310-008", "task mismatch")
require(contract["github_issue"] == 1014, "issue mismatch")
require(contract["negative_suite_status"] == "deterministic_fail_closed_forbidden_execution", "suite status mismatch")
deps = {(item["task_id"], item["github_issue"], item["required_status"]) for item in contract["depends_on"]}
for number, task_id in [(1007, "V310-001"), (1008, "V310-002"), (1009, "V310-003"), (1010, "V310-004"), (1011, "V310-005"), (1012, "V310-006"), (1013, "V310-007")]:
    require((task_id, number, "closed") in deps, f"dependency missing: {task_id}")
for item in contract["forbidden_execution_flags"]:
    if item["flag"] in {"backend_go_live_claim", "product_grade_trading_terminal_claim"}:
        require(item["opened_status"] == "fail_closed_forbidden_live_trading_claim", f"claim reason mismatch: {item['flag']}")
    else:
        require(item["opened_status"] == "fail_closed_forbidden_execution_boundary", f"execution reason mismatch: {item['flag']}")
for case in contract["missing_gate_cases"]:
    require(case["expected_status"].startswith("fail_closed_"), f"missing gate case not fail-closed: {case['case_id']}")
surfaces = {surface["surface"]: surface for surface in contract["ingestion_surfaces"]}
for surface in ["source_artifact_schema_validator", "release_gate", "dashboard_ingestion", "admin_workbench_ingestion", "read_model_ingestion"]:
    require(surface in surfaces, f"surface missing: {surface}")
    require(surfaces[surface]["forbidden_true_flags_fail_closed"] is True, f"surface not fail-closed: {surface}")
for key, value in contract["release_claim_guards"].items():
    require(value is False, f"release claim guard must be false: {key}")
for key, value in contract["runtime_boundary_flags"].items():
    require(value is False, f"runtime flag must be false: {key}")
forbidden_flags = {item["flag"] for item in contract["forbidden_execution_flags"]}
v31_contracts = [
    "docs/rust-cutover/release/v0_31_0_production_enablement_boundary_contract.json",
    "docs/rust-cutover/release/v0_31_0_operator_approval_freeze_change_window_lifecycle.json",
    "docs/rust-cutover/release/v0_31_0_risk_audit_go_no_go_control_contract.json",
    "docs/rust-cutover/release/v0_31_0_canary_rollback_dr_execution_boundary.json",
    "docs/rust-cutover/release/v0_31_0_production_config_venue_readiness_provenance_gate.json",
    "docs/rust-cutover/release/v0_31_0_telemetry_slo_incident_enablement_gate.json",
    "docs/rust-cutover/release/v0_31_0_backend_enablement_state_read_model_admin_bridge.json",
]
for path in v31_contracts:
    data = json.loads(Path(path).read_text())
    flags = data.get("runtime_boundary_flags", {})
    for flag in forbidden_flags:
        if flag in flags:
            require(flags[flag] is False, f"{path} opens forbidden flag {flag}")
require(contract["runtime_behavior_changed"] is False, "runtime behavior changed")
require(contract["trading_behavior_changed"] is False, "trading behavior changed")
for marker in [
    "negative_suite_status = deterministic_fail_closed_forbidden_execution",
    "forbidden true flags -> fail_closed_forbidden_execution_boundary",
    "forbidden live trading claims -> fail_closed_forbidden_live_trading_claim",
    "source artifact schema validator coverage = true",
    "release gate coverage = true",
    "dashboard ingestion coverage = true",
    "admin workbench ingestion coverage = true",
    "read model ingestion coverage = true",
    "missing scoped approval -> fail_closed_missing_scoped_approval",
    "missing risk gate -> fail_closed_missing_risk_gate",
    "missing audit gate -> fail_closed_missing_audit_gate",
    "missing rollback readiness -> fail_closed_missing_rollback_path",
    "missing telemetry/SLO gate -> fail_closed_missing_telemetry_slo_gate",
    "stale config -> fail_closed_stale_config",
    "adapter_send_allowed = false",
    "live_exchange_request_allowed = false",
    "retry_scheduler_enabled = false",
    "automatic_remediation_allowed = false",
    "dashboard_trading_controls_enabled = false",
    "backend_go_live_claim = false",
    "product_grade_trading_terminal_claim = false",
]:
    require_contains(md, marker, "contract markdown")
    require_contains(evidence, marker, "evidence")
for label, text in {"task": task, "evidence": evidence, "README": readme}.items():
    require_contains(text, "V310-008", label)
    require_contains(text, "v0.31.0", label)
PY

current_issue_json="$(gh_with_retry issue view "$CURRENT_ISSUE" --repo "$REPO" --json number,state,title)"
prev_issue_json="$(gh_with_retry issue view "$PREV_ISSUE" --repo "$REPO" --json number,state,title)"
CURRENT_ISSUE_JSON="$current_issue_json" PREV_ISSUE_JSON="$prev_issue_json" python3 <<'PY'
import json, os
current = json.loads(os.environ["CURRENT_ISSUE_JSON"])
prev = json.loads(os.environ["PREV_ISSUE_JSON"])
if current.get("number") != 1014:
    raise SystemExit("current issue mismatch")
if prev.get("number") != 1013 or prev.get("state") != "CLOSED":
    raise SystemExit("V310-007 must be closed")
print("v31_forbidden_execution_negative_live current_issue_state=%s v310_007_state=CLOSED forbidden_flags_fail_closed=true ingestion_surfaces=5" % current.get("state"))
PY
echo "v31_forbidden_execution_negative=pass task=V310-008 issue=1014 backend_go_live=false trading_claim=false"
