#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V231_RELEASE_REPO:-atxinbao/NTPRO}"
RELEASE_VERSION="${NTPRO_V231_RELEASE_VERSION:-v0.23.1}"
RELEASE_TAG="${NTPRO_V231_RELEASE_TAG:-ntpro-rust-only-v0.23.1}"
RELEASE_NAME="${NTPRO_V231_RELEASE_NAME:-NTPRO Rust-only v0.23.1}"
BASE_RELEASE_TAG="${NTPRO_V231_BASE_RELEASE_TAG:-ntpro-rust-only-v0.23.0}"
MANIFEST_PATH="${NTPRO_V231_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_23_1_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V231_RELEASE_NOTES:-docs/rust-cutover/release/v0_23_1_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V231_READINESS_REPORT:-docs/rust-cutover/release/v0_23_1_readiness_report.md}"

fail() {
  echo "v23.1 release gate failed: $*" >&2
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
    fail "stale marker in $path: $marker"
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
  docs/rust-cutover/release/v0_23_0_release_manifest.json \
  docs/rust-cutover/release/v0_23_0_release_closeout_evidence.md \
  docs/rust-cutover/release/v0_23_0_gate_phase_split.md \
  docs/rust-cutover/release/v0_23_0_evidence_replay_only_boundary.md \
  docs/rust-cutover/release/v0_23_0_publication_evidence_audit_path.md \
  scripts/ai/verify_v23_1_release_gates.sh \
  scripts/ai/verify_v23_1_strict_provenance.sh; do
  require_file "$path"
done

for task_id in V231-001 V231-002 V231-003 V231-004 V231-005 V231-006; do
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
  "v0.23.1 is a patch closeout release" \
  "This release does not add submit capability" \
  "This release is not a product-grade live trading terminal" \
  "V231-001" \
  "V231-006" \
  "scripts/ai/verify_release.sh v23.1-release-gates" \
  "scripts/ai/verify_release.sh v23.1-strict-provenance" \
  "scripts/ai/verify_v23_1_release_gates.sh" \
  "scripts/ai/verify_v23_1_strict_provenance.sh" \
  "v0.24.0 remains blocked"; do
  require_contains "$RELEASE_NOTES_PATH" "$marker"
done

for marker in \
  "Milestone: \`$RELEASE_TAG\`" \
  "Status: RELEASED" \
  "V231-001 evidence" \
  "V231-006 evidence" \
  "v23.1 release gates = required" \
  "v23.1 strict provenance = required" \
  "release surface current guard = required" \
  "v0.23.0 GitHub Release = required published" \
  "#742 V231-006 = stays open until tag, hosted gate, public release, and publication evidence are recorded" \
  "No V240 implementation starts until all V231 issues are closed and v0.23.1 release evidence is published"; do
  require_contains "$READINESS_REPORT_PATH" "$marker"
done

stale_markers=(
  "ntpro-rust-only-v0.23.0-candidate"
  "public release publication = pending"
  "tag gate run = pending"
  "tag gate result = pending"
  "RELEASE GATE CORRECTIVE FIX IN PROGRESS"
  "corrective fix in progress"
)

for path in "$RELEASE_NOTES_PATH" "$READINESS_REPORT_PATH" "$MANIFEST_PATH"; do
  for marker in "${stale_markers[@]}"; do
    require_not_contains "$path" "$marker"
  done
done

scripts/ai/ntpro_governance.sh golden-trace-release-scope
scripts/ai/verify_release.sh v23.1-release-closeout-evidence
scripts/ai/verify_release.sh v23.1-stale-provenance-cleanup
scripts/ai/verify_release.sh v23.1-gate-phase-split
scripts/ai/verify_release.sh v23.1-evidence-replay-only-boundary
scripts/ai/verify_release.sh v23.1-publication-evidence-audit-path
NTPRO_RELEASE_SURFACE_ALLOW_MISSING_TAG=1 scripts/ai/verify_release.sh release-surface-current-guard
scripts/ai/verify_release.sh release-publish-after-gate

RELEASE_VERSION="$RELEASE_VERSION" \
RELEASE_TAG="$RELEASE_TAG" \
RELEASE_NAME="$RELEASE_NAME" \
MANIFEST_PATH="$MANIFEST_PATH" \
python3 <<'PY'
import copy
import json
import os
from pathlib import Path

manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def require_false(mapping: dict, key: str) -> None:
    require(mapping.get(key) is False, f"boundary flag must be false: {key}")


def require_true(mapping: dict, key: str) -> None:
    require(mapping.get(key) is True, f"capability flag must be true: {key}")


def validate(candidate: dict) -> None:
    require(candidate.get("schema_version") == "ntpro.v231_patch_release_manifest.v1", "manifest schema mismatch")
    require(candidate.get("task_id") == "V231-006", "manifest task mismatch")
    require(candidate.get("product_version") == os.environ["RELEASE_VERSION"], "manifest product version mismatch")
    require(candidate.get("release_status") == "published", "manifest release status mismatch")
    require(candidate.get("patch_scope") == "v230_post_release_governance_closeout_patch", "manifest patch scope mismatch")

    base = candidate.get("base_release") or {}
    require(base.get("tag") == "ntpro-rust-only-v0.23.0", "base release tag mismatch")
    require(Path(base.get("release_manifest_path", "")).is_file(), "base release manifest missing")
    require(Path(base.get("closeout_evidence_path", "")).is_file(), "base closeout evidence missing")

    planned = candidate.get("planned_release") or {}
    require(planned.get("tag") == os.environ["RELEASE_TAG"], "planned release tag mismatch")
    require(planned.get("name") == os.environ["RELEASE_NAME"], "planned release name mismatch")
    require(planned.get("github_release_url") == f"https://github.com/atxinbao/NTPRO/releases/tag/{os.environ['RELEASE_TAG']}", "planned release URL mismatch")
    require(planned.get("target_commitish") == "main", "planned target mismatch")
    require(planned.get("draft") is False, "planned draft flag mismatch")
    require(planned.get("prerelease") is False, "planned prerelease flag mismatch")

    expected_evidence = {
        "V231-001": 737,
        "V231-002": 738,
        "V231-003": 739,
        "V231-004": 740,
        "V231-005": 741,
        "V231-006": 742,
    }
    evidence = candidate.get("v231_evidence") or []
    require(len(evidence) == len(expected_evidence), "V231 evidence count mismatch")
    for item in evidence:
        task_id = item.get("task_id")
        require(expected_evidence.get(task_id) == item.get("issue"), f"V231 evidence issue mismatch: {task_id}")
        path = Path(item.get("path", ""))
        require(path.is_file(), f"V231 evidence file missing: {path}")
        require(task_id in path.read_text(encoding="utf-8"), f"V231 evidence task marker missing: {path}")

    inputs = candidate.get("release_inputs") or {}
    for key in (
        "release_notes_path",
        "readiness_report_path",
        "release_manifest_path",
        "base_release_manifest_path",
        "base_release_closeout_evidence_path",
        "gate_phase_split_path",
        "evidence_replay_only_boundary_path",
        "publication_evidence_audit_path",
        "golden_trace_manifest_path",
        "v231_release_gates_path",
        "v231_strict_provenance_path",
    ):
        path = Path(inputs.get(key, ""))
        require(path.is_file(), f"release input missing: {key} -> {path}")

    commands = {
        gate.get("command")
        for gate in candidate.get("release_gates", [])
        if gate.get("required") is True
    }
    for command in (
        "scripts/ai/verify_release.sh v23.1-release-closeout-evidence",
        "scripts/ai/verify_release.sh v23.1-stale-provenance-cleanup",
        "scripts/ai/verify_release.sh v23.1-gate-phase-split",
        "scripts/ai/verify_release.sh v23.1-evidence-replay-only-boundary",
        "scripts/ai/verify_release.sh v23.1-publication-evidence-audit-path",
        "scripts/ai/verify_release.sh v23-release-gates",
        "scripts/ai/verify_release.sh v23-strict-provenance",
        "scripts/ai/verify_release.sh release-surface-current-guard",
        "scripts/ai/verify_release.sh release-publish-after-gate",
        "scripts/ai/verify_release.sh v23.1-release-gates",
        "scripts/ai/verify_release.sh v23.1-strict-provenance",
        "scripts/ai/verify_v23_1_release_gates.sh",
        "scripts/ai/verify_v23_1_strict_provenance.sh",
    ):
        require(command in commands, f"required release gate missing: {command}")

    capability = candidate.get("capability") or {}
    for key in (
        "patch_closeout_only",
        "release_governance_hardening",
        "strict_provenance",
        "gate_before_publish",
        "v0_24_start_gate_defined",
    ):
        require_true(capability, key)
    require_false(capability, "v0_24_implementation_started")
    for key in (
        "product_grade_live_trading_terminal",
        "new_submit_capability",
        "production_order_mutation_expansion",
        "dashboard_operation_controls",
        "complete_executable_read_model_runtime",
    ):
        require_false(capability, key)

    boundary = candidate.get("boundary_flags") or {}
    for key in (
        "new_submit_capability",
        "production_order_submission_allowed",
        "production_order_mutation_allowed",
        "ungated_submit_allowed",
        "ungated_cancel_allowed",
        "ungated_retry_allowed",
        "ungated_replace_allowed",
        "ungated_amend_allowed",
        "ungated_flatten_allowed",
        "automatic_cancel_allowed",
        "automatic_remediation_allowed",
        "automatic_operation_action_allowed",
        "strategy_driven_production_execution_allowed",
        "cross_account_implicit_operation_allowed",
        "cross_strategy_implicit_operation_allowed",
        "cross_venue_implicit_operation_allowed",
        "cross_node_implicit_operation_allowed",
        "shared_approval_consumption_allowed",
        "dashboard_order_controls_enabled",
        "dashboard_approval_controls_enabled",
        "dashboard_cancel_controls_enabled",
        "dashboard_retry_controls_enabled",
        "dashboard_submit_controls_enabled",
        "dashboard_replace_controls_enabled",
        "dashboard_amend_controls_enabled",
        "dashboard_flatten_controls_enabled",
        "trader_terminal_order_ticket_enabled",
        "trader_terminal_live_trading_claim",
        "manual_operation_entry_enabled",
        "manual_operation_submit_allowed",
        "manual_operation_cancel_allowed",
        "manual_operation_retry_allowed",
        "manual_operation_replace_allowed",
        "manual_operation_amend_allowed",
        "manual_operation_flatten_allowed",
        "product_grade_trading_terminal_claim",
    ):
        require_false(boundary, key)

    publication = candidate.get("publication_governance") or {}
    require_true(publication, "gate_before_publish")
    require_true(publication, "public_release_requires_successful_hosted_gate_for_same_tag_commit")
    require_true(publication, "release_gate_success_before_publication_required")
    require(publication.get("publication_evidence_strategy") == "source_tree_plus_github_remote", "publication strategy mismatch")
    require(publication.get("local_generated_evidence_required_in_source_tree") is False, "local generated evidence must not be required")
    require(publication.get("remote_reconstruction_required") is True, "remote reconstruction must be required")

    surface = candidate.get("release_surface_current_guard") or {}
    require(surface.get("current_release_version") == "v0.23.1", "release surface version mismatch")
    require(surface.get("current_release_tag") == "ntpro-rust-only-v0.23.1", "release surface tag mismatch")
    require(surface.get("next_capability_version") == "v0.24.0", "next capability mismatch")

    next_tracks = candidate.get("next_tracks") or {}
    require(next_tracks.get("capability") == "v0.24.0", "next capability track mismatch")
    require(next_tracks.get("start_gate") == "blocked_until_v231_release_evidence_published", "v0.24 start gate mismatch")
    require(next_tracks.get("implementation_started") is False, "v0.24 implementation must not be started")
    require(next_tracks.get("inherits_production_submit") is False, "v0.24 must not inherit production submit")
    require(next_tracks.get("inherits_dashboard_operation_controls") is False, "v0.24 must not inherit Dashboard controls")


validate(manifest)

if os.environ.get("NTPRO_V231_RELEASE_SELFTEST", "1") == "1":
    missing_evidence = copy.deepcopy(manifest)
    missing_evidence["v231_evidence"] = missing_evidence["v231_evidence"][:-1]
    try:
        validate(missing_evidence)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: missing V231 evidence")

    opened_boundary = copy.deepcopy(manifest)
    opened_boundary["boundary_flags"]["dashboard_submit_controls_enabled"] = True
    try:
        validate(opened_boundary)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: open Dashboard submit boundary")
PY

if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
  for issue in 737 738 739 740 741; do
    state="$(gh_with_retry issue view "$issue" --repo "$REPO" --json state --jq .state)" || fail "could not read GitHub issue #$issue"
    [[ "$state" == "CLOSED" ]] || fail "GitHub issue #$issue must be CLOSED before $RELEASE_VERSION release gates, got $state"
  done
  current_state="$(gh_with_retry issue view 742 --repo "$REPO" --json state --jq .state)" || fail "could not read GitHub issue #742"
  [[ "$current_state" == "OPEN" || "$current_state" == "CLOSED" ]] || fail "unexpected current release issue state: $current_state"
else
  fail "gh authentication is required for v23.1 release gate issue proof"
fi

echo "v23_1_release_gates status=ok release_tag=$RELEASE_TAG base_release=$BASE_RELEASE_TAG current_issue_state=$current_state negative_selftest=${NTPRO_V231_RELEASE_SELFTEST:-1}"
