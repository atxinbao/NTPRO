#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V300_RELEASE_REPO:-atxinbao/NTPRO}"
RELEASE_VERSION="${NTPRO_V300_RELEASE_VERSION:-v0.30.0}"
RELEASE_TAG="${NTPRO_V300_RELEASE_TAG:-ntpro-rust-only-v0.30.0}"
RELEASE_NAME="${NTPRO_V300_RELEASE_NAME:-NTPRO Rust-only v0.30.0}"
BASE_RELEASE_TAG="${NTPRO_V300_BASE_RELEASE_TAG:-ntpro-rust-only-v0.29.1}"
MANIFEST_PATH="${NTPRO_V300_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_30_0_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V300_RELEASE_NOTES:-docs/rust-cutover/release/v0_30_0_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V300_READINESS_REPORT:-docs/rust-cutover/release/v0_30_0_readiness_report.md}"
CLOSEOUT_PATH="${NTPRO_V300_CLOSEOUT:-docs/rust-cutover/release/v0_30_0_release_closeout_evidence.md}"
V31_HANDOFF_PATH="${NTPRO_V300_V31_HANDOFF:-docs/rust-cutover/release/v0_30_0_v31_production_enablement_handoff.md}"
V31_HANDOFF_JSON="${NTPRO_V300_V31_HANDOFF_JSON:-docs/rust-cutover/release/v0_30_0_v31_production_enablement_handoff.json}"
CURRENT_ISSUE="${NTPRO_V300_CURRENT_ISSUE:-980}"
MILESTONE_NUMBER="${NTPRO_V300_MILESTONE_NUMBER:-26}"
MILESTONE_TITLE="${NTPRO_V300_MILESTONE_TITLE:-v0.30.0}"

fail() {
  echo "v30 release gate failed: $*" >&2
  exit 1
}

require_file() {
  local path="$1"
  [[ -f "$path" ]] || fail "missing required file: $path"
}

require_contains() {
  local path="$1"
  local marker="$2"
  if ! grep -F -- "$marker" "$path" >/dev/null; then
    fail "missing marker in $path: $marker"
  fi
}

require_not_contains() {
  local path="$1"
  local marker="$2"
  if grep -F -- "$marker" "$path" >/dev/null; then
    fail "forbidden marker in $path: $marker"
  fi
}

require_release_status_marker() {
  local path="$1"
  if grep -F -- "Status: RELEASE GATE READY" "$path" >/dev/null; then
    return 0
  fi
  if grep -F -- "Status: RELEASED" "$path" >/dev/null; then
    return 0
  fi
  fail "missing release status marker in $path"
}

gh_with_retry() {
  local attempt=1
  local max_attempts=4
  while true; do
    if GODEBUG=http2client=0 gh "$@"; then
      return 0
    fi
    if (( attempt >= max_attempts )); then
      return 1
    fi
    sleep "$((attempt * 2))"
    attempt=$((attempt + 1))
  done
}

required_files=(
  "$MANIFEST_PATH"
  "$RELEASE_NOTES_PATH"
  "$READINESS_REPORT_PATH"
  "$CLOSEOUT_PATH"
  "$V31_HANDOFF_PATH"
  "$V31_HANDOFF_JSON"
  docs/rust-cutover/release/v0_29_1_release_manifest.json
  docs/rust-cutover/release/v0_29_1_release_closeout_evidence.md
  docs/rust-cutover/release/README.md
  scripts/ai/check_github_release_published.sh
  scripts/ai/check_release_surface_current.sh
  scripts/ai/publish_ntpro_release_after_gate.sh
  scripts/ai/verify_v30_release_gates.sh
  scripts/ai/verify_v30_strict_provenance.sh
  .github/workflows/release-tag.yml
)

