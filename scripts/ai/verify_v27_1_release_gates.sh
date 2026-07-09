#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V271_RELEASE_REPO:-atxinbao/NTPRO}"
RELEASE_VERSION="${NTPRO_V271_RELEASE_VERSION:-v0.27.1}"
RELEASE_TAG="${NTPRO_V271_RELEASE_TAG:-ntpro-rust-only-v0.27.1}"
RELEASE_NAME="${NTPRO_V271_RELEASE_NAME:-NTPRO Rust-only v0.27.1}"
BASE_RELEASE_TAG="${NTPRO_V271_BASE_RELEASE_TAG:-ntpro-rust-only-v0.27.0}"
MANIFEST_PATH="${NTPRO_V271_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_27_1_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V271_RELEASE_NOTES:-docs/rust-cutover/release/v0_27_1_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V271_READINESS_REPORT:-docs/rust-cutover/release/v0_27_1_readiness_report.md}"
CURRENT_ISSUE="${NTPRO_V271_CURRENT_ISSUE:-892}"
MILESTONE_NUMBER="${NTPRO_V271_MILESTONE_NUMBER:-21}"
MILESTONE_TITLE="${NTPRO_V271_MILESTONE_TITLE:-v0.27.1}"

fail() {
  echo "v27.1 release gate failed: $*" >&2
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
  docs/rust-cutover/release/v0_27_1_release_closeout_evidence.md \
  docs/rust-cutover/release/v0_27_0_release_manifest.json \
  docs/rust-cutover/release/v0_27_0_readiness_report.md \
  docs/rust-cutover/release/v0_27_0_release_notes.md \
  docs/rust-cutover/release/v0_27_0_release_closeout_evidence.md \
  docs/rust-cutover/release/v0_27_0_publication_entry_provenance.md \
  README.md \
  ROADMAP.md \
  docs/versioning.md \
  docs/rust-cutover/release/README.md \
  scripts/ai/check_release_surface_current.sh \
  scripts/ai/check_github_release_published.sh \
  scripts/ai/verify_release_publish_after_gate.sh \
  scripts/ai/publish_ntpro_release_after_gate.sh \
  scripts/ai/verify_v27_release_gates.sh \
  scripts/ai/verify_v27_strict_provenance.sh \
  scripts/ai/verify_v27_1_release_gates.sh \
  scripts/ai/verify_v27_1_strict_provenance.sh \
  scripts/ai/verify_release.sh; do
  require_file "$path"
done

for task_id in V271-001 V271-002 V271-003 V271-004 V271-005 V271-006; do
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
  "v0.27.1 is a patch governance and evidence hardening release" \
  "V271-001" \
  "V271-006" \
  "v27.1 release gates = required" \
  "v27.1 strict provenance = required" \
  "release surface current guard = required" \
  "release publication guard = required" \
  "release publish after gate = required" \
  "v28 intake gate = hard-blocked until v0.27.1 publication evidence exists" \
  "publication evidence strategy = source_tree_plus_github_remote" \
  "local generated publication evidence required in source tree = false" \
  "remote reconstruction required = true" \
  "scripts/ai/verify_release.sh v27.1-release-gates" \
  "scripts/ai/verify_release.sh v27.1-strict-provenance" \
  "scripts/ai/verify_v27_1_release_gates.sh" \
  "scripts/ai/verify_v27_1_strict_provenance.sh" \
  "scripts/ai/publish_ntpro_release_after_gate.sh" \
  "v0.28.0 start gate = blocked until v0.27.1 release gate passes"; do
  require_contains "$RELEASE_NOTES_PATH" "$marker"
done

for marker in \
  "Milestone: \`$RELEASE_TAG\`" \
  "Status: RELEASED" \
  "V271-001 evidence" \
  "V271-006 evidence" \
  "v27.1 release gates = required" \
  "v27.1 strict provenance = required" \
  "#892 V271-006 = closed" \
  "V271 final release scope issue count = 6" \
  "V271 final release scope evidence count = 6" \
  "V271 exact milestone issue set = #887-#892" \
  "V271 registered corrective-scope exception count = 0" \
  "registered corrective-scope exceptions required = true" \
  "unregistered corrective milestone issues fail closed = true" \
  "v0.28.0 start gate = blocked until v0.27.1 release evidence is published"; do
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
    "product_grade_trading_terminal_claim = true"; do
    require_not_contains "$path" "$marker"
  done
done

# v0.27.0 is a historical prerequisite for the v0.27.1 tag gate. Its source
# surface has intentionally been superseded by v0.27.1, so the historical gate
# skips only that current-surface assertion while retaining live release proof.
NTPRO_RELEASE_GATE=0 \
  NTPRO_V270_RELEASE_SKIP_CURRENT_SURFACE_GUARD=1 \
  scripts/ai/verify_release.sh v27-release-gates
NTPRO_RELEASE_GATE=0 \
  NTPRO_RELEASE_STRICT_REQUIRE_HEAD_TAG=0 \
  scripts/ai/verify_release.sh v27-strict-provenance

if [[ "${NTPRO_V271_RELEASE_SKIP_CURRENT_SURFACE_GUARD:-0}" == "1" ]] || \
  grep -F "Current source tag: ntpro-rust-only-v0.28.0" README.md >/dev/null; then
  echo "v27_1_release_gates current_surface_guard=skipped reason=current_release_surface_superseded"
else
  NTPRO_CURRENT_RELEASE_VERSION="$RELEASE_VERSION" \
    NTPRO_CURRENT_RELEASE_TAG="$RELEASE_TAG" \
    NTPRO_NEXT_PATCH_VERSION="v0.27.2" \
    NTPRO_NEXT_CAPABILITY_VERSION="v0.28.0" \
    NTPRO_CURRENT_RELEASE_CAPABILITY="v0.27.1 Product Operations Runtime Integration Closeout Patch" \
    NTPRO_RELEASE_SURFACE_ALLOW_MISSING_TAG=1 \
    scripts/ai/verify_release.sh release-surface-current-guard
fi

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

RELEASE_VERSION="$RELEASE_VERSION" RELEASE_TAG="$RELEASE_TAG" RELEASE_NAME="$RELEASE_NAME" MANIFEST_PATH="$MANIFEST_PATH" python3 <<'PY'
import copy
import json
import os
from pathlib import Path

manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))

