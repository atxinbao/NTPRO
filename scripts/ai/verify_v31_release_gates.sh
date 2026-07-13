#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V310_RELEASE_REPO:-atxinbao/NTPRO}"
RELEASE_VERSION="${NTPRO_V310_RELEASE_VERSION:-v0.31.0}"
RELEASE_TAG="${NTPRO_V310_RELEASE_TAG:-ntpro-rust-only-v0.31.0}"
RELEASE_NAME="${NTPRO_V310_RELEASE_NAME:-NTPRO Rust-only v0.31.0}"
BASE_RELEASE_TAG="${NTPRO_V310_BASE_RELEASE_TAG:-ntpro-rust-only-v0.30.1}"
MANIFEST_PATH="${NTPRO_V310_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_31_0_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V310_RELEASE_NOTES:-docs/rust-cutover/release/v0_31_0_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V310_READINESS_REPORT:-docs/rust-cutover/release/v0_31_0_readiness_report.md}"
V32_HANDOFF_PATH="${NTPRO_V310_V32_HANDOFF:-docs/rust-cutover/release/v0_31_0_v32_backend_production_closeout_handoff.md}"
V32_HANDOFF_JSON="${NTPRO_V310_V32_HANDOFF_JSON:-docs/rust-cutover/release/v0_31_0_v32_backend_production_closeout_handoff.json}"
CURRENT_ISSUE="${NTPRO_V310_CURRENT_ISSUE:-1015}"
MILESTONE_TITLE="${NTPRO_V310_MILESTONE_TITLE:-v0.31.0}"

