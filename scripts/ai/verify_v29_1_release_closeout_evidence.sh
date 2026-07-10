#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V291_CLOSEOUT_REPO:-atxinbao/NTPRO}"
RELEASE_VERSION="${NTPRO_V291_CLOSEOUT_RELEASE_VERSION:-v0.29.0}"
RELEASE_TAG="${NTPRO_V291_CLOSEOUT_RELEASE_TAG:-ntpro-rust-only-v0.29.0}"
RELEASE_NAME="${NTPRO_V291_CLOSEOUT_RELEASE_NAME:-NTPRO Rust-only v0.29.0}"
RELEASE_URL="${NTPRO_V291_CLOSEOUT_RELEASE_URL:-https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.29.0}"
TAG_OBJECT="${NTPRO_V291_CLOSEOUT_TAG_OBJECT:-25cccef7a99c6f231dac7f915f24abe882ad7f2c}"
TAG_SHA="${NTPRO_V291_CLOSEOUT_TAG_SHA:-85110d29867763f8d3b6395f4ff8154378b475b9}"
TAG_TREE="${NTPRO_V291_CLOSEOUT_TAG_TREE:-8c6529cba8366e191ee5b301254b1acdc7dab74a}"
PUBLISHED_AT="${NTPRO_V291_CLOSEOUT_PUBLISHED_AT:-2026-07-10T13:44:23Z}"
GATE_RUN_ID="${NTPRO_V291_CLOSEOUT_GATE_RUN_ID:-29091765148}"
GATE_URL="${NTPRO_V291_CLOSEOUT_GATE_URL:-https://github.com/atxinbao/NTPRO/actions/runs/29091765148}"
GATE_COMPLETED_AT="${NTPRO_V291_CLOSEOUT_GATE_COMPLETED_AT:-2026-07-10T13:43:15Z}"
GATE_JOBS_TOTAL="${NTPRO_V291_CLOSEOUT_GATE_JOBS_TOTAL:-88}"
GATE_JOBS_SUCCESS="${NTPRO_V291_CLOSEOUT_GATE_JOBS_SUCCESS:-88}"
MILESTONE_NUMBER="${NTPRO_V291_CLOSEOUT_MILESTONE_NUMBER:-24}"
MILESTONE_TITLE="${NTPRO_V291_CLOSEOUT_MILESTONE_TITLE:-v0.29.0}"

MANIFEST_PATH="${NTPRO_V291_CLOSEOUT_MANIFEST:-docs/rust-cutover/release/v0_29_0_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V291_CLOSEOUT_RELEASE_NOTES:-docs/rust-cutover/release/v0_29_0_release_notes.md}"
CLOSEOUT_EVIDENCE_PATH="${NTPRO_V291_CLOSEOUT_EVIDENCE:-docs/rust-cutover/release/v0_29_0_release_closeout_evidence.md}"
V291_EVIDENCE_PATH="${NTPRO_V291_EVIDENCE:-docs/rust-cutover/evidence/V291-001.md}"
V291_TASK_PATH="${NTPRO_V291_TASK:-docs/rust-cutover/tasks/V291-001.md}"

