#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

RELEASE_VERSION="${NTPRO_V231_CLOSEOUT_RELEASE_VERSION:-v0.23.0}"
RELEASE_TAG="${NTPRO_V231_CLOSEOUT_RELEASE_TAG:-ntpro-rust-only-v0.23.0}"
RELEASE_NAME="${NTPRO_V231_CLOSEOUT_RELEASE_NAME:-NTPRO Rust-only v0.23.0}"
TAG_SHA="${NTPRO_V231_CLOSEOUT_TAG_SHA:-783b024621116d50feaf418f12cb95fb95f87575}"
TAG_TREE="${NTPRO_V231_CLOSEOUT_TAG_TREE:-1d40d9c962d7500a8b7cbcdf42a47ec30ea8a5f9}"
RELEASE_URL="${NTPRO_V231_CLOSEOUT_RELEASE_URL:-https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.23.0}"
PUBLISHED_AT="${NTPRO_V231_CLOSEOUT_PUBLISHED_AT:-2026-07-03T18:34:39Z}"
GATE_RUN_ID="${NTPRO_V231_CLOSEOUT_GATE_RUN_ID:-28673868094}"
GATE_URL="${NTPRO_V231_CLOSEOUT_GATE_URL:-https://github.com/atxinbao/NTPRO/actions/runs/28673868094}"
GATE_COMPLETED_AT="${NTPRO_V231_CLOSEOUT_GATE_COMPLETED_AT:-2026-07-03T18:29:30Z}"
GATE_JOBS_TOTAL="${NTPRO_V231_CLOSEOUT_GATE_JOBS_TOTAL:-66}"
GATE_JOBS_SUCCESS="${NTPRO_V231_CLOSEOUT_GATE_JOBS_SUCCESS:-66}"
MILESTONE_NUMBER="${NTPRO_V231_CLOSEOUT_MILESTONE_NUMBER:-11}"

MANIFEST_PATH="${NTPRO_V231_CLOSEOUT_MANIFEST:-docs/rust-cutover/release/v0_23_0_release_manifest.json}"
READINESS_PATH="${NTPRO_V231_CLOSEOUT_READINESS:-docs/rust-cutover/release/v0_23_0_readiness_report.md}"
V230_EVIDENCE_PATH="${NTPRO_V231_CLOSEOUT_V230_EVIDENCE:-docs/rust-cutover/evidence/V230-007.md}"
CLOSEOUT_EVIDENCE_PATH="${NTPRO_V231_CLOSEOUT_EVIDENCE:-docs/rust-cutover/release/v0_23_0_release_closeout_evidence.md}"
PUBLICATION_AUDIT_PATH="${NTPRO_V231_CLOSEOUT_PUBLICATION_AUDIT:-docs/rust-cutover/release/v0_23_0_publication_evidence_audit_path.md}"
V231_EVIDENCE_PATH="${NTPRO_V231_EVIDENCE:-docs/rust-cutover/evidence/V231-001.md}"
V231_TASK_PATH="${NTPRO_V231_TASK:-docs/rust-cutover/tasks/V231-001.md}"

