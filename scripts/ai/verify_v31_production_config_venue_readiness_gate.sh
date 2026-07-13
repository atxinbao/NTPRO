#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V31_CONFIG_VENUE_REPO:-atxinbao/NTPRO}"
CURRENT_ISSUE="${NTPRO_V31_CONFIG_VENUE_CURRENT_ISSUE:-1011}"
PREV_ISSUE="${NTPRO_V31_CONFIG_VENUE_PREV_ISSUE:-1010}"
CONTRACT_JSON="${NTPRO_V31_CONFIG_VENUE_JSON:-docs/rust-cutover/release/v0_31_0_production_config_venue_readiness_provenance_gate.json}"
CONTRACT_MD="${NTPRO_V31_CONFIG_VENUE_MD:-docs/rust-cutover/release/v0_31_0_production_config_venue_readiness_provenance_gate.md}"
TASK_DOC="${NTPRO_V31_CONFIG_VENUE_TASK:-docs/rust-cutover/tasks/V310-005.md}"
EVIDENCE_DOC="${NTPRO_V31_CONFIG_VENUE_EVIDENCE:-docs/rust-cutover/evidence/V310-005.md}"

fail() { echo "v31 production config venue readiness gate failed: $*" >&2; exit 1; }
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
  docs/rust-cutover/release/v0_31_0_canary_rollback_dr_execution_boundary.json \
  docs/rust-cutover/tasks/V310-004.md docs/rust-cutover/evidence/V310-004.md \
  docs/rust-cutover/release/README.md scripts/ai/verify_v31_canary_rollback_dr_execution_boundary.sh \
  scripts/ai/verify_v31_production_config_venue_readiness_gate.sh; do
  require_file "$path"
done

scripts/ai/verify_v31_canary_rollback_dr_execution_boundary.sh >/dev/null

CONTRACT_JSON="$CONTRACT_JSON" CONTRACT_MD="$CONTRACT_MD" TASK_DOC="$TASK_DOC" EVIDENCE_DOC="$EVIDENCE_DOC" python3 <<'PY'
import json, os
from pathlib import Path

def require(condition, message):
    if not condition:
        raise SystemExit(message)

def require_contains(text, marker, label):
    require(marker in text, f"{label} missing marker: {marker}")

def classify(case):
    if case["config_freshness"] == "stale":
        return "fail_closed_stale_config"
    if case["venue_binding"] == "mismatched":
        return "fail_closed_mismatched_venue"
    if case["redaction"] == "missing":
        return "fail_closed_missing_redaction"
    if case["environment_source"] == "unproven":
        return "fail_closed_unproven_environment_source"
    return "config_venue_ready_no_adapter_send"

contract = json.loads(Path(os.environ["CONTRACT_JSON"]).read_text())
md = Path(os.environ["CONTRACT_MD"]).read_text()
task = Path(os.environ["TASK_DOC"]).read_text()
evidence = Path(os.environ["EVIDENCE_DOC"]).read_text()
readme = Path("docs/rust-cutover/release/README.md").read_text()

require(contract["schema_version"] == "ntpro.v310_config_venue_readiness_gate.v1", "schema mismatch")
require(contract["task_id"] == "V310-005", "task mismatch")
require(contract["github_issue"] == 1011, "issue mismatch")
require(contract["gate_status"] == "production_config_venue_readiness_required_no_adapter_send", "gate status mismatch")
deps = {(item["task_id"], item["github_issue"], item["required_status"]) for item in contract["depends_on"]}
require(("V310-003", 1009, "closed") in deps, "V310-003 dependency missing")
require(("V310-004", 1010, "closed") in deps, "V310-004 dependency missing")
config = contract["production_config_evidence"]
venue = contract["venue_readiness_evidence"]
for key in [
    "required",
    "source_provenance_required",
    "environment_provenance_required",
    "redaction_required",
    "sensitive_values_forbidden",
    "release_tag_consistency_required",
    "config_digest_required",
    "freshness_required",
]:
    require(config[key] is True, f"config flag must be true: {key}")
for key in [
    "required",
    "source_provenance_required",
    "venue_binding_required",
    "environment_binding_required",
    "venue_release_consistency_required",
    "credential_material_redacted_required",
]:
    require(venue[key] is True, f"venue flag must be true: {key}")
require(venue["endpoint_class"] == "read_only_or_probe_plan", "endpoint class mismatch")
require(venue["adapter_send_authorized"] is False, "adapter send must be false")
require(venue["live_exchange_request_authorized"] is False, "live exchange must be false")
for case in contract["decision_cases"]:
    require(classify(case) == case["expected_status"], f"case mismatch {case['case_id']}")
for key, value in contract["runtime_boundary_flags"].items():
    require(value is False, f"runtime flag must be false: {key}")
require(contract["runtime_behavior_changed"] is False, "runtime behavior changed")
require(contract["trading_behavior_changed"] is False, "trading behavior changed")
for marker in [
    "gate_status = production_config_venue_readiness_required_no_adapter_send",
    "production config evidence required = true",
    "venue readiness evidence required = true",
    "environment provenance required = true",
    "redaction required = true",
    "sensitive values forbidden = true",
    "stale config -> fail_closed_stale_config",
    "mismatched venue -> fail_closed_mismatched_venue",
    "missing redaction -> fail_closed_missing_redaction",
    "unproven environment source -> fail_closed_unproven_environment_source",
    "adapter_send_allowed = false",
    "live_exchange_request_allowed = false",
    "backend_go_live_claim = false",
]:
    require_contains(md, marker, "contract markdown")
    require_contains(evidence, marker, "evidence")
for label, text in {"task": task, "evidence": evidence, "README": readme}.items():
    require_contains(text, "V310-005", label)
    require_contains(text, "v0.31.0", label)
PY

current_issue_json="$(gh_with_retry issue view "$CURRENT_ISSUE" --repo "$REPO" --json number,state,title)"
prev_issue_json="$(gh_with_retry issue view "$PREV_ISSUE" --repo "$REPO" --json number,state,title)"
CURRENT_ISSUE_JSON="$current_issue_json" PREV_ISSUE_JSON="$prev_issue_json" python3 <<'PY'
import json, os
current = json.loads(os.environ["CURRENT_ISSUE_JSON"])
prev = json.loads(os.environ["PREV_ISSUE_JSON"])
if current.get("number") != 1011:
    raise SystemExit("current issue mismatch")
if prev.get("number") != 1010 or prev.get("state") != "CLOSED":
    raise SystemExit("V310-004 must be closed")
print("v31_config_venue_gate_live current_issue_state=%s v310_004_state=CLOSED redaction_required=true venue_readiness_required=true" % current.get("state"))
PY
echo "v31_config_venue_gate=pass task=V310-005 issue=1011 adapter_send=false live_exchange=false"
