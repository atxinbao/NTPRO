#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

RELEASE_VERSION="${NTPRO_V23_RELEASE_VERSION:-v0.23.0}"
RELEASE_TAG="${NTPRO_V23_RELEASE_TAG:-ntpro-rust-only-v0.23.0}"
PREVIOUS_VERSION="${NTPRO_V23_PREVIOUS_VERSION:-v0.22.1}"
PREVIOUS_TAG="${NTPRO_V23_PREVIOUS_TAG:-ntpro-rust-only-v0.22.1}"
MANIFEST_PATH="${NTPRO_V23_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_23_0_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V23_RELEASE_NOTES:-docs/rust-cutover/release/v0_23_0_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V23_READINESS_REPORT:-docs/rust-cutover/release/v0_23_0_readiness_report.md}"
CONTRACT_PATH="${NTPRO_V23_CONTRACT:-docs/rust-cutover/release/v0_23_0_multi_node_isolation_contract.md}"
CONTRACT_MANIFEST_PATH="${NTPRO_V23_CONTRACT_MANIFEST:-docs/rust-cutover/release/v0_23_0_isolation_contract_manifest.json}"
PREVIOUS_MANIFEST_PATH="${NTPRO_V23_PREVIOUS_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_22_1_release_manifest.json}"
GOLDEN_TRACE_MANIFEST_PATH="${NTPRO_V23_GOLDEN_TRACE_MANIFEST:-docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"
CURRENT_ISSUE="${NTPRO_V23_CURRENT_ISSUE:-718}"

fail() {
  echo "v23 release gate failed: $*" >&2
  exit 1
}

require_file() {
  local path="$1"
  [[ -f "$path" ]] || fail "missing required file: $path"
}

require_contains() {
  local path="$1"
  local needle="$2"
  if ! grep -F -- "$needle" "$path" >/dev/null; then
    fail "missing marker in $path: $needle"
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

require_file "$MANIFEST_PATH"
require_file "$RELEASE_NOTES_PATH"
require_file "$READINESS_REPORT_PATH"
require_file "$CONTRACT_PATH"
require_file "$CONTRACT_MANIFEST_PATH"
require_file "$PREVIOUS_MANIFEST_PATH"
require_file "$GOLDEN_TRACE_MANIFEST_PATH"

for task_id in V230-000 V230-001 V230-002 V230-003 V230-004 V230-005 V230-006 V230-007; do
  require_file "docs/rust-cutover/evidence/${task_id}.md"
  require_contains "docs/rust-cutover/evidence/${task_id}.md" "$task_id"
done

for task_id in V230-000 V230-001 V230-002 V230-003 V230-004 V230-005 V230-006 V230-007; do
  require_file "docs/rust-cutover/tasks/${task_id}.md"
  require_contains "docs/rust-cutover/tasks/${task_id}.md" "$task_id"
done

for path in \
  docs/rust-cutover/scope/v0_23_0_multi_node_isolation_scope.md \
  docs/rust-cutover/release/v0_23_0_multi_account_read_model_partitioning.md \
  docs/rust-cutover/release/v0_23_0_multi_strategy_supervisor_isolation.md \
  docs/rust-cutover/release/v0_23_0_multi_venue_node_lifecycle_boundary.md \
  docs/rust-cutover/release/v0_23_0_multi_node_orchestration_control_plane_gating.md \
  docs/rust-cutover/release/v0_23_0_dashboard_observability_surface.md; do
  require_file "$path"
done

for marker in \
  "Status: RELEASED" \
  "Tag: \`$RELEASE_TAG\`" \
  "Release name: \`NTPRO Rust-only $RELEASE_VERSION\`" \
  "Multi-Account / Multi-Strategy / Multi-Venue Node Isolation" \
  "This release does not add submit capability" \
  "This release is not a product-grade live trading terminal" \
  "V230-000" \
  "V230-007" \
  "scripts/ai/verify_release.sh v23-release-gates" \
  "scripts/ai/verify_release.sh v23-strict-provenance" \
  "scripts/ai/verify_release.sh v23.1-gate-phase-split" \
  "scripts/ai/verify_release.sh v23.1-evidence-replay-only-boundary" \
  "scripts/ai/verify_v23_release_gates.sh" \
  "scripts/ai/verify_v23_strict_provenance.sh" \
  "scripts/ai/publish_ntpro_release_after_gate.sh"; do
  require_contains "$RELEASE_NOTES_PATH" "$marker"
done

for marker in \
  "Milestone: \`$RELEASE_TAG\`" \
  "Status: RELEASED" \
  "V230-000 evidence" \
  "V230-007 evidence" \
  "v23 release gates = required" \
  "v23 strict provenance = required" \
  "v23.1 gate phase split = required" \
  "v23.1 evidence replay only boundary = required" \
  "release publish after gate = required" \
  "#718 V230-007 = closed after tag, hosted gate, public release, and publication evidence were recorded" \
  "release closeout evidence = docs/rust-cutover/release/v0_23_0_release_closeout_evidence.md"; do
  require_contains "$READINESS_REPORT_PATH" "$marker"
done

python3 scripts/ai/validate_golden_trace_release_scope.py
cargo test -p nautilus-cli --test golden_trace_read_model_projection -- --nocapture
scripts/ai/verify_v23_dashboard_observability_smoke.sh
scripts/ai/verify_release.sh release-publish-after-gate
scripts/ai/verify_v23_1_gate_phase_split.sh post-release-live
scripts/ai/verify_v23_1_evidence_replay_only_boundary.sh

if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
  for issue in 705 706 707 708 709 710 711 712 713 714 715 716 717; do
    state="$(gh_with_retry issue view "$issue" --json state --jq .state)" || fail "could not read GitHub issue #$issue"
    [[ "$state" == "CLOSED" ]] || fail "GitHub issue #$issue must be CLOSED before $RELEASE_VERSION release gates, got $state"
  done
  current_state="$(gh_with_retry issue view "$CURRENT_ISSUE" --json state --jq .state)" || fail "could not read GitHub issue #$CURRENT_ISSUE"
  [[ "$current_state" == "OPEN" || "$current_state" == "CLOSED" ]] || fail "unexpected current release issue state: $current_state"
else
  fail "gh authentication is required for v23 release gate issue proof"
fi

RELEASE_VERSION="$RELEASE_VERSION" \
RELEASE_TAG="$RELEASE_TAG" \
PREVIOUS_VERSION="$PREVIOUS_VERSION" \
PREVIOUS_TAG="$PREVIOUS_TAG" \
MANIFEST_PATH="$MANIFEST_PATH" \
RELEASE_NOTES_PATH="$RELEASE_NOTES_PATH" \
READINESS_REPORT_PATH="$READINESS_REPORT_PATH" \
CONTRACT_PATH="$CONTRACT_PATH" \
CONTRACT_MANIFEST_PATH="$CONTRACT_MANIFEST_PATH" \
PREVIOUS_MANIFEST_PATH="$PREVIOUS_MANIFEST_PATH" \
GOLDEN_TRACE_MANIFEST_PATH="$GOLDEN_TRACE_MANIFEST_PATH" \
python3 <<'PY'
import json
import os
from collections import Counter
from pathlib import Path

manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))
contract_manifest = json.loads(Path(os.environ["CONTRACT_MANIFEST_PATH"]).read_text(encoding="utf-8"))
previous_manifest = json.loads(Path(os.environ["PREVIOUS_MANIFEST_PATH"]).read_text(encoding="utf-8"))
golden_trace_manifest = json.loads(Path(os.environ["GOLDEN_TRACE_MANIFEST_PATH"]).read_text(encoding="utf-8"))


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def require_false(mapping: dict, key: str) -> None:
    require(mapping.get(key) is False, f"boundary flag must be false: {key}")