BOUNDARY_FALSE_FLAGS = [
    "new_submit_capability",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
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
    "product_grade_trading_terminal_claim",
]


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def validate(candidate: dict) -> None:
    require(candidate.get("schema_version") == "ntpro.v271_patch_release_manifest.v1", "manifest schema mismatch")
    require(candidate.get("task_id") == "V271-006", "manifest task mismatch")
    require(candidate.get("product_version") == os.environ["RELEASE_VERSION"], "manifest version mismatch")
    require(candidate.get("release_status") == "released", "manifest release status mismatch")
    planned = candidate.get("planned_release") or {}
    require(planned.get("tag") == os.environ["RELEASE_TAG"], "planned tag mismatch")
    require(planned.get("name") == os.environ["RELEASE_NAME"], "planned name mismatch")
    require(planned.get("draft") is False and planned.get("prerelease") is False, "planned release flags mismatch")
    base = candidate.get("base_release") or {}
    require(base.get("tag") == "ntpro-rust-only-v0.27.0", "base release tag mismatch")
    release_inputs = candidate.get("release_inputs") or {}
    require(
        release_inputs.get("release_closeout_evidence_path")
        == "docs/rust-cutover/release/v0_27_1_release_closeout_evidence.md",
        "release closeout evidence path missing",
    )
    published = candidate.get("published_release") or {}
    require(published.get("tag") == os.environ["RELEASE_TAG"], "published release tag mismatch")
    require(published.get("draft") is False and published.get("prerelease") is False, "published release flags mismatch")
    require(published.get("tag_sha") == "0fdc11dc983bbfb9fe124a3f171a58fb1e7ccf19", "published tag SHA mismatch")
    require(
        published.get("release_body_matches_tracked_release_notes") is True,
        "published release body/source match missing",
    )
    evidence = candidate.get("v271_evidence") or []
    expected = {
        "V271-001": 887,
        "V271-002": 888,
        "V271-003": 889,
        "V271-004": 890,
        "V271-005": 891,
        "V271-006": 892,
    }
    require(len(evidence) == 6, "V271 evidence count mismatch")
    for item in evidence:
        task_id = item.get("task_id")
        require(expected.get(task_id) == item.get("issue"), f"V271 issue mismatch: {task_id}")
        path = Path(item.get("path", ""))
        require(path.is_file(), f"missing V271 evidence file: {path}")
    scope = candidate.get("release_scope") or {}
    require(scope.get("final_release_scope_issue_count") == 6, "final release scope issue count mismatch")
    require(scope.get("final_release_scope_evidence_count") == 6, "final release scope evidence count mismatch")
    require(scope.get("exact_milestone_issue_numbers") == [887, 888, 889, 890, 891, 892], "exact milestone issue numbers mismatch")
    require(scope.get("exact_milestone_issue_set") == "#887-#892", "exact milestone issue set mismatch")
    require(scope.get("registered_corrective_scope_exception_count") == 0, "registered corrective exception count mismatch")
    require(scope.get("unregistered_corrective_milestone_issues_fail_closed") is True, "unregistered corrective fail-closed rule missing")
    require(scope.get("future_release_gates_must_register_corrective_issues") is True, "future corrective registration rule missing")
    require(scope.get("v27_0_dependency_proven") is True, "v27.0 dependency proof missing")
    require(scope.get("v27_0_release_evidence_published") is True, "v27.0 release evidence missing")
    require(scope.get("capability_scope_expands_trading") is False, "release gate must not expand trading")
    require(scope.get("runtime_behavior_changed_by_release_gate") is False, "release gate must not change runtime")
    require(scope.get("trading_behavior_changed_by_release_gate") is False, "release gate must not change trading")
    commands = {gate.get("command") for gate in candidate.get("release_gates", []) if gate.get("required") is True}
    for command in (
        "scripts/ai/verify_release.sh v27-release-gates",
        "scripts/ai/verify_release.sh v27-strict-provenance",
        "scripts/ai/verify_release.sh v27.1-release-gates",
        "scripts/ai/verify_release.sh v27.1-strict-provenance",
        "scripts/ai/verify_release.sh release-surface-current-guard",
        "scripts/ai/verify_release.sh release-publication-guard",
        "scripts/ai/verify_release.sh release-publish-after-gate",
        "scripts/ai/verify_v27_1_release_gates.sh",
        "scripts/ai/verify_v27_1_strict_provenance.sh",
    ):
        require(command in commands, f"missing release gate command: {command}")
    capability = candidate.get("capability") or {}
    require(capability.get("patch_hardening_only") is True, "patch hardening flag missing")
    require(capability.get("release_governance_hardening") is True, "release governance flag missing")
    require(capability.get("strict_provenance") is True, "strict provenance flag missing")
    require(capability.get("v270_closeout_reconciliation") is True, "v270 closeout flag missing")
    require(capability.get("v0_28_start_gate_defined") is True, "v28 start gate flag missing")
    require(capability.get("v0_28_implementation_started") is False, "v28 implementation must not start")
    for key in ("product_grade_live_trading_terminal", "new_submit_capability", "production_order_mutation_expansion", "dashboard_operation_controls", "automatic_remediation_runtime"):
        require(capability.get(key) is False, f"capability flag must be false: {key}")
    boundary = candidate.get("boundary_flags") or {}
    for key in BOUNDARY_FALSE_FLAGS:
        require(boundary.get(key) is False, f"boundary must be false: {key}")
    publication = candidate.get("publication_governance") or {}
    require(publication.get("gate_before_publish") is True, "gate before publish missing")
    require(publication.get("release_gate_success_before_publication_required") is True, "publication ordering missing")
    require(publication.get("remote_reconstruction_required") is True, "remote reconstruction missing")
    surface = candidate.get("release_surface_current_guard") or {}
    require(surface.get("current_release_version") == "v0.27.1", "current surface version mismatch")
    require(surface.get("next_patch_version") == "v0.27.2", "next patch mismatch")
    require(surface.get("next_capability_version") == "v0.28.0", "next capability mismatch")
    next_tracks = candidate.get("next_tracks") or {}
    require(next_tracks.get("capability") == "v0.28.0", "next capability track mismatch")
    require(next_tracks.get("start_gate") == "blocked_until_v271_release_evidence_published", "v28 start gate missing")
    require(next_tracks.get("implementation_started") is False, "v28 implementation must not start")
    for key in (
        "inherits_production_submit",
        "inherits_production_mutation",
        "inherits_adapter_send",
        "inherits_live_exchange_request",
        "inherits_retry_scheduler",
        "inherits_automatic_remediation",
        "inherits_dashboard_trading_controls",
        "inherits_admin_workbench_trading_controls",
    ):
        require(next_tracks.get(key) is False, f"next track must not inherit: {key}")
    requirements = candidate.get("post_publication_requirements") or {}
    require(requirements.get("all_v271_issues_closed_required") is True, "V271 closeout requirement missing")
    require(requirements.get("exact_milestone_issue_set_required") is True, "exact scope requirement missing")
    require(requirements.get("hosted_release_gate_success_required") is True, "hosted gate requirement missing")
    require(requirements.get("strict_release_body_match_required") is True, "strict body match requirement missing")
    require(requirements.get("publication_after_hosted_gate_required") is True, "publication ordering requirement missing")
    require(requirements.get("source_controlled_closeout_evidence_required") is True, "source closeout requirement missing")
    require(requirements.get("v0_28_start_gate_fails_without_v271_release_evidence") is True, "v28 hard-block requirement missing")
    closeout = candidate.get("post_publication_closeout") or {}
    require(closeout.get("task_id") == "V281-003", "post-publication closeout task mismatch")
    require(closeout.get("issue") == 921, "post-publication closeout issue mismatch")
    require(closeout.get("source_controlled_closeout_evidence") is True, "source-controlled closeout proof missing")
    require(
        closeout.get("source_controlled_closeout_evidence_path")
        == "docs/rust-cutover/release/v0_27_1_release_closeout_evidence.md",
        "source-controlled closeout path mismatch",
    )
    gate = closeout.get("hosted_release_gate") or {}
    require(gate.get("run_id") == 28940442369, "closeout hosted gate run mismatch")
    require(gate.get("conclusion") == "success", "closeout hosted gate conclusion mismatch")
    require(gate.get("jobs_success") == 82 and gate.get("jobs_failed") == 0, "closeout hosted gate job count mismatch")


