#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V281_RELEASE_REPO:-atxinbao/NTPRO}"
RELEASE_VERSION="${NTPRO_V281_RELEASE_VERSION:-v0.28.1}"
RELEASE_TAG="${NTPRO_V281_RELEASE_TAG:-ntpro-rust-only-v0.28.1}"
RELEASE_NAME="${NTPRO_V281_RELEASE_NAME:-NTPRO Rust-only v0.28.1}"
BASE_RELEASE_TAG="${NTPRO_V281_BASE_RELEASE_TAG:-ntpro-rust-only-v0.28.0}"
MANIFEST_PATH="${NTPRO_V281_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_28_1_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V281_RELEASE_NOTES:-docs/rust-cutover/release/v0_28_1_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V281_READINESS_REPORT:-docs/rust-cutover/release/v0_28_1_readiness_report.md}"
CLOSEOUT_PATH="${NTPRO_V281_CLOSEOUT:-docs/rust-cutover/release/v0_28_1_release_closeout_evidence.md}"
CURRENT_ISSUE="${NTPRO_V281_CURRENT_ISSUE:-925}"
MILESTONE_NUMBER="${NTPRO_V281_MILESTONE_NUMBER:-23}"
MILESTONE_TITLE="${NTPRO_V281_MILESTONE_TITLE:-v0.28.1}"