fail() {
  echo "v31 release gate failed: $*" >&2
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

for path in \
  "$MANIFEST_PATH" \
  "$RELEASE_NOTES_PATH" \
  "$READINESS_REPORT_PATH" \
  "$V32_HANDOFF_PATH" \
  "$V32_HANDOFF_JSON" \
  docs/rust-cutover/release/v0_30_1_release_manifest.json \
  docs/rust-cutover/release/v0_30_1_release_notes.md \
  docs/rust-cutover/release/v0_30_1_release_closeout_evidence.md \
  docs/rust-cutover/release/README.md \
  scripts/ai/check_github_release_published.sh \
  scripts/ai/publish_ntpro_release_after_gate.sh \
  scripts/ai/verify_v31_forbidden_production_execution_negative_tests.sh \
  scripts/ai/verify_v31_release_gates.sh \
  scripts/ai/verify_v31_strict_provenance.sh \
  .github/workflows/release-tag.yml \
  .github/workflows/release-publish.yml; do
  require_file "$path"
done

for task_id in V310-000 V310-001 V310-002 V310-003 V310-004 V310-005 V310-006 V310-007 V310-008 V310-009; do
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
  "v0.31.0 publishes the Controlled Backend Production Enablement Candidate Foundation" \
  "V310-000" \
  "V310-009" \
  "V310 final release scope issue count = 10" \
  "V310 final release scope evidence count = 10" \
  "V310 exact milestone issue set = #1006-#1015" \
  "V310 registered corrective-scope exception count = 0" \
  "v31 release gates = required" \
  "v31 strict provenance = required" \
  "v32 handoff = hard-blocked until v0.31.0 release evidence and explicit scoped approval" \
  "publication evidence strategy = source_tree_plus_github_remote" \
  "generated publication evidence sole proof allowed = false" \
  "scripts/ai/verify_v31_release_gates.sh" \
  "scripts/ai/verify_v31_strict_provenance.sh" \
  "scripts/ai/publish_ntpro_release_after_gate.sh"; do
  require_contains "$RELEASE_NOTES_PATH" "$marker"
done

require_release_status_marker "$READINESS_REPORT_PATH"
for marker in \
  "V310-000 evidence" \
  "V310-009 evidence" \
  "#1015 V310-009 = must be closed before v0.31.0 tag gate is accepted" \
  "V310 final release scope issue count = 10" \
  "V310 final release scope evidence count = 10" \
  "V310 exact milestone issue set = #1006-#1015" \
  "source-controlled release manifest = docs/rust-cutover/release/v0_31_0_release_manifest.json" \
  "source-controlled v32 handoff = docs/rust-cutover/release/v0_31_0_v32_backend_production_closeout_handoff.md"; do
  require_contains "$READINESS_REPORT_PATH" "$marker"
done

for marker in \
  "handoff_status = hard_blocked_until_v31_release_evidence_and_explicit_scoped_approval" \
  "v31 release evidence required = true" \
  "hosted v31 release gate success required = true" \
  "publication after hosted gate required = true" \
  "explicit scoped issue required = true" \
  "owner operator approval required = true" \
  "inherits_submit = false" \
  "inherits_mutation = false" \
  "inherits_adapter_send = false" \
  "inherits_live_exchange_request = false" \
  "inherits_retry_scheduler = false" \
  "inherits_automatic_remediation = false" \
  "inherits_trading_controls = false"; do
  require_contains "$V32_HANDOFF_PATH" "$marker"
done

for path in "$RELEASE_NOTES_PATH" "$READINESS_REPORT_PATH" "$MANIFEST_PATH" "$V32_HANDOFF_PATH"; do
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

scripts/ai/verify_v31_forbidden_production_execution_negative_tests.sh >/dev/null

NTPRO_CURRENT_RELEASE_VERSION="$RELEASE_VERSION" \
  NTPRO_CURRENT_RELEASE_TAG="$RELEASE_TAG" \
  NTPRO_CURRENT_RELEASE_NAME="$RELEASE_NAME" \
  NTPRO_RELEASE_PUBLICATION_ALLOW_OFFLINE=1 \
  NTPRO_RELEASE_PUBLICATION_PREPUBLISH_TAG_GATE="${NTPRO_RELEASE_GATE:-0}" \
  scripts/ai/check_github_release_published.sh >/dev/null

MANIFEST_PATH="$MANIFEST_PATH" V32_HANDOFF_JSON="$V32_HANDOFF_JSON" python3 <<'PY'
import copy
import json
import os
from pathlib import Path

manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))
handoff = json.loads(Path(os.environ["V32_HANDOFF_JSON"]).read_text(encoding="utf-8"))
expected = {
    "V310-000": 1006,
    "V310-001": 1007,
    "V310-002": 1008,
    "V310-003": 1009,
    "V310-004": 1010,
    "V310-005": 1011,
    "V310-006": 1012,
    "V310-007": 1013,
    "V310-008": 1014,
    "V310-009": 1015,
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
    "automatic_recovery_allowed",
    "dashboard_operation_controls_enabled",
    "dashboard_trading_controls_enabled",
    "admin_workbench_operation_controls_enabled",
    "admin_workbench_trading_controls_enabled",
    "trader_terminal_order_ticket_enabled",
    "manual_operation_submit_allowed",
    "backend_go_live_claim",
    "actual_backend_production_go_live_allowed",
    "product_grade_trading_terminal_claim",
    "product_grade_live_trading_terminal_claim",
    "default_production_execution_allowed",
]

def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)

