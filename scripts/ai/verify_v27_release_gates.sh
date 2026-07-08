#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V270_RELEASE_REPO:-atxinbao/NTPRO}"
RELEASE_VERSION="${NTPRO_V270_RELEASE_VERSION:-v0.27.0}"
RELEASE_TAG="${NTPRO_V270_RELEASE_TAG:-ntpro-rust-only-v0.27.0}"
RELEASE_NAME="${NTPRO_V270_RELEASE_NAME:-NTPRO Rust-only v0.27.0}"
BASE_RELEASE_TAG="${NTPRO_V270_BASE_RELEASE_TAG:-ntpro-rust-only-v0.26.1}"
MANIFEST_PATH="${NTPRO_V270_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_27_0_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V270_RELEASE_NOTES:-docs/rust-cutover/release/v0_27_0_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V270_READINESS_REPORT:-docs/rust-cutover/release/v0_27_0_readiness_report.md}"
TRACE_PATH="${NTPRO_V270_RELEASE_TRACE:-tests/golden/v270_release_gates_strict_provenance.jsonl}"
REPLAY_SCOPE_PATH="${NTPRO_V270_REPLAY_SCOPE:-docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"
CURRENT_ISSUE="${NTPRO_V270_CURRENT_ISSUE:-885}"
MILESTONE_NUMBER="${NTPRO_V270_MILESTONE_NUMBER:-20}"
MILESTONE_TITLE="${NTPRO_V270_MILESTONE_TITLE:-v0.27.0}"

