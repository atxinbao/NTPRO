#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V320_RELEASE_REPO:-atxinbao/NTPRO}"
RELEASE_VERSION="${NTPRO_V320_RELEASE_VERSION:-v0.32.0}"
RELEASE_TAG="${NTPRO_V320_RELEASE_TAG:-ntpro-rust-only-v0.32.0}"
RELEASE_NAME="${NTPRO_V320_RELEASE_NAME:-NTPRO Rust-only v0.32.0}"
BASE_RELEASE_TAG="${NTPRO_V320_BASE_RELEASE_TAG:-ntpro-rust-only-v0.31.1}"
MANIFEST_PATH="${NTPRO_V320_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_32_0_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V320_RELEASE_NOTES:-docs/rust-cutover/release/v0_32_0_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V320_READINESS_REPORT:-docs/rust-cutover/release/v0_32_0_readiness_report.md}"
RELEASE_CLOSEOUT_PATH="${NTPRO_V320_CLOSEOUT_EVIDENCE:-docs/rust-cutover/release/v0_32_0_release_closeout_evidence.md}"
CURRENT_ISSUE="${NTPRO_V320_CURRENT_ISSUE:-1051}"
MILESTONE_TITLE="${NTPRO_V320_MILESTONE_TITLE:-v0.32.0}"

fail() { echo "v32 release gate failed: $*" >&2; exit 1; }
require_file() { [[ -f "$1" ]] || fail "missing required file: $1"; }
require_contains() {
  local path="$1"
  local marker="$2"
  grep -F -- "$marker" "$path" >/dev/null || fail "missing marker in $path: $marker"
}
require_not_contains() {
  local path="$1"
  local marker="$2"
  if grep -F -- "$marker" "$path" >/dev/null; then
    fail "forbidden marker in $path: $marker"
  fi
}
gh_with_retry() {
  local attempt=1
  while true; do
    if GODEBUG=http2client=0 gh "$@"; then return 0; fi
    (( attempt >= 4 )) && return 1
    sleep "$((attempt * 2))"
    attempt=$((attempt + 1))
  done
}

for path in \
  "$MANIFEST_PATH" "$RELEASE_NOTES_PATH" "$READINESS_REPORT_PATH" "$RELEASE_CLOSEOUT_PATH" \
  docs/rust-cutover/release/v0_31_1_release_manifest.json \
  docs/rust-cutover/release/README.md \
  scripts/ai/check_release_surface_current.sh \
  scripts/ai/check_github_release_published.sh \
  scripts/ai/publish_ntpro_release_after_gate.sh \
  scripts/ai/verify_v32_intake_gate.sh \
  scripts/ai/verify_v32_backend_closeout_boundary_contract.sh \
  scripts/ai/verify_v32_owner_operator_change_window_closeout.sh \
  scripts/ai/verify_v32_risk_audit_go_no_go_closeout.sh \
  scripts/ai/verify_v32_config_venue_credential_provenance.sh \
  scripts/ai/verify_v32_canary_rollback_dr_closeout.sh \
  scripts/ai/verify_v32_telemetry_slo_incident_closeout.sh \
  scripts/ai/verify_v32_backend_enablement_read_model_admin_bridge_closeout.sh \
  scripts/ai/verify_v32_fail_closed_negative_tests.sh \
  scripts/ai/verify_v32_release_gates.sh \
  scripts/ai/verify_v32_strict_provenance.sh; do
  require_file "$path"
done

for task_id in V320-000 V320-001 V320-002 V320-003 V320-004 V320-005 V320-006 V320-007 V320-008 V320-009; do
  require_file "docs/rust-cutover/evidence/${task_id}.md"
  require_contains "docs/rust-cutover/evidence/${task_id}.md" "$task_id"
  require_file "docs/rust-cutover/tasks/${task_id}.md"
  require_contains "docs/rust-cutover/tasks/${task_id}.md" "$task_id"
done

