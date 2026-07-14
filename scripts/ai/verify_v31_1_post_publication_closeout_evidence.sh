#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

MODE="${1:-source}"
REPO="${NTPRO_V311_POST_PUBLICATION_REPO:-atxinbao/NTPRO}"
RELEASE_VERSION="${NTPRO_V311_POST_PUBLICATION_VERSION:-v0.31.0}"
RELEASE_TAG="${NTPRO_V311_POST_PUBLICATION_TAG:-ntpro-rust-only-v0.31.0}"
RELEASE_NAME="${NTPRO_V311_POST_PUBLICATION_NAME:-NTPRO Rust-only v0.31.0}"
RELEASE_URL="${NTPRO_V311_POST_PUBLICATION_URL:-https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.31.0}"
TAG_SHA="${NTPRO_V311_POST_PUBLICATION_TAG_SHA:-14e582868cb9c18b8b26a1fd50ab21cb56f8f1a1}"
TAG_OBJECT="${NTPRO_V311_POST_PUBLICATION_TAG_OBJECT:-8c0d71f6e6ef2a890daf1e07299c658fa187a262}"
TAG_TREE="${NTPRO_V311_POST_PUBLICATION_TAG_TREE:-7ace21b252c8a14b66eae9642baa2fd4ad3b895a}"
PUBLISHED_AT="${NTPRO_V311_POST_PUBLICATION_PUBLISHED_AT:-2026-07-13T22:42:06Z}"
GATE_RUN_ID="${NTPRO_V311_POST_PUBLICATION_GATE_RUN_ID:-29285960500}"
GATE_URL="${NTPRO_V311_POST_PUBLICATION_GATE_URL:-https://github.com/atxinbao/NTPRO/actions/runs/29285960500}"
GATE_COMPLETED_AT="${NTPRO_V311_POST_PUBLICATION_GATE_COMPLETED_AT:-2026-07-13T22:41:01Z}"
GATE_JOBS_TOTAL="${NTPRO_V311_POST_PUBLICATION_GATE_JOBS_TOTAL:-96}"
GATE_JOBS_SUCCESS="${NTPRO_V311_POST_PUBLICATION_GATE_JOBS_SUCCESS:-96}"
PUBLISH_RUN_ID="${NTPRO_V311_POST_PUBLICATION_PUBLISH_RUN_ID:-29290691138}"
PUBLISH_URL="${NTPRO_V311_POST_PUBLICATION_PUBLISH_URL:-https://github.com/atxinbao/NTPRO/actions/runs/29290691138}"
PUBLISH_COMPLETED_AT="${NTPRO_V311_POST_PUBLICATION_PUBLISH_COMPLETED_AT:-2026-07-13T22:42:11Z}"
MILESTONE_NUMBER="${NTPRO_V311_POST_PUBLICATION_MILESTONE_NUMBER:-28}"
MILESTONE_TITLE="${NTPRO_V311_POST_PUBLICATION_MILESTONE_TITLE:-v0.31.0}"

MANIFEST_PATH="${NTPRO_V311_POST_PUBLICATION_MANIFEST:-docs/rust-cutover/release/v0_31_0_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V311_POST_PUBLICATION_NOTES:-docs/rust-cutover/release/v0_31_0_release_notes.md}"
READINESS_PATH="${NTPRO_V311_POST_PUBLICATION_READINESS:-docs/rust-cutover/release/v0_31_0_readiness_report.md}"
CLOSEOUT_PATH="${NTPRO_V311_POST_PUBLICATION_CLOSEOUT:-docs/rust-cutover/release/v0_31_0_release_closeout_evidence.md}"
TASK_PATH="${NTPRO_V311_POST_PUBLICATION_TASK:-docs/rust-cutover/tasks/V311-001.md}"
EVIDENCE_PATH="${NTPRO_V311_POST_PUBLICATION_EVIDENCE:-docs/rust-cutover/evidence/V311-001.md}"

fail() {
  echo "v31.1 post-publication closeout evidence failed: $*" >&2
  exit 1
}