v300_scripts=(
  scripts/ai/verify_v30_backend_go_live_candidate_boundary_contract.sh
  scripts/ai/verify_v30_production_deployment_plan_environment_readiness.sh
  scripts/ai/verify_v30_runtime_enablement_boundary_controlled_feature_flags.sh
  scripts/ai/verify_v30_operator_approval_freeze_change_window_lifecycle.sh
  scripts/ai/verify_v30_canary_execution_preflight_no_default_execution_gate.sh
  scripts/ai/verify_v30_rollback_disaster_recovery_execution_boundary.sh
  scripts/ai/verify_v30_production_config_provenance_venue_connectivity_readiness.sh
  scripts/ai/verify_v30_telemetry_slo_gate_incident_freeze_integration.sh
  scripts/ai/verify_v30_audit_retention_evidence_export_readiness.sh
  scripts/ai/verify_v30_go_no_go_runbook_live_readiness_decision_record.sh
)

for path in "${required_files[@]}" "${v300_scripts[@]}"; do
  require_file "$path"
done

for task_id in V300-000 V300-001 V300-002 V300-003 V300-004 V300-005 V300-006 V300-007 V300-008 V300-009 V300-010 V300-011; do
  require_file "docs/rust-cutover/evidence/${task_id}.md"
  require_contains "docs/rust-cutover/evidence/${task_id}.md" "$task_id"
  require_file "docs/rust-cutover/tasks/${task_id}.md"
  require_contains "docs/rust-cutover/tasks/${task_id}.md" "$task_id"
done

require_release_status_marker "$RELEASE_NOTES_PATH"
for marker in \
  "Tag: \`$RELEASE_TAG\`" \
  "Release name: \`$RELEASE_NAME\`" \
  "Release URL: \`https://github.com/atxinbao/NTPRO/releases/tag/$RELEASE_TAG\`" \
  "Base release: \`$BASE_RELEASE_TAG\`" \
  "v0.30.0 publishes the Backend Production Go-Live Candidate Foundation" \
  "V300-000" \
  "V300-011" \
  "V300 final release scope issue count = 12" \
  "V300 final release scope evidence count = 12" \
  "V300 exact milestone issue set = #969-#980" \
  "V300 registered corrective-scope exception count = 0" \
  "v30 release gates = required" \
  "v30 strict provenance = required" \
  "v31 production enablement track = hard-blocked until v0.30.0 release gate passes" \
  "publication evidence strategy = source_tree_plus_github_remote" \
  "generated publication evidence sole proof allowed = false" \
  "scripts/ai/verify_v30_release_gates.sh" \
  "scripts/ai/verify_v30_strict_provenance.sh" \
  "scripts/ai/publish_ntpro_release_after_gate.sh"; do
  require_contains "$RELEASE_NOTES_PATH" "$marker"
done

require_release_status_marker "$READINESS_REPORT_PATH"
for marker in \
  "V300-000 evidence" \
  "V300-011 evidence" \
  "#980 V300-011 = must be closed before v0.30.0 tag gate is accepted" \
  "V300 final release scope issue count = 12" \
  "V300 final release scope evidence count = 12" \
  "V300 exact milestone issue set = #969-#980" \
  "v31 production enablement = hard-blocked until v0.30.0 release evidence and explicit scoped approval" \
  "source-controlled closeout evidence = docs/rust-cutover/release/v0_30_0_release_closeout_evidence.md"; do
  require_contains "$READINESS_REPORT_PATH" "$marker"
done

for marker in \
  "handoff_status = hard_blocked_until_v30_release_evidence_and_explicit_scoped_approval" \
  "explicit_scoped_issue_required = true" \
  "owner_operator_approval_required = true" \
  "inherits_submit = false" \
  "inherits_mutation = false" \
  "inherits_adapter_send = false" \
  "inherits_live_exchange_request = false" \
  "inherits_automatic_remediation = false" \
  "inherits_trading_controls = false"; do
  require_contains "$V31_HANDOFF_PATH" "$marker"
done