for marker in \
  "Status: RELEASED" \
  "Tag: \`$RELEASE_TAG\`" \
  "Release name: \`$RELEASE_NAME\`" \
  "Release URL: \`https://github.com/atxinbao/NTPRO/releases/tag/$RELEASE_TAG\`" \
  "Base release: \`$BASE_RELEASE_TAG\`" \
  "v0.32.0 publishes the Backend Production Closeout version" \
  "V320-000" \
  "V320-009" \
  "V320 final release scope issue count = 10" \
  "V320 final release scope evidence count = 10" \
  "V320 exact milestone issue set = #1042-#1051" \
  "V320 registered corrective-scope exception count = 0" \
  "v32 release gates = required" \
  "v32 strict provenance = required" \
  "release surface current guard = required" \
  "release publication guard = required" \
  "release publish after gate = required" \
  "publish after hosted gate success = required" \
  "publication evidence strategy = source_tree_plus_github_remote" \
  "generated publication evidence sole proof allowed = false" \
  "v0.33.0 inheritance = separately scoped only" \
  "scripts/ai/verify_v32_release_gates.sh" \
  "scripts/ai/verify_v32_strict_provenance.sh"; do
  require_contains "$RELEASE_NOTES_PATH" "$marker"
done

for marker in \
  "V320-000 evidence" \
  "V320-009 evidence" \
  "#1051 V320-009 = must be closed before v0.32.0 tag gate is accepted" \
  "V320 final release scope issue count = 10" \
  "V320 final release scope evidence count = 10" \
  "V320 exact milestone issue set = #1042-#1051" \
  "strict provenance manifest = target/ntpro-v320/v0_32_0_strict_release_manifest.json" \
  "v0.33.0 inheritance = separately scoped only"; do
  require_contains "$READINESS_REPORT_PATH" "$marker"
done

for marker in \
  "release tag = ntpro-rust-only-v0.32.0" \
  "hosted release gate required = true" \
  "published after hosted gate = required" \
  "release body must match tracked notes = docs/rust-cutover/release/v0_32_0_release_notes.md" \
  "remote reconstruction required = true" \
  "generated publication evidence sole proof allowed = false"; do
  require_contains "$RELEASE_CLOSEOUT_PATH" "$marker"
done

for path in "$RELEASE_NOTES_PATH" "$READINESS_REPORT_PATH" "$MANIFEST_PATH" "$RELEASE_CLOSEOUT_PATH"; do
  for marker in \
    "new_submit_capability = true" \
    "production_order_submission_allowed = true" \
    "production_order_mutation_allowed = true" \
    "execution_adapter_call_allowed = true" \
    "adapter_send_allowed = true" \
    "live_exchange_request_allowed = true" \
    "network_attempted = true" \
    "retry_scheduler_enabled = true" \
    "automatic_remediation_allowed = true" \
    "dashboard_trading_controls_enabled = true" \
    "admin_workbench_trading_controls_enabled = true" \
    "trader_terminal_order_ticket_enabled = true" \
    "manual_operation_submit_allowed = true" \
    "backend_go_live_claim = true" \
    "actual_backend_production_go_live_allowed = true" \
    "frontend_completion_claim = true" \
    "product_grade_trading_terminal_claim = true"; do
    require_not_contains "$path" "$marker"
  done
done

# The final V320-008 verifier recursively proves the V320 dependency chain.
# Running every stage here would repeat the same live GitHub checks many times.
scripts/ai/verify_v32_fail_closed_negative_tests.sh >/dev/null

NTPRO_CURRENT_RELEASE_VERSION="$RELEASE_VERSION" \
  NTPRO_CURRENT_RELEASE_TAG="$RELEASE_TAG" \
  NTPRO_CURRENT_RELEASE_NAME="$RELEASE_NAME" \
  NTPRO_POST_BASELINE_GOVERNANCE_TRACK="backend-freeze-governance" \
  NTPRO_NEXT_CAPABILITY_VERSION="v0.33.0+" \
  NTPRO_CURRENT_RELEASE_CAPABILITY="v0.32.0 Backend Production Closeout" \
  NTPRO_RELEASE_SURFACE_ALLOW_MISSING_TAG="${NTPRO_RELEASE_SURFACE_ALLOW_MISSING_TAG:-1}" \
  scripts/ai/check_release_surface_current.sh >/dev/null

