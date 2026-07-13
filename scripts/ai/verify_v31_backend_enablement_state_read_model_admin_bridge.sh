#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V31_READ_MODEL_REPO:-atxinbao/NTPRO}"
CURRENT_ISSUE="${NTPRO_V31_READ_MODEL_CURRENT_ISSUE:-1013}"
PREV_ISSUE="${NTPRO_V31_READ_MODEL_PREV_ISSUE:-1012}"
CONTRACT_JSON="${NTPRO_V31_READ_MODEL_JSON:-docs/rust-cutover/release/v0_31_0_backend_enablement_state_read_model_admin_bridge.json}"
CONTRACT_MD="${NTPRO_V31_READ_MODEL_MD:-docs/rust-cutover/release/v0_31_0_backend_enablement_state_read_model_admin_bridge.md}"
TASK_DOC="${NTPRO_V31_READ_MODEL_TASK:-docs/rust-cutover/tasks/V310-007.md}"
EVIDENCE_DOC="${NTPRO_V31_READ_MODEL_EVIDENCE:-docs/rust-cutover/evidence/V310-007.md}"

fail() { echo "v31 backend enablement state read model admin bridge failed: $*" >&2; exit 1; }
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
  docs/rust-cutover/release/v0_31_0_telemetry_slo_incident_enablement_gate.json \
  docs/rust-cutover/tasks/V310-006.md docs/rust-cutover/evidence/V310-006.md \
  docs/rust-cutover/release/README.md scripts/ai/verify_v31_telemetry_slo_incident_enablement_gate.sh \
  scripts/ai/verify_v31_backend_enablement_state_read_model_admin_bridge.sh; do
  require_file "$path"
done

scripts/ai/verify_v31_telemetry_slo_incident_enablement_gate.sh >/dev/null

CONTRACT_JSON="$CONTRACT_JSON" CONTRACT_MD="$CONTRACT_MD" TASK_DOC="$TASK_DOC" EVIDENCE_DOC="$EVIDENCE_DOC" python3 <<'PY'
import json, os
from pathlib import Path

def require(condition, message):
    if not condition:
        raise SystemExit(message)

def require_contains(text, marker, label):
    require(marker in text, f"{label} missing marker: {marker}")

def classify(case):
    if case["artifact"] == "missing":
        return "fail_closed_missing_state_artifact"
    if case["artifact"] == "malformed":
        return "fail_closed_malformed_state_artifact"
    if case["state_freshness"] == "stale":
        return "fail_closed_stale_state_artifact"
    if case["controls"] == "enabled":
        return "fail_closed_forbidden_control"
    return "read_only_enablement_state_visible_no_mutation"

contract = json.loads(Path(os.environ["CONTRACT_JSON"]).read_text())
md = Path(os.environ["CONTRACT_MD"]).read_text()
task = Path(os.environ["TASK_DOC"]).read_text()
evidence = Path(os.environ["EVIDENCE_DOC"]).read_text()
readme = Path("docs/rust-cutover/release/README.md").read_text()

require(contract["schema_version"] == "ntpro.v310_backend_enablement_state_read_model_admin_bridge.v1", "schema mismatch")
require(contract["task_id"] == "V310-007", "task mismatch")
require(contract["github_issue"] == 1013, "issue mismatch")
require(contract["read_model_status"] == "read_only_enablement_state_visible_no_mutation", "read model status mismatch")
deps = {(item["task_id"], item["github_issue"], item["required_status"]) for item in contract["depends_on"]}
for number, task_id in [(1007, "V310-001"), (1008, "V310-002"), (1009, "V310-003"), (1010, "V310-004"), (1011, "V310-005"), (1012, "V310-006")]:
    require((task_id, number, "closed") in deps, f"dependency missing: {task_id}")
expected_components = {
    "enablement_state",
    "approval_state",
    "risk_audit_state",
    "canary_state",
    "rollback_state",
    "telemetry_slo_state",
    "boundary_flags",
}
require(set(contract["required_components"]) == expected_components, "required components mismatch")
for key, value in contract["component_requirements"].items():
    require(value is True, f"component requirement must be true: {key}")
bridge = contract["admin_bridge"]
for key in ["artifact_ingestion_required", "read_only", "rendering_evidence_required", "operator_visibility_only"]:
    require(bridge[key] is True, f"bridge flag must be true: {key}")
require(bridge["mutation_controls_allowed"] is False, "mutation controls must be false")
for control in ["submit", "cancel", "retry", "replace", "amend", "flatten", "order_ticket"]:
    require(control in contract["disabled_controls"], f"missing disabled control: {control}")
for state in contract["candidate_states"]:
    require(state["production_state_mutation_allowed"] is False, "production mutation must be false")
for case in contract["decision_cases"]:
    require(classify(case) == case["expected_status"], f"case mismatch {case['case_id']}")
flags = contract["runtime_boundary_flags"]
require(flags["admin_bridge_read_only"] is True, "admin bridge read-only must be true")
for key, value in flags.items():
    if key == "admin_bridge_read_only":
        continue
    require(value is False, f"runtime flag must be false: {key}")
require(contract["runtime_behavior_changed"] is False, "runtime behavior changed")
require(contract["trading_behavior_changed"] is False, "trading behavior changed")
for marker in [
    "read_model_status = read_only_enablement_state_visible_no_mutation",
    "source provenance required = true",
    "lineage required = true",
    "freshness required = true",
    "redaction required = true",
    "artifact ingestion required = true",
    "admin bridge read-only = true",
    "mutation controls allowed = false",
    "missing artifact -> fail_closed_missing_state_artifact",
    "malformed artifact -> fail_closed_malformed_state_artifact",
    "stale artifact -> fail_closed_stale_state_artifact",
    "forbidden control -> fail_closed_forbidden_control",
    "submit control disabled = true",
    "cancel control disabled = true",
    "retry control disabled = true",
    "replace control disabled = true",
    "amend control disabled = true",
    "flatten control disabled = true",
    "order ticket disabled = true",
    "admin_workbench_trading_controls_enabled = false",
    "backend_go_live_claim = false",
]:
    require_contains(md, marker, "contract markdown")
    require_contains(evidence, marker, "evidence")
for label, text in {"task": task, "evidence": evidence, "README": readme}.items():
    require_contains(text, "V310-007", label)
    require_contains(text, "v0.31.0", label)
PY

current_issue_json="$(gh_with_retry issue view "$CURRENT_ISSUE" --repo "$REPO" --json number,state,title)"
prev_issue_json="$(gh_with_retry issue view "$PREV_ISSUE" --repo "$REPO" --json number,state,title)"
CURRENT_ISSUE_JSON="$current_issue_json" PREV_ISSUE_JSON="$prev_issue_json" python3 <<'PY'
import json, os
current = json.loads(os.environ["CURRENT_ISSUE_JSON"])
prev = json.loads(os.environ["PREV_ISSUE_JSON"])
if current.get("number") != 1013:
    raise SystemExit("current issue mismatch")
if prev.get("number") != 1012 or prev.get("state") != "CLOSED":
    raise SystemExit("V310-006 must be closed")
print("v31_enablement_state_read_model_live current_issue_state=%s v310_006_state=CLOSED read_only=true controls_disabled=true" % current.get("state"))
PY
echo "v31_enablement_state_read_model=pass task=V310-007 issue=1013 mutation_controls=false trading_controls=false"