validate(manifest)
if os.environ.get("NTPRO_V271_RELEASE_SELFTEST", "1") == "1":
    missing = copy.deepcopy(manifest)
    missing["v271_evidence"] = missing["v271_evidence"][:-1]
    try:
        validate(missing)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: missing V271 evidence")
    opened = copy.deepcopy(manifest)
    opened["boundary_flags"]["dashboard_trading_controls_enabled"] = True
    try:
        validate(opened)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: dashboard trading controls enabled")
    unblocked = copy.deepcopy(manifest)
    unblocked["post_publication_requirements"]["v0_28_start_gate_fails_without_v271_release_evidence"] = False
    try:
        validate(unblocked)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: v28 gate not hard-blocked")
    extra_issue = copy.deepcopy(manifest)
    extra_issue["release_scope"]["exact_milestone_issue_numbers"] = extra_issue["release_scope"]["exact_milestone_issue_numbers"] + [999]
    try:
        validate(extra_issue)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: unregistered extra milestone issue")
PY

if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
  for issue in 887 888 889 890 891; do
    state="$(gh_with_retry issue view "$issue" --repo "$REPO" --json state --jq .state)" || fail "could not read GitHub issue #$issue"
    [[ "$state" == "CLOSED" ]] || fail "GitHub issue #$issue must be CLOSED before $RELEASE_VERSION release gates, got $state"
  done
  current_state="$(gh_with_retry issue view "$CURRENT_ISSUE" --repo "$REPO" --json state --jq .state)" || fail "could not read GitHub issue #$CURRENT_ISSUE"
  if [[ "${NTPRO_RELEASE_GATE:-0}" == "1" || "${NTPRO_V271_RELEASE_REQUIRE_CLOSEOUT:-0}" == "1" ]]; then
    [[ "$current_state" == "CLOSED" ]] || fail "GitHub issue #$CURRENT_ISSUE must be CLOSED for the tag gate, got $current_state"
  else
    [[ "$current_state" == "OPEN" || "$current_state" == "CLOSED" ]] || fail "unexpected current issue state: $current_state"
  fi
  if [[ "$current_state" == "CLOSED" ]]; then
    v271_issue_summary="6/6_closed"
  else
    v271_issue_summary="5/6_closed_or_current"
  fi
  milestone_json="$(gh_with_retry api "/repos/$REPO/milestones/$MILESTONE_NUMBER")" || fail "could not read GitHub milestone #$MILESTONE_NUMBER"
  milestone_issues_json="$(gh_with_retry issue list --repo "$REPO" --milestone "$MILESTONE_TITLE" --state all --limit 100 --json number,state,title)" || fail "could not read GitHub milestone issues for $MILESTONE_TITLE"
  MILESTONE_JSON="$milestone_json" MILESTONE_ISSUES_JSON="$milestone_issues_json" RELEASE_GATE="${NTPRO_RELEASE_GATE:-0}" REQUIRE_CLOSEOUT="${NTPRO_V271_RELEASE_REQUIRE_CLOSEOUT:-0}" CURRENT_ISSUE="$CURRENT_ISSUE" python3 <<'PY'