NTPRO_CURRENT_RELEASE_VERSION="$RELEASE_VERSION" \
  NTPRO_CURRENT_RELEASE_TAG="$RELEASE_TAG" \
  NTPRO_CURRENT_RELEASE_NAME="$RELEASE_NAME" \
  NTPRO_RELEASE_PUBLICATION_ALLOW_OFFLINE=1 \
  NTPRO_RELEASE_PUBLICATION_PREPUBLISH_TAG_GATE="${NTPRO_RELEASE_GATE:-1}" \
  scripts/ai/check_github_release_published.sh >/dev/null

MANIFEST_PATH="$MANIFEST_PATH" python3 <<'PY'
import copy
import json
from pathlib import Path

manifest = json.loads(Path(__import__("os").environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))
expected = {
    "V320-000": 1042,
    "V320-001": 1043,
    "V320-002": 1044,
    "V320-003": 1045,
    "V320-004": 1046,
    "V320-005": 1047,
    "V320-006": 1048,
    "V320-007": 1049,
    "V320-008": 1050,
    "V320-009": 1051,
}
false_flags = [
    "new_submit_capability",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "cancel_order_allowed",
    "replace_order_allowed",
    "amend_order_allowed",
    "flatten_position_allowed",
    "execution_adapter_call_allowed",
    "adapter_send_allowed",
    "live_exchange_request_allowed",
    "network_attempted",
    "retry_scheduler_enabled",
    "automatic_remediation_allowed",
    "automatic_operation_action_allowed",
    "automatic_recovery_allowed",
    "dashboard_operation_controls_enabled",
    "dashboard_trading_controls_enabled",
    "admin_workbench_operation_controls_enabled",
    "admin_workbench_trading_controls_enabled",
    "trader_terminal_order_ticket_enabled",
    "manual_operation_submit_allowed",
    "backend_go_live_claim",
    "actual_backend_production_go_live_allowed",
    "frontend_completion_claim",
    "product_grade_trading_terminal_claim",
    "product_grade_live_trading_terminal_claim",
    "default_production_execution_allowed",
]

def require(condition, message):
    if not condition:
        raise SystemExit(message)

def validate(candidate):
    require(candidate.get("schema_version") == "ntpro.v320_backend_closeout_release_manifest.v1", "manifest schema mismatch")
    require(candidate.get("task_id") == "V320-009", "manifest task mismatch")
    require(candidate.get("product_version") == "v0.32.0", "product version mismatch")
    require(candidate.get("release_status") in {"release_gate_ready", "released"}, "release status mismatch")
    planned = candidate.get("planned_release") or {}
    require(planned.get("tag") == "ntpro-rust-only-v0.32.0", "planned release tag mismatch")
    evidence = candidate.get("v320_evidence") or []
    require(len(evidence) == 10, "V320 evidence count mismatch")
    for item in evidence:
        task_id = item.get("task_id")
        require(expected.get(task_id) == item.get("issue"), f"V320 issue mismatch: {task_id}")
        require(Path(item.get("path", "")).is_file(), f"missing V320 evidence: {item}")
    scope = candidate.get("release_scope") or {}
    require(scope.get("exact_milestone_issue_numbers") == list(expected.values()), "exact issue numbers mismatch")
    require(scope.get("exact_milestone_issue_set") == "#1042-#1051", "exact issue set mismatch")
    require(scope.get("final_release_scope_issue_count") == 10, "final issue count mismatch")
    require(scope.get("final_release_scope_evidence_count") == 10, "final evidence count mismatch")
    require(scope.get("backend_closeout_version_only") is True, "backend closeout only flag missing")
    requirements = candidate.get("post_publication_requirements") or {}
    require(requirements.get("all_v320_issues_closed_required") is True, "V320 closeout requirement missing")
    require(requirements.get("hosted_release_gate_success_required") is True, "hosted gate requirement missing")
    require(requirements.get("publication_after_hosted_gate_required") is True, "publication after gate requirement missing")
    require(requirements.get("generated_publication_evidence_sole_proof_allowed") is False, "generated-only proof must be false")
    next_tracks = candidate.get("next_tracks") or {}
    require(next_tracks.get("capability") == "v0.33.0", "next capability mismatch")
    require(next_tracks.get("capability_entry") == "separately_scoped_only", "next capability entry mismatch")
    for key in false_flags:
        require((candidate.get("boundary_flags") or {}).get(key) is False, f"boundary must remain false: {key}")

