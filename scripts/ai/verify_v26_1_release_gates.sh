#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V261_RELEASE_REPO:-atxinbao/NTPRO}"
RELEASE_VERSION="${NTPRO_V261_RELEASE_VERSION:-v0.26.1}"
RELEASE_TAG="${NTPRO_V261_RELEASE_TAG:-ntpro-rust-only-v0.26.1}"
RELEASE_NAME="${NTPRO_V261_RELEASE_NAME:-NTPRO Rust-only v0.26.1}"
BASE_RELEASE_TAG="${NTPRO_V261_BASE_RELEASE_TAG:-ntpro-rust-only-v0.26.0}"
MANIFEST_PATH="${NTPRO_V261_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_26_1_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V261_RELEASE_NOTES:-docs/rust-cutover/release/v0_26_1_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V261_READINESS_REPORT:-docs/rust-cutover/release/v0_26_1_readiness_report.md}"
CURRENT_ISSUE="${NTPRO_V261_CURRENT_ISSUE:-852}"
MILESTONE_NUMBER="${NTPRO_V261_MILESTONE_NUMBER:-19}"

fail() {
  echo "v26.1 release gate failed: $*" >&2
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
  docs/rust-cutover/release/v0_26_0_release_manifest.json \
  docs/rust-cutover/release/v0_26_0_release_closeout_evidence.md \
  scripts/ai/verify_v26_release_gates.sh \
  scripts/ai/verify_v26_strict_provenance.sh \
  scripts/ai/verify_v26_1_final_scope_integration.sh \
  scripts/ai/verify_v26_1_stale_v260_evidence_cleanup.sh \
  scripts/ai/verify_v26_1_post_publication_strict_gate.sh \
  scripts/ai/verify_v26_1_release_gates.sh \
  scripts/ai/verify_v26_1_strict_provenance.sh \
  scripts/ai/verify_v27_intake_gate.sh; do
  require_file "$path"
done

for task_id in V261-001 V261-002 V261-003 V261-004 V261-005 V261-006; do
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
  "v0.26.1 is a patch governance and evidence hardening release" \
  "V261-001" \
  "V261-006" \
  "v26.1 release gates = required" \
  "v26.1 strict provenance = required" \
  "release surface current guard = required" \
  "release publication guard = required" \
  "release publish after gate = required" \
  "v27 intake gate = hard-blocked until v0.26.1 publication evidence exists" \
  "publication evidence strategy = source_tree_plus_github_remote" \
  "v0.27.0 start gate = blocked until v0.26.1 release gate passes" \
  "scripts/ai/verify_release.sh v26.1-release-gates" \
  "scripts/ai/verify_release.sh v26.1-strict-provenance" \
  "scripts/ai/verify_v26_1_release_gates.sh" \
  "scripts/ai/verify_v26_1_strict_provenance.sh" \
  "scripts/ai/verify_v27_intake_gate.sh" \
  "scripts/ai/publish_ntpro_release_after_gate.sh"; do
  require_contains "$RELEASE_NOTES_PATH" "$marker"
done

for marker in \
  "Milestone: \`$RELEASE_TAG\`" \
  "Status: RELEASED" \
  "V261-001 evidence" \
  "V261-006 evidence" \
  "v26.1 release gates = required" \
  "v26.1 strict provenance = required" \
  "#852 V261-006 = must be closed before v0.26.1 tag gate is accepted" \
  "No V270 implementation starts until all V261 issues are closed and v0.26.1"; do
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
    "trader_terminal_order_ticket_enabled = true" \
    "manual_operation_submit_allowed = true" \
    "product_grade_trading_terminal_claim = true"; do
    require_not_contains "$path" "$marker"
  done
done

# v26.0 is a historical prerequisite for the v26.1 tag gate. Do not let the
# current tag workflow's release-gate mode force v26.0 HEAD/tag equality checks.
NTPRO_RELEASE_GATE=0 scripts/ai/verify_release.sh v26-release-gates
NTPRO_RELEASE_GATE=0 NTPRO_RELEASE_STRICT_REQUIRE_HEAD_TAG=0 scripts/ai/verify_release.sh v26-strict-provenance
scripts/ai/verify_v26_1_final_scope_integration.sh
scripts/ai/verify_v26_1_stale_v260_evidence_cleanup.sh
scripts/ai/verify_v26_1_post_publication_strict_gate.sh

if [[ "${NTPRO_V261_RELEASE_SKIP_CURRENT_SURFACE_GUARD:-0}" == "1" ]]; then
  echo "v26_1_release_gates historical_current_surface_guard=skipped reason=current_release_surface_superseded"
elif [[ "${NTPRO_RELEASE_GATE:-0}" != "1" && "${NTPRO_V261_RELEASE_REQUIRE_CURRENT_SURFACE:-0}" != "1" ]]; then
  echo "v26_1_release_gates current_surface_guard=deferred reason=pre_tag_release_surface"
else
  NTPRO_CURRENT_RELEASE_VERSION="$RELEASE_VERSION" \
    NTPRO_CURRENT_RELEASE_TAG="$RELEASE_TAG" \
    NTPRO_NEXT_PATCH_VERSION="v0.26.2" \
    NTPRO_NEXT_CAPABILITY_VERSION="v0.27.0" \
    NTPRO_CURRENT_RELEASE_CAPABILITY="v0.26.1 Product Hardening Foundation Closeout Patch" \
    NTPRO_RELEASE_SURFACE_ALLOW_MISSING_TAG=1 \
    scripts/ai/verify_release.sh release-surface-current-guard
fi

NTPRO_CURRENT_RELEASE_VERSION="$RELEASE_VERSION" \
  NTPRO_CURRENT_RELEASE_TAG="$RELEASE_TAG" \
  NTPRO_CURRENT_RELEASE_NAME="$RELEASE_NAME" \
  NTPRO_RELEASE_PUBLICATION_ALLOW_OFFLINE=1 \
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


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def validate(candidate: dict) -> None:
    require(candidate.get("schema_version") == "ntpro.v261_patch_release_manifest.v1", "manifest schema mismatch")
    require(candidate.get("task_id") == "V261-006", "manifest task mismatch")
    require(candidate.get("product_version") == os.environ["RELEASE_VERSION"], "manifest version mismatch")
    require(candidate.get("release_status") in {"release_gate_ready", "released"}, "manifest release status mismatch")
    planned = candidate.get("planned_release") or {}
    require(planned.get("tag") == os.environ["RELEASE_TAG"], "planned tag mismatch")
    require(planned.get("name") == os.environ["RELEASE_NAME"], "planned name mismatch")
    require(planned.get("draft") is False and planned.get("prerelease") is False, "planned release flags mismatch")
    evidence = candidate.get("v261_evidence") or []
    require(len(evidence) == 6, "V261 evidence count mismatch")
    expected = {"V261-001": 847, "V261-002": 848, "V261-003": 849, "V261-004": 850, "V261-005": 851, "V261-006": 852}
    for item in evidence:
        task_id = item.get("task_id")
        require(expected.get(task_id) == item.get("issue"), f"V261 issue mismatch: {task_id}")
        path = Path(item.get("path", ""))
        require(path.is_file(), f"missing V261 evidence file: {path}")
    commands = {gate.get("command") for gate in candidate.get("release_gates", []) if gate.get("required") is True}
    for command in (
        "scripts/ai/verify_release.sh v26-release-gates",
        "scripts/ai/verify_release.sh v26-strict-provenance",
        "scripts/ai/verify_release.sh v26.1-release-gates",
        "scripts/ai/verify_release.sh v26.1-strict-provenance",
        "scripts/ai/verify_release.sh release-surface-current-guard",
        "scripts/ai/verify_release.sh release-publication-guard",
        "scripts/ai/verify_release.sh release-publish-after-gate",
        "scripts/ai/verify_v26_1_release_gates.sh",
        "scripts/ai/verify_v26_1_strict_provenance.sh",
        "scripts/ai/verify_v27_intake_gate.sh",
    ):
        require(command in commands, f"missing release gate command: {command}")
    inputs = candidate.get("release_inputs") or {}
    require(inputs.get("v27_intake_gate_path") == "scripts/ai/verify_v27_intake_gate.sh", "v27 intake path missing")
    capability = candidate.get("capability") or {}
    require(capability.get("patch_hardening_only") is True, "patch hardening flag missing")
    require(capability.get("strict_provenance") is True, "strict provenance flag missing")
    require(capability.get("v0_27_start_gate_defined") is True, "v27 start gate flag missing")
    require(capability.get("v0_27_implementation_started") is False, "v27 must not start")
    for key in ("new_submit_capability", "production_order_mutation_expansion", "dashboard_operation_controls", "automatic_remediation_runtime"):
        require(capability.get(key) is False, f"capability must be false: {key}")
    boundary = candidate.get("boundary_flags") or {}
    for key in (
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
        "trader_terminal_order_ticket_enabled",
        "manual_operation_submit_allowed",
        "product_grade_trading_terminal_claim",
    ):
        require(boundary.get(key) is False, f"boundary must be false: {key}")
    next_tracks = candidate.get("next_tracks") or {}
    require(next_tracks.get("capability") == "v0.27.0", "next capability mismatch")
    require(next_tracks.get("start_gate") == "blocked_until_v261_release_evidence_published", "next start gate mismatch")
    require(next_tracks.get("implementation_started") is False, "v0.27 must not start")
    requirements = candidate.get("post_publication_requirements") or {}
    require(requirements.get("all_v261_issues_closed_required") is True, "V261 closeout requirement missing")
    require(requirements.get("v0_27_start_gate_fails_without_v261_release_evidence") is True, "v27 hard block requirement missing")


validate(manifest)
if os.environ.get("NTPRO_V261_RELEASE_SELFTEST", "1") == "1":
    missing = copy.deepcopy(manifest)
    missing["v261_evidence"] = missing["v261_evidence"][:-1]
    try:
        validate(missing)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: missing V261 evidence")
    opened = copy.deepcopy(manifest)
    opened["boundary_flags"]["dashboard_trading_controls_enabled"] = True
    try:
        validate(opened)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: dashboard trading controls enabled")
    unblocked = copy.deepcopy(manifest)
    unblocked["post_publication_requirements"]["v0_27_start_gate_fails_without_v261_release_evidence"] = False
    try:
        validate(unblocked)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: missing v27 hard block")
PY

if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
  for issue in 847 848 849 850 851; do
    state="$(gh_with_retry issue view "$issue" --repo "$REPO" --json state --jq .state)" || fail "could not read GitHub issue #$issue"
    [[ "$state" == "CLOSED" ]] || fail "GitHub issue #$issue must be CLOSED before $RELEASE_VERSION release gates, got $state"
  done
  current_state="$(gh_with_retry issue view "$CURRENT_ISSUE" --repo "$REPO" --json state --jq .state)" || fail "could not read GitHub issue #$CURRENT_ISSUE"
  if [[ "${NTPRO_RELEASE_GATE:-0}" == "1" || "${NTPRO_V261_RELEASE_REQUIRE_CLOSEOUT:-0}" == "1" ]]; then
    [[ "$current_state" == "CLOSED" ]] || fail "GitHub issue #$CURRENT_ISSUE must be CLOSED for the tag gate, got $current_state"
  else
    [[ "$current_state" == "OPEN" || "$current_state" == "CLOSED" ]] || fail "unexpected current issue state: $current_state"
  fi
  milestone_json="$(gh_with_retry api "/repos/$REPO/milestones/$MILESTONE_NUMBER")" || fail "could not read GitHub milestone #$MILESTONE_NUMBER"
  MILESTONE_JSON="$milestone_json" RELEASE_GATE="${NTPRO_RELEASE_GATE:-0}" REQUIRE_CLOSEOUT="${NTPRO_V261_RELEASE_REQUIRE_CLOSEOUT:-0}" python3 <<'PY'
import json
import os

milestone = json.loads(os.environ["MILESTONE_JSON"])
if milestone["title"] != "v0.26.1":
    raise SystemExit(milestone)
if os.environ["RELEASE_GATE"] == "1" or os.environ["REQUIRE_CLOSEOUT"] == "1":
    if milestone["state"] != "closed" or milestone["open_issues"] != 0 or milestone["closed_issues"] != 6:
        raise SystemExit(f"v0.26.1 milestone must be closed with 6 closed issues for tag gate: {milestone}")
else:
    if milestone["state"] not in {"open", "closed"}:
        raise SystemExit(milestone)
PY
else
  fail "gh authentication is required for v26.1 release gate issue proof"
fi

echo "v26_1_release_gates status=ok release_tag=$RELEASE_TAG base_release=$BASE_RELEASE_TAG current_issue_state=$current_state v261_issues=5/6_closed_or_current final_scope_issues=6 negative_selftest=${NTPRO_V261_RELEASE_SELFTEST:-1}"
