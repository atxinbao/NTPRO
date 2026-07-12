#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V291_RELEASE_REPO:-atxinbao/NTPRO}"
RELEASE_VERSION="${NTPRO_V291_RELEASE_VERSION:-v0.29.1}"
RELEASE_TAG="${NTPRO_V291_RELEASE_TAG:-ntpro-rust-only-v0.29.1}"
RELEASE_NAME="${NTPRO_V291_RELEASE_NAME:-NTPRO Rust-only v0.29.1}"
BASE_RELEASE_TAG="${NTPRO_V291_BASE_RELEASE_TAG:-ntpro-rust-only-v0.29.0}"
MANIFEST_PATH="${NTPRO_V291_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_29_1_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V291_RELEASE_NOTES:-docs/rust-cutover/release/v0_29_1_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V291_READINESS_REPORT:-docs/rust-cutover/release/v0_29_1_readiness_report.md}"
CLOSEOUT_PATH="${NTPRO_V291_CLOSEOUT:-docs/rust-cutover/release/v0_29_1_release_closeout_evidence.md}"
CURRENT_ISSUE="${NTPRO_V291_CURRENT_ISSUE:-968}"
MILESTONE_NUMBER="${NTPRO_V291_MILESTONE_NUMBER:-25}"
MILESTONE_TITLE="${NTPRO_V291_MILESTONE_TITLE:-v0.29.1}"

fail() {
  echo "v29.1 release gate failed: $*" >&2
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
  "$CLOSEOUT_PATH" \
  docs/rust-cutover/release/v0_29_1_v30_start_gate.md \
  docs/rust-cutover/release/v0_29_1_v30_start_gate_requirements.json \
  docs/rust-cutover/release/v0_29_0_release_manifest.json \
  docs/rust-cutover/release/v0_29_0_release_closeout_evidence.md \
  docs/rust-cutover/release/README.md \
  scripts/ai/check_github_release_published.sh \
  scripts/ai/publish_ntpro_release_after_gate.sh \
  scripts/ai/verify_v29_release_gates.sh \
  scripts/ai/verify_v29_strict_provenance.sh \
  scripts/ai/verify_v29_1_v30_start_gate.sh \
  scripts/ai/verify_v29_1_release_gates.sh \
  scripts/ai/verify_v29_1_strict_provenance.sh; do
  require_file "$path"
done

for task_id in V291-001 V291-002 V291-003 V291-004 V291-005 V291-006; do
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
  "v0.29.1 is a release governance and v0.30.0 start-gate hardening patch" \
  "V291-001" \
  "V291-006" \
  "V291 final release scope issue count = 6" \
  "V291 final release scope evidence count = 6" \
  "V291 exact milestone issue set = #963-#968" \
  "V291 registered corrective-scope exception count = 0" \
  "v29.1 release gates = required" \
  "v29.1 strict provenance = required" \
  "v30 start gate = hard-blocked until v0.29.1 publication evidence exists" \
  "publication evidence strategy = source_tree_plus_github_remote" \
  "generated publication evidence sole proof allowed = false" \
  "scripts/ai/verify_v29_1_release_gates.sh" \
  "scripts/ai/verify_v29_1_strict_provenance.sh" \
  "scripts/ai/verify_v29_1_v30_start_gate.sh"; do
  require_contains "$RELEASE_NOTES_PATH" "$marker"
done

require_release_status_marker "$READINESS_REPORT_PATH"
for marker in \
  "V291-001 evidence" \
  "V291-006 evidence" \
  "#968 V291-006 = must be closed before v0.29.1 tag gate is accepted" \
  "V291 final release scope issue count = 6" \
  "V291 final release scope evidence count = 6" \
  "V291 exact milestone issue set = #963-#968" \
  "v0.30.0 start gate = blocked until v0.29.1 release evidence is published" \
  "source-controlled closeout evidence = docs/rust-cutover/release/v0_29_1_release_closeout_evidence.md"; do
  require_contains "$READINESS_REPORT_PATH" "$marker"
done

for path in "$RELEASE_NOTES_PATH" "$READINESS_REPORT_PATH" "$MANIFEST_PATH" "$CLOSEOUT_PATH"; do
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
    "product_grade_trading_terminal_claim = true"; do
    require_not_contains "$path" "$marker"
  done
done

scripts/ai/verify_v29_1_release_closeout_evidence.sh >/dev/null
scripts/ai/verify_v29_1_post_publication_closeout_gate.sh source >/dev/null
NTPRO_RELEASE_GATE=0 NTPRO_RELEASE_STRICT_REQUIRE_HEAD_TAG=0 scripts/ai/verify_v29_strict_provenance.sh
scripts/ai/verify_v29_1_v30_start_gate.sh

NTPRO_CURRENT_RELEASE_VERSION="$RELEASE_VERSION" \
  NTPRO_CURRENT_RELEASE_TAG="$RELEASE_TAG" \
  NTPRO_CURRENT_RELEASE_NAME="$RELEASE_NAME" \
  NTPRO_RELEASE_PUBLICATION_ALLOW_OFFLINE=1 \
  NTPRO_RELEASE_PUBLICATION_PREPUBLISH_TAG_GATE="${NTPRO_RELEASE_GATE:-0}" \
  scripts/ai/check_github_release_published.sh

MANIFEST_PATH="$MANIFEST_PATH" python3 <<'PY'
import copy
import json
import os
from pathlib import Path

manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))
expected = {
    "V291-001": 963,
    "V291-002": 964,
    "V291-003": 965,
    "V291-004": 966,
    "V291-005": 967,
    "V291-006": 968,
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
    "dashboard_operation_controls_enabled",
    "dashboard_trading_controls_enabled",
    "admin_workbench_operation_controls_enabled",
    "admin_workbench_trading_controls_enabled",
    "trader_terminal_order_ticket_enabled",
    "manual_operation_submit_allowed",
    "backend_go_live_claim",
    "product_grade_trading_terminal_claim",
]