fail() {
  echo "v28.1 release gate failed: $*" >&2
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
  "$CLOSEOUT_PATH" \
  docs/rust-cutover/release/v0_28_0_release_manifest.json \
  docs/rust-cutover/release/v0_28_0_release_closeout_evidence.md \
  README.md \
  ROADMAP.md \
  docs/versioning.md \
  docs/rust-cutover/release/README.md \
  scripts/ai/check_release_surface_current.sh \
  scripts/ai/check_github_release_published.sh \
  scripts/ai/verify_release_publish_after_gate.sh \
  scripts/ai/publish_ntpro_release_after_gate.sh \
  scripts/ai/verify_v28_release_gates.sh \
  scripts/ai/verify_v28_strict_provenance.sh \
  scripts/ai/verify_v28_1_release_body_hash_normalization.sh \
  scripts/ai/verify_v28_1_runtime_closed_terminology.sh \
  scripts/ai/verify_v28_1_release_gates.sh \
  scripts/ai/verify_v28_1_strict_provenance.sh \
  scripts/ai/verify_v29_intake_gate.sh \
  scripts/ai/verify_release.sh; do
  require_file "$path"
done

for task_id in V281-001 V281-002 V281-003 V281-004 V281-005 V281-006 V281-007; do
  require_file "docs/rust-cutover/evidence/${task_id}.md"
  require_contains "docs/rust-cutover/evidence/${task_id}.md" "$task_id"
  require_file "docs/rust-cutover/tasks/${task_id}.md"
  require_contains "docs/rust-cutover/tasks/${task_id}.md" "$task_id"
done

for marker in \
  "Status: RELEASE GATE READY" \
  "Tag: \`$RELEASE_TAG\`" \
  "Release name: \`$RELEASE_NAME\`" \
  "Release URL: \`https://github.com/atxinbao/NTPRO/releases/tag/$RELEASE_TAG\`" \
  "Base release: \`$BASE_RELEASE_TAG\`" \
  "v0.28.1 is a patch governance and provenance hardening release" \
  "V281-001" \
  "V281-007" \
  "v28.1 release gates = required" \
  "v28.1 strict provenance = required" \
  "v29 intake gate = hard-blocked until v0.28.1 publication evidence exists" \
  "release surface current guard = required" \
  "release publication guard = required" \
  "release publish after gate = required" \
  "post-publication closeout evidence path = docs/rust-cutover/release/v0_28_1_release_closeout_evidence.md" \
  "publication evidence strategy = source_tree_plus_github_remote" \
  "local generated publication evidence required in source tree = false" \
  "remote reconstruction required = true" \
  "scripts/ai/verify_release.sh v28.1-release-gates" \
  "scripts/ai/verify_release.sh v28.1-strict-provenance" \
  "scripts/ai/verify_v29_intake_gate.sh" \
  "v0.29.0 start gate = blocked until v0.28.1 release gate passes"; do
  require_contains "$RELEASE_NOTES_PATH" "$marker"
done

for marker in \
  "Milestone: \`$RELEASE_TAG\`" \
  "Status: RELEASE GATE READY" \
  "V281-001 evidence" \
  "V281-007 evidence" \
  "v28.1 release gates = required" \
  "v28.1 strict provenance = required" \
  "#925 V281-007 = must be closed before v0.28.1 tag gate is accepted" \
  "V281 final release scope issue count = 7" \
  "V281 final release scope evidence count = 7" \
  "V281 exact milestone issue set = #919-#925" \
  "V281 registered corrective-scope exception count = 0" \
  "v0.29.0 start gate = blocked until v0.28.1 release evidence is published" \
  "source-controlled closeout evidence = docs/rust-cutover/release/v0_28_1_release_closeout_evidence.md"; do
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
    "product_grade_trading_terminal_claim = true"; do
    require_not_contains "$path" "$marker"
  done
done

NTPRO_RELEASE_PUBLISH_AFTER_GATE_LIVE_CURRENT=0 scripts/ai/verify_release.sh v28-release-gates
NTPRO_RELEASE_STRICT_REQUIRE_HEAD_TAG=0 scripts/ai/verify_release.sh v28-strict-provenance
scripts/ai/verify_release.sh v28.1-release-body-hash-normalization
scripts/ai/verify_release.sh v28.1-runtime-closed-terminology
NTPRO_RELEASE_PUBLISH_AFTER_GATE_LIVE_CURRENT=0 scripts/ai/verify_release.sh v28.1-release-publish-after-gate-current-binding

NTPRO_CURRENT_RELEASE_VERSION="$RELEASE_VERSION" \
  NTPRO_CURRENT_RELEASE_TAG="$RELEASE_TAG" \
  NTPRO_NEXT_PATCH_VERSION="v0.28.2" \
  NTPRO_NEXT_CAPABILITY_VERSION="v0.29.0" \
  NTPRO_CURRENT_RELEASE_CAPABILITY="v0.28.1 Backend Closure Governance Closeout Patch" \
  NTPRO_RELEASE_SURFACE_ALLOW_MISSING_TAG=1 \
  scripts/ai/verify_release.sh release-surface-current-guard

NTPRO_CURRENT_RELEASE_VERSION="$RELEASE_VERSION" \
  NTPRO_CURRENT_RELEASE_TAG="$RELEASE_TAG" \
  NTPRO_CURRENT_RELEASE_NAME="$RELEASE_NAME" \
  NTPRO_RELEASE_PUBLICATION_ALLOW_OFFLINE=1 \
  NTPRO_RELEASE_PUBLICATION_PREPUBLISH_TAG_GATE="${NTPRO_RELEASE_GATE:-0}" \
  scripts/ai/verify_release.sh release-publication-guard
scripts/ai/verify_release.sh release-publish-after-gate
NTPRO_V29_INTAKE_ALLOW_UNPUBLISHED=1 scripts/ai/verify_release.sh v29-intake-gate

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
expected = {
    "V281-001": 919,
    "V281-002": 920,
    "V281-003": 921,
    "V281-004": 922,
    "V281-005": 923,
    "V281-006": 924,
    "V281-007": 925,
}
false_flags = [
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
    require(candidate.get("schema_version") == "ntpro.v281_patch_release_manifest.v1", "manifest schema mismatch")
    require(candidate.get("task_id") == "V281-007", "manifest task mismatch")
    require(candidate.get("product_version") == os.environ["RELEASE_VERSION"], "manifest version mismatch")
    require(candidate.get("release_status") == "release_gate_ready", "manifest release status mismatch")
    planned = candidate.get("planned_release") or {}
    require(planned.get("tag") == os.environ["RELEASE_TAG"], "planned tag mismatch")
    require(planned.get("name") == os.environ["RELEASE_NAME"], "planned name mismatch")
    require(planned.get("draft") is False and planned.get("prerelease") is False, "planned release flags mismatch")
    base = candidate.get("base_release") or {}
    require(base.get("tag") == "ntpro-rust-only-v0.28.0", "base release tag mismatch")
    require(base.get("release_closeout_evidence_path") == "docs/rust-cutover/release/v0_28_0_release_closeout_evidence.md", "base closeout path missing")
    evidence = candidate.get("v281_evidence") or []
    require(len(evidence) == 7, "V281 evidence count mismatch")
    for item in evidence:
        task_id = item.get("task_id")
        require(expected.get(task_id) == item.get("issue"), f"V281 issue mismatch: {task_id}")
        require(Path(item.get("path", "")).is_file(), f"missing V281 evidence file: {item}")
    scope = candidate.get("release_scope") or {}
    require(scope.get("exact_milestone_issue_numbers") == list(expected.values()), "exact issue numbers mismatch")
    require(scope.get("exact_milestone_issue_set") == "#919-#925", "exact issue set mismatch")
    require(scope.get("final_release_scope_issue_count") == 7, "final issue count mismatch")
    require(scope.get("final_release_scope_evidence_count") == 7, "final evidence count mismatch")
    require(scope.get("registered_corrective_scope_exception_count") == 0, "corrective exception count mismatch")
    require(scope.get("unregistered_corrective_milestone_issues_fail_closed") is True, "unregistered corrective fail-closed rule missing")
    requirements = candidate.get("post_publication_requirements") or {}
    require(requirements.get("all_v281_issues_closed_required") is True, "V281 closeout requirement missing")
    require(requirements.get("hosted_release_gate_success_required") is True, "hosted gate requirement missing")
    require(requirements.get("publication_after_hosted_gate_required") is True, "publication after gate requirement missing")
    require(requirements.get("source_controlled_closeout_evidence_required") is True, "source closeout requirement missing")
    require(requirements.get("generated_publication_evidence_sole_proof_allowed") is False, "generated-only proof must be disallowed")
    require(requirements.get("v0_29_start_gate_fails_without_v281_release_evidence") is True, "v29 hard-block requirement missing")
    closeout = candidate.get("post_publication_closeout") or {}
    require(closeout.get("source_controlled_closeout_evidence") is True, "source-controlled closeout proof missing")
    require(closeout.get("source_controlled_closeout_evidence_path") == "docs/rust-cutover/release/v0_28_1_release_closeout_evidence.md", "closeout path mismatch")
    require(closeout.get("generated_evidence_is_sole_proof") is False, "generated evidence sole proof must be false")
    next_tracks = candidate.get("next_tracks") or {}
    require(next_tracks.get("capability") == "v0.29.0", "next capability mismatch")
    require(next_tracks.get("start_gate") == "blocked_until_v281_release_evidence_published", "next start gate mismatch")
    require(next_tracks.get("implementation_started") is False, "v29 implementation must not start")
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
missing_evidence["v281_evidence"] = missing_evidence["v281_evidence"][:-1]
try:
    validate(missing_evidence)
except SystemExit:
    pass
else:
    raise SystemExit("negative self-test unexpectedly allowed missing V281 evidence")
PY

if command -v gh >/dev/null 2>&1 && gh_with_retry auth status >/dev/null 2>&1; then
  issue_json="$(gh_with_retry issue list --repo "$REPO" --state all --milestone "$MILESTONE_TITLE" --limit 50 --json number,state)"
  ISSUE_JSON="$issue_json" CURRENT_ISSUE="$CURRENT_ISSUE" TAG_GATE="${NTPRO_RELEASE_GATE:-0}" python3 <<'PY'
import json
import os

issues = json.loads(os.environ["ISSUE_JSON"])
states = {int(item["number"]): item["state"] for item in issues}
expected = set(range(919, 926))
current_issue = int(os.environ["CURRENT_ISSUE"])
tag_gate = os.environ.get("TAG_GATE") == "1"
if set(states) != expected:
    raise SystemExit(f"V281 milestone issue set mismatch: {sorted(states)}")
for number in sorted(expected):
    state = states[number]
    if tag_gate or number != current_issue:
        if state != "CLOSED":
            raise SystemExit(f"V281 issue must be closed before tag gate: #{number} state={state}")
    elif state not in {"OPEN", "CLOSED"}:
        raise SystemExit(f"unexpected current issue state: #{number} state={state}")
closed = sum(1 for state in states.values() if state == "CLOSED")
mode = "tag_gate" if tag_gate else "pr_mode"
print(f"v281_issue_scope={mode} closed={closed}/7 current_issue_state={states[current_issue]}")
PY
  if [[ "${NTPRO_RELEASE_GATE:-0}" == "1" ]]; then
    milestone_json="$(gh_with_retry api "repos/$REPO/milestones/$MILESTONE_NUMBER")"
    MILESTONE_JSON="$milestone_json" MILESTONE_TITLE="$MILESTONE_TITLE" python3 <<'PY'
import json
import os

payload = json.loads(os.environ["MILESTONE_JSON"])
if payload.get("title") != os.environ["MILESTONE_TITLE"]:
    raise SystemExit(f"milestone title mismatch: {payload.get('title')}")
if payload.get("open_issues") != 0:
    raise SystemExit(f"milestone open issue count must be 0 before tag gate: {payload.get('open_issues')}")
print("v281_milestone_open_issues=0")
PY
  fi
elif [[ "${NTPRO_RELEASE_GATE:-0}" == "1" ]]; then
  fail "gh authentication is required for tag-gate issue closeout proof"
else
  echo "v281_issue_scope=offline_skip reason=gh_unavailable_or_unauthenticated"
fi

echo "v28_1_release_gates=pass release_tag=$RELEASE_TAG final_scope_issues=7 final_scope_evidence=7 negative_selftest=1"
