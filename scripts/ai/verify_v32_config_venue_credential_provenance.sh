#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V32_CONFIG_REPO:-atxinbao/NTPRO}"
CURRENT_ISSUE="${NTPRO_V32_CONFIG_CURRENT_ISSUE:-1046}"
PREV_ISSUE="${NTPRO_V32_CONFIG_PREV_ISSUE:-1045}"
V320_MILESTONE_TITLE="${NTPRO_V32_CONFIG_MILESTONE_TITLE:-v0.32.0}"
CONTRACT_JSON="${NTPRO_V32_CONFIG_JSON:-docs/rust-cutover/release/v0_32_0_config_venue_credential_provenance.json}"
CONTRACT_MD="${NTPRO_V32_CONFIG_MD:-docs/rust-cutover/release/v0_32_0_config_venue_credential_provenance.md}"
TASK_DOC="${NTPRO_V32_CONFIG_TASK:-docs/rust-cutover/tasks/V320-004.md}"
EVIDENCE_DOC="${NTPRO_V32_CONFIG_EVIDENCE:-docs/rust-cutover/evidence/V320-004.md}"

fail() { echo "v32 config venue credential provenance failed: $*" >&2; exit 1; }
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
  docs/rust-cutover/release/v0_32_0_risk_audit_go_no_go_closeout.json \
  docs/rust-cutover/tasks/V320-003.md docs/rust-cutover/evidence/V320-003.md \
  docs/rust-cutover/release/README.md scripts/ai/verify_v32_risk_audit_go_no_go_closeout.sh \
  scripts/ai/verify_v32_config_venue_credential_provenance.sh; do
  require_file "$path"
done

scripts/ai/verify_v32_risk_audit_go_no_go_closeout.sh >/dev/null

CONTRACT_JSON="$CONTRACT_JSON" CONTRACT_MD="$CONTRACT_MD" TASK_DOC="$TASK_DOC" EVIDENCE_DOC="$EVIDENCE_DOC" python3 <<'PY'
import json, os
from pathlib import Path

def require(condition, message):
    if not condition:
        raise SystemExit(message)

def require_contains(text, marker, label):
    require(marker in text, f"{label} missing marker: {marker}")

def classify(case):
    if case["config_state"] == "missing":
        return "fail_closed_missing_config"
    if case["environment"] == "mismatched":
        return "fail_closed_wrong_environment"
    if case["venue_readiness"] == "stale":
        return "fail_closed_stale_venue_readiness"
    if case["credential_scope"] == "mismatched":
        return "fail_closed_credential_scope_mismatch"
    if case["redaction"] == "missing":
        return "fail_closed_missing_redaction"
    if case["raw_secret_persisted"] is True:
        return "fail_closed_raw_secret_persisted"
    if case["unrestricted_payload_persisted"] is True:
        return "fail_closed_unrestricted_payload_persisted"
    return "config_venue_credential_ready_no_adapter_send"

contract = json.loads(Path(os.environ["CONTRACT_JSON"]).read_text())
md = Path(os.environ["CONTRACT_MD"]).read_text()
task = Path(os.environ["TASK_DOC"]).read_text()
evidence = Path(os.environ["EVIDENCE_DOC"]).read_text()
readme = Path("docs/rust-cutover/release/README.md").read_text()

require(contract["schema_version"] == "ntpro.v320_config_venue_credential_provenance.v1", "schema mismatch")
require(contract["task_id"] == "V320-004", "task mismatch")
require(contract["github_issue"] == 1046, "issue mismatch")
require(contract["milestone"] == "v0.32.0", "milestone mismatch")
require(contract["gate_status"] == "production_config_venue_credential_environment_provenance_required_no_adapter_send", "gate status mismatch")
deps = {(item["task_id"], item["github_issue"], item["required_status"]) for item in contract["depends_on"]}
require(("V320-003", 1045, "closed") in deps, "V320-003 dependency missing")
config = contract["production_config_evidence"]
venue = contract["venue_readiness_evidence"]
credential = contract["credential_provenance"]
for key in ["required", "source_provenance_required", "environment_provenance_required", "redaction_required", "sensitive_values_forbidden", "config_digest_required", "release_tag_consistency_required", "freshness_required"]:
    require(config[key] is True, f"config flag must be true: {key}")