fail() {
  echo "v27 release gate failed: $*" >&2
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

gh_with_retry() {
  local attempt=1
  local max_attempts=4
  while true; do
    if gh "$@"; then
      return 0
    fi
    if (( attempt >= max_attempts )); then
      return 1
    fi
    sleep "$((attempt * 2))"
    attempt=$((attempt + 1))
  done
}

for path in \
  "$MANIFEST_PATH" \
  "$RELEASE_NOTES_PATH" \
  "$READINESS_REPORT_PATH" \
  "$TRACE_PATH" \
  "$REPLAY_SCOPE_PATH" \
  docs/rust-cutover/release/v0_26_1_release_manifest.json \
  docs/rust-cutover/release/v0_26_1_readiness_report.md \
  docs/rust-cutover/release/v0_26_1_release_notes.md \
  docs/rust-cutover/release/v0_27_0_intake_gate.md \
  docs/rust-cutover/release/v0_27_0_product_operations_runtime_integration_boundary_contract.md \
  docs/rust-cutover/release/v0_27_0_external_identity_permission_foundation.md \
  docs/rust-cutover/release/v0_27_0_persistent_operation_audit_storage_foundation.md \
  docs/rust-cutover/release/v0_27_0_deployment_orchestration_foundation.md \
  docs/rust-cutover/release/v0_27_0_long_run_telemetry_slo_runtime_evidence.md \
  docs/rust-cutover/release/v0_27_0_admin_workbench_runtime_state_bridge.md \
  docs/rust-cutover/release/v0_27_0_runtime_integration_fail_closed_hardening.md \
  scripts/ai/verify_v27_intake_gate.sh \
  scripts/ai/verify_v27_product_operations_runtime_integration_boundary_contract.sh \
  scripts/ai/verify_v27_external_identity_permission_foundation.sh \
  scripts/ai/verify_v27_persistent_audit_storage_foundation.sh \
  scripts/ai/verify_v27_deployment_orchestration_foundation.sh \
  scripts/ai/verify_v27_long_run_telemetry_slo_runtime_evidence.sh \
  scripts/ai/verify_v27_admin_workbench_runtime_state_bridge.sh \
  scripts/ai/verify_v27_runtime_integration_fail_closed_hardening.sh \
  scripts/ai/verify_v27_release_gates.sh \
  scripts/ai/verify_v27_strict_provenance.sh \
  scripts/ai/verify_release.sh \
  scripts/ai/publish_ntpro_release_after_gate.sh; do
  require_file "$path"
done

for task_id in V270-000 V270-001 V270-002 V270-003 V270-004 V270-005 V270-006 V270-007 V270-008 V270-009 V270-010; do
  require_file "docs/rust-cutover/evidence/${task_id}.md"
  require_contains "docs/rust-cutover/evidence/${task_id}.md" "$task_id"
  require_file "docs/rust-cutover/tasks/${task_id}.md"
  require_contains "docs/rust-cutover/tasks/${task_id}.md" "$task_id"
done

if [[ "${NTPRO_V270_ALLOW_STALE_EVIDENCE:-0}" != "1" ]]; then
  for marker in \
    "current_issue_state=OPEN" \
    "v270_issues=8/9_closed_or_current" \
    "v270_issues=10/11_closed_or_current" \
    "final_scope_issues=9" \
    "final_scope_issues=10" \
    "tag_exists=false" \
    "source_dirty=true" \
    "offline_skip missing local tag" \
    "pre_tag_mode missing_tag" \
    "Pending final validation"; do
    require_not_contains "docs/rust-cutover/evidence/V270-008.md" "$marker"
    require_not_contains "docs/rust-cutover/evidence/V270-009.md" "$marker"
    require_not_contains "docs/rust-cutover/evidence/V270-010.md" "$marker"
  done
  require_contains "docs/rust-cutover/evidence/V270-008.md" "final_scope_issues=11"
  require_contains "docs/rust-cutover/evidence/V270-009.md" "final_scope_issues=11"
  require_contains "docs/rust-cutover/evidence/V270-010.md" "final_scope_issues=11"
  for marker in \
    "#885 V270-010 = must be closed before v0.27.0 tag gate is accepted" \
    "v0.27.0 milestone = must be closed before public publication"; do
    require_not_contains "$READINESS_REPORT_PATH" "$marker"
  done
fi

for marker in \
  "Status: RELEASED" \
  "Tag: \`$RELEASE_TAG\`" \
  "Release name: \`$RELEASE_NAME\`" \
  "Release URL: \`https://github.com/atxinbao/NTPRO/releases/tag/$RELEASE_TAG\`" \
  "Base release: \`$BASE_RELEASE_TAG\`" \
  "v0.27.0 publishes the Product Operations Runtime Integration Foundation" \
  "V270-000" \
  "V270-010" \
  "V270 final release scope issue count = 11" \
  "V270 final release scope evidence count = 11" \
  "v27 release gates = required" \
  "v27 strict provenance = required" \
  "release surface current guard = required" \
  "release publication guard = required" \
  "release publish after gate = required" \
  "hosted release gate success before public GitHub Release = required" \
  "publication evidence strategy = source_tree_plus_github_remote" \
  "local generated publication evidence required in source tree = false" \
  "remote reconstruction required = true" \
  "new_submit_capability = false" \
  "production_order_submission_allowed = false" \
  "production_order_mutation_allowed = false" \
  "execution_adapter_call_allowed = false" \
  "adapter_send_allowed = false" \
  "live_exchange_request_allowed = false" \
  "retry_scheduler_enabled = false" \
  "automatic_remediation_allowed = false" \
  "dashboard_operation_controls_enabled = false" \
  "dashboard_trading_controls_enabled = false" \
  "admin_workbench_operation_controls_enabled = false" \
  "admin_workbench_trading_controls_enabled = false" \
  "trader_terminal_order_ticket_enabled = false" \
  "manual_operation_submit_allowed = false" \
  "product_grade_trading_terminal_claim = false" \
  "scripts/ai/verify_release.sh v27-release-gates" \
  "scripts/ai/verify_release.sh v27-strict-provenance" \
  "scripts/ai/verify_v27_release_gates.sh" \
  "scripts/ai/verify_v27_strict_provenance.sh" \
  "scripts/ai/check_github_release_published.sh" \
  "scripts/ai/golden_trace_runner.py" \
  "scripts/ai/publish_ntpro_release_after_gate.sh"; do
  require_contains "$RELEASE_NOTES_PATH" "$marker"
done

for marker in \
  "Milestone: \`$RELEASE_TAG\`" \
  "Status: RELEASED" \
  "V270-000 evidence" \
  "V270-010 evidence" \
  "v27 release gates = required" \
  "v27 strict provenance = required" \
  "#885 V270-010 = closed before v0.27.0 tag gate was accepted" \
  "V270 final release scope issue count = 11" \
  "V270 final release scope evidence count = 11" \
  "V270 exact milestone issue set = #853-#861,#883,#885" \
  "V270 registered corrective-scope exception count = 0" \
  "registered corrective-scope exceptions required = true" \
  "unregistered corrective milestone issues fail closed = true" \
  "v0.27.0 milestone = closed before public publication"; do
  require_contains "$READINESS_REPORT_PATH" "$marker"
done

for path in "$RELEASE_NOTES_PATH" "$READINESS_REPORT_PATH" "$MANIFEST_PATH"; do
  for marker in \
    "new_submit_capability = true" \
    "production_order_submission_allowed = true" \
    "production_order_mutation_allowed = true" \
    "execution_adapter_call_allowed = true" \
    "adapter_send_allowed = true" \
    "live_exchange_request_allowed = true" \
    "retry_scheduler_enabled = true" \
    "automatic_remediation_allowed = true" \
    "dashboard_operation_controls_enabled = true" \
    "dashboard_trading_controls_enabled = true" \
    "admin_workbench_operation_controls_enabled = true" \
    "admin_workbench_trading_controls_enabled = true" \
    "trader_terminal_order_ticket_enabled = true" \
    "manual_operation_submit_allowed = true" \
    "product_grade_trading_terminal_claim = true"; do
    require_not_contains "$path" "$marker"
  done
done

scripts/ai/verify_release.sh v27-intake-gate
scripts/ai/verify_release.sh v27-product-operations-boundary-contract
scripts/ai/verify_release.sh v27-external-identity-permission-foundation
scripts/ai/verify_release.sh v27-persistent-audit-storage-foundation
scripts/ai/verify_release.sh v27-deployment-orchestration-foundation
scripts/ai/verify_release.sh v27-long-run-telemetry-slo-runtime-evidence
scripts/ai/verify_release.sh v27-admin-workbench-runtime-state-bridge
scripts/ai/verify_release.sh v27-runtime-integration-fail-closed-hardening

NTPRO_CURRENT_RELEASE_VERSION="$RELEASE_VERSION" \
  NTPRO_CURRENT_RELEASE_TAG="$RELEASE_TAG" \
  NTPRO_NEXT_PATCH_VERSION="v0.27.1" \
  NTPRO_NEXT_CAPABILITY_VERSION="v0.28.0" \
  NTPRO_CURRENT_RELEASE_CAPABILITY="v0.27.0 Product Operations Runtime Integration Foundation" \
  NTPRO_RELEASE_SURFACE_ALLOW_MISSING_TAG=1 \
  scripts/ai/verify_release.sh release-surface-current-guard

NTPRO_CURRENT_RELEASE_VERSION="$RELEASE_VERSION" \
  NTPRO_CURRENT_RELEASE_TAG="$RELEASE_TAG" \
  NTPRO_CURRENT_RELEASE_NAME="$RELEASE_NAME" \
  NTPRO_RELEASE_PUBLICATION_ALLOW_OFFLINE=1 \
  NTPRO_RELEASE_PUBLICATION_PREPUBLISH_TAG_GATE="${NTPRO_RELEASE_GATE:-0}" \
  scripts/ai/verify_release.sh release-publication-guard
scripts/ai/verify_release.sh release-publish-after-gate

if [[ "${NTPRO_RELEASE_GATE:-0}" == "1" ]]; then
  git rev-parse -q --verify "${RELEASE_TAG}^{commit}" >/dev/null || fail "missing local release tag: $RELEASE_TAG"
  tag_commit="$(git rev-list -n 1 "$RELEASE_TAG")"
  head_commit="$(git rev-parse HEAD)"
  [[ "$head_commit" == "$tag_commit" ]] || fail "HEAD $head_commit does not match $RELEASE_TAG commit $tag_commit"
fi

RELEASE_VERSION="$RELEASE_VERSION" \
RELEASE_TAG="$RELEASE_TAG" \
RELEASE_NAME="$RELEASE_NAME" \
MANIFEST_PATH="$MANIFEST_PATH" \
TRACE_PATH="$TRACE_PATH" \
REPLAY_SCOPE_PATH="$REPLAY_SCOPE_PATH" \
python3 <<'PY'
import copy
import json
import os
from pathlib import Path

manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))