fail() {
  echo "v23.1 release closeout evidence failed: $*" >&2
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
    fail "stale marker in $path: $marker"
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
  "$READINESS_PATH" \
  "$V230_EVIDENCE_PATH" \
  "$CLOSEOUT_EVIDENCE_PATH" \
  "$PUBLICATION_AUDIT_PATH" \
  "$V231_EVIDENCE_PATH" \
  "$V231_TASK_PATH"; do
  require_file "$path"
done

for marker in \
  "$RELEASE_TAG" \
  "$RELEASE_NAME" \
  "$RELEASE_URL" \
  "$PUBLISHED_AT" \
  "$TAG_SHA" \
  "$GATE_URL" \
  "$GATE_COMPLETED_AT" \
  "hosted release gate jobs = ${GATE_JOBS_SUCCESS}/${GATE_JOBS_TOTAL} success" \
  "publication evidence strategy = source_tree_plus_github_remote" \
  "publication evidence audit path = $PUBLICATION_AUDIT_PATH" \
  "local generated publication evidence required in source tree = false" \
  "V230 issue set = 8/8 closed" \
  "v0.23.0 milestone state = closed" \
  "v0.24.0 start rule = blocked until all V231 issues are closed and v0.23.1 release evidence is published"; do
  require_contains "$CLOSEOUT_EVIDENCE_PATH" "$marker"
done

for marker in \
  "Status: RELEASED - CLOSEOUT EVIDENCE RECORDED" \
  "final tag gate run = $GATE_RUN_ID" \
  "final tag gate result = success" \
  "final tag gate jobs = ${GATE_JOBS_SUCCESS}/${GATE_JOBS_TOTAL} success" \
  "public release publication = published_after_gate" \
  "#718 closeout = closed after tag, hosted gate, public release, and publication evidence were recorded"; do
  require_contains "$V230_EVIDENCE_PATH" "$marker"
done

for marker in \
  "#718 V230-007 = closed after tag, hosted gate, public release, and publication evidence were recorded" \
  "V230 issue set = 8/8 closed" \
  "v0.23.0 milestone = closed" \
  "release closeout evidence = $CLOSEOUT_EVIDENCE_PATH" \
  "hosted release gate jobs = ${GATE_JOBS_SUCCESS}/${GATE_JOBS_TOTAL} success" \
  "tag SHA = $TAG_SHA"; do
  require_contains "$READINESS_PATH" "$marker"
done

for marker in \
  "public release publication = pending" \
  "tag gate run = pending" \
  "tag gate result = pending" \
  "RELEASE GATE CORRECTIVE FIX IN PROGRESS" \
  "corrective fix in progress" \
  "#718 V230-007 = stays open until tag"; do
  require_not_contains "$V230_EVIDENCE_PATH" "$marker"
  require_not_contains "$READINESS_PATH" "$marker"
  require_not_contains "$CLOSEOUT_EVIDENCE_PATH" "$marker"
done

RELEASE_VERSION="$RELEASE_VERSION" \
RELEASE_TAG="$RELEASE_TAG" \
RELEASE_NAME="$RELEASE_NAME" \
RELEASE_URL="$RELEASE_URL" \
PUBLISHED_AT="$PUBLISHED_AT" \
TAG_SHA="$TAG_SHA" \
TAG_TREE="$TAG_TREE" \
GATE_RUN_ID="$GATE_RUN_ID" \
GATE_URL="$GATE_URL" \
GATE_COMPLETED_AT="$GATE_COMPLETED_AT" \
GATE_JOBS_TOTAL="$GATE_JOBS_TOTAL" \
GATE_JOBS_SUCCESS="$GATE_JOBS_SUCCESS" \
MANIFEST_PATH="$MANIFEST_PATH" \
CLOSEOUT_EVIDENCE_PATH="$CLOSEOUT_EVIDENCE_PATH" \
python3 <<'PY'
import json
import os
from pathlib import Path

manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


require(manifest.get("schema_version") == "ntpro.v230_release_manifest.v1", "manifest schema mismatch")
require(manifest.get("product_version") == os.environ["RELEASE_VERSION"], "manifest version mismatch")
require(manifest.get("release_status") == "released", "manifest release status mismatch")

closeout = manifest.get("post_release_closeout") or {}
require(closeout.get("closeout_evidence_path") == os.environ["CLOSEOUT_EVIDENCE_PATH"], "closeout evidence path mismatch")

release = closeout.get("github_release") or {}
require(release.get("tag") == os.environ["RELEASE_TAG"], "release tag mismatch")
require(release.get("name") == os.environ["RELEASE_NAME"], "release name mismatch")
require(release.get("url") == os.environ["RELEASE_URL"], "release URL mismatch")
require(release.get("published_at") == os.environ["PUBLISHED_AT"], "published_at mismatch")
require(release.get("draft") is False, "release must not be draft")
require(release.get("prerelease") is False, "release must not be prerelease")
require(release.get("target_commitish") == "main", "target_commitish mismatch")

tag = closeout.get("tag") or {}
require(tag.get("sha") == os.environ["TAG_SHA"], "tag SHA mismatch")
require(tag.get("tree") == os.environ["TAG_TREE"], "tag tree mismatch")
require(tag.get("matches_origin_main") is True, "tag/main match flag mismatch")

gate = closeout.get("hosted_release_gate") or {}
require(str(gate.get("run_id")) == os.environ["GATE_RUN_ID"], "gate run id mismatch")
require(gate.get("url") == os.environ["GATE_URL"], "gate URL mismatch")
require(gate.get("completed_at") == os.environ["GATE_COMPLETED_AT"], "gate completed_at mismatch")
require(gate.get("conclusion") == "success", "gate conclusion mismatch")
require(str(gate.get("jobs_total")) == os.environ["GATE_JOBS_TOTAL"], "gate jobs_total mismatch")
require(str(gate.get("jobs_success")) == os.environ["GATE_JOBS_SUCCESS"], "gate jobs_success mismatch")

publication = closeout.get("publication_evidence") or {}
require(publication.get("status") == "published_after_gate", "publication status mismatch")
require(publication.get("release_publication_after_gate") == "pass", "publication after gate mismatch")
require(publication.get("audit_source") == "source_tree_plus_github_remote", "publication audit source mismatch")
require(publication.get("tracked_audit_path") == "docs/rust-cutover/release/v0_23_0_publication_evidence_audit_path.md", "publication audit path mismatch")
require(publication.get("local_generated_evidence_required_in_source_tree") is False, "local generated evidence must not be required in source tree")
require(publication.get("remote_reconstruction_required") is True, "remote reconstruction must be required")
require(publication.get("secret_material_allowed") is False, "publication evidence must not allow secret material")

issues = closeout.get("issue_closeout") or {}
require(issues.get("closed_issues") == [711, 712, 713, 714, 715, 716, 717, 718], "closed issue set mismatch")
require(issues.get("closed_count") == 8, "closed issue count mismatch")
require(issues.get("open_count") == 0, "open issue count mismatch")

milestone = closeout.get("milestone_closeout") or {}
require(milestone.get("number") == 11, "milestone number mismatch")
require(milestone.get("title") == "v0.23.0", "milestone title mismatch")
require(milestone.get("state") == "closed", "milestone state mismatch")
require(milestone.get("open_issues") == 0, "milestone open issue count mismatch")
require(milestone.get("closed_issues") == 8, "milestone closed issue count mismatch")
PY

if ! command -v gh >/dev/null 2>&1; then
  fail "gh is required for live release closeout proof"
fi
gh auth status >/dev/null 2>&1 || fail "gh authentication is required for live release closeout proof"

release_json="$(gh_with_retry release view "$RELEASE_TAG" --json tagName,name,isDraft,isPrerelease,url,publishedAt,targetCommitish)"
RELEASE_JSON="$release_json" \
RELEASE_TAG="$RELEASE_TAG" \
RELEASE_NAME="$RELEASE_NAME" \
RELEASE_URL="$RELEASE_URL" \
PUBLISHED_AT="$PUBLISHED_AT" \
python3 <<'PY'
import json
import os

release = json.loads(os.environ["RELEASE_JSON"])
assert release["tagName"] == os.environ["RELEASE_TAG"], release
assert release["name"] == os.environ["RELEASE_NAME"], release
assert release["url"] == os.environ["RELEASE_URL"], release
assert release["publishedAt"] == os.environ["PUBLISHED_AT"], release
assert release["isDraft"] is False, release
assert release["isPrerelease"] is False, release
assert release["targetCommitish"] == "main", release
PY

remote_tag_sha="$(git ls-remote --tags origin "refs/tags/$RELEASE_TAG" | awk '{print $1}')"
[[ "$remote_tag_sha" == "$TAG_SHA" ]] || fail "remote tag SHA mismatch: $remote_tag_sha"

run_json="$(gh_with_retry run view "$GATE_RUN_ID" --json status,conclusion,updatedAt,url,headSha,jobs)"
RUN_JSON="$run_json" \
GATE_RUN_ID="$GATE_RUN_ID" \
GATE_URL="$GATE_URL" \
GATE_COMPLETED_AT="$GATE_COMPLETED_AT" \
TAG_SHA="$TAG_SHA" \
GATE_JOBS_TOTAL="$GATE_JOBS_TOTAL" \
GATE_JOBS_SUCCESS="$GATE_JOBS_SUCCESS" \
python3 <<'PY'
import json
import os

run = json.loads(os.environ["RUN_JSON"])
assert run["status"] == "completed", run
assert run["conclusion"] == "success", run
assert run["url"] == os.environ["GATE_URL"], run
assert run["updatedAt"] == os.environ["GATE_COMPLETED_AT"], run
assert run["headSha"] == os.environ["TAG_SHA"], run
jobs = run.get("jobs") or []
assert len(jobs) == int(os.environ["GATE_JOBS_TOTAL"]), len(jobs)
assert sum(1 for job in jobs if job.get("conclusion") == "success") == int(os.environ["GATE_JOBS_SUCCESS"]), jobs
PY

for issue in 711 712 713 714 715 716 717 718; do
  state="$(gh_with_retry issue view "$issue" --json state --jq .state)" || fail "could not read issue #$issue"
  [[ "$state" == "CLOSED" ]] || fail "issue #$issue must be CLOSED, got $state"
done

milestone_json="$(gh api "repos/atxinbao/NTPRO/milestones/$MILESTONE_NUMBER")"
MILESTONE_JSON="$milestone_json" python3 <<'PY'
import json
import os

milestone = json.loads(os.environ["MILESTONE_JSON"])
assert milestone["title"] == "v0.23.0", milestone
assert milestone["state"] == "closed", milestone
assert milestone["open_issues"] == 0, milestone
assert milestone["closed_issues"] == 8, milestone
PY

echo "v23_1_release_closeout_evidence status=ok release_tag=$RELEASE_TAG tag_sha=$TAG_SHA release_gate_run=$GATE_RUN_ID jobs=${GATE_JOBS_SUCCESS}/${GATE_JOBS_TOTAL} milestone=v0.23.0:closed issues=8/8 release_url=$RELEASE_URL"
