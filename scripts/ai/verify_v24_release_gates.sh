#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V24_RELEASE_REPO:-atxinbao/NTPRO}"
RELEASE_VERSION="${NTPRO_V24_RELEASE_VERSION:-v0.24.0}"
RELEASE_TAG="${NTPRO_V24_RELEASE_TAG:-ntpro-rust-only-v0.24.0}"
RELEASE_NAME="${NTPRO_V24_RELEASE_NAME:-NTPRO Rust-only v0.24.0}"
BASE_RELEASE_TAG="${NTPRO_V24_BASE_RELEASE_TAG:-ntpro-rust-only-v0.23.1}"
MANIFEST_PATH="${NTPRO_V24_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_24_0_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V24_RELEASE_NOTES:-docs/rust-cutover/release/v0_24_0_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V24_READINESS_REPORT:-docs/rust-cutover/release/v0_24_0_readiness_report.md}"
CURRENT_ISSUE="${NTPRO_V24_CURRENT_ISSUE:-752}"

fail() {
  echo "v24 release gate failed: $*" >&2
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
  docs/rust-cutover/release/v0_23_1_release_manifest.json \
  docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json \
  tests/golden/v240_order_intent_execution_policy.jsonl \
  tests/golden/v240_rate_limit_throttle_gate.jsonl \
  tests/golden/v240_order_slicing_preview.jsonl \
  tests/golden/v240_cancel_replace_amend_preview.jsonl \
  tests/golden/v240_retry_policy_ledger.jsonl \
  tests/golden/v240_readback_audit_evidence.jsonl \
  tests/golden/v240_dashboard_workbench_order_control_preview.json \
  scripts/ai/verify_v24_release_gates.sh \
  scripts/ai/verify_v24_strict_provenance.sh; do
  require_file "$path"
done

for task_id in V240-000 V240-001 V240-002 V240-003 V240-004 V240-005 V240-006 V240-007 V240-008 V240-009; do
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
  "Execution Algorithms And Order Control Foundation" \
  "This release does not add submit capability" \
  "This release is not a product-grade live trading terminal" \
  "V240-000" \
  "V240-009" \
  "v24 release gates = required" \
  "v24 strict provenance = required" \
  "release publish after gate = required" \
  "publication evidence strategy = source_tree_plus_github_remote" \
  "v0.25.0 start gate = blocked until v0.24.0 release evidence is published" \
  "scripts/ai/verify_release.sh v24-release-gates" \
  "scripts/ai/verify_release.sh v24-strict-provenance" \
  "scripts/ai/verify_v24_release_gates.sh" \
  "scripts/ai/verify_v24_strict_provenance.sh" \
  "scripts/ai/publish_ntpro_release_after_gate.sh"; do
  require_contains "$RELEASE_NOTES_PATH" "$marker"
done

for marker in \
  "Milestone: \`$RELEASE_TAG\`" \
  "Status: RELEASED" \
  "V240-000 evidence" \
  "V240-009 evidence" \
  "v24 release gates = required" \
  "v24 strict provenance = required" \
  "release surface current guard = required" \
  "release publication guard = required" \
  "release publish after gate = required" \
  "#752 V240-009 = stays open until tag, hosted gate, public release, strict provenance, and publication evidence are recorded" \
  "No V250 implementation starts until all V240 issues are closed and v0.24.0 release evidence is published"; do
  require_contains "$READINESS_REPORT_PATH" "$marker"
done

for path in "$RELEASE_NOTES_PATH" "$READINESS_REPORT_PATH" "$MANIFEST_PATH"; do
  for marker in \
    "new_submit_capability = true" \
    "production_order_submission_allowed = true" \
    "production_order_mutation_allowed = true" \
    "execution_adapter_call_allowed = true" \
    "live_exchange_request_allowed = true" \
    "implicit_retry_allowed = true" \
    "retry_scheduler_enabled = true" \
    "cancel_replace_amend_send_allowed = true" \
    "flatten_allowed = true" \
    "dashboard_operation_controls_enabled = true" \
    "trader_terminal_order_ticket_enabled = true" \
    "manual_operation_submit_allowed = true" \
    "product_grade_trading_terminal_claim = true"; do
    require_not_contains "$path" "$marker"
  done
done

python3 scripts/ai/validate_golden_trace_release_scope.py --manifest docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json --trace-glob 'tests/golden/*.jsonl'
scripts/ai/verify_release.sh v24-intake-gate
scripts/ai/verify_release.sh v24-order-control-contract
scripts/ai/verify_release.sh v24-order-intent-policy
scripts/ai/verify_release.sh v24-rate-limit-throttle-gate
scripts/ai/verify_release.sh v24-order-slicing-preview
scripts/ai/verify_release.sh v24-cancel-replace-amend-preview
scripts/ai/verify_release.sh v24-retry-policy-ledger
scripts/ai/verify_release.sh v24-readback-audit-evidence
scripts/ai/verify_release.sh v24-dashboard-workbench-preview
NTPRO_RELEASE_SURFACE_ALLOW_MISSING_TAG=1 scripts/ai/verify_release.sh release-surface-current-guard
NTPRO_RELEASE_PUBLICATION_ALLOW_OFFLINE=1 scripts/ai/verify_release.sh release-publication-guard
scripts/ai/verify_release.sh release-publish-after-gate

if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
  for issue in 743 744 745 746 747 748 749 750 751; do
    state="$(gh_with_retry issue view "$issue" --repo "$REPO" --json state --jq .state)" || fail "could not read GitHub issue #$issue"
    [[ "$state" == "CLOSED" ]] || fail "GitHub issue #$issue must be CLOSED before $RELEASE_VERSION release gates, got $state"
  done
  current_state="$(gh_with_retry issue view "$CURRENT_ISSUE" --repo "$REPO" --json state --jq .state)" || fail "could not read GitHub issue #$CURRENT_ISSUE"
  [[ "$current_state" == "OPEN" || "$current_state" == "CLOSED" ]] || fail "unexpected current release issue state: $current_state"
else
  fail "gh authentication is required for v24 release gate issue proof"
fi

RELEASE_VERSION="$RELEASE_VERSION" \
RELEASE_TAG="$RELEASE_TAG" \
RELEASE_NAME="$RELEASE_NAME" \
BASE_RELEASE_TAG="$BASE_RELEASE_TAG" \
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
    require(candidate.get("schema_version") == "ntpro.v240_release_manifest.v1", "manifest schema mismatch")
    require(candidate.get("task_id") == "V240-009", "manifest task mismatch")
    require(candidate.get("product_version") == os.environ["RELEASE_VERSION"], "manifest product version mismatch")
    require(candidate.get("release_status") == "released", "manifest release status mismatch")
    require(candidate.get("release_scope") == "execution_algorithms_and_order_control_foundation", "manifest release scope mismatch")
    require(candidate.get("capability_class") == "preview_evidence_replay_readonly_order_control_foundation", "manifest capability class mismatch")

    base = candidate.get("base_release") or {}
    require(base.get("tag") == os.environ["BASE_RELEASE_TAG"], "base release tag mismatch")
    require(Path(base.get("release_manifest_path", "")).is_file(), "base release manifest missing")

    planned = candidate.get("planned_release") or {}
    require(planned.get("tag") == os.environ["RELEASE_TAG"], "planned release tag mismatch")
    require(planned.get("name") == os.environ["RELEASE_NAME"], "planned release name mismatch")
    require(planned.get("github_release_url") == f"https://github.com/atxinbao/NTPRO/releases/tag/{os.environ['RELEASE_TAG']}", "planned release URL mismatch")
    require(planned.get("target_commitish") == "main", "planned target mismatch")
    require(planned.get("draft") is False, "planned draft flag mismatch")
    require(planned.get("prerelease") is False, "planned prerelease flag mismatch")

    expected_evidence = {
        "V240-000": 743,
        "V240-001": 744,
        "V240-002": 745,
        "V240-003": 746,
        "V240-004": 747,
        "V240-005": 748,
        "V240-006": 749,
        "V240-007": 750,
        "V240-008": 751,
        "V240-009": 752,
    }
    evidence = candidate.get("v240_evidence") or []
    require(len(evidence) == len(expected_evidence), "V240 evidence count mismatch")
    for item in evidence:
        task_id = item.get("task_id")
        require(expected_evidence.get(task_id) == item.get("issue"), f"V240 evidence issue mismatch: {task_id}")
        path = Path(item.get("path", ""))
        require(path.is_file(), f"V240 evidence file missing: {path}")
        require(task_id in path.read_text(encoding="utf-8"), f"V240 evidence task marker missing: {path}")

    inputs = candidate.get("release_inputs") or {}
    for key in (
        "release_notes_path",
        "readiness_report_path",
        "release_manifest_path",
        "base_release_manifest_path",
        "golden_trace_manifest_path",
        "dashboard_workbench_fixture_path",
        "v24_release_gates_path",
        "v24_strict_provenance_path",
    ):
        path = Path(inputs.get(key, ""))
        require(path.is_file(), f"release input missing: {key} -> {path}")

    commands = {
        gate.get("command")
        for gate in candidate.get("release_gates", [])
        if gate.get("required") is True
    }
    for command in (
        "scripts/ai/verify_release.sh v24-intake-gate",
        "scripts/ai/verify_release.sh v24-order-control-contract",
        "scripts/ai/verify_release.sh v24-order-intent-policy",
        "scripts/ai/verify_release.sh v24-rate-limit-throttle-gate",
        "scripts/ai/verify_release.sh v24-order-slicing-preview",
        "scripts/ai/verify_release.sh v24-cancel-replace-amend-preview",
        "scripts/ai/verify_release.sh v24-retry-policy-ledger",
        "scripts/ai/verify_release.sh v24-readback-audit-evidence",
        "scripts/ai/verify_release.sh v24-dashboard-workbench-preview",
        "scripts/ai/verify_release.sh v24-release-gates",
        "scripts/ai/verify_release.sh v24-strict-provenance",
        "scripts/ai/verify_release.sh release-surface-current-guard",
        "scripts/ai/verify_release.sh release-publication-guard",
        "scripts/ai/verify_release.sh release-publish-after-gate",
        "scripts/ai/verify_v24_release_gates.sh",
        "scripts/ai/verify_v24_strict_provenance.sh",
    ):
        require(command in commands, f"required release gate missing: {command}")

    capability = candidate.get("capability") or {}
    for key in (
        "order_control_foundation_preview_only",
        "preview_evidence_only",
        "release_governance_hardening",
        "strict_provenance",
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
        "ungated_submit_allowed",
        "ungated_cancel_allowed",
        "ungated_retry_allowed",
        "ungated_replace_allowed",
        "ungated_amend_allowed",
        "ungated_flatten_allowed",
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
    require(surface.get("current_release_version") == "v0.24.0", "release surface version mismatch")
    require(surface.get("current_release_tag") == "ntpro-rust-only-v0.24.0", "release surface tag mismatch")
    require(surface.get("next_capability_version") == "v0.25.0", "next capability mismatch")

    next_tracks = candidate.get("next_tracks") or {}
    require(next_tracks.get("capability") == "v0.25.0", "next capability track mismatch")
    require(next_tracks.get("start_gate") == "blocked_until_v240_release_evidence_published", "v0.25 start gate mismatch")
    require(next_tracks.get("implementation_started") is False, "v0.25 implementation must not be started")
    require(next_tracks.get("inherits_production_submit") is False, "v0.25 must not inherit production submit")
    require(next_tracks.get("inherits_production_mutation") is False, "v0.25 must not inherit production mutation")
    require(next_tracks.get("inherits_retry_scheduler") is False, "v0.25 must not inherit retry scheduler")
    require(next_tracks.get("inherits_dashboard_operation_controls") is False, "v0.25 must not inherit Dashboard controls")


validate(manifest)

negative = copy.deepcopy(manifest)
negative["boundary_flags"]["new_submit_capability"] = True
try:
    validate(negative)
except SystemExit:
    pass
else:
    raise SystemExit("negative selftest failed: true submit boundary was accepted")
PY

echo "v24_release_gates status=ok release_tag=$RELEASE_TAG current_issue_state=$current_state negative_selftest=1"