EXPECTED_CASES = [
    "release.v270.release_gates.ready.001",
    "release.v270.release_gates.missing_v270_evidence_fail_closed.001",
    "release.v270.release_gates.forbidden_dashboard_control_fail_closed.001",
    "release.v270.strict_provenance.source_tree_bound.001",
]
BOUNDARY_FALSE_FLAGS = [
    "new_submit_capability",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "execution_adapter_call_allowed",
    "adapter_send_allowed",
    "live_exchange_request_allowed",
    "retry_scheduler_enabled",
    "automatic_remediation_allowed",
    "dashboard_operation_controls_enabled",
    "dashboard_trading_controls_enabled",
    "admin_workbench_operation_controls_enabled",
    "admin_workbench_trading_controls_enabled",
    "trader_terminal_order_ticket_enabled",
    "manual_operation_submit_allowed",
    "product_grade_trading_terminal_claim",
]


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def validate(candidate: dict) -> None:
    require(candidate.get("schema_version") == "ntpro.v270_release_manifest.v1", "manifest schema mismatch")
    require(candidate.get("task_id") == "V270-008", "manifest task mismatch")
    require(candidate.get("product_version") == os.environ["RELEASE_VERSION"], "manifest version mismatch")
    require(candidate.get("release_status") in {"release_gate_ready", "released"}, "manifest release status mismatch")
    planned = candidate.get("planned_release") or {}
    require(planned.get("tag") == os.environ["RELEASE_TAG"], "planned tag mismatch")
    require(planned.get("name") == os.environ["RELEASE_NAME"], "planned name mismatch")
    require(planned.get("draft") is False and planned.get("prerelease") is False, "planned release flags mismatch")
    base = candidate.get("base_release") or {}
    require(base.get("tag") == "ntpro-rust-only-v0.26.1", "base release tag mismatch")
    evidence = candidate.get("v270_evidence") or []
    expected = {
        "V270-000": 853,
        "V270-001": 854,
        "V270-002": 855,
        "V270-003": 856,
        "V270-004": 857,
        "V270-005": 858,
        "V270-006": 859,
        "V270-007": 860,
        "V270-008": 861,
        "V270-009": 883,
        "V270-010": 885,
    }
    require(len(evidence) == 11, "V270 evidence count mismatch")
    for item in evidence:
        task_id = item.get("task_id")
        require(expected.get(task_id) == item.get("issue"), f"V270 issue mismatch: {task_id}")
        path = Path(item.get("path", ""))
        require(path.is_file(), f"missing V270 evidence file: {path}")
    scope = candidate.get("release_scope") or {}
    require(scope.get("final_release_scope_issue_count") == 11, "final release scope issue count mismatch")
    require(scope.get("final_release_scope_evidence_count") == 11, "final release scope evidence count mismatch")
    require(scope.get("exact_milestone_issue_numbers") == [853, 854, 855, 856, 857, 858, 859, 860, 861, 883, 885], "exact milestone issue numbers mismatch")
    require(scope.get("exact_milestone_issue_set") == "#853-#861,#883,#885", "exact milestone issue set mismatch")
    require(scope.get("registered_corrective_scope_exception_count") == 0, "registered corrective exception count mismatch")
    require(scope.get("unregistered_corrective_milestone_issues_fail_closed") is True, "unregistered corrective fail-closed rule missing")
    require(scope.get("future_release_gates_must_register_corrective_issues") is True, "future corrective registration rule missing")
    require(scope.get("v26_1_dependency_proven") is True, "v26.1 dependency proof missing")
    require(scope.get("v26_1_release_evidence_published") is True, "v26.1 release evidence missing")
    require(scope.get("capability_scope_expands_trading") is False, "release gate must not expand trading")
    require(scope.get("runtime_behavior_changed_by_release_gate") is False, "release gate must not change runtime")
    require(scope.get("trading_behavior_changed_by_release_gate") is False, "release gate must not change trading")
    commands = {gate.get("command") for gate in candidate.get("release_gates", []) if gate.get("required") is True}
    for command in (
        "scripts/ai/verify_release.sh v27-intake-gate",
        "scripts/ai/verify_release.sh v27-product-operations-boundary-contract",
        "scripts/ai/verify_release.sh v27-external-identity-permission-foundation",
        "scripts/ai/verify_release.sh v27-persistent-audit-storage-foundation",
        "scripts/ai/verify_release.sh v27-deployment-orchestration-foundation",
        "scripts/ai/verify_release.sh v27-long-run-telemetry-slo-runtime-evidence",
        "scripts/ai/verify_release.sh v27-admin-workbench-runtime-state-bridge",
        "scripts/ai/verify_release.sh v27-runtime-integration-fail-closed-hardening",
        "scripts/ai/verify_release.sh v27-release-gates",
        "scripts/ai/verify_release.sh v27-strict-provenance",
        "scripts/ai/verify_release.sh release-surface-current-guard",
        "scripts/ai/verify_release.sh release-publication-guard",
        "scripts/ai/verify_release.sh release-publish-after-gate",
        "scripts/ai/verify_v27_release_gates.sh",
        "scripts/ai/verify_v27_strict_provenance.sh",
    ):
        require(command in commands, f"missing release gate command: {command}")
    capability = candidate.get("capability") or {}
    for key in (
        "product_operations_runtime_integration_foundation",
        "release_governance",
        "strict_provenance",
        "external_identity_permission_foundation",
        "persistent_operation_audit_storage_foundation",
        "deployment_upgrade_rollback_orchestration_foundation",
        "long_run_telemetry_slo_runtime_evidence",
        "admin_workbench_runtime_state_bridge",
        "runtime_integration_fail_closed_hardening",
    ):
        require(capability.get(key) is True, f"capability flag must be true: {key}")
    for key in ("product_grade_live_trading_terminal", "new_submit_capability", "production_order_mutation_expansion", "dashboard_operation_controls", "automatic_remediation_runtime"):
        require(capability.get(key) is False, f"capability flag must be false: {key}")
    boundary = candidate.get("boundary_flags") or {}
    for key in BOUNDARY_FALSE_FLAGS:
        require(boundary.get(key) is False, f"boundary must be false: {key}")
    publication = candidate.get("publication_governance") or {}
    require(publication.get("gate_before_publish") is True, "gate before publish missing")
    require(publication.get("release_gate_success_before_publication_required") is True, "publication ordering missing")
    requirements = candidate.get("post_publication_requirements") or {}
    require(requirements.get("all_v270_issues_closed_required") is True, "V270 closeout requirement missing")
    require(requirements.get("hosted_release_gate_success_required") is True, "hosted gate requirement missing")
    require(requirements.get("strict_release_body_match_required") is True, "strict body match requirement missing")