def require_true(mapping: dict, key: str) -> None:
    require(mapping.get(key) is True, f"capability flag must be true: {key}")


require(manifest.get("schema_version") == "ntpro.v230_release_manifest.v1", "manifest schema mismatch")
require(manifest.get("task_id") == "V230-007", "manifest task_id mismatch")
require(manifest.get("product_version") == os.environ["RELEASE_VERSION"], "manifest product version mismatch")
require(manifest.get("release_status") == "released", "manifest release status mismatch")
require(manifest.get("release_scope") == "multi_node_isolation_and_readonly_observability", "manifest release scope mismatch")
require(manifest.get("capability_class") == "evidence_replay_readonly_observability_only", "manifest capability class mismatch")

previous = manifest.get("previous_release") or {}
require(previous.get("tag") == os.environ["PREVIOUS_TAG"], "previous release tag mismatch")
require(previous.get("version") == os.environ["PREVIOUS_VERSION"], "previous release version mismatch")
require(previous_manifest.get("release_status") == "published", "previous release manifest status mismatch")
require((previous_manifest.get("planned_release") or {}).get("tag") == os.environ["PREVIOUS_TAG"], "previous release planned tag mismatch")

planned = manifest.get("planned_release") or {}
require(planned.get("tag") == os.environ["RELEASE_TAG"], "planned release tag mismatch")
require(planned.get("name") == f"NTPRO Rust-only {os.environ['RELEASE_VERSION']}", "planned release name mismatch")
require(planned.get("github_release_url") == f"https://github.com/atxinbao/NTPRO/releases/tag/{os.environ['RELEASE_TAG']}", "planned release URL mismatch")
require(planned.get("target_commitish") == "main", "planned target_commitish mismatch")
require(planned.get("draft") is False, "planned release must not be draft")
require(planned.get("prerelease") is False, "planned release must not be prerelease")

