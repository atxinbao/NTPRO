#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V290_RELEASE_REPO:-atxinbao/NTPRO}"
RELEASE_VERSION="${NTPRO_V290_RELEASE_VERSION:-v0.29.0}"
RELEASE_TAG="${NTPRO_V290_RELEASE_TAG:-ntpro-rust-only-v0.29.0}"
RELEASE_NAME="${NTPRO_V290_RELEASE_NAME:-NTPRO Rust-only v0.29.0}"
BASE_RELEASE_TAG="${NTPRO_V290_BASE_RELEASE_TAG:-ntpro-rust-only-v0.28.1}"
MANIFEST_PATH="${NTPRO_V290_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_29_0_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V290_RELEASE_NOTES:-docs/rust-cutover/release/v0_29_0_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V290_READINESS_REPORT:-docs/rust-cutover/release/v0_29_0_readiness_report.md}"
HANDOFF_PATH="${NTPRO_V290_HANDOFF:-docs/rust-cutover/release/v0_29_0_v30_go_live_candidate_handoff.md}"
MATRIX_PATH="${NTPRO_V290_BACKEND_READINESS_MATRIX:-docs/rust-cutover/release/v0_29_0_backend_production_readiness_matrix.json}"
CURRENT_ISSUE="${NTPRO_V290_CURRENT_ISSUE:-961}"
MILESTONE_NUMBER="${NTPRO_V290_MILESTONE_NUMBER:-24}"
MILESTONE_TITLE="${NTPRO_V290_MILESTONE_TITLE:-v0.29.0}"
RELEASE_GATE_RUN_ID="${NTPRO_V290_RELEASE_GATE_RUN_ID:-29091765148}"
RELEASE_TAG_SHA="${NTPRO_V290_RELEASE_TAG_SHA:-85110d29867763f8d3b6395f4ff8154378b475b9}"
RELEASE_CLOSEOUT_PATH="${NTPRO_V290_RELEASE_CLOSEOUT:-docs/rust-cutover/release/v0_29_0_release_closeout_evidence.md}"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