import json
import os

milestone = json.loads(os.environ["MILESTONE_JSON"])
milestone_issues = json.loads(os.environ["MILESTONE_ISSUES_JSON"])
expected = {887, 888, 889, 890, 891, 892}
current_issue = int(os.environ["CURRENT_ISSUE"])
if milestone["title"] != "v0.27.1":
    raise SystemExit(milestone)
if os.environ["RELEASE_GATE"] == "1" or os.environ["REQUIRE_CLOSEOUT"] == "1":
    if milestone["state"] != "closed" or milestone["open_issues"] != 0 or milestone["closed_issues"] != len(expected):
        raise SystemExit(f"v0.27.1 milestone must be closed with exactly registered issue scope for tag gate: {milestone}")
else:
    if milestone["state"] not in {"open", "closed"}:
        raise SystemExit(milestone)
numbers = {issue.get("number") for issue in milestone_issues}
if numbers != expected:
    raise SystemExit(f"v0.27.1 milestone issue set mismatch: {sorted(numbers)}")
for issue in milestone_issues:
    number = issue.get("number")
    state = issue.get("state")
    if number == current_issue and os.environ["RELEASE_GATE"] != "1" and os.environ["REQUIRE_CLOSEOUT"] != "1":
        if state not in {"OPEN", "CLOSED"}:
            raise SystemExit(f"unexpected current issue state: #{number} {state}")
    elif state != "CLOSED":
        raise SystemExit(f"v0.27.1 milestone issue must be closed: #{number}")
PY
else
  fail "gh authentication is required for v27.1 release gate issue proof"
fi

echo "v27_1_release_gates status=ok release_tag=$RELEASE_TAG base_release=$BASE_RELEASE_TAG current_issue_state=$current_state v271_issues=$v271_issue_summary final_scope_issues=6 negative_selftest=${NTPRO_V271_RELEASE_SELFTEST:-1}"