for path in "$RELEASE_NOTES_PATH" "$READINESS_REPORT_PATH" "$MANIFEST_PATH" "$CLOSEOUT_PATH" "$V31_HANDOFF_PATH"; do
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
    "automatic_operation_action_allowed = true" \
    "dashboard_operation_controls_enabled = true" \
    "dashboard_trading_controls_enabled = true" \
    "admin_workbench_operation_controls_enabled = true" \
    "admin_workbench_trading_controls_enabled = true" \
    "trader_terminal_order_ticket_enabled = true" \
    "manual_operation_submit_allowed = true" \
    "backend_go_live_claim = true" \
    "actual_backend_production_go_live_allowed = true" \
    "product_grade_trading_terminal_claim = true"; do
    require_not_contains "$path" "$marker"
  done
done

for script in "${v300_scripts[@]}"; do
  "$script" >/dev/null
done

NTPRO_CURRENT_RELEASE_VERSION="$RELEASE_VERSION" \
  NTPRO_CURRENT_RELEASE_TAG="$RELEASE_TAG" \
  NTPRO_CURRENT_RELEASE_NAME="$RELEASE_NAME" \
  NTPRO_RELEASE_PUBLICATION_ALLOW_OFFLINE=1 \
  NTPRO_RELEASE_PUBLICATION_PREPUBLISH_TAG_GATE="${NTPRO_RELEASE_GATE:-0}" \
  scripts/ai/check_github_release_published.sh >/dev/null

MANIFEST_PATH="$MANIFEST_PATH" V31_HANDOFF_JSON="$V31_HANDOFF_JSON" python3 <<'PY'
import copy
import json
import os
from pathlib import Path

manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))
handoff = json.loads(Path(os.environ["V31_HANDOFF_JSON"]).read_text(encoding="utf-8"))
expected = {
    "V300-000": 969,
    "V300-001": 970,
    "V300-002": 971,
    "V300-003": 972,
    "V300-004": 973,
    "V300-005": 974,
    "V300-006": 975,
    "V300-007": 976,
    "V300-008": 977,
    "V300-009": 978,
    "V300-010": 979,
    "V300-011": 980,
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
    "dashboard_operation_controls_enabled",
    "dashboard_trading_controls_enabled",
    "admin_workbench_operation_controls_enabled",
    "admin_workbench_trading_controls_enabled",
    "trader_terminal_order_ticket_enabled",
    "manual_operation_submit_allowed",
    "backend_go_live_claim",
    "ambiguous_backend_go_live_claim",
    "actual_backend_production_go_live_allowed",
    "production_runtime_enablement_allowed",
    "product_grade_trading_terminal_claim",
    "product_grade_live_trading_terminal_claim",
    "default_production_execution_allowed",
    "adapter_send_attempted",
    "live_exchange_request_attempted",
    "automatic_remediation_attempted",
    "go_no_go_record_enables_execution",
    "decision_record_backend_go_live_allowed",
]

def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)