def validate(candidate: dict) -> None:
    require(candidate.get("schema_version") == "ntpro.v310_release_manifest.v1", "manifest schema mismatch")
    require(candidate.get("task_id") == "V310-009", "manifest task mismatch")
    require(candidate.get("product_version") == "v0.31.0", "manifest product version mismatch")
    require(candidate.get("release_status") in {"release_gate_ready", "released"}, "manifest release status mismatch")
    planned = candidate.get("planned_release") or {}
    require(planned.get("tag") == "ntpro-rust-only-v0.31.0", "planned release tag mismatch")
    evidence = candidate.get("v310_evidence") or []
    require(len(evidence) == 10, "V310 evidence count mismatch")
    for item in evidence:
        task_id = item.get("task_id")
        require(expected.get(task_id) == item.get("issue"), f"V310 issue mismatch: {task_id}")
        require(Path(item.get("path", "")).is_file(), f"missing V310 evidence: {item}")
    scope = candidate.get("release_scope") or {}
    require(scope.get("exact_milestone_issue_numbers") == list(expected.values()), "V310 exact issue numbers mismatch")
    require(scope.get("exact_milestone_issue_set") == "#1006-#1015", "V310 exact issue set mismatch")
    require(scope.get("final_release_scope_issue_count") == 10, "final issue count mismatch")
    require(scope.get("final_release_scope_evidence_count") == 10, "final evidence count mismatch")
    require(scope.get("registered_corrective_scope_exception_count") == 0, "corrective exception count mismatch")
    requirements = candidate.get("post_publication_requirements") or {}
    require(requirements.get("all_v310_issues_closed_required") is True, "V310 closeout requirement missing")
    require(requirements.get("hosted_release_gate_success_required") is True, "hosted gate requirement missing")
    require(requirements.get("publication_after_hosted_gate_required") is True, "publication after gate requirement missing")
    require(requirements.get("same_tag_commit_required") is True, "same tag commit requirement missing")
    require(requirements.get("generated_publication_evidence_sole_proof_allowed") is False, "generated-only proof must be false")
    require(requirements.get("v0_32_start_gate_fails_without_v310_release_evidence") is True, "v32 blocker missing")
    next_tracks = candidate.get("next_tracks") or {}
    require(next_tracks.get("capability") == "v0.32.0", "next capability mismatch")
    require(next_tracks.get("start_gate") == "blocked_until_v310_release_evidence_and_explicit_scoped_approval", "next start gate mismatch")
    require(next_tracks.get("implementation_started") is False, "v32 implementation must not start")
    for key in false_flags:
        require((candidate.get("boundary_flags") or {}).get(key) is False, f"boundary must remain false: {key}")

validate(manifest)

require(handoff.get("schema_version") == "ntpro.v310_v32_backend_production_closeout_handoff.v1", "handoff schema mismatch")
require(handoff.get("next_track", {}).get("handoff_status") == "hard_blocked_until_v31_release_evidence_and_explicit_scoped_approval", "handoff status mismatch")
for key, value in (handoff.get("non_inheritance") or {}).items():
    require(value is False, f"handoff inheritance must be false: {key}")

bad_boundary = copy.deepcopy(manifest)
bad_boundary["boundary_flags"]["adapter_send_allowed"] = True
try:
    validate(bad_boundary)
except SystemExit:
    pass
else:
    raise SystemExit("negative self-test unexpectedly allowed adapter_send_allowed")

bad_scope = copy.deepcopy(manifest)
bad_scope["release_scope"]["exact_milestone_issue_numbers"].append(9999)
try:
    validate(bad_scope)
except SystemExit:
    pass
else:
    raise SystemExit("negative self-test unexpectedly allowed issue-scope drift")
PY

if [[ "${NTPRO_RELEASE_GATE:-0}" == "1" ]]; then
  current_issue_json="$(gh_with_retry issue view "$CURRENT_ISSUE" --repo "$REPO" --json number,state,title)"
  milestone_json="$(gh_with_retry api "repos/$REPO/milestones?state=all" --jq ".[] | select(.title == \"$MILESTONE_TITLE\")")"
  CURRENT_ISSUE_JSON="$current_issue_json" MILESTONE_JSON="$milestone_json" python3 <<'PY'
import json
import os

current = json.loads(os.environ["CURRENT_ISSUE_JSON"])
milestone = json.loads(os.environ["MILESTONE_JSON"])
if current.get("number") != 1015 or current.get("state") != "CLOSED":
    raise SystemExit("V310-009 must be closed for release gate")
if milestone.get("open_issues") != 0:
    raise SystemExit("v0.31.0 milestone must have zero open issues for release gate")
PY
fi

echo "v31_release_gates=pass release_tag=$RELEASE_TAG final_scope_issues=10 final_scope_evidence=10 v32_handoff=hard_blocked negative_selftest=2"
