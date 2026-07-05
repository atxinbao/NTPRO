#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V241_RELEASE_REPO:-atxinbao/NTPRO}"
RELEASE_VERSION="${NTPRO_V241_RELEASE_VERSION:-v0.24.1}"
RELEASE_TAG="${NTPRO_V241_RELEASE_TAG:-ntpro-rust-only-v0.24.1}"
RELEASE_NAME="${NTPRO_V241_RELEASE_NAME:-NTPRO Rust-only v0.24.1}"
BASE_RELEASE_TAG="${NTPRO_V241_BASE_RELEASE_TAG:-ntpro-rust-only-v0.24.0}"
MANIFEST_PATH="${NTPRO_V241_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_24_1_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V241_RELEASE_NOTES:-docs/rust-cutover/release/v0_24_1_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V241_READINESS_REPORT:-docs/rust-cutover/release/v0_24_1_readiness_report.md}"
CURRENT_ISSUE="${NTPRO_V241_CURRENT_ISSUE:-775}"
MILESTONE_NUMBER="${NTPRO_V241_MILESTONE_NUMBER:-15}"

fail() {
  echo "v24.1 release gate failed: $*" >&2
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
  docs/rust-cutover/release/v0_24_0_release_manifest.json \
  docs/rust-cutover/release/v0_24_0_release_closeout_evidence.md \
  docs/rust-cutover/release/v0_24_0_provenance_reconciliation.md \
  docs/rust-cutover/release/v0_24_1_schema_replay_classification.md \
  docs/rust-cutover/release/v0_24_1_dashboard_artifact_ingestion.md \
  docs/rust-cutover/release/v0_24_1_dashboard_fixture_ref_integrity.md \
  docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json \
  tests/golden/v241_dashboard_order_control_artifact_ingestion.json \
  scripts/ai/verify_v24_1_release_gates.sh \
  scripts/ai/verify_v24_1_strict_provenance.sh; do
  require_file "$path"
done

for task_id in V241-001 V241-002 V241-003 V241-004 V241-005 V241-006 V241-007; do
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
  "v0.24.1 is a hardening patch" \
  "This release does not add submit capability" \
  "This release is not a product-grade live trading terminal" \
  "V241-001" \
  "V241-007" \
  "v24.1 release gates = required" \
  "v24.1 strict provenance = required" \
  "release surface current guard = required" \
  "release publication guard = required" \
  "release publish after gate = required" \
  "publication evidence strategy = source_tree_plus_github_remote" \
  "v0.25.0 start gate = blocked until v0.24.1 release evidence is published" \
  "scripts/ai/verify_release.sh v24.1-release-gates" \
  "scripts/ai/verify_release.sh v24.1-strict-provenance" \
  "scripts/ai/verify_v24_1_release_gates.sh" \
  "scripts/ai/verify_v24_1_strict_provenance.sh" \
  "scripts/ai/publish_ntpro_release_after_gate.sh"; do
  require_contains "$RELEASE_NOTES_PATH" "$marker"
done

for marker in \
  "Milestone: \`$RELEASE_TAG\`" \
  "Status: RELEASED" \
  "V241-001 evidence" \
  "V241-007 evidence" \
  "v24.1 release gates = required" \
  "v24.1 strict provenance = required" \
  "release surface current guard = required" \
  "release publication guard = required" \
  "release publish after gate = required" \
  "#775 V241-006 = must be closed before v0.24.1 tag gate is accepted" \
  "No V250 implementation starts until all V241 issues are closed and v0.24.1 release evidence is published"; do
  require_contains "$READINESS_REPORT_PATH" "$marker"
done

for path in "$RELEASE_NOTES_PATH" "$READINESS_REPORT_PATH" "$MANIFEST_PATH"; do
  for marker in \
    "new_submit_capability = true" \
    "production_order_submission_allowed = true" \
    "production_order_mutation_allowed = true" \
    "execution_adapter_call_allowed = true" \
    "live_exchange_request_allowed = true" \
    "retry_scheduler_enabled = true" \
    "dashboard_operation_controls_enabled = true" \
    "trader_terminal_order_ticket_enabled = true" \
    "manual_operation_submit_allowed = true" \
    "product_grade_trading_terminal_claim = true"; do
    require_not_contains "$path" "$marker"
  done
done

scripts/ai/verify_release.sh v24.1-release-closeout-evidence
scripts/ai/verify_release.sh v24.1-provenance-reconciliation
scripts/ai/verify_release.sh v24.1-stale-pretag-cleanup
scripts/ai/verify_release.sh v24.1-schema-replay-classification
scripts/ai/verify_release.sh v24.1-dashboard-artifact-ingestion
scripts/ai/verify_release.sh v24.1-dashboard-fixture-ref-integrity
NTPRO_RELEASE_SURFACE_ALLOW_MISSING_TAG=1 scripts/ai/verify_release.sh release-surface-current-guard
NTPRO_RELEASE_PUBLICATION_ALLOW_OFFLINE=1 scripts/ai/verify_release.sh release-publication-guard
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
    require(candidate.get("schema_version") == "ntpro.v241_patch_release_manifest.v1", "manifest schema mismatch")
    require(candidate.get("task_id") == "V241-006", "manifest task mismatch")
    require(candidate.get("product_version") == os.environ["RELEASE_VERSION"], "manifest product version mismatch")
    require(candidate.get("release_status") == "released", "manifest release status mismatch")
    require(candidate.get("patch_scope") == "v240_post_release_governance_and_dashboard_hardening_patch", "manifest patch scope mismatch")

    base = candidate.get("base_release") or {}
    require(base.get("tag") == "ntpro-rust-only-v0.24.0", "base release tag mismatch")
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
        "V241-001": 770,
        "V241-002": 771,
        "V241-003": 772,
        "V241-004": 773,
        "V241-005": 774,
        "V241-006": 775,
        "V241-007": 776,
    }
    evidence = candidate.get("v241_evidence") or []
    require(len(evidence) == len(expected_evidence), "V241 evidence count mismatch")
    for item in evidence:
        task_id = item.get("task_id")
        require(expected_evidence.get(task_id) == item.get("issue"), f"V241 evidence issue mismatch: {task_id}")
        path = Path(item.get("path", ""))
        require(path.is_file(), f"V241 evidence file missing: {path}")
        require(task_id in path.read_text(encoding="utf-8"), f"V241 evidence task marker missing: {path}")

    inputs = candidate.get("release_inputs") or {}
    for key in (
        "release_notes_path",
        "readiness_report_path",
        "release_manifest_path",
        "base_release_manifest_path",
        "base_release_closeout_evidence_path",
        "provenance_reconciliation_path",
        "schema_replay_classification_path",
        "dashboard_artifact_ingestion_path",
        "dashboard_fixture_ref_integrity_path",
        "golden_trace_manifest_path",
        "dashboard_artifact_fixture_path",
        "v241_release_gates_path",
        "v241_strict_provenance_path",
    ):
        path = Path(inputs.get(key, ""))
        require(path.is_file(), f"release input missing: {key} -> {path}")

    commands = {
        gate.get("command")
        for gate in candidate.get("release_gates", [])
        if gate.get("required") is True
    }
    for command in (
        "scripts/ai/verify_release.sh v24.1-release-closeout-evidence",
        "scripts/ai/verify_release.sh v24.1-provenance-reconciliation",
        "scripts/ai/verify_release.sh v24.1-stale-pretag-cleanup",
        "scripts/ai/verify_release.sh v24.1-schema-replay-classification",
        "scripts/ai/verify_release.sh v24.1-dashboard-artifact-ingestion",
        "scripts/ai/verify_release.sh v24.1-dashboard-fixture-ref-integrity",
        "scripts/ai/verify_release.sh v24.1-release-gates",
        "scripts/ai/verify_release.sh v24.1-strict-provenance",
        "scripts/ai/verify_release.sh release-surface-current-guard",
        "scripts/ai/verify_release.sh release-publication-guard",
        "scripts/ai/verify_release.sh release-publish-after-gate",
        "scripts/ai/verify_v24_1_release_gates.sh",
        "scripts/ai/verify_v24_1_strict_provenance.sh",
    ):
        require(command in commands, f"required release gate missing: {command}")

    capability = candidate.get("capability") or {}
    for key in (
        "patch_hardening_only",
        "release_governance_hardening",
        "strict_provenance",
        "schema_replay_classification_hardening",
        "dashboard_artifact_ingestion_hardening",
        "dashboard_fixture_ref_integrity_hardening",
        "gate_before_publish",
        "v0_25_start_gate_defined",
    ):
        require_true(capability, key)
    for key in (
        "v0_25_implementation_started",
        "product_grade_live_trading_terminal",
        "complete_executable_order_control_runtime",
        "new_submit_capability",
        "production_order_mutation_expansion",
        "dashboard_operation_controls",
    ):
        require_false(capability, key)

    boundary = candidate.get("boundary_flags") or {}
    for key in (
        "new_submit_capability",
        "production_order_submission_allowed",
        "production_order_mutation_allowed",
        "execution_adapter_call_allowed",
        "live_exchange_request_allowed",
        "network_attempted",
        "implicit_retry_allowed",
        "retry_scheduler_enabled",
        "automatic_cancel_allowed",
        "automatic_retry_allowed",
        "automatic_remediation_allowed",
        "automatic_operation_action_allowed",
        "strategy_driven_production_execution_allowed",
        "shared_approval_consumption_allowed",
        "cancel_replace_amend_send_allowed",
        "flatten_allowed",
        "dashboard_operation_controls_enabled",
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
    require(surface.get("current_release_version") == "v0.24.1", "release surface version mismatch")
    require(surface.get("current_release_tag") == "ntpro-rust-only-v0.24.1", "release surface tag mismatch")
    require(surface.get("next_patch_version") == "v0.24.2", "next patch mismatch")
    require(surface.get("next_capability_version") == "v0.25.0", "next capability mismatch")

    next_tracks = candidate.get("next_tracks") or {}
    require(next_tracks.get("capability") == "v0.25.0", "next capability track mismatch")
    require(next_tracks.get("start_gate") == "blocked_until_v241_release_evidence_published", "v0.25 start gate mismatch")
    require(next_tracks.get("implementation_started") is False, "v0.25 implementation must not be started")
    require(next_tracks.get("inherits_production_submit") is False, "v0.25 must not inherit production submit")
    require(next_tracks.get("inherits_production_mutation") is False, "v0.25 must not inherit production mutation")
    require(next_tracks.get("inherits_retry_scheduler") is False, "v0.25 must not inherit retry scheduler")
    require(next_tracks.get("inherits_dashboard_operation_controls") is False, "v0.25 must not inherit Dashboard controls")

    requirements = candidate.get("post_publication_requirements") or {}
    require(requirements.get("release_tag") == os.environ["RELEASE_TAG"], "post-publication release tag mismatch")
    require(requirements.get("milestone_number") == 15, "post-publication milestone number mismatch")
    require(requirements.get("all_v241_issues_closed_required") is True, "issue closeout requirement missing")
    require(requirements.get("github_release_published_required") is True, "GitHub release publication requirement missing")
    require(requirements.get("v0_25_start_gate_fails_without_v241_release_evidence") is True, "v0.25 start gate requirement missing")


validate(manifest)

if os.environ.get("NTPRO_V241_RELEASE_SELFTEST", "1") == "1":
    missing_evidence = copy.deepcopy(manifest)
    missing_evidence["v241_evidence"] = missing_evidence["v241_evidence"][:-1]
    try:
        validate(missing_evidence)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: missing V241 evidence")

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
  for issue in 770 771 772 773 774 776; do
    state="$(gh_with_retry issue view "$issue" --repo "$REPO" --json state --jq .state)" || fail "could not read GitHub issue #$issue"
    [[ "$state" == "CLOSED" ]] || fail "GitHub issue #$issue must be CLOSED before $RELEASE_VERSION release gates, got $state"
  done
  current_state="$(gh_with_retry issue view "$CURRENT_ISSUE" --repo "$REPO" --json state --jq .state)" || fail "could not read GitHub issue #$CURRENT_ISSUE"
  if [[ "${NTPRO_RELEASE_GATE:-0}" == "1" || "${NTPRO_V241_RELEASE_REQUIRE_CLOSEOUT:-0}" == "1" ]]; then
    [[ "$current_state" == "CLOSED" ]] || fail "GitHub issue #$CURRENT_ISSUE must be CLOSED for the tag gate, got $current_state"
  else
    [[ "$current_state" == "OPEN" || "$current_state" == "CLOSED" ]] || fail "unexpected current release issue state: $current_state"
  fi

  milestone_json="$(gh_with_retry api "/repos/$REPO/milestones/$MILESTONE_NUMBER")" || fail "could not read GitHub milestone #$MILESTONE_NUMBER"
  MILESTONE_JSON="$milestone_json" \
  RELEASE_GATE="${NTPRO_RELEASE_GATE:-0}" \
  REQUIRE_CLOSEOUT="${NTPRO_V241_RELEASE_REQUIRE_CLOSEOUT:-0}" \
  python3 <<'PY'
import json
import os

milestone = json.loads(os.environ["MILESTONE_JSON"])
if milestone["title"] != "v0.24.1":
    raise SystemExit(milestone)
if os.environ["RELEASE_GATE"] == "1" or os.environ["REQUIRE_CLOSEOUT"] == "1":
    if milestone["state"] != "closed" or milestone["open_issues"] != 0 or milestone["closed_issues"] != 7:
        raise SystemExit(f"v0.24.1 milestone must be closed with 7 closed issues for release gate: {milestone}")
else:
    if milestone["state"] not in {"open", "closed"}:
        raise SystemExit(milestone)
PY
else
  fail "gh authentication is required for v24.1 release gate issue proof"
fi

if [[ "${NTPRO_V241_RELEASE_REQUIRE_PUBLICATION:-0}" == "1" ]]; then
  NTPRO_CURRENT_RELEASE_VERSION="$RELEASE_VERSION" \
    NTPRO_CURRENT_RELEASE_TAG="$RELEASE_TAG" \
    NTPRO_CURRENT_RELEASE_NAME="$RELEASE_NAME" \
    NTPRO_RELEASE_PUBLICATION_STRICT_BODY=1 \
    scripts/ai/check_github_release_published.sh
fi

echo "v24_1_release_gates status=ok release_tag=$RELEASE_TAG base_release=$BASE_RELEASE_TAG current_issue_state=$current_state negative_selftest=${NTPRO_V241_RELEASE_SELFTEST:-1}"