def validate_manifest(candidate: dict) -> None:
    require(candidate.get("schema_version") == "ntpro.v300_backend_go_live_candidate_release_manifest.v1", "manifest schema mismatch")
    require(candidate.get("task_id") == "V300-011", "manifest task mismatch")
    require(candidate.get("product_version") == "v0.30.0", "manifest product version mismatch")
    release_status = candidate.get("release_status")
    require(release_status in {"release_gate_ready", "released"}, "manifest release status mismatch")
    if release_status == "released":
        post_closeout = candidate.get("post_release_closeout") or {}
        require(post_closeout.get("closeout_evidence_path") == "docs/rust-cutover/release/v0_30_0_release_closeout_evidence.md", "release closeout path mismatch")
        require((post_closeout.get("github_release") or {}).get("published_at") == "2026-07-11T05:37:06Z", "published release timestamp mismatch")
        require((post_closeout.get("hosted_release_gate") or {}).get("run_id") == 29139384219, "release gate run mismatch")
        require((post_closeout.get("milestone_closeout") or {}).get("exact_issue_set") == "#969-#980", "milestone closeout issue set mismatch")
    require(candidate.get("release_scope") == "backend_production_go_live_candidate_foundation_only", "release scope mismatch")
    planned = candidate.get("planned_release") or {}
    require(planned.get("tag") == "ntpro-rust-only-v0.30.0", "planned release tag mismatch")
    evidence = candidate.get("v300_evidence") or []
    require(len(evidence) == 12, "V300 evidence count mismatch")
    for item in evidence:
        task_id = item.get("task_id")
        require(expected.get(task_id) == item.get("issue"), f"V300 issue mismatch: {task_id}")
        require(Path(item.get("path", "")).is_file(), f"missing V300 evidence: {item}")
    scope = candidate.get("release_scope_facts") or {}
    require(scope.get("exact_milestone_issue_numbers") == list(expected.values()), "V300 exact issue numbers mismatch")
    require(scope.get("exact_milestone_issue_set") == "#969-#980", "V300 exact issue set mismatch")
    require(scope.get("final_release_scope_issue_count") == 12, "final issue count mismatch")
    require(scope.get("final_release_scope_evidence_count") == 12, "final evidence count mismatch")
    require(scope.get("registered_corrective_scope_exception_count") == 0, "corrective exception count mismatch")
    require(scope.get("actual_backend_production_go_live") is False, "actual backend go-live must be false")
    requirements = candidate.get("post_publication_requirements") or {}
    require(requirements.get("all_v300_issues_closed_required") is True, "V300 closeout requirement missing")
    require(requirements.get("hosted_release_gate_success_required") is True, "hosted gate requirement missing")
    require(requirements.get("publication_after_hosted_gate_required") is True, "publication after gate requirement missing")
    require(requirements.get("source_controlled_closeout_evidence_required") is True, "source closeout requirement missing")
    require(requirements.get("generated_publication_evidence_sole_proof_allowed") is False, "generated-only proof must be false")
    require(requirements.get("v31_handoff_fails_without_v300_release_evidence") is True, "v31 blocker missing")
    next_tracks = candidate.get("next_tracks") or {}
    require(next_tracks.get("capability") == "v0.31.0", "next capability mismatch")
    require(next_tracks.get("start_gate") == "hard_blocked_until_v30_release_evidence_and_explicit_scoped_approval", "next start gate mismatch")
    require(next_tracks.get("requires_scoped_issue") is True, "next track scoped issue missing")
    for key in false_flags:
        require((candidate.get("boundary_flags") or {}).get(key) is False, f"boundary must remain false: {key}")

def classify_handoff(candidate: dict) -> str:
    if candidate.get("schema_version") != "ntpro.v300.v31_production_enablement_handoff.v1":
        return "fail_closed_forbidden_boundary"
    if candidate.get("handoff_status") != "hard_blocked_until_v30_release_evidence_and_explicit_scoped_approval":
        return "fail_closed_forbidden_boundary"
    gates = {gate.get("gate_id"): gate for gate in candidate.get("future_enablement_gates") or []}
    if not gates.get("v30_release_evidence", {}).get("required"):
        return "fail_closed_missing_v30_release_evidence"
    if not gates.get("v30_1_release_evidence", {}).get("required"):
        return "fail_closed_missing_v30_1_release_evidence"
    if not gates.get("explicit_scoped_issue", {}).get("required"):
        return "fail_closed_missing_scoped_approval"
    if not gates.get("risk_gate", {}).get("required") or not gates.get("audit_gate", {}).get("required"):
        return "fail_closed_missing_risk_or_audit_gate"
    flags = candidate.get("boundary_flags") or {}
    inherited_keys = [
        "inherits_submit",
        "inherits_mutation",
        "inherits_adapter_send",
        "inherits_live_exchange_request",
        "inherits_automatic_remediation",
        "inherits_trading_controls",
    ]
    if any(flags.get(key) is not False for key in inherited_keys):
        return "fail_closed_inherited_execution"
    for key, value in flags.items():
        if value is not False:
            return "fail_closed_forbidden_boundary"
    return "v31_handoff_hard_blocked"

def merge(base, override):
    if isinstance(base, dict) and isinstance(override, dict):
        result = copy.deepcopy(base)
        for key, value in override.items():
            result[key] = merge(result.get(key), value)
        return result
    return copy.deepcopy(override)