fail() {
  echo "v29.1 release closeout evidence failed: $*" >&2
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
  "$CLOSEOUT_EVIDENCE_PATH" \
  "$V291_EVIDENCE_PATH" \
  "$V291_TASK_PATH" \
  docs/rust-cutover/release/README.md; do
  require_file "$path"
done

for marker in \
  "$RELEASE_TAG" \
  "$RELEASE_NAME" \
  "$RELEASE_URL" \
  "$PUBLISHED_AT" \
  "$TAG_OBJECT" \
  "$TAG_SHA" \
  "$TAG_TREE" \
  "$GATE_URL" \
  "$GATE_COMPLETED_AT" \
  "hosted release gate jobs = ${GATE_JOBS_SUCCESS}/${GATE_JOBS_TOTAL} success" \
  "release publication after gate = pass" \
  "publication status = published_after_gate" \
  "published after hosted gate = true" \
  "release body matches tracked release notes = true" \
  "source-controlled closeout evidence = $CLOSEOUT_EVIDENCE_PATH" \
  "release-publication-evidence/ntpro-rust-only-v0.29.0.json = generated artifact, not sole proof" \
  "generated publication evidence sole proof allowed = false" \
  "V290 final release issue set = 12/12 closed" \
  "V290 exact milestone issue set = #926-#936, #961" \
  "v0.29.0 milestone state = closed" \
  "v0.30.0 start rule = hard-blocked until v0.29.1 release evidence is published"; do
  require_contains "$CLOSEOUT_EVIDENCE_PATH" "$marker"
done

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
  require_not_contains "$MANIFEST_PATH" "$marker"
  require_not_contains "$CLOSEOUT_EVIDENCE_PATH" "$marker"
done

python3 -m json.tool "$MANIFEST_PATH" >/dev/null

command -v gh >/dev/null 2>&1 || fail "gh is required for live closeout proof"
gh_with_retry auth status >/dev/null 2>&1 || fail "gh authentication is required for live closeout proof"

release_json="$(gh_with_retry api "repos/$REPO/releases/tags/$RELEASE_TAG")" || fail "missing GitHub Release: $RELEASE_TAG"
run_json="$(gh_with_retry run view "$GATE_RUN_ID" --repo "$REPO" --json status,conclusion,headSha,url,createdAt,updatedAt,workflowName,jobs)" || fail "missing hosted release gate run: $GATE_RUN_ID"
milestone_json="$(gh_with_retry api "repos/$REPO/milestones/$MILESTONE_NUMBER")" || fail "missing milestone: $MILESTONE_NUMBER"
issues_json="$(gh_with_retry issue list --repo "$REPO" --milestone "$MILESTONE_TITLE" --state all --limit 100 --json number,title,state)" || fail "missing milestone issues"
remote_tags="$(git ls-remote --tags origin "refs/tags/$RELEASE_TAG" "refs/tags/$RELEASE_TAG^{}")"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT
printf '%s' "$release_json" >"$tmp_dir/release.json"
printf '%s' "$run_json" >"$tmp_dir/run.json"
printf '%s' "$milestone_json" >"$tmp_dir/milestone.json"
printf '%s' "$issues_json" >"$tmp_dir/issues.json"
printf '%s' "$remote_tags" >"$tmp_dir/remote_tags.txt"

RELEASE_JSON_PATH="$tmp_dir/release.json" \
RUN_JSON_PATH="$tmp_dir/run.json" \
MILESTONE_JSON_PATH="$tmp_dir/milestone.json" \
ISSUES_JSON_PATH="$tmp_dir/issues.json" \
REMOTE_TAGS_PATH="$tmp_dir/remote_tags.txt" \
RELEASE_VERSION="$RELEASE_VERSION" \
RELEASE_TAG="$RELEASE_TAG" \
RELEASE_NAME="$RELEASE_NAME" \
RELEASE_URL="$RELEASE_URL" \
TAG_OBJECT="$TAG_OBJECT" \
TAG_SHA="$TAG_SHA" \
TAG_TREE="$TAG_TREE" \
PUBLISHED_AT="$PUBLISHED_AT" \
GATE_RUN_ID="$GATE_RUN_ID" \
GATE_URL="$GATE_URL" \
GATE_COMPLETED_AT="$GATE_COMPLETED_AT" \
GATE_JOBS_TOTAL="$GATE_JOBS_TOTAL" \
GATE_JOBS_SUCCESS="$GATE_JOBS_SUCCESS" \
MANIFEST_PATH="$MANIFEST_PATH" \
RELEASE_NOTES_PATH="$RELEASE_NOTES_PATH" \
CLOSEOUT_EVIDENCE_PATH="$CLOSEOUT_EVIDENCE_PATH" \
python3 <<'PY'
import hashlib
import json
import os
from datetime import datetime, timezone
from pathlib import Path


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def parse_ts(value: str) -> datetime:
    parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)


def normalize(text: str) -> str:
    return "\n".join(line.rstrip() for line in text.splitlines()).strip()


release = json.loads(Path(os.environ["RELEASE_JSON_PATH"]).read_text(encoding="utf-8"))
run = json.loads(Path(os.environ["RUN_JSON_PATH"]).read_text(encoding="utf-8"))
milestone = json.loads(Path(os.environ["MILESTONE_JSON_PATH"]).read_text(encoding="utf-8"))
issues = json.loads(Path(os.environ["ISSUES_JSON_PATH"]).read_text(encoding="utf-8"))
remote_tags = Path(os.environ["REMOTE_TAGS_PATH"]).read_text(encoding="utf-8")
manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))
notes = Path(os.environ["RELEASE_NOTES_PATH"]).read_text(encoding="utf-8")
closeout = Path(os.environ["CLOSEOUT_EVIDENCE_PATH"]).read_text(encoding="utf-8")

require(manifest.get("schema_version") == "ntpro.v290_release_manifest.v1", "manifest schema mismatch")
require(manifest.get("product_version") == os.environ["RELEASE_VERSION"], "manifest version mismatch")
require(manifest.get("release_status") == "released", "manifest release status mismatch")
post = manifest.get("post_release_closeout") or {}
require(post.get("closeout_evidence_path") == os.environ["CLOSEOUT_EVIDENCE_PATH"], "manifest closeout path mismatch")

release_manifest = post.get("github_release") or {}
require(release_manifest.get("tag") == os.environ["RELEASE_TAG"], "manifest release tag mismatch")
require(release_manifest.get("published_at") == os.environ["PUBLISHED_AT"], "manifest published_at mismatch")

require(release.get("tag_name") == os.environ["RELEASE_TAG"], "release tag mismatch")
require(release.get("name") == os.environ["RELEASE_NAME"], "release name mismatch")
require(release.get("html_url") == os.environ["RELEASE_URL"], "release URL mismatch")
require(release.get("draft") is False, "release must not be draft")
require(release.get("prerelease") is False, "release must not be prerelease")
require(release.get("target_commitish") == "main", "target commitish mismatch")
require(release.get("published_at") == os.environ["PUBLISHED_AT"], "published_at mismatch")
require(str(release.get("id")) == str(release_manifest.get("id")), "release id mismatch")
require(release.get("node_id") == release_manifest.get("node_id"), "release node id mismatch")
require((release.get("author") or {}).get("login") == "atxinbao", "release author mismatch")