def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)

def validate(candidate: dict) -> None:
    require(candidate.get("schema_version") == "ntpro.v291_patch_release_manifest.v1", "manifest schema mismatch")
    require(candidate.get("task_id") == "V291-006", "manifest task mismatch")
    require(candidate.get("product_version") == "v0.29.1", "manifest product version mismatch")
    require(candidate.get("release_status") in {"release_gate_ready", "released"}, "manifest release status mismatch")
    if candidate.get("release_status") == "released":
        published = candidate.get("published_release") or {}
        post_pub = candidate.get("post_publication_closeout") or {}
        contract = candidate.get("authoritative_predecessor_closeout_contract") or {}
        require(published.get("tag") == "ntpro-rust-only-v0.29.1", "published release tag mismatch")
        require(published.get("tag_sha") == "a831d802e4321f50ed6e10481aea35b15a74b01e", "published tag SHA mismatch")
        require(post_pub.get("status") == "source_controlled_closeout_recorded", "post-publication closeout status mismatch")
        require(post_pub.get("release_gate_run_id") == 29130876713, "post-publication release gate run mismatch")
        require(post_pub.get("published_after_hosted_gate") is True, "post-publication ordering missing")
        require(contract.get("contract_id") == "v0_29_1_authoritative_closeout_contract", "authoritative contract missing")
        require(contract.get("v30_intake_consumes_contract") is True, "v30 intake contract marker missing")
    planned = candidate.get("planned_release") or {}
    require(planned.get("tag") == "ntpro-rust-only-v0.29.1", "planned release tag mismatch")
    evidence = candidate.get("v291_evidence") or []
    require(len(evidence) == 6, "V291 evidence count mismatch")
    for item in evidence:
        task_id = item.get("task_id")
        require(expected.get(task_id) == item.get("issue"), f"V291 issue mismatch: {task_id}")
        require(Path(item.get("path", "")).is_file(), f"missing V291 evidence: {item}")
    scope = candidate.get("release_scope") or {}
    require(scope.get("exact_milestone_issue_numbers") == list(expected.values()), "V291 exact issue numbers mismatch")
    require(scope.get("exact_milestone_issue_set") == "#963-#968", "V291 exact issue set mismatch")
    require(scope.get("final_release_scope_issue_count") == 6, "final issue count mismatch")
    require(scope.get("final_release_scope_evidence_count") == 6, "final evidence count mismatch")
    require(scope.get("registered_corrective_scope_exception_count") == 0, "corrective exception count mismatch")
    requirements = candidate.get("post_publication_requirements") or {}
    require(requirements.get("all_v291_issues_closed_required") is True, "V291 closeout requirement missing")
    require(requirements.get("hosted_release_gate_success_required") is True, "hosted gate requirement missing")
    require(requirements.get("publication_after_hosted_gate_required") is True, "publication after gate requirement missing")
    require(requirements.get("source_controlled_closeout_evidence_required") is True, "source closeout requirement missing")
    require(requirements.get("generated_publication_evidence_sole_proof_allowed") is False, "generated-only proof must be false")
    require(requirements.get("v0_30_start_gate_fails_without_v291_release_evidence") is True, "v30 blocker missing")
    next_tracks = candidate.get("next_tracks") or {}
    require(next_tracks.get("capability") == "v0.30.0", "next capability mismatch")
    require(next_tracks.get("start_gate") == "blocked_until_v291_release_evidence_published", "next start gate mismatch")
    require(next_tracks.get("implementation_started") is False, "v30 implementation must not start")
    for key in false_flags:
        require((candidate.get("boundary_flags") or {}).get(key) is False, f"boundary must remain false: {key}")