def apply_gate_overrides(candidate: dict, overrides: dict) -> dict:
    result = copy.deepcopy(candidate)
    gates = result["future_enablement_gates"]
    for gate_id, override in overrides.items():
        for index, gate in enumerate(gates):
            if gate.get("gate_id") == gate_id:
                gates[index] = merge(gate, override)
                break
    return result

def validate_handoff(candidate: dict) -> None:
    require(candidate.get("next_capability_track") == "v0.31.0", "handoff next track mismatch")
    require(candidate.get("v31_start_gate_status") == "blocked_until_v301_release_evidence_published", "v31 start gate status mismatch")
    require(candidate.get("v31_start_gate_contract") == "docs/rust-cutover/release/v0_30_1_v31_start_gate.json", "v31 start gate contract mismatch")
    require("v30_1_release_closeout" in (candidate.get("required_future_inputs") or []), "v30.1 closeout input missing")
    require(len(candidate.get("required_future_inputs") or []) == 10, "future input count mismatch")
    require(classify_handoff(candidate) == "v31_handoff_hard_blocked", "handoff baseline mismatch")
    for case in candidate.get("readiness_cases") or []:
        scenario = copy.deepcopy(candidate)
        if case.get("gate_overrides"):
            scenario = apply_gate_overrides(scenario, case["gate_overrides"])
        if case.get("boundary_flags_override"):
            scenario["boundary_flags"] = merge(scenario["boundary_flags"], case["boundary_flags_override"])
        status = classify_handoff(scenario)
        require(status == case.get("expected_status"), f"handoff case {case.get('case_id')} expected {case.get('expected_status')} got {status}")

validate_manifest(manifest)
validate_handoff(handoff)

bad_boundary = copy.deepcopy(manifest)
bad_boundary["boundary_flags"]["adapter_send_allowed"] = True
try:
    validate_manifest(bad_boundary)
except SystemExit:
    pass
else:
    raise SystemExit("negative selftest failed: open boundary accepted")
PY

if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1 && [[ "${NTPRO_V300_SKIP_GH_SCOPE:-0}" != "1" ]]; then
  closed_count=0
  current_state=""
  for issue in {969..980}; do
    state="$(gh_with_retry issue view "$issue" --repo "$REPO" --json state --jq .state)"
    if [[ "$issue" == "$CURRENT_ISSUE" ]]; then
      current_state="$state"
    fi
    if [[ "$state" == "CLOSED" ]]; then
      closed_count=$((closed_count + 1))
    elif [[ "${NTPRO_RELEASE_GATE:-0}" == "1" || "$issue" != "$CURRENT_ISSUE" ]]; then
      fail "V300 issue must be closed before tag gate: #$issue state=$state"
    fi
  done
  if [[ "${NTPRO_RELEASE_GATE:-0}" == "1" && "$closed_count" != "12" ]]; then
    fail "V300 release gate requires all issues closed: closed=$closed_count/12"
  fi
  milestone_json="$(gh_with_retry api "repos/$REPO/milestones/$MILESTONE_NUMBER")"
  milestone_title="$(python3 -c 'import json,sys; print(json.load(sys.stdin)["title"])' <<<"$milestone_json")"
  [[ "$milestone_title" == "$MILESTONE_TITLE" ]] || fail "milestone title mismatch: $milestone_title"
  if [[ "${NTPRO_RELEASE_GATE:-0}" == "1" ]]; then
    echo "v300_issue_scope=tag_gate closed=$closed_count/12 current_issue_state=$current_state"
  else
    echo "v300_issue_scope=pr_mode closed=$closed_count/12 current_issue_state=$current_state"
  fi
else
  echo "v300_issue_scope=offline_skip"
fi

echo "v30_release_gates=pass release_tag=$RELEASE_TAG final_scope_issues=12 final_scope_evidence=12 v31_handoff=hard_blocked negative_selftest=1"