expected_evidence = {
    "V230-000": 711,
    "V230-001": 712,
    "V230-002": 713,
    "V230-003": 714,
    "V230-004": 715,
    "V230-005": 716,
    "V230-006": 717,
    "V230-007": 718,
}
evidence = manifest.get("v230_evidence") or []
require(len(evidence) == len(expected_evidence), "V230 evidence count mismatch")
for item in evidence:
    task_id = item.get("task_id")
    require(expected_evidence.get(task_id) == item.get("issue"), f"V230 evidence issue mismatch: {task_id}")
    path = Path(item.get("path", ""))
    require(path.is_file(), f"V230 evidence file missing: {path}")
    require(task_id in path.read_text(encoding="utf-8"), f"V230 evidence task marker missing: {path}")

required_inputs = manifest.get("release_inputs") or {}
for key in (
    "release_notes_path",
    "readiness_report_path",
    "release_manifest_path",
    "contract_path",
    "contract_manifest_path",
    "gate_phase_split_path",
    "golden_trace_manifest_path",
    "dashboard_observability_smoke_path",
    "closeout_evidence_path",
):
    path = Path(required_inputs.get(key, ""))
    require(path.is_file(), f"release input missing: {key} -> {path}")

commands = {
    gate.get("command")
    for gate in manifest.get("release_gates", [])
    if gate.get("required") is True
}
for command in (
    "scripts/ai/verify_release.sh v23-release-gates",
    "scripts/ai/verify_release.sh v23-strict-provenance",
    "scripts/ai/verify_release.sh v23.1-gate-phase-split",
    "scripts/ai/verify_release.sh v23.1-evidence-replay-only-boundary",
    "scripts/ai/verify_v23_release_gates.sh",
    "scripts/ai/verify_v23_strict_provenance.sh",
    "scripts/ai/verify_v23_dashboard_observability_smoke.sh",
    "scripts/ai/verify_release.sh release-publish-after-gate",
):
    require(command in commands, f"required release gate missing: {command}")

capability = manifest.get("capability") or {}
for key in (
    "multi_account_isolation",
    "multi_strategy_isolation",
    "multi_venue_node_isolation",
    "cross_node_read_model_aggregation",
    "read_only_dashboard_observability",
    "owner_approved_control_contract_defined",
    "gate_before_publish",
    "strict_provenance",
):
    require_true(capability, key)
for key in (
    "product_grade_live_trading_terminal",
    "new_submit_capability",
    "production_order_mutation_expansion",
    "dashboard_operation_controls",
    "complete_executable_read_model_runtime",
):
    require_false(capability, key)

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
    require_false(manifest.get("boundary_flags") or {}, key)

runtime_claims = manifest.get("runtime_claims") or {}
for key in (
    "production_multi_node_runtime",
    "runtime_integrated_multi_node_execution",
    "runtime_implementation_complete",
    "product_grade_terminal_ready",
    "v24_inherits_runtime_capability_from_v23",
):
    require_false(runtime_claims, key)

next_tracks = manifest.get("next_tracks") or {}
require(
    next_tracks.get("capability_entry") == "future_contract_and_gated_implementation_only",
    "next v0.24.0 capability entry mismatch",
)
require(next_tracks.get("inherits_runtime_capability") is False, "v0.24.0 runtime inheritance must be false")

scope_cases = golden_trace_manifest.get("cases") or []
status_counts = Counter(item.get("status") for item in scope_cases)
read_model_cases = [item for item in scope_cases if item.get("category") == "read_model"]
read_model_counts = Counter(item.get("status") for item in read_model_cases)
replay = manifest.get("read_model_replay") or {}
require(len(scope_cases) == replay.get("manifest_cases") == 100, "release scope manifest case count mismatch")
require(status_counts.get("executable_replay") == replay.get("manifest_executable_replay_cases") == 95, "release scope executable count mismatch")
require(status_counts.get("schema_only_scoped") == replay.get("manifest_schema_only_scoped_cases") == 5, "release scope schema-only count mismatch")
require(len(read_model_cases) == replay.get("read_model_cases") == 49, "read-model case count mismatch")
require(read_model_counts.get("executable_replay") == replay.get("read_model_executable_replay_rows") == 45, "read-model executable count mismatch")
require(read_model_counts.get("schema_only_scoped") == replay.get("read_model_schema_only_scoped_rows") == 4, "read-model schema-only count mismatch")

require(contract_manifest.get("release") == "v0.23.0", "contract manifest release mismatch")
required_markers = set(contract_manifest.get("required_validation_markers") or [])
for marker in (
    "identity_keys_present",
    "isolation_scope_key_present_when_crossing_boundaries",
    "missing_key_fail_closed_or_degraded_unavailable",
    "mismatched_key_fail_closed",
    "dashboard_has_no_operation_controls",
    "release_claims_do_not_exceed_contract",
):
    require(marker in required_markers, f"contract validation marker missing: {marker}")
PY

echo "v23_release_gates status=ok product_version=$RELEASE_VERSION release_tag=$RELEASE_TAG manifest=$MANIFEST_PATH current_issue_state=$current_state"