require_file() {
  local path="$1"
  [[ -f "$path" ]] || fail "missing required file: $path"
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
  "$READINESS_PATH" \
  "$CLOSEOUT_PATH" \
  "$TASK_PATH" \
  "$EVIDENCE_PATH"; do
  require_file "$path"
done

run_source_validation() {
  MANIFEST_PATH="$MANIFEST_PATH" \
  RELEASE_NOTES_PATH="$RELEASE_NOTES_PATH" \
  READINESS_PATH="$READINESS_PATH" \
  CLOSEOUT_PATH="$CLOSEOUT_PATH" \
  TASK_PATH="$TASK_PATH" \
  EVIDENCE_PATH="$EVIDENCE_PATH" \
  RELEASE_VERSION="$RELEASE_VERSION" \
  RELEASE_TAG="$RELEASE_TAG" \
  RELEASE_NAME="$RELEASE_NAME" \
  RELEASE_URL="$RELEASE_URL" \
  TAG_SHA="$TAG_SHA" \
  TAG_OBJECT="$TAG_OBJECT" \
  TAG_TREE="$TAG_TREE" \
  PUBLISHED_AT="$PUBLISHED_AT" \
  GATE_RUN_ID="$GATE_RUN_ID" \
  GATE_URL="$GATE_URL" \
  GATE_COMPLETED_AT="$GATE_COMPLETED_AT" \
  GATE_JOBS_TOTAL="$GATE_JOBS_TOTAL" \
  GATE_JOBS_SUCCESS="$GATE_JOBS_SUCCESS" \
  PUBLISH_RUN_ID="$PUBLISH_RUN_ID" \
  PUBLISH_URL="$PUBLISH_URL" \
  PUBLISH_COMPLETED_AT="$PUBLISH_COMPLETED_AT" \
  MILESTONE_NUMBER="$MILESTONE_NUMBER" \
  MILESTONE_TITLE="$MILESTONE_TITLE" \
  python3 <<'PY'
import json
import os
from datetime import datetime, timezone
from pathlib import Path


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def require_contains(text: str, marker: str, label: str) -> None:
    require(marker in text, f"{label} missing marker: {marker}")


def parse_ts(value: str) -> datetime:
    parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)


manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))
notes = Path(os.environ["RELEASE_NOTES_PATH"]).read_text(encoding="utf-8")
readiness = Path(os.environ["READINESS_PATH"]).read_text(encoding="utf-8")
closeout = Path(os.environ["CLOSEOUT_PATH"]).read_text(encoding="utf-8")
task = Path(os.environ["TASK_PATH"]).read_text(encoding="utf-8")
evidence = Path(os.environ["EVIDENCE_PATH"]).read_text(encoding="utf-8")

expected_issues = [1006, 1007, 1008, 1009, 1010, 1011, 1012, 1013, 1014, 1015, 1033]
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

require(manifest.get("schema_version") == "ntpro.v310_release_manifest.v1", "manifest schema mismatch")
require(manifest.get("product_version") == os.environ["RELEASE_VERSION"], "manifest version mismatch")
require(manifest.get("release_status") == "released", "manifest must be released after publication")

planned = manifest.get("planned_release") or {}
require(planned.get("tag") == os.environ["RELEASE_TAG"], "planned release tag mismatch")
require(planned.get("name") == os.environ["RELEASE_NAME"], "planned release name mismatch")

published = manifest.get("published_release") or {}
require(published.get("tag") == os.environ["RELEASE_TAG"], "published release tag mismatch")
require(published.get("name") == os.environ["RELEASE_NAME"], "published release name mismatch")
require(published.get("github_release_url") == os.environ["RELEASE_URL"], "published release URL mismatch")
require(published.get("draft") is False, "published release must not be draft")
require(published.get("prerelease") is False, "published release must not be prerelease")
require(published.get("target_commitish") == "main", "published target commitish mismatch")
require(published.get("published_at") == os.environ["PUBLISHED_AT"], "published_at mismatch")
require(published.get("tag_object_sha") == os.environ["TAG_OBJECT"], "tag object mismatch")
require(published.get("tag_sha") == os.environ["TAG_SHA"], "tag SHA mismatch")
require(published.get("tag_tree_sha") == os.environ["TAG_TREE"], "tag tree mismatch")
require(published.get("release_gate_run_id") == int(os.environ["GATE_RUN_ID"]), "release gate run mismatch")
require(published.get("release_gate_url") == os.environ["GATE_URL"], "release gate URL mismatch")
require(published.get("release_gate_completed_at") == os.environ["GATE_COMPLETED_AT"], "release gate completed_at mismatch")
require(published.get("release_gate_conclusion") == "success", "release gate conclusion mismatch")
require(published.get("release_gate_jobs_success") == int(os.environ["GATE_JOBS_SUCCESS"]), "release gate jobs_success mismatch")
require(published.get("release_gate_jobs_total") == int(os.environ["GATE_JOBS_TOTAL"]), "release gate jobs_total mismatch")
require(published.get("publish_workflow_run_id") == int(os.environ["PUBLISH_RUN_ID"]), "publish workflow run mismatch")
require(published.get("publish_workflow_url") == os.environ["PUBLISH_URL"], "publish workflow URL mismatch")
require(published.get("publish_workflow_completed_at") == os.environ["PUBLISH_COMPLETED_AT"], "publish workflow completed_at mismatch")
require(published.get("publish_workflow_conclusion") == "success", "publish workflow conclusion mismatch")
require(published.get("published_after_hosted_gate") is True, "published after hosted gate missing")
require(parse_ts(published["published_at"]) >= parse_ts(published["release_gate_completed_at"]), "release must publish after gate")

