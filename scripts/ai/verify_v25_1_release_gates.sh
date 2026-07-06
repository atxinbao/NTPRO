#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V251_RELEASE_REPO:-atxinbao/NTPRO}"
RELEASE_VERSION="${NTPRO_V251_RELEASE_VERSION:-v0.25.1}"
RELEASE_TAG="${NTPRO_V251_RELEASE_TAG:-ntpro-rust-only-v0.25.1}"
RELEASE_NAME="${NTPRO_V251_RELEASE_NAME:-NTPRO Rust-only v0.25.1}"
BASE_RELEASE_TAG="${NTPRO_V251_BASE_RELEASE_TAG:-ntpro-rust-only-v0.25.0}"
MANIFEST_PATH="${NTPRO_V251_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_25_1_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V251_RELEASE_NOTES:-docs/rust-cutover/release/v0_25_1_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V251_READINESS_REPORT:-docs/rust-cutover/release/v0_25_1_readiness_report.md}"
CURRENT_ISSUE="${NTPRO_V251_CURRENT_ISSUE:-811}"
MILESTONE_NUMBER="${NTPRO_V251_MILESTONE_NUMBER:-17}"

fail() {
  echo "v25.1 release gate failed: $*" >&2
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
  docs/rust-cutover/release/v0_25_0_release_manifest.json \
  docs/rust-cutover/release/v0_25_0_release_closeout_evidence.md \
  scripts/ai/verify_v25_1_release_gates.sh \
  scripts/ai/verify_v25_1_strict_provenance.sh; do
  require_file "$path"
done

for task_id in V251-001 V251-002 V251-003 V251-004 V251-005 V251-006; do
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
  "v0.25.1 is a patch governance and evidence hardening release" \
  "V251-001" \
  "V251-006" \
  "v25.1 release gates = required" \
  "v25.1 strict provenance = required" \
  "release surface current guard = required" \
  "release publication guard = required" \
  "release publish after gate = required" \
  "publication evidence strategy = source_tree_plus_github_remote" \
  "v0.26.0 start gate = blocked until v0.25.1 release gate passes" \
  "scripts/ai/verify_release.sh v25.1-release-gates" \
  "scripts/ai/verify_release.sh v25.1-strict-provenance" \
  "scripts/ai/verify_v25_1_release_gates.sh" \
  "scripts/ai/verify_v25_1_strict_provenance.sh" \
  "scripts/ai/publish_ntpro_release_after_gate.sh"; do
  require_contains "$RELEASE_NOTES_PATH" "$marker"
done

for marker in \
  "Milestone: \`$RELEASE_TAG\`" \
  "Status: RELEASED" \
  "V251-001 evidence" \
  "V251-006 evidence" \
  "v25.1 release gates = required" \
  "v25.1 strict provenance = required" \
  "#811 V251-006 = must be closed before v0.25.1 tag gate is accepted" \
  "No V260 implementation starts until all V251 issues are closed and v0.25.1"; do
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

scripts/ai/verify_release.sh v25.1-release-closeout-evidence
scripts/ai/verify_release.sh v25.1-corrective-release-scope
scripts/ai/verify_release.sh v25.1-stale-pretag-cleanup
scripts/ai/verify_release.sh v25.1-dashboard-source-ref-integrity
scripts/ai/verify_release.sh v25.1-post-release-gate-split
NTPRO_CURRENT_RELEASE_VERSION="$RELEASE_VERSION" \
  NTPRO_CURRENT_RELEASE_TAG="$RELEASE_TAG" \
  NTPRO_NEXT_PATCH_VERSION="v0.25.2" \
  NTPRO_NEXT_CAPABILITY_VERSION="v0.26.0" \
  NTPRO_CURRENT_RELEASE_CAPABILITY="v0.25.1 Monitoring Incident DR Foundation Hardening Patch" \
  NTPRO_RELEASE_SURFACE_ALLOW_MISSING_TAG=1 \
  scripts/ai/verify_release.sh release-surface-current-guard
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
    require(candidate.get("schema_version") == "ntpro.v251_patch_release_manifest.v1", "manifest schema mismatch")
    require(candidate.get("task_id") == "V251-006", "manifest task mismatch")
    require(candidate.get("product_version") == os.environ["RELEASE_VERSION"], "manifest version mismatch")
    require(candidate.get("release_status") == "released", "manifest release status mismatch")
    planned = candidate.get("planned_release") or {}
    require(planned.get("tag") == os.environ["RELEASE_TAG"], "planned tag mismatch")
    require(planned.get("name") == os.environ["RELEASE_NAME"], "planned name mismatch")
    require(planned.get("draft") is False and planned.get("prerelease") is False, "planned release flags mismatch")
    evidence = candidate.get("v251_evidence") or []
    require(len(evidence) == 6, "V251 evidence count mismatch")
    expected = {"V251-001": 806, "V251-002": 807, "V251-003": 808, "V251-004": 809, "V251-005": 810, "V251-006": 811}
    for item in evidence:
        task_id = item.get("task_id")
        require(expected.get(task_id) == item.get("issue"), f"V251 issue mismatch: {task_id}")
        path = Path(item.get("path", ""))
        require(path.is_file(), f"missing V251 evidence file: {path}")
    commands = {gate.get("command") for gate in candidate.get("release_gates", []) if gate.get("required") is True}
    for command in (
        "scripts/ai/verify_release.sh v25.1-release-closeout-evidence",
        "scripts/ai/verify_release.sh v25.1-corrective-release-scope",
        "scripts/ai/verify_release.sh v25.1-stale-pretag-cleanup",
        "scripts/ai/verify_release.sh v25.1-dashboard-source-ref-integrity",
        "scripts/ai/verify_release.sh v25.1-post-release-gate-split",
        "scripts/ai/verify_release.sh v25.1-release-gates",
        "scripts/ai/verify_release.sh v25.1-strict-provenance",
        "scripts/ai/verify_release.sh release-surface-current-guard",
        "scripts/ai/verify_release.sh release-publication-guard",
        "scripts/ai/verify_release.sh release-publish-after-gate",
        "scripts/ai/verify_v25_1_release_gates.sh",
        "scripts/ai/verify_v25_1_strict_provenance.sh",
    ):
        require(command in commands, f"missing release gate command: {command}")
    capability = candidate.get("capability") or {}
    require(capability.get("patch_hardening_only") is True, "patch hardening flag missing")
    require(capability.get("strict_provenance") is True, "strict provenance flag missing")
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
    require(next_tracks.get("capability") == "v0.26.0", "next capability mismatch")
    require(next_tracks.get("start_gate") == "blocked_until_v251_release_evidence_published", "next start gate mismatch")
    require(next_tracks.get("implementation_started") is False, "v0.26 must not start")


validate(manifest)
if os.environ.get("NTPRO_V251_RELEASE_SELFTEST", "1") == "1":
    missing = copy.deepcopy(manifest)
    missing["v251_evidence"] = missing["v251_evidence"][:-1]
    try:
        validate(missing)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: missing V251 evidence")
    opened = copy.deepcopy(manifest)
    opened["boundary_flags"]["dashboard_trading_controls_enabled"] = True
    try:
        validate(opened)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: dashboard trading controls enabled")
PY

if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
  for issue in 806 807 808 809 810; do
    state="$(gh_with_retry issue view "$issue" --repo "$REPO" --json state --jq .state)" || fail "could not read GitHub issue #$issue"
    [[ "$state" == "CLOSED" ]] || fail "GitHub issue #$issue must be CLOSED before $RELEASE_VERSION release gates, got $state"
  done
  current_state="$(gh_with_retry issue view "$CURRENT_ISSUE" --repo "$REPO" --json state --jq .state)" || fail "could not read GitHub issue #$CURRENT_ISSUE"
  if [[ "${NTPRO_RELEASE_GATE:-0}" == "1" || "${NTPRO_V251_RELEASE_REQUIRE_CLOSEOUT:-0}" == "1" ]]; then
    [[ "$current_state" == "CLOSED" ]] || fail "GitHub issue #$CURRENT_ISSUE must be CLOSED for the tag gate, got $current_state"
  else
    [[ "$current_state" == "OPEN" || "$current_state" == "CLOSED" ]] || fail "unexpected current issue state: $current_state"
  fi
  milestone_json="$(gh_with_retry api "/repos/$REPO/milestones/$MILESTONE_NUMBER")" || fail "could not read GitHub milestone #$MILESTONE_NUMBER"
  MILESTONE_JSON="$milestone_json" RELEASE_GATE="${NTPRO_RELEASE_GATE:-0}" REQUIRE_CLOSEOUT="${NTPRO_V251_RELEASE_REQUIRE_CLOSEOUT:-0}" python3 <<'PY'
import json
import os

milestone = json.loads(os.environ["MILESTONE_JSON"])
if milestone["title"] != "v0.25.1":
    raise SystemExit(milestone)
if os.environ["RELEASE_GATE"] == "1" or os.environ["REQUIRE_CLOSEOUT"] == "1":
    if milestone["state"] != "closed" or milestone["open_issues"] != 0 or milestone["closed_issues"] != 6:
        raise SystemExit(f"v0.25.1 milestone must be closed with 6 closed issues for tag gate: {milestone}")
else:
    if milestone["state"] not in {"open", "closed"}:
        raise SystemExit(milestone)
PY
else
  fail "gh authentication is required for v25.1 release gate issue proof"
fi

echo "v25_1_release_gates status=ok release_tag=$RELEASE_TAG base_release=$BASE_RELEASE_TAG current_issue_state=$current_state v251_issues=5/6_closed_or_current final_scope_issues=6 negative_selftest=${NTPRO_V251_RELEASE_SELFTEST:-1}"