validate(manifest)

bad_boundary = copy.deepcopy(manifest)
bad_boundary["boundary_flags"]["adapter_send_allowed"] = True
try:
    validate(bad_boundary)
except SystemExit:
    pass
else:
    raise SystemExit("negative self-test unexpectedly allowed adapter_send_allowed")

bad_scope = copy.deepcopy(manifest)
bad_scope["release_scope"]["exact_milestone_issue_numbers"].append(9999)
try:
    validate(bad_scope)
except SystemExit:
    pass
else:
    raise SystemExit("negative self-test unexpectedly allowed issue-scope drift")
PY

if [[ "${NTPRO_RELEASE_GATE:-0}" == "1" ]]; then
  current_issue_json="$(gh_with_retry issue view "$CURRENT_ISSUE" --repo "$REPO" --json number,state,title)"
  milestone_json="$(gh_with_retry api "repos/$REPO/milestones?state=all" --jq ".[] | select(.title == \"$MILESTONE_TITLE\")")"
  issues_json="$(gh_with_retry issue list --repo "$REPO" --state all --milestone "$MILESTONE_TITLE" --limit 100 --json number,state,title)"
  CURRENT_ISSUE_JSON="$current_issue_json" MILESTONE_JSON="$milestone_json" ISSUES_JSON="$issues_json" python3 <<'PY'
import json
import os
current = json.loads(os.environ["CURRENT_ISSUE_JSON"])
milestone = json.loads(os.environ["MILESTONE_JSON"])
issues = json.loads(os.environ["ISSUES_JSON"])
expected = set(range(1042, 1052))
issue_map = {item["number"]: item["state"] for item in issues}
if current.get("number") != 1051 or current.get("state") != "CLOSED":
    raise SystemExit(f"V320 issue must be closed before tag gate: #1051 state={current.get('state')}")
if milestone.get("open_issues") != 0:
    raise SystemExit("v0.32.0 milestone must have zero open issues for release gate")
if set(issue_map) != expected:
    raise SystemExit(f"V320 milestone issue set mismatch: {sorted(issue_map)}")
for number in sorted(expected):
    if issue_map[number] != "CLOSED":
        raise SystemExit(f"V320 issue must be closed before tag gate: #{number} state={issue_map[number]}")
PY
else
  issues_json="$(gh_with_retry issue list --repo "$REPO" --state all --milestone "$MILESTONE_TITLE" --limit 100 --json number,state,title 2>/dev/null || printf '[]')"
  current_issue_json="$(gh_with_retry issue view "$CURRENT_ISSUE" --repo "$REPO" --json number,state,title 2>/dev/null || printf '{}')"
  ISSUES_JSON="$issues_json" CURRENT_ISSUE_JSON="$current_issue_json" python3 <<'PY'
import json
import os
issues = json.loads(os.environ["ISSUES_JSON"])
current = json.loads(os.environ["CURRENT_ISSUE_JSON"] or "{}")
expected = set(range(1042, 1052))
states = {item["number"]: item["state"] for item in issues if item["number"] in expected}
closed = sum(1 for value in states.values() if value == "CLOSED")
state = current.get("state", "UNKNOWN")
print(f"v32_issue_scope=pr_mode closed={closed}/10 current_issue_state={state}")
PY
fi

echo "v32_release_gates=pass release_tag=$RELEASE_TAG final_scope_issues=10 final_scope_evidence=10 backend_closeout=only negative_selftest=2"