release_hash = "1fed8bfaa9f73d24d4392ed203cbf12d6c90d0cdabcbc29ec9c87151041aa355"
release_raw_hash = "92bd2f48a42fc706173aa1971efe95c1f3559a86fa57e54f71b8d3692b922744"
require(published.get("release_body_hash_semantics") == "normalized_sha256", "release body hash semantics mismatch")
require(published.get("release_body_normalized_sha256") == release_hash, "release body normalized hash mismatch")
require(published.get("release_body_raw_sha256") == release_raw_hash, "release body raw hash mismatch")
require(published.get("tracked_release_notes_normalized_sha256_at_publication") == release_hash, "publication notes normalized hash mismatch")
require(published.get("tracked_release_notes_raw_sha256_at_publication") == release_raw_hash, "publication notes raw hash mismatch")
require(published.get("release_body_matches_tracked_release_notes_at_publication") is True, "publication body match marker missing")
require(published.get("release_body_raw_matches_tracked_release_notes_at_publication") is True, "publication raw body match marker missing")
require(published.get("current_release_body_reconciliation_required") is True, "body reconciliation marker missing")
require(published.get("current_release_body_reconciliation_issue") == 1038, "body reconciliation issue mismatch")

scope = manifest.get("release_scope") or {}
require(scope.get("exact_milestone_issue_numbers") == expected_issues, "exact V310 issue set mismatch")
require(scope.get("final_release_scope_issue_count") == 11, "final issue count mismatch")
require(scope.get("final_release_scope_evidence_count") == 11, "final evidence count mismatch")

post = manifest.get("post_publication_closeout") or {}
require(post.get("status") == "source_controlled_closeout_recorded", "post-publication status mismatch")
require(post.get("source_controlled_closeout_evidence_path") == os.environ["CLOSEOUT_PATH"], "source closeout path mismatch")
require(post.get("generated_publication_evidence_sole_proof_allowed") is False, "generated-only proof must be false")
require(post.get("release_gate_run_id") == int(os.environ["GATE_RUN_ID"]), "post-publication gate run mismatch")
require(post.get("publish_workflow_run_id") == int(os.environ["PUBLISH_RUN_ID"]), "post-publication publish run mismatch")
require(post.get("milestone_number") == int(os.environ["MILESTONE_NUMBER"]), "milestone number mismatch")
require(post.get("milestone_title") == os.environ["MILESTONE_TITLE"], "milestone title mismatch")
require(post.get("milestone_state") == "closed", "milestone state mismatch")
require(post.get("milestone_open_issues") == 0, "milestone open issues mismatch")
require(post.get("milestone_closed_issues") == 11, "milestone closed issues mismatch")
require(post.get("exact_issue_numbers") == expected_issues, "post-publication exact issue numbers mismatch")
require(post.get("release_body_normalized_sha256") == release_hash, "post-publication body hash mismatch")
require(post.get("release_body_raw_sha256") == release_raw_hash, "post-publication raw body hash mismatch")
require(post.get("v32_backend_closeout_blocked_until_v311_release_evidence") is True, "v32 v311 blocker missing")

requirements = manifest.get("post_publication_requirements") or {}
for key in [
    "all_v310_issues_closed_required",
    "hosted_release_gate_success_required",
    "publication_after_hosted_gate_required",
    "same_tag_commit_required",
    "source_controlled_closeout_evidence_required",
    "publish_workflow_success_required",
    "release_body_hash_record_required",
]:
    require(requirements.get(key) is True, f"post-publication requirement missing: {key}")