def load_jsonl(path: Path) -> list[dict]:
    rows = []
    for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        try:
            row = json.loads(line)
        except json.JSONDecodeError as exc:
            raise SystemExit(f"{path}:{line_number}: invalid JSON: {exc}") from exc
        rows.append(row)
    return rows


def validate_trace() -> None:
    trace_path = Path(os.environ["TRACE_PATH"])
    rows = load_jsonl(trace_path)
    found = [row.get("case_id") for row in rows]
    require(found == EXPECTED_CASES, f"release trace cases mismatch: {found}")
    for row in rows:
        case_id = row["case_id"]
        require(row.get("category") == "release_governance", f"{case_id}: category mismatch")
        expected = row.get("expected", {}).get("events", [{}])[0].get("payload", {})
        status = expected.get("effective_release_gate_status")
        if case_id.endswith("ready.001") or case_id.endswith("source_tree_bound.001"):
            require(status in {"release_gate_ready", "strict_provenance_ready"}, f"{case_id}: ready status mismatch")
            require(expected.get("fail_closed") is False, f"{case_id}: must not fail closed")
        else:
            require(str(status).startswith("fail_closed_"), f"{case_id}: fail-closed status mismatch")
            require(expected.get("fail_closed") is True, f"{case_id}: must fail closed")
    replay_scope = json.loads(Path(os.environ["REPLAY_SCOPE_PATH"]).read_text(encoding="utf-8"))
    entries = {case.get("case_id"): case for case in replay_scope.get("cases", [])}
    for case_id in EXPECTED_CASES:
        entry = entries.get(case_id)
        require(entry is not None, f"missing replay scope entry: {case_id}")
        require(entry.get("trace") == trace_path.as_posix(), f"{case_id}: replay scope trace mismatch")
        require(entry.get("status") == "validator_executable_replay", f"{case_id}: replay status mismatch")
        require(entry.get("evidence_id") == "V270-008", f"{case_id}: evidence mismatch")
        require(entry.get("harness") == "scripts/ai/verify_release.sh v27-release-gates", f"{case_id}: harness mismatch")
        require(entry.get("runtime_adapter_integration") is False, f"{case_id}: runtime adapter flag mismatch")