for key in ["required", "source_provenance_required", "venue_binding_required", "environment_binding_required", "venue_release_consistency_required", "credential_material_redacted_required"]:
    require(venue[key] is True, f"venue flag must be true: {key}")
for key in ["required", "credential_scope_digest_required", "raw_secret_forbidden", "raw_signature_forbidden", "raw_credential_forbidden", "unrestricted_payload_forbidden"]:
    require(credential[key] is True, f"credential flag must be true: {key}")
require(venue["endpoint_class"] == "read_only_or_probe_plan", "endpoint class mismatch")
require(venue["adapter_send_authorized"] is False, "adapter send must be false")
require(venue["live_exchange_request_authorized"] is False, "live exchange must be false")
for field in ["environment_id", "venue_id", "account_scope", "strategy_scope", "config_digest", "redaction_digest", "source_provenance", "environment_provenance"]:
    require(field in set(config["required_fields"]), f"config field missing: {field}")
for field in ["credential_scope_digest", "credential_redaction_digest", "environment_id", "venue_id", "account_scope", "strategy_scope"]:
    require(field in set(credential["required_fields"]), f"credential field missing: {field}")
for case in contract["decision_cases"]:
    require(classify(case) == case["expected_status"], f"case mismatch {case['case_id']}")
for key, value in contract["runtime_boundary_flags"].items():
    require(value is False, f"runtime flag must be false: {key}")
require(contract["runtime_behavior_changed"] is False, "runtime behavior changed")
require(contract["trading_behavior_changed"] is False, "trading behavior changed")
for marker in [
    "gate_status = production_config_venue_credential_environment_provenance_required_no_adapter_send",
    "depends_on_issue_1045 = closed",
    "production config evidence required = true",
    "venue readiness evidence required = true",
    "credential provenance required = true",
    "environment provenance required = true",
    "redaction required = true",
    "sensitive values forbidden = true",
    "raw secret forbidden = true",
    "raw signature forbidden = true",
    "raw credential forbidden = true",
    "unrestricted payload forbidden = true",
    "credential scope digest required = true",
    "missing config -> fail_closed_missing_config",
    "wrong environment -> fail_closed_wrong_environment",
    "stale venue readiness -> fail_closed_stale_venue_readiness",
    "credential scope mismatch -> fail_closed_credential_scope_mismatch",
    "raw secret persisted -> fail_closed_raw_secret_persisted",
    "unrestricted payload persisted -> fail_closed_unrestricted_payload_persisted",
    "adapter_send_allowed = false",
    "live_exchange_request_allowed = false",
    "backend_go_live_claim = false",
]:
    require_contains(md, marker, "contract markdown")
    require_contains(evidence, marker, "evidence")
for label, text in {"task": task, "evidence": evidence, "README": readme}.items():
    require_contains(text, "V320-004", label)
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
if current.get("number") != 1046:
    raise SystemExit("current issue mismatch")
if (current.get("milestone") or {}).get("title") != "v0.32.0":
    raise SystemExit("current milestone mismatch")
if prev.get("number") != 1045 or prev.get("state") != "CLOSED":
    raise SystemExit("V320-003 must be closed")
issue_map = {item["number"]: item for item in issues}
missing = [number for number in range(1042, 1052) if number not in issue_map]
if missing:
    raise SystemExit(f"missing V320 issues: {missing}")
print("v32_config_venue_credential_live previous_issue=1045:CLOSED current_issue=1046:%s v320_issues=10 redaction_required=true venue_readiness_required=true" % current.get("state"))
PY
echo "v32_config_venue_credential_provenance=pass task=V320-004 issue=1046 adapter_send=false live_exchange=false credential_scope_bound=true"