validate(manifest)

bad_boundary = copy.deepcopy(manifest)
bad_boundary["boundary_flags"]["adapter_send_allowed"] = True
try:
    validate(bad_boundary)
except SystemExit:
    pass
else:
    raise SystemExit("negative self-test unexpectedly allowed adapter_send_allowed")

missing_evidence = copy.deepcopy(manifest)
missing_evidence["v291_evidence"] = missing_evidence["v291_evidence"][:-1]
try:
    validate(missing_evidence)
except SystemExit:
    pass
else:
    raise SystemExit("negative self-test unexpectedly allowed missing V291 evidence")
PY

if command -v gh >/dev/null 2>&1 && gh_with_retry auth status >/dev/null 2>&1; then
  issue_json="$(gh_with_retry issue list --repo "$REPO" --state all --milestone "$MILESTONE_TITLE" --limit 50 --json number,state)"
  ISSUE_JSON="$issue_json" CURRENT_ISSUE="$CURRENT_ISSUE" TAG_GATE="${NTPRO_RELEASE_GATE:-0}" python3 <<'PY'
import json
import os

issues = json.loads(os.environ["ISSUE_JSON"])
states = {int(item["number"]): item["state"] for item in issues}
expected = [963, 964, 965, 966, 967, 968]
if sorted(states) != expected:
    raise SystemExit(f"V291 milestone issue scope mismatch: got={sorted(states)} expected={expected}")
closed = sum(1 for issue in expected if states[issue] == "CLOSED")
current_issue = int(os.environ["CURRENT_ISSUE"])
tag_gate = os.environ["TAG_GATE"] == "1"
if tag_gate:
    for issue in expected:
        if states[issue] != "CLOSED":
            raise SystemExit(f"V291 issue must be closed before tag gate: #{issue} state={states[issue]}")
else:
    for issue in expected:
        if issue == current_issue:
            continue
        if states[issue] != "CLOSED":
            raise SystemExit(f"V291 dependency issue is not closed: #{issue} state={states[issue]}")
    print(f"v291_issue_scope=pr_mode closed={closed}/6 current_issue_state={states.get(current_issue)}")
PY
fi

if [[ "${NTPRO_RELEASE_GATE:-0}" == "1" ]]; then
  git rev-parse -q --verify "${RELEASE_TAG}^{commit}" >/dev/null || fail "missing local release tag: $RELEASE_TAG"
  tag_commit="$(git rev-list -n 1 "$RELEASE_TAG")"
  head_commit="$(git rev-parse HEAD)"
  [[ "$head_commit" == "$tag_commit" ]] || fail "HEAD $head_commit does not match $RELEASE_TAG commit $tag_commit"
fi

echo "v29_1_release_gates=pass release_tag=$RELEASE_TAG final_scope_issues=6 final_scope_evidence=6 negative_selftest=1"