require(requirements.get("generated_publication_evidence_sole_proof_allowed") is False, "generated-only requirement mismatch")
require(requirements.get("v0_32_start_gate_fails_without_v311_release_evidence") is True, "v32 v311 start gate requirement missing")

for key in false_flags:
    require(manifest["boundary_flags"].get(key) is False, f"boundary flag must remain false: {key}")

for label, text in {"release notes": notes, "readiness report": readiness}.items():
    require_contains(text, "Status: RELEASED", label)
    require("Status: RELEASE GATE READY" not in text, f"{label} still uses release-gate-ready status")
    require_contains(text, "published release status = published_after_gate", label)
    require_contains(text, "published release closeout evidence = docs/rust-cutover/release/v0_31_0_release_closeout_evidence.md", label)
    require_contains(text, "hosted release gate run = 29285960500", label)
    require_contains(text, "publish workflow run = 29290691138", label)
    require_contains(text, "release body hash semantics = normalized_sha256", label)
    require_contains(text, "GitHub Release body released-state reconciliation = V311-003 / #1038", label)

for marker in [
    "Status: CLOSEOUT EVIDENCE RECORDED",
    "release tag = ntpro-rust-only-v0.31.0",
    "hosted release gate jobs = 96/96 success",
    "publish workflow jobs = 1/1 success",
    "release body normalized sha256 = 1fed8bfaa9f73d24d4392ed203cbf12d6c90d0cdabcbc29ec9c87151041aa355",
    "V310 final release issue set = 11/11 closed",
    "v0.31.0 milestone state = closed",
    "v0.32.0 backend closeout start rule = blocked until all V311 issues close and ntpro-rust-only-v0.31.1 release evidence is published",
    "generated-evidence-only proof accepted = false",
]:
    require_contains(closeout, marker, "closeout evidence")

for marker in [
    "GitHub issue: `#1036`",
    "Blocks v0.32.0 backend production closeout until all V311 issues close",
    "GitHub Release body reconciliation, which is tracked by V311-003 / #1038",
]:
    require_contains(task, marker, "V311-001 task")

for marker in [
    "Task: `V311-001` / GitHub issue `#1036`",
    "hosted release gate = 96/96 success",
    "publish workflow = 1/1 success",
    "v0.32.0 remains blocked by v0.31.1 release evidence = true",
]:
    require_contains(evidence, marker, "V311-001 evidence")
PY
}

run_live_validation() {
  command -v gh >/dev/null 2>&1 || fail "gh_unavailable"
  gh auth status >/dev/null 2>&1 || fail "gh_auth_unavailable"

  release_json="$(gh_with_retry release view "$RELEASE_TAG" --repo "$REPO" --json tagName,name,isDraft,isPrerelease,url,publishedAt,createdAt,targetCommitish,author,body,databaseId,id)"
  gate_json="$(gh_with_retry run view "$GATE_RUN_ID" --repo "$REPO" --json status,conclusion,headSha,url,createdAt,updatedAt,workflowName,jobs --jq '{workflowName,status,conclusion,headSha,url,createdAt,updatedAt,jobs_success:([.jobs[] | select(.conclusion=="success")] | length),jobs_total:(.jobs | length)}')"
  publish_json="$(gh_with_retry run view "$PUBLISH_RUN_ID" --repo "$REPO" --json status,conclusion,headSha,url,createdAt,updatedAt,workflowName,jobs --jq '{workflowName,status,conclusion,headSha,url,createdAt,updatedAt,jobs_success:([.jobs[] | select(.conclusion=="success")] | length),jobs_total:(.jobs | length)}')"
  milestone_json="$(gh_with_retry api "repos/$REPO/milestones/$MILESTONE_NUMBER" --jq '{number,title,state,open_issues,closed_issues,closed_at}')"
  issues_json="$(gh_with_retry issue list --repo "$REPO" --milestone "$MILESTONE_TITLE" --state all --limit 100 --json number,state)"

  MANIFEST_PATH="$MANIFEST_PATH" \
  RELEASE_JSON="$release_json" \
  GATE_JSON="$gate_json" \
  PUBLISH_JSON="$publish_json" \
  MILESTONE_JSON="$milestone_json" \
  ISSUES_JSON="$issues_json" \
  RELEASE_TAG="$RELEASE_TAG" \
  RELEASE_NAME="$RELEASE_NAME" \
  RELEASE_URL="$RELEASE_URL" \
  TAG_SHA="$TAG_SHA" \
  PUBLISHED_AT="$PUBLISHED_AT" \
  GATE_RUN_ID="$GATE_RUN_ID" \
  GATE_JOBS_TOTAL="$GATE_JOBS_TOTAL" \
  PUBLISH_RUN_ID="$PUBLISH_RUN_ID" \
  PUBLISH_COMPLETED_AT="$PUBLISH_COMPLETED_AT" \
  MILESTONE_NUMBER="$MILESTONE_NUMBER" \
  MILESTONE_TITLE="$MILESTONE_TITLE" \
  python3 <<'PY'
import hashlib
import json
import os
from pathlib import Path


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def normalize(value: str) -> str:
    return "\n".join(line.rstrip() for line in value.splitlines()).strip()


manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))
release = json.loads(os.environ["RELEASE_JSON"])
gate = json.loads(os.environ["GATE_JSON"])
publish = json.loads(os.environ["PUBLISH_JSON"])
milestone = json.loads(os.environ["MILESTONE_JSON"])
issues = json.loads(os.environ["ISSUES_JSON"])
expected_issues = [1006, 1007, 1008, 1009, 1010, 1011, 1012, 1013, 1014, 1015, 1033]