validate(manifest)
validate_trace()
if os.environ.get("NTPRO_V270_RELEASE_SELFTEST", "1") == "1":
    missing = copy.deepcopy(manifest)
    missing["v270_evidence"] = missing["v270_evidence"][:-1]
    try:
        validate(missing)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: missing V270 evidence")
    opened = copy.deepcopy(manifest)
    opened["boundary_flags"]["dashboard_trading_controls_enabled"] = True
    try:
        validate(opened)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: dashboard trading controls enabled")
    unproven = copy.deepcopy(manifest)
    unproven["release_scope"]["v26_1_dependency_proven"] = False
    try:
        validate(unproven)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: missing v26.1 dependency proof")
    extra_issue = copy.deepcopy(manifest)
    extra_issue["release_scope"]["exact_milestone_issue_numbers"] = extra_issue["release_scope"]["exact_milestone_issue_numbers"] + [999]
    try:
        validate(extra_issue)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: unregistered extra milestone issue")
PY

python3 scripts/ai/validate_golden_trace_release_scope.py >/dev/null

if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
  for issue in 853 854 855 856 857 858 859 860 861 883; do
    state="$(gh_with_retry issue view "$issue" --repo "$REPO" --json state --jq .state)" || fail "could not read GitHub issue #$issue"
    [[ "$state" == "CLOSED" ]] || fail "GitHub issue #$issue must be CLOSED before $RELEASE_VERSION release gates, got $state"
  done
  current_state="$(gh_with_retry issue view "$CURRENT_ISSUE" --repo "$REPO" --json state --jq .state)" || fail "could not read GitHub issue #$CURRENT_ISSUE"
  if [[ "${NTPRO_RELEASE_GATE:-0}" == "1" || "${NTPRO_V270_RELEASE_REQUIRE_CLOSEOUT:-0}" == "1" ]]; then
    [[ "$current_state" == "CLOSED" ]] || fail "GitHub issue #$CURRENT_ISSUE must be CLOSED for the tag gate, got $current_state"
  else
    [[ "$current_state" == "OPEN" || "$current_state" == "CLOSED" ]] || fail "unexpected current issue state: $current_state"
  fi
  if [[ "$current_state" == "CLOSED" ]]; then
    v270_issue_summary="11/11_closed"
  else
    fail "GitHub issue #$CURRENT_ISSUE must be CLOSED for exact v27 scope, got $current_state"
  fi
  milestone_json="$(gh_with_retry api "/repos/$REPO/milestones/$MILESTONE_NUMBER")" || fail "could not read GitHub milestone #$MILESTONE_NUMBER"
  milestone_issues_json="$(gh_with_retry issue list --repo "$REPO" --milestone "$MILESTONE_TITLE" --state all --limit 100 --json number,state,title)" || fail "could not read GitHub milestone issues for $MILESTONE_TITLE"
  MILESTONE_JSON="$milestone_json" MILESTONE_ISSUES_JSON="$milestone_issues_json" RELEASE_GATE="${NTPRO_RELEASE_GATE:-0}" REQUIRE_CLOSEOUT="${NTPRO_V270_RELEASE_REQUIRE_CLOSEOUT:-0}" python3 <<'PY'
