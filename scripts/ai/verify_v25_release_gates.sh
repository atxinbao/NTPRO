#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V250_RELEASE_REPO:-atxinbao/NTPRO}"
RELEASE_VERSION="${NTPRO_V250_RELEASE_VERSION:-v0.25.0}"
RELEASE_TAG="${NTPRO_V250_RELEASE_TAG:-ntpro-rust-only-v0.25.0}"
RELEASE_NAME="${NTPRO_V250_RELEASE_NAME:-NTPRO Rust-only v0.25.0}"
BASE_RELEASE_TAG="${NTPRO_V250_BASE_RELEASE_TAG:-ntpro-rust-only-v0.24.1}"
MANIFEST_PATH="${NTPRO_V250_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_25_0_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V250_RELEASE_NOTES:-docs/rust-cutover/release/v0_25_0_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V250_READINESS_REPORT:-docs/rust-cutover/release/v0_25_0_readiness_report.md}"
CURRENT_ISSUE="${NTPRO_V250_CURRENT_ISSUE:-785}"
MILESTONE_NUMBER="${NTPRO_V250_MILESTONE_NUMBER:-16}"

fail() {
  echo "v25 release gate failed: $*" >&2
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
  docs/rust-cutover/release/v0_24_1_release_manifest.json \
  docs/rust-cutover/release/v0_24_1_readiness_report.md \
  docs/rust-cutover/release/v0_25_0_intake_gate.md \
  docs/rust-cutover/release/v0_25_0_monitoring_observability_contract.md \
  docs/rust-cutover/release/v0_25_0_alert_taxonomy_routing.md \
  docs/rust-cutover/release/v0_25_0_incident_lifecycle_acknowledgement.md \
  docs/rust-cutover/release/v0_25_0_runbook_audit_evidence.md \
  docs/rust-cutover/release/v0_25_0_dr_preview_drill_evidence.md \
  docs/rust-cutover/release/v0_25_0_dashboard_monitoring_surface.md \
  docs/rust-cutover/release/v0_25_0_slo_freshness_diagnostics_gate.md \
  docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json \
  scripts/ai/verify_v25_release_gates.sh \
  scripts/ai/verify_v25_strict_provenance.sh; do
  require_file "$path"
done

for task_id in V250-000 V250-001 V250-002 V250-003 V250-004 V250-005 V250-006 V250-007 V250-008 V250-009; do
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
  "v0.25.0 publishes the Monitoring, Incident, and Disaster-Recovery Foundation" \
  "This release does not add submit capability" \
  "This release is not a product-grade live trading terminal" \
  "V250-000" \
  "V250-008" \
  "V250-009" \
  "V250 final release scope issue count = 10" \
  "V250 final release scope evidence count = 10" \
  "V250-009 failed release gate run = https://github.com/atxinbao/NTPRO/actions/runs/28762387835" \
  "V250-009 final success release gate run = https://github.com/atxinbao/NTPRO/actions/runs/28764231552" \
  "V250-009 capability expansion = false" \
  "v25 release gates = required" \
  "v25 strict provenance = required" \
  "release surface current guard = required" \
  "release publication guard = required" \
  "release publish after gate = required" \
  "publication evidence strategy = source_tree_plus_github_remote" \
  "local generated publication evidence required in source tree = false" \
  "remote reconstruction required = true" \
  "new_submit_capability = false" \
  "production_order_mutation_allowed = false" \
  "execution_adapter_call_allowed = false" \
  "adapter_send_allowed = false" \
  "live_exchange_request_allowed = false" \
  "retry_scheduler_enabled = false" \
  "dashboard_operation_controls_enabled = false" \
  "dashboard_trading_controls_enabled = false" \
  "trader_terminal_order_ticket_enabled = false" \
  "product_grade_trading_terminal_claim = false" \
  "scripts/ai/verify_release.sh v25-release-gates" \
  "scripts/ai/verify_release.sh v25-strict-provenance" \
  "scripts/ai/verify_v25_release_gates.sh" \
  "scripts/ai/verify_v25_strict_provenance.sh" \
  "scripts/ai/publish_ntpro_release_after_gate.sh"; do
  require_contains "$RELEASE_NOTES_PATH" "$marker"
done

for marker in \
  "Milestone: \`$RELEASE_TAG\`" \
  "Status: RELEASED" \
  "V250-000 evidence" \
  "V250-008 evidence" \
  "V250-009 corrective evidence" \
  "v25 release gates = required" \
  "v25 strict provenance = required" \
  "release surface current guard = required" \
  "release publication guard = required" \
  "release publish after gate = required" \
  "#785 V250-008 = closed" \
  "#804 V250-009 corrective issue = closed" \
  "failed release gate run = https://github.com/atxinbao/NTPRO/actions/runs/28762387835" \
  "final success release gate run = https://github.com/atxinbao/NTPRO/actions/runs/28764231552" \
  "V250 corrective issue set = #804 closed before final publication" \
  "V250 final release scope issue count = 10" \
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

scripts/ai/verify_release.sh v25-intake-gate
scripts/ai/verify_release.sh v25-monitoring-observability-contract
scripts/ai/verify_release.sh v25-alert-taxonomy-routing
scripts/ai/verify_release.sh v25-incident-lifecycle-acknowledgement
scripts/ai/verify_release.sh v25-runbook-audit-evidence
scripts/ai/verify_release.sh v25-dr-preview-drill-evidence
scripts/ai/verify_release.sh v25-dashboard-monitoring-surface
scripts/ai/verify_release.sh v25-slo-freshness-diagnostics-gate
if [[ "${NTPRO_V250_RELEASE_SKIP_CURRENT_SURFACE_GUARD:-0}" == "1" ]]; then
  echo "v25_release_gates historical_current_surface_guard=skipped reason=current_release_surface_superseded"
else
  NTPRO_CURRENT_RELEASE_VERSION="$RELEASE_VERSION" \
    NTPRO_CURRENT_RELEASE_TAG="$RELEASE_TAG" \
    NTPRO_NEXT_PATCH_VERSION="v0.25.1" \
    NTPRO_NEXT_CAPABILITY_VERSION="v0.26.0" \
    NTPRO_CURRENT_RELEASE_CAPABILITY="v0.25.0 Monitoring, Incident, and Disaster-Recovery Foundation" \
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
    require(mapping.get(key) is False, f"flag must be false: {key}")


def require_true(mapping: dict, key: str) -> None:
    require(mapping.get(key) is True, f"flag must be true: {key}")


def validate(candidate: dict) -> None:
    require(candidate.get("schema_version") == "ntpro.v250_release_manifest.v1", "manifest schema mismatch")
    require(candidate.get("task_id") == "V250-008", "manifest task mismatch")
    require(candidate.get("product_version") == os.environ["RELEASE_VERSION"], "manifest product version mismatch")
    require(candidate.get("release_status") == "released", "manifest release status mismatch")
    require(candidate.get("capability_scope") == "monitoring_incident_disaster_recovery_foundation", "capability scope mismatch")

    base = candidate.get("base_release") or {}
    require(base.get("tag") == "ntpro-rust-only-v0.24.1", "base release tag mismatch")
    require(Path(base.get("release_manifest_path", "")).is_file(), "base release manifest missing")
    require(Path(base.get("readiness_report_path", "")).is_file(), "base readiness report missing")

    planned = candidate.get("planned_release") or {}
    require(planned.get("tag") == os.environ["RELEASE_TAG"], "planned release tag mismatch")
    require(planned.get("name") == os.environ["RELEASE_NAME"], "planned release name mismatch")
    require(planned.get("github_release_url") == f"https://github.com/atxinbao/NTPRO/releases/tag/{os.environ['RELEASE_TAG']}", "planned release URL mismatch")
    require(planned.get("target_commitish") == "main", "planned target mismatch")
    require(planned.get("draft") is False, "planned draft flag mismatch")
    require(planned.get("prerelease") is False, "planned prerelease flag mismatch")

    expected_evidence = {
        "V250-000": 777,
        "V250-001": 778,
        "V250-002": 779,
        "V250-003": 780,
        "V250-004": 781,
        "V250-005": 782,
        "V250-006": 783,
        "V250-007": 784,
        "V250-008": 785,
        "V250-009": 804,
    }
    evidence = candidate.get("v250_evidence") or []
    require(len(evidence) == len(expected_evidence), "V250 evidence count mismatch")
    for item in evidence:
        task_id = item.get("task_id")
        require(expected_evidence.get(task_id) == item.get("issue"), f"V250 evidence issue mismatch: {task_id}")
        path = Path(item.get("path", ""))
        require(path.is_file(), f"V250 evidence file missing: {path}")
        require(task_id in path.read_text(encoding="utf-8"), f"V250 evidence task marker missing: {path}")

    release_scope = candidate.get("release_scope") or {}
    require(release_scope.get("milestone_issue_count") == 9, "milestone issue count mismatch")
    require(release_scope.get("corrective_issue_count") == 1, "corrective issue count mismatch")
    require(release_scope.get("final_release_scope_issue_count") == 10, "final release scope issue count mismatch")
    require(release_scope.get("final_release_scope_evidence_count") == 10, "final release scope evidence count mismatch")
    require(release_scope.get("corrective_issue") == 804, "corrective issue mismatch")
    require(release_scope.get("corrective_pull_request") == 805, "corrective pull request mismatch")
    require(release_scope.get("corrective_failed_run_id") == 28762387835, "corrective failed run mismatch")
    require(release_scope.get("final_success_run_id") == 28764231552, "final success run mismatch")
    require(release_scope.get("corrective_scope_expands_capability") is False, "corrective scope must not expand capability")
    require(release_scope.get("corrective_scope_changes_runtime_behavior") is False, "corrective scope must not change runtime behavior")
    require(release_scope.get("corrective_scope_changes_trading_behavior") is False, "corrective scope must not change trading behavior")

    corrective = candidate.get("corrective_release_scope") or {}
    require(corrective.get("task_id") == "V250-009", "corrective task mismatch")
    require(corrective.get("issue") == 804, "corrective issue mismatch")
    require(corrective.get("task_path") == "docs/rust-cutover/tasks/V250-009.md", "corrective task path mismatch")
    require(corrective.get("evidence_path") == "docs/rust-cutover/evidence/V250-009.md", "corrective evidence path mismatch")
    require(corrective.get("pull_request") == 805, "corrective PR mismatch")
    require(corrective.get("failed_release_gate_run") == 28762387835, "corrective failed release run mismatch")
    require(corrective.get("final_success_release_gate_run") == 28764231552, "corrective final success run mismatch")
    require(corrective.get("merge_commit") == "eedcdab1d3ca85d6f51b368b5f36208a7b591026", "corrective merge commit mismatch")
    require(corrective.get("included_in_release_tag") is True, "corrective scope must be included in release tag")
    require(corrective.get("capability_expansion") is False, "corrective scope must not expand capability")
    require(corrective.get("runtime_behavior_changed") is False, "corrective scope must not change runtime behavior")
    require(corrective.get("trading_behavior_changed") is False, "corrective scope must not change trading behavior")

    inputs = candidate.get("release_inputs") or {}
    for key in (
        "release_notes_path",
        "readiness_report_path",
        "release_manifest_path",
        "base_release_manifest_path",
        "base_readiness_report_path",
        "golden_trace_manifest_path",
        "v25_intake_path",
        "v25_monitoring_path",
        "v25_alert_path",
        "v25_incident_path",
        "v25_runbook_path",
        "v25_dr_preview_path",
        "v25_dashboard_path",
        "v25_slo_path",
        "v250_release_gates_path",
        "v250_strict_provenance_path",
    ):
        path = Path(inputs.get(key, ""))
        require(path.is_file(), f"release input missing: {key} -> {path}")

    commands = {
        gate.get("command")
        for gate in candidate.get("release_gates", [])
        if gate.get("required") is True
    }
    for command in (
        "scripts/ai/verify_release.sh v25-intake-gate",
        "scripts/ai/verify_release.sh v25-monitoring-observability-contract",
        "scripts/ai/verify_release.sh v25-alert-taxonomy-routing",
        "scripts/ai/verify_release.sh v25-incident-lifecycle-acknowledgement",
        "scripts/ai/verify_release.sh v25-runbook-audit-evidence",
        "scripts/ai/verify_release.sh v25-dr-preview-drill-evidence",
        "scripts/ai/verify_release.sh v25-dashboard-monitoring-surface",
        "scripts/ai/verify_release.sh v25-slo-freshness-diagnostics-gate",
        "scripts/ai/verify_release.sh v25-release-gates",
        "scripts/ai/verify_release.sh v25-strict-provenance",
        "scripts/ai/verify_release.sh release-surface-current-guard",
        "scripts/ai/verify_release.sh release-publication-guard",
        "scripts/ai/verify_release.sh release-publish-after-gate",
        "scripts/ai/verify_v25_release_gates.sh",
        "scripts/ai/verify_v25_strict_provenance.sh",
    ):
        require(command in commands, f"required release gate missing: {command}")

    capability = candidate.get("capability") or {}
    for key in (
        "monitoring_incident_dr_foundation",
        "release_governance",
        "strict_provenance",
        "monitoring_observability_contract",
        "alert_taxonomy_routing",
        "incident_lifecycle_acknowledgement",
        "runbook_audit_evidence",
        "dr_preview_drill_evidence",
        "dashboard_monitoring_readonly_surface",
        "slo_freshness_diagnostics_gate",
        "gate_before_publish",
    ):
        require_true(capability, key)
    for key in (
        "product_grade_live_trading_terminal",
        "new_submit_capability",
        "production_order_mutation_expansion",
        "dashboard_trading_controls",
        "automatic_remediation_runtime",
    ):
        require_false(capability, key)

    boundary = candidate.get("boundary_flags") or {}
    for key in (
        "new_submit_capability",
        "production_order_submission_allowed",
        "production_order_mutation_allowed",
        "execution_adapter_call_allowed",
        "adapter_send_allowed",
        "live_exchange_request_allowed",
        "network_attempted",
        "implicit_retry_allowed",
        "retry_scheduler_enabled",
        "automatic_cancel_allowed",
        "automatic_retry_allowed",
        "automatic_remediation_allowed",
        "automatic_recovery_allowed",
        "automatic_operation_action_allowed",
        "strategy_driven_production_execution_allowed",
        "shared_approval_consumption_allowed",
        "cancel_replace_amend_send_allowed",
        "flatten_allowed",
        "dashboard_operation_controls_enabled",
        "dashboard_trading_controls_enabled",
        "dashboard_order_controls_enabled",
        "dashboard_approval_controls_enabled",
        "dashboard_cancel_controls_enabled",
        "dashboard_retry_controls_enabled",
        "dashboard_submit_controls_enabled",
        "dashboard_replace_controls_enabled",
        "dashboard_amend_controls_enabled",
        "dashboard_flatten_controls_enabled",
        "dashboard_remediation_controls_enabled",
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
    require(surface.get("current_release_version") == "v0.25.0", "release surface version mismatch")
    require(surface.get("current_release_tag") == "ntpro-rust-only-v0.25.0", "release surface tag mismatch")
    require(surface.get("next_patch_version") == "v0.25.1", "next patch mismatch")
    require(surface.get("next_capability_version") == "v0.26.0", "next capability mismatch")

    next_tracks = candidate.get("next_tracks") or {}
    require(next_tracks.get("patch") == "v0.25.1", "next patch track mismatch")
    require(next_tracks.get("capability") == "v0.26.0", "next capability track mismatch")
    require(next_tracks.get("start_gate") == "blocked_until_v250_release_evidence_published", "v0.26 start gate mismatch")
    for key in (
        "implementation_started",
        "inherits_production_submit",
        "inherits_production_mutation",
        "inherits_adapter_send",
        "inherits_live_exchange_request",
        "inherits_retry_scheduler",
        "inherits_automatic_remediation",
        "inherits_dashboard_trading_controls",
    ):
        require_false(next_tracks, key)

    requirements = candidate.get("post_publication_requirements") or {}
    require(requirements.get("release_tag") == os.environ["RELEASE_TAG"], "post-publication release tag mismatch")
    require(requirements.get("milestone_number") == 16, "post-publication milestone number mismatch")
    require(requirements.get("all_v250_milestone_issues_closed_required") is True, "milestone issue closeout requirement missing")
    require(requirements.get("v250_milestone_issue_count") == 9, "milestone issue count requirement mismatch")
    require(requirements.get("corrective_release_scope_closed_required") is True, "corrective release scope closeout requirement missing")
    require(requirements.get("corrective_issue") == 804, "corrective issue requirement mismatch")
    require(requirements.get("corrective_pull_request") == 805, "corrective PR requirement mismatch")
    require(requirements.get("final_release_scope_issue_count") == 10, "final release scope issue count requirement mismatch")
    require(requirements.get("github_release_published_required") is True, "GitHub release publication requirement missing")
    require(requirements.get("hosted_release_gate_success_required") is True, "hosted gate requirement missing")
    require(requirements.get("strict_release_body_match_required") is True, "strict release body requirement missing")
    require(requirements.get("v0_26_start_gate_fails_without_v250_release_evidence") is True, "v0.26 start gate requirement missing")


validate(manifest)

if os.environ.get("NTPRO_V250_RELEASE_SELFTEST", "1") == "1":
    missing_evidence = copy.deepcopy(manifest)
    missing_evidence["v250_evidence"] = missing_evidence["v250_evidence"][:-1]
    try:
        validate(missing_evidence)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: missing V250 evidence")

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
  for issue in 777 778 779 780 781 782 783 784; do
    state="$(gh_with_retry issue view "$issue" --repo "$REPO" --json state --jq .state)" || fail "could not read GitHub issue #$issue"
    [[ "$state" == "CLOSED" ]] || fail "GitHub issue #$issue must be CLOSED before $RELEASE_VERSION release gates, got $state"
  done
  current_state="$(gh_with_retry issue view "$CURRENT_ISSUE" --repo "$REPO" --json state --jq .state)" || fail "could not read GitHub issue #$CURRENT_ISSUE"
  if [[ "${NTPRO_RELEASE_GATE:-0}" == "1" || "${NTPRO_V250_RELEASE_REQUIRE_CLOSEOUT:-0}" == "1" ]]; then
    [[ "$current_state" == "CLOSED" ]] || fail "GitHub issue #$CURRENT_ISSUE must be CLOSED for the tag gate, got $current_state"
  else
    [[ "$current_state" == "OPEN" || "$current_state" == "CLOSED" ]] || fail "unexpected current release issue state: $current_state"
  fi

  corrective_state="$(gh_with_retry issue view 804 --repo "$REPO" --json state --jq .state)" || fail "could not read GitHub issue #804"
  [[ "$corrective_state" == "CLOSED" ]] || fail "GitHub issue #804 must be CLOSED for the corrective release scope, got $corrective_state"

  corrective_pr_json="$(gh_with_retry api "/repos/$REPO/pulls/805")" || fail "could not read GitHub PR #805"
  CORRECTIVE_PR_JSON="$corrective_pr_json" python3 <<'PY'
import json
import os

pr = json.loads(os.environ["CORRECTIVE_PR_JSON"])
if pr["number"] != 805 or pr["state"] != "closed":
    raise SystemExit(pr)
if pr["merged_at"] != "2026-07-06T02:36:12Z":
    raise SystemExit(pr)
if pr["merge_commit_sha"] != "eedcdab1d3ca85d6f51b368b5f36208a7b591026":
    raise SystemExit(pr)
PY

  milestone_json="$(gh_with_retry api "/repos/$REPO/milestones/$MILESTONE_NUMBER")" || fail "could not read GitHub milestone #$MILESTONE_NUMBER"
  MILESTONE_JSON="$milestone_json" \
  RELEASE_GATE="${NTPRO_RELEASE_GATE:-0}" \
  REQUIRE_CLOSEOUT="${NTPRO_V250_RELEASE_REQUIRE_CLOSEOUT:-0}" \
  python3 <<'PY'
import json
import os

milestone = json.loads(os.environ["MILESTONE_JSON"])
if milestone["title"] != "v0.25.0":
    raise SystemExit(milestone)
if os.environ["RELEASE_GATE"] == "1" or os.environ["REQUIRE_CLOSEOUT"] == "1":
    if milestone["state"] != "closed" or milestone["open_issues"] != 0 or milestone["closed_issues"] != 9:
        raise SystemExit(f"v0.25.0 milestone must be closed with 9 closed issues for release gate: {milestone}")
else:
    if milestone["state"] not in {"open", "closed"}:
        raise SystemExit(milestone)
PY
else
  fail "gh authentication is required for v25 release gate issue proof"
fi

if [[ "${NTPRO_V250_RELEASE_REQUIRE_PUBLICATION:-0}" == "1" ]]; then
  NTPRO_CURRENT_RELEASE_VERSION="$RELEASE_VERSION" \
    NTPRO_CURRENT_RELEASE_TAG="$RELEASE_TAG" \
    NTPRO_CURRENT_RELEASE_NAME="$RELEASE_NAME" \
    NTPRO_RELEASE_PUBLICATION_STRICT_BODY=1 \
    scripts/ai/check_github_release_published.sh
fi

echo "v25_release_gates status=ok release_tag=$RELEASE_TAG base_release=$BASE_RELEASE_TAG current_issue_state=$current_state corrective_issue_state=$corrective_state corrective_pr=805:merged final_scope_issues=10 negative_selftest=${NTPRO_V250_RELEASE_SELFTEST:-1}"