require(release["tagName"] == os.environ["RELEASE_TAG"], "live release tag mismatch")
require(release["name"] == os.environ["RELEASE_NAME"], "live release name mismatch")
require(release["url"] == os.environ["RELEASE_URL"], "live release URL mismatch")
require(release["isDraft"] is False, "live release must not be draft")
require(release["isPrerelease"] is False, "live release must not be prerelease")
require(release["publishedAt"] == os.environ["PUBLISHED_AT"], "live publishedAt mismatch")
require(release["targetCommitish"] == "main", "live target commitish mismatch")

body_hash = hashlib.sha256(normalize(release.get("body") or "").encode("utf-8")).hexdigest()
require(body_hash == manifest["published_release"]["release_body_normalized_sha256"], "live release body hash mismatch")

require(gate["workflowName"] == "Rust Cutover Release Gate", "live gate workflow mismatch")
require(gate["status"] == "completed", "live gate status mismatch")
require(gate["conclusion"] == "success", "live gate conclusion mismatch")
require(gate["headSha"] == os.environ["TAG_SHA"], "live gate head SHA mismatch")
require(gate["jobs_success"] == int(os.environ["GATE_JOBS_TOTAL"]), "live gate jobs_success mismatch")
require(gate["jobs_total"] == int(os.environ["GATE_JOBS_TOTAL"]), "live gate jobs_total mismatch")

require(publish["workflowName"] == "Rust Cutover Publish Release", "live publish workflow mismatch")
require(publish["status"] == "completed", "live publish status mismatch")
require(publish["conclusion"] == "success", "live publish conclusion mismatch")
require(publish["headSha"] == os.environ["TAG_SHA"], "live publish head SHA mismatch")
require(publish["updatedAt"] == os.environ["PUBLISH_COMPLETED_AT"], "live publish completed_at mismatch")
require(publish["jobs_success"] == 1 and publish["jobs_total"] == 1, "live publish jobs mismatch")

require(milestone["number"] == int(os.environ["MILESTONE_NUMBER"]), "live milestone number mismatch")
require(milestone["title"] == os.environ["MILESTONE_TITLE"], "live milestone title mismatch")
require(milestone["state"] == "closed", "live milestone state mismatch")
require(milestone["open_issues"] == 0, "live milestone open issue mismatch")
require(milestone["closed_issues"] == 11, "live milestone closed issue mismatch")

observed = sorted(item["number"] for item in issues)
states = {item["number"]: item["state"] for item in issues}
require(observed == expected_issues, f"live issue set mismatch: {observed}")
for number in expected_issues:
    require(states[number] == "CLOSED", f"live issue must be closed: {number}")
PY
}

case "$MODE" in
  source)
    run_source_validation
    ;;
  live)
    run_source_validation
    run_live_validation
    ;;
  *)
    fail "unsupported mode: $MODE"
    ;;
esac

echo "v31_1_post_publication_closeout_evidence status=ok mode=$MODE release_tag=$RELEASE_TAG release_gate_run=$GATE_RUN_ID publish_run=$PUBLISH_RUN_ID milestone=$MILESTONE_TITLE:closed issues=11/11"