body = release.get("body") or ""
body_normalized = normalize(body)
notes_normalized = normalize(notes)
body_normalized_sha = hashlib.sha256(body_normalized.encode("utf-8")).hexdigest()
notes_normalized_sha = hashlib.sha256(notes_normalized.encode("utf-8")).hexdigest()
body_raw_sha = hashlib.sha256(body.encode("utf-8")).hexdigest()
notes_raw_sha = hashlib.sha256(notes.encode("utf-8")).hexdigest()
body_manifest = post.get("release_body") or {}
require(body_normalized_sha == notes_normalized_sha, "normalized release body hash mismatch")
require(body_raw_sha == notes_raw_sha, "raw release body hash mismatch")
require(body_normalized_sha == body_manifest.get("normalized_sha256"), "manifest normalized hash mismatch")
require(body_raw_sha == body_manifest.get("raw_sha256"), "manifest raw hash mismatch")
require(len(body_normalized.splitlines()) == body_manifest.get("normalized_line_count"), "normalized line count mismatch")

require(run.get("status") == "completed", "run status mismatch")
require(run.get("conclusion") == "success", "run conclusion mismatch")
require(run.get("workflowName") == "Rust Cutover Release Gate", "workflow mismatch")
require(run.get("headSha") == os.environ["TAG_SHA"], "run head SHA mismatch")
require(run.get("url") == os.environ["GATE_URL"], "run URL mismatch")
require(run.get("updatedAt") == os.environ["GATE_COMPLETED_AT"], "run completed time mismatch")
jobs = run.get("jobs") or []
jobs_success = sum(1 for item in jobs if item.get("conclusion") == "success")
require(len(jobs) == int(os.environ["GATE_JOBS_TOTAL"]), "run job count mismatch")
require(jobs_success == int(os.environ["GATE_JOBS_SUCCESS"]), "run success count mismatch")
require(parse_ts(release.get("published_at")) >= parse_ts(run.get("updatedAt")), "release must be published after hosted gate")

require(milestone.get("number") == 24, "milestone number mismatch")
require(milestone.get("title") == "v0.29.0", "milestone title mismatch")
require(milestone.get("state") == "closed", "milestone must be closed")
require(milestone.get("open_issues") == 0, "milestone open issue count mismatch")
require(milestone.get("closed_issues") == 12, "milestone closed issue count mismatch")
require(milestone.get("closed_at") == "2026-07-10T13:45:02Z", "milestone closed_at mismatch")

expected_issues = set(range(926, 937)) | {961}
issue_states = {int(item["number"]): item["state"] for item in issues}
require(set(issue_states) == expected_issues, f"issue set mismatch: {sorted(issue_states)}")
require(all(state == "CLOSED" for state in issue_states.values()), f"not all V290 issues closed: {issue_states}")

require(os.environ["TAG_OBJECT"] in remote_tags, "remote annotated tag object missing")
require(os.environ["TAG_SHA"] in remote_tags, "remote peeled tag commit missing")
tag_manifest = post.get("tag") or {}
require(tag_manifest.get("object") == os.environ["TAG_OBJECT"], "manifest tag object mismatch")
require(tag_manifest.get("peeled_commit") == os.environ["TAG_SHA"], "manifest peeled tag mismatch")
require(tag_manifest.get("tree") == os.environ["TAG_TREE"], "manifest tag tree mismatch")

for marker in [
    "new_submit_capability = false",
    "production_order_submission_allowed = false",
    "production_order_mutation_allowed = false",
    "execution_adapter_call_allowed = false",
    "adapter_send_allowed = false",
    "live_exchange_request_allowed = false",
    "network_attempted = false",
    "retry_scheduler_enabled = false",
    "automatic_remediation_allowed = false",
    "dashboard_operation_controls_enabled = false",
    "dashboard_trading_controls_enabled = false",
    "admin_workbench_trading_controls_enabled = false",
    "trader_terminal_order_ticket_enabled = false",
    "manual_operation_submit_allowed = false",
    "backend_go_live_claim = false",
    "product_grade_trading_terminal_claim = false",
]:
    require(marker in closeout, f"missing boundary marker: {marker}")
PY

NTPRO_CURRENT_RELEASE_VERSION="$RELEASE_VERSION" \
  NTPRO_CURRENT_RELEASE_TAG="$RELEASE_TAG" \
  NTPRO_CURRENT_RELEASE_NAME="$RELEASE_NAME" \
  scripts/ai/verify_release.sh release-publication-guard

echo "v29_1_release_closeout_evidence status=ok release_tag=$RELEASE_TAG tag_sha=$TAG_SHA release_gate_run=$GATE_RUN_ID jobs=${GATE_JOBS_SUCCESS}/${GATE_JOBS_TOTAL} milestone=v0.29.0:closed issues=12/12 release_url=$RELEASE_URL"