import json
import os

milestone = json.loads(os.environ["MILESTONE_JSON"])
milestone_issues = json.loads(os.environ["MILESTONE_ISSUES_JSON"])
expected = {853, 854, 855, 856, 857, 858, 859, 860, 861, 883, 885}
if milestone["title"] != "v0.27.0":
    raise SystemExit(milestone)
if os.environ["RELEASE_GATE"] == "1" or os.environ["REQUIRE_CLOSEOUT"] == "1":
    if milestone["state"] != "closed" or milestone["open_issues"] != 0 or milestone["closed_issues"] != len(expected):
        raise SystemExit(f"v0.27.0 milestone must be closed with exactly registered issue scope for tag gate: {milestone}")
else:
    if milestone["state"] not in {"open", "closed"}:
        raise SystemExit(milestone)
numbers = {issue.get("number") for issue in milestone_issues}
if numbers != expected:
    raise SystemExit(f"v0.27.0 milestone issue set mismatch: {sorted(numbers)}")
for issue in milestone_issues:
    if issue.get("state") != "CLOSED":
        raise SystemExit(f"v0.27.0 milestone issue must be closed: #{issue.get('number')}")
PY
else
  fail "gh authentication is required for v27 release gate issue proof"
fi

echo "v27_release_gates status=ok release_tag=$RELEASE_TAG base_release=$BASE_RELEASE_TAG current_issue_state=$current_state v270_issues=$v270_issue_summary final_scope_issues=11 negative_selftest=${NTPRO_V270_RELEASE_SELFTEST:-1}"