fail() {
  echo "v29 release gate failed: $*" >&2
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

publish_after_gate_has_v29_binding() {
  local output="$1"
  grep -F "release_publish_after_gate_current_binding=pass release_tag=$RELEASE_TAG release_gate_run_id=$RELEASE_GATE_RUN_ID tag_sha=$RELEASE_TAG_SHA" "$output" >/dev/null &&
    ! grep -F "release_tag=ntpro-rust-only-v0.28.0" "$output" >/dev/null &&
    ! grep -F "release_gate_run_id=28969059200" "$output" >/dev/null
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
  "$HANDOFF_PATH" \
  "$MATRIX_PATH" \
  docs/rust-cutover/release/v0_28_1_release_manifest.json \
  docs/rust-cutover/release/v0_28_1_release_closeout_evidence.md \
  README.md \
  ROADMAP.md \
  docs/versioning.md \
  docs/rust-cutover/release/README.md \
  scripts/ai/check_release_surface_current.sh \
  scripts/ai/check_github_release_published.sh \
  scripts/ai/verify_release_publish_after_gate.sh \
  scripts/ai/publish_ntpro_release_after_gate.sh \
  scripts/ai/verify_v29_intake_gate.sh \
  scripts/ai/verify_v29_backend_production_readiness_boundary_contract.sh \
  scripts/ai/verify_v29_persistent_audit_storage_production_readiness.sh \
  scripts/ai/verify_v29_telemetry_slo_ingestion_production_readiness.sh \
  scripts/ai/verify_v29_permission_source_production_readiness.sh \
  scripts/ai/verify_v29_read_only_backend_api_production_readiness.sh \
  scripts/ai/verify_v29_deployment_config_runbook_production_readiness.sh \
  scripts/ai/verify_v29_monitoring_alert_incident_production_readiness.sh \
  scripts/ai/verify_v29_canary_rollback_dr_preflight_readiness.sh \
  scripts/ai/verify_v29_backend_production_readiness_fail_closed_hardening.sh \
  scripts/ai/verify_v29_release_gates.sh \
  scripts/ai/verify_v29_strict_provenance.sh \
  scripts/ai/verify_release.sh; do
  require_file "$path"
done

for task_id in V290-000 V290-001 V290-002 V290-003 V290-004 V290-005 V290-006 V290-007 V290-008 V290-009 V290-010 V290-011; do
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
  "v0.29.0 publishes the Backend Production Readiness Foundation track" \
  "V290-000" \
  "V290-010" \
  "V290-011" \
  "V290 final release scope issue count = 12" \
  "V290 final release scope evidence count = 12" \
  "V290 exact milestone issue set = #926-#936, #961" \
  "V290 registered corrective-scope exception count = 1" \
  "v29 release gates = required" \
  "v29 strict provenance = required" \
  "backend production readiness boundary contract = required" \
  "backend production readiness fail-closed hardening = required" \
  "release surface current guard = required" \
  "release publication guard = required" \
  "release publish after gate = required" \
  "post-publication closeout gate = required" \
  "hosted release gate success before public GitHub Release = required" \
  "publication evidence strategy = source_tree_plus_github_remote" \
  "local generated publication evidence required in source tree = false" \
  "remote reconstruction required = true" \
  "generated publication evidence sole proof allowed = false" \
  "published release closeout evidence = docs/rust-cutover/release/v0_29_0_release_closeout_evidence.md" \
  "published release status = published_after_gate" \
  "hosted release gate run = 29091765148" \
  "release body hash semantics = normalized_sha256" \
  "scripts/ai/verify_release.sh v29-release-gates" \
  "scripts/ai/verify_release.sh v29-strict-provenance" \
  "scripts/ai/verify_release.sh v29.1-post-publication-closeout-gate" \
  "scripts/ai/verify_v29_release_gates.sh" \
  "scripts/ai/verify_v29_strict_provenance.sh" \
  "scripts/ai/verify_v29_1_post_publication_closeout_gate.sh" \
  "v0.30.0 backend production go-live candidate = next track" \
  "The next patch track is \`v0.29.1\`" \
  "The next capability track is \`v0.30.0\`"; do
  require_contains "$RELEASE_NOTES_PATH" "$marker"
done

for marker in \
  "Milestone: \`$RELEASE_TAG\`" \
  "Status: RELEASED" \
  "V290-000 evidence" \
  "V290-010 evidence" \
  "V290-011 evidence" \
  "v29 release gates = required" \
  "v29 strict provenance = required" \
  "#936 V290-010 = must be closed before v0.29.0 tag gate is accepted" \
  "#961 V290-011 = corrective release-gate blocker, must be closed before v0.29.0 tag gate is accepted" \
  "V290 final release scope issue count = 12" \
  "V290 final release scope evidence count = 12" \
  "V290 exact milestone issue set = #926-#936, #961" \
  "V290 registered corrective-scope exception count = 1" \
  "production_ready_count = 11" \
  "readiness_preview_count = 2" \
  "blocked_count = 0" \
  "deferred_count = 0" \
  "post-publication closeout gate = required" \
  "publication evidence strategy = source_tree_plus_github_remote" \
  "generated publication evidence sole proof allowed = false" \
  "published release closeout evidence = docs/rust-cutover/release/v0_29_0_release_closeout_evidence.md" \
  "published release status = published_after_gate" \
  "hosted release gate run = 29091765148" \
  "release body hash semantics = normalized_sha256" \
  "scripts/ai/verify_release.sh v29.1-post-publication-closeout-gate" \
  "scripts/ai/verify_v29_1_post_publication_closeout_gate.sh" \
  "v0.30.0 go-live candidate start = blocked until v0.29.0 publication evidence exists"; do
  require_contains "$READINESS_REPORT_PATH" "$marker"
done

for marker in \
  "v0.30.0 backend production go-live candidate = next track" \
  "v0.30.0 default trading controls = false" \
  "v0.30.0 backend go-live claim inherited from v0.29.0 = false" \
  "v0.30.0 requires new scoped issues before any production enablement = true"; do
  require_contains "$HANDOFF_PATH" "$marker"
done

for path in "$RELEASE_NOTES_PATH" "$READINESS_REPORT_PATH" "$MANIFEST_PATH" "$HANDOFF_PATH"; do
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

scripts/ai/verify_release.sh v29-intake-gate
scripts/ai/verify_release.sh v29-backend-production-readiness-boundary-contract
scripts/ai/verify_release.sh v29-persistent-audit-storage-production-readiness
scripts/ai/verify_release.sh v29-telemetry-slo-ingestion-production-readiness
scripts/ai/verify_release.sh v29-permission-source-production-readiness
scripts/ai/verify_release.sh v29-read-only-backend-api-production-readiness
scripts/ai/verify_release.sh v29-deployment-config-runbook-production-readiness
scripts/ai/verify_release.sh v29-monitoring-alert-incident-production-readiness
scripts/ai/verify_release.sh v29-canary-rollback-dr-preflight-readiness
scripts/ai/verify_release.sh v29-backend-production-readiness-fail-closed-hardening

NTPRO_CURRENT_RELEASE_VERSION="$RELEASE_VERSION" \
  NTPRO_CURRENT_RELEASE_TAG="$RELEASE_TAG" \
  NTPRO_NEXT_PATCH_VERSION="v0.29.1" \
  NTPRO_NEXT_CAPABILITY_VERSION="v0.30.0" \
  NTPRO_CURRENT_RELEASE_CAPABILITY="v0.29.0 Backend Production Readiness Foundation" \
  NTPRO_RELEASE_SURFACE_ALLOW_MISSING_TAG=1 \
  scripts/ai/verify_release.sh release-surface-current-guard

NTPRO_CURRENT_RELEASE_VERSION="$RELEASE_VERSION" \
  NTPRO_CURRENT_RELEASE_TAG="$RELEASE_TAG" \
  NTPRO_CURRENT_RELEASE_NAME="$RELEASE_NAME" \
  NTPRO_RELEASE_PUBLICATION_ALLOW_OFFLINE=1 \
  NTPRO_RELEASE_PUBLICATION_PREPUBLISH_TAG_GATE="${NTPRO_RELEASE_GATE:-0}" \
  scripts/ai/verify_release.sh release-publication-guard

NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_VERSION="$RELEASE_VERSION" \
  NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_TAG="$RELEASE_TAG" \
  NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_NAME="$RELEASE_NAME" \
  NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_NOTES="$RELEASE_NOTES_PATH" \
  NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_MANIFEST="$MANIFEST_PATH" \
  NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_CLOSEOUT="$RELEASE_CLOSEOUT_PATH" \
  NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_GATE_RUN_ID="$RELEASE_GATE_RUN_ID" \
  NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_TAG_SHA="$RELEASE_TAG_SHA" \
  NTPRO_RELEASE_PUBLISH_AFTER_GATE_LIVE_CURRENT=0 \
  NTPRO_RELEASE_PUBLISH_AFTER_GATE_REQUIRE_LIVE_CURRENT=0 \
  scripts/ai/verify_release.sh release-publish-after-gate | tee "$tmp_dir/v29-publish-after-gate.out"
if ! publish_after_gate_has_v29_binding "$tmp_dir/v29-publish-after-gate.out"; then
  fail "v29 publish-after-gate output did not prove $RELEASE_TAG run $RELEASE_GATE_RUN_ID"
fi

if NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_VERSION="v0.28.0" \
  NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_TAG="ntpro-rust-only-v0.28.0" \
  NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_NAME="NTPRO Rust-only v0.28.0" \
  NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_NOTES="docs/rust-cutover/release/v0_28_0_release_notes.md" \
  NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_MANIFEST="docs/rust-cutover/release/v0_28_0_release_manifest.json" \
  NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_CLOSEOUT="docs/rust-cutover/release/v0_28_0_release_closeout_evidence.md" \
  NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_GATE_RUN_ID="28969059200" \
  NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_TAG_SHA="41ef23417a4f21226cbc069de8cc31d0fa5e696e" \
  NTPRO_RELEASE_PUBLISH_AFTER_GATE_LIVE_CURRENT=0 \
  NTPRO_RELEASE_PUBLISH_AFTER_GATE_REQUIRE_LIVE_CURRENT=0 \
  scripts/ai/verify_release.sh release-publish-after-gate >"$tmp_dir/v28-publish-after-gate.out" 2>&1; then
  if publish_after_gate_has_v29_binding "$tmp_dir/v28-publish-after-gate.out"; then
    fail "negative self-test unexpectedly accepted v0.28.0 publish-after-gate output as v29 current binding"
  fi
fi
echo "v29_publish_after_gate_current_binding=pass release_tag=$RELEASE_TAG release_gate_run_id=$RELEASE_GATE_RUN_ID tag_sha=$RELEASE_TAG_SHA historical_v28_fallback_rejected=true"

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
    "V290-000": 926,
    "V290-001": 927,
    "V290-002": 928,
    "V290-003": 929,
    "V290-004": 930,
    "V290-005": 931,
    "V290-006": 932,
    "V290-007": 933,
    "V290-008": 934,
    "V290-009": 935,
    "V290-010": 936,
    "V290-011": 961,
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
    require(candidate.get("schema_version") == "ntpro.v290_release_manifest.v1", "manifest schema mismatch")
    require(candidate.get("task_id") == "V290-011", "manifest task mismatch")
    require(candidate.get("product_version") == os.environ["RELEASE_VERSION"], "manifest version mismatch")
    require(candidate.get("release_status") == "released", "manifest release status mismatch")
    planned = candidate.get("planned_release") or {}
    require(planned.get("tag") == os.environ["RELEASE_TAG"], "planned tag mismatch")
    require(planned.get("name") == os.environ["RELEASE_NAME"], "planned name mismatch")
    require(planned.get("draft") is False and planned.get("prerelease") is False, "planned release flags mismatch")
    base = candidate.get("base_release") or {}
    require(base.get("tag") == "ntpro-rust-only-v0.28.1", "base release tag mismatch")
    require(base.get("release_closeout_evidence_path") == "docs/rust-cutover/release/v0_28_1_release_closeout_evidence.md", "base closeout path missing")
    evidence = candidate.get("v290_evidence") or []
    require(len(evidence) == 12, "V290 evidence count mismatch")
    for item in evidence:
        task_id = item.get("task_id")
        require(expected.get(task_id) == item.get("issue"), f"V290 issue mismatch: {task_id}")
        require(Path(item.get("path", "")).is_file(), f"missing V290 evidence file: {item}")
    scope = candidate.get("release_scope") or {}
    require(scope.get("exact_milestone_issue_numbers") == list(expected.values()), "exact issue numbers mismatch")
    require(scope.get("exact_milestone_issue_set") == "#926-#936, #961", "exact issue set mismatch")
    require(scope.get("final_release_scope_issue_count") == 12, "final issue count mismatch")
    require(scope.get("final_release_scope_evidence_count") == 12, "final evidence count mismatch")
    require(scope.get("registered_corrective_scope_exception_count") == 1, "corrective exception count mismatch")
    require(scope.get("registered_corrective_scope_exception_issue_numbers") == [961], "corrective exception issue numbers mismatch")
    require(scope.get("unregistered_corrective_milestone_issues_fail_closed") is True, "unregistered corrective fail-closed rule missing")
    require(scope.get("production_ready_count") == 11, "production-ready count mismatch")
    require(scope.get("readiness_preview_count") == 2, "readiness-preview count mismatch")
    require(scope.get("blocked_count") == 0, "blocked count mismatch")
    require(scope.get("deferred_count") == 0, "deferred count mismatch")
    requirements = candidate.get("post_publication_requirements") or {}
    require(requirements.get("all_v290_issues_closed_required") is True, "V290 closeout requirement missing")
    require(requirements.get("hosted_release_gate_success_required") is True, "hosted gate requirement missing")
    require(requirements.get("publication_after_hosted_gate_required") is True, "publication after gate requirement missing")
    require(requirements.get("remote_reconstruction_required") is True, "remote reconstruction requirement missing")
    post_closeout = candidate.get("post_release_closeout") or {}
    require(post_closeout.get("closeout_evidence_path") == "docs/rust-cutover/release/v0_29_0_release_closeout_evidence.md", "post-release closeout evidence missing")
    publication = post_closeout.get("publication_evidence") or {}
    require(publication.get("status") == "published_after_gate", "publication evidence status mismatch")
    require(publication.get("audit_source") == "source_tree_plus_github_remote", "publication audit source mismatch")
    require(publication.get("generated_publication_evidence_sole_proof_allowed") is False, "generated-only proof must be disallowed")
    closeout_gate = candidate.get("post_publication_closeout_gate") or {}
    require(closeout_gate.get("task_id") == "V291-004", "post-publication closeout gate task mismatch")
    require(closeout_gate.get("issue") == 966, "post-publication closeout gate issue mismatch")
    require(closeout_gate.get("rejects_release_gate_ready_only") is True, "release_gate_ready rejection missing")
    require(closeout_gate.get("requires_source_tree_plus_github_remote") is True, "source_tree_plus_github_remote requirement missing")
    require(closeout_gate.get("generated_evidence_only_allowed") is False, "generated-only proof must be rejected")
    next_tracks = candidate.get("next_tracks") or {}
    require(next_tracks.get("capability") == "v0.30.0", "next capability mismatch")
    require(next_tracks.get("capability_entry") == "backend_production_go_live_candidate_after_v290_release_evidence", "next capability entry mismatch")
    require(next_tracks.get("start_gate") == "blocked_until_v290_release_evidence_published", "next start gate mismatch")
    require(next_tracks.get("implementation_started") is False, "v30 implementation must not start")
    require(next_tracks.get("inherits_backend_go_live_claim") is False, "v30 must not inherit backend go-live claim")
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
missing_evidence["v290_evidence"] = missing_evidence["v290_evidence"][:-1]
try:
    validate(missing_evidence)
except SystemExit:
    pass
else:
    raise SystemExit("negative self-test unexpectedly allowed missing V290 evidence")

gate_ready_only = copy.deepcopy(manifest)
gate_ready_only["release_status"] = "release_gate_ready"
gate_ready_only.pop("post_release_closeout", None)
try:
    validate(gate_ready_only)
except SystemExit:
    pass
else:
    raise SystemExit("negative self-test unexpectedly allowed release_gate_ready-only manifest")
PY

if command -v gh >/dev/null 2>&1 && gh_with_retry auth status >/dev/null 2>&1; then
  issue_json="$(gh_with_retry issue list --repo "$REPO" --state all --milestone "$MILESTONE_TITLE" --limit 50 --json number,state)"
  ISSUE_JSON="$issue_json" CURRENT_ISSUE="$CURRENT_ISSUE" TAG_GATE="${NTPRO_RELEASE_GATE:-0}" python3 <<'PY'
import json
import os

issues = json.loads(os.environ["ISSUE_JSON"])
states = {int(item["number"]): item["state"] for item in issues}
expected = set(range(926, 937)) | {961}
current_issue = int(os.environ["CURRENT_ISSUE"])
tag_gate = os.environ.get("TAG_GATE") == "1"
if set(states) != expected:
    raise SystemExit(f"V290 milestone issue set mismatch: {sorted(states)}")
for number in sorted(expected):
    state = states[number]
    if tag_gate or number != current_issue:
        if state != "CLOSED":
            raise SystemExit(f"V290 issue must be closed before tag gate: #{number} state={state}")
    elif state not in {"OPEN", "CLOSED"}:
        raise SystemExit(f"unexpected current issue state: #{number} state={state}")
closed = sum(1 for state in states.values() if state == "CLOSED")
mode = "tag_gate" if tag_gate else "pr_mode"
print(f"v290_issue_scope={mode} closed={closed}/12 current_issue_state={states[current_issue]}")
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
print("v290_milestone_open_issues=0")
PY
  fi
elif [[ "${NTPRO_RELEASE_GATE:-0}" == "1" ]]; then
  fail "gh authentication is required for tag-gate issue closeout proof"
else
  echo "v290_issue_scope=offline_skip reason=gh_unavailable_or_unauthenticated"
fi

echo "v29_release_gates=pass release_tag=$RELEASE_TAG release_gate_run_id=$RELEASE_GATE_RUN_ID publish_after_gate_current_binding=pass final_scope_issues=12 final_scope_evidence=12 negative_selftest=1"
