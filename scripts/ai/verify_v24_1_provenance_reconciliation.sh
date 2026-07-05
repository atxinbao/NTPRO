#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V241_PROVENANCE_REPO:-atxinbao/NTPRO}"
RELEASE_TAG="${NTPRO_V241_PROVENANCE_RELEASE_TAG:-ntpro-rust-only-v0.24.0}"
RELEASE_NAME="${NTPRO_V241_PROVENANCE_RELEASE_NAME:-NTPRO Rust-only v0.24.0}"
RELEASE_URL="${NTPRO_V241_PROVENANCE_RELEASE_URL:-https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.24.0}"
PUBLISHED_AT="${NTPRO_V241_PROVENANCE_PUBLISHED_AT:-2026-07-05T03:59:29Z}"

TAG_SHA="${NTPRO_V241_PROVENANCE_TAG_SHA:-fff22c4e36b85098b4b32a35762a873f93d16587}"
TAG_TREE="${NTPRO_V241_PROVENANCE_TAG_TREE:-287adca8a02aaada2bc78d49277568751a4bbe46}"
TAG_NOTES_SHA256="${NTPRO_V241_PROVENANCE_TAG_NOTES_SHA256:-92cb335a5d7a071cde4be738f3d632a3b64ed56e8812f001704ae64bdd4756ca}"
TAG_NOTES_BYTES="${NTPRO_V241_PROVENANCE_TAG_NOTES_BYTES:-5263}"

PR769_NUMBER="${NTPRO_V241_PROVENANCE_PR769_NUMBER:-769}"
PR769_HEAD_SHA="${NTPRO_V241_PROVENANCE_PR769_HEAD_SHA:-7f33daaee9071792541e7152af3ecdd0124fb4d5}"
PR769_MERGE_SHA="${NTPRO_V241_PROVENANCE_PR769_MERGE_SHA:-f590023fd8e62323f3a3a5f08e970e5376ba73cb}"
PR769_MERGED_AT="${NTPRO_V241_PROVENANCE_PR769_MERGED_AT:-2026-07-05T04:03:58Z}"
PR769_NOTES_SHA256="${NTPRO_V241_PROVENANCE_PR769_NOTES_SHA256:-53c7c59d2585c7b8e710c59b0707156e6c9f3107eeb9e0decf8cbc0a3c4a5570}"
PR769_NOTES_BYTES="${NTPRO_V241_PROVENANCE_PR769_NOTES_BYTES:-5261}"

PR786_NUMBER="${NTPRO_V241_PROVENANCE_PR786_NUMBER:-786}"
PR786_HEAD_SHA="${NTPRO_V241_PROVENANCE_PR786_HEAD_SHA:-57966f5c44d1a10a6a43f2f0c7ecd70c352736fc}"
PR786_MERGE_SHA="${NTPRO_V241_PROVENANCE_PR786_MERGE_SHA:-581d5775a3f3589e16dfbb2758432869b78a1212}"
PR786_MERGED_AT="${NTPRO_V241_PROVENANCE_PR786_MERGED_AT:-2026-07-05T10:24:49Z}"

RELEASE_BODY_SHA256="${NTPRO_V241_PROVENANCE_RELEASE_BODY_SHA256:-53c7c59d2585c7b8e710c59b0707156e6c9f3107eeb9e0decf8cbc0a3c4a5570}"
RELEASE_BODY_BYTES="${NTPRO_V241_PROVENANCE_RELEASE_BODY_BYTES:-5261}"

RELEASE_NOTES_PATH="${NTPRO_V241_PROVENANCE_RELEASE_NOTES:-docs/rust-cutover/release/v0_24_0_release_notes.md}"
MANIFEST_PATH="${NTPRO_V241_PROVENANCE_MANIFEST:-docs/rust-cutover/release/v0_24_0_release_manifest.json}"
REPORT_PATH="${NTPRO_V241_PROVENANCE_REPORT:-docs/rust-cutover/release/v0_24_0_provenance_reconciliation.md}"
EVIDENCE_PATH="${NTPRO_V241_PROVENANCE_EVIDENCE:-docs/rust-cutover/evidence/V241-002.md}"
TASK_PATH="${NTPRO_V241_PROVENANCE_TASK:-docs/rust-cutover/tasks/V241-002.md}"
CLOSEOUT_EVIDENCE_PATH="${NTPRO_V241_CLOSEOUT_EVIDENCE:-docs/rust-cutover/release/v0_24_0_release_closeout_evidence.md}"

fail() {
  echo "v24.1 provenance reconciliation failed: $*" >&2
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
  "$RELEASE_NOTES_PATH" \
  "$MANIFEST_PATH" \
  "$REPORT_PATH" \
  "$EVIDENCE_PATH" \
  "$TASK_PATH" \
  "$CLOSEOUT_EVIDENCE_PATH"; do
  require_file "$path"
done

for marker in \
  "$RELEASE_TAG" \
  "$TAG_SHA" \
  "$TAG_TREE" \
  "$TAG_NOTES_SHA256" \
  "$PR769_MERGE_SHA" \
  "$PR769_HEAD_SHA" \
  "$PR769_MERGED_AT" \
  "$PR769_NOTES_SHA256" \
  "$PR786_MERGE_SHA" \
  "$PR786_HEAD_SHA" \
  "$PR786_MERGED_AT" \
  "$RELEASE_BODY_SHA256" \
  "strategy = patch_closeout_record_not_retag" \
  "retag required = false" \
  "current source-tree release notes equal GitHub Release body = true" \
  "release notes changed after PR #769 = false" \
  "runtime files changed by PR #769 = false" \
  "trading behavior changed by PR #769 = false"; do
  require_contains "$REPORT_PATH" "$marker"
done

for marker in \
  "$RELEASE_TAG" \
  "$TAG_SHA" \
  "$TAG_NOTES_SHA256" \
  "$PR769_MERGE_SHA" \
  "$PR769_NOTES_SHA256" \
  "$RELEASE_BODY_SHA256" \
  "$PR786_MERGE_SHA" \
  "retag required = false" \
  "strategy = patch_closeout_record_not_retag"; do
  require_contains "$EVIDENCE_PATH" "$marker"
done

if ! command -v gh >/dev/null 2>&1; then
  fail "gh is required for live release body proof"
fi
gh auth status >/dev/null 2>&1 || fail "gh authentication is required for live release body proof"

release_json="$(gh_with_retry release view "$RELEASE_TAG" --repo "$REPO" --json tagName,name,isDraft,isPrerelease,url,publishedAt,targetCommitish,body)"
pr769_json="$(gh_with_retry pr view "$PR769_NUMBER" --repo "$REPO" --json number,title,state,mergedAt,mergeCommit,headRefOid,changedFiles,additions,deletions,files,url)"
pr786_json="$(gh_with_retry pr view "$PR786_NUMBER" --repo "$REPO" --json number,title,state,mergedAt,mergeCommit,headRefOid,changedFiles,additions,deletions,url)"

RELEASE_JSON="$release_json" \
PR769_JSON="$pr769_json" \
PR786_JSON="$pr786_json" \
RELEASE_TAG="$RELEASE_TAG" \
RELEASE_NAME="$RELEASE_NAME" \
RELEASE_URL="$RELEASE_URL" \
PUBLISHED_AT="$PUBLISHED_AT" \
TAG_SHA="$TAG_SHA" \
TAG_TREE="$TAG_TREE" \
TAG_NOTES_SHA256="$TAG_NOTES_SHA256" \
TAG_NOTES_BYTES="$TAG_NOTES_BYTES" \
PR769_HEAD_SHA="$PR769_HEAD_SHA" \
PR769_MERGE_SHA="$PR769_MERGE_SHA" \
PR769_MERGED_AT="$PR769_MERGED_AT" \
PR769_NOTES_SHA256="$PR769_NOTES_SHA256" \
PR769_NOTES_BYTES="$PR769_NOTES_BYTES" \
PR786_HEAD_SHA="$PR786_HEAD_SHA" \
PR786_MERGE_SHA="$PR786_MERGE_SHA" \
PR786_MERGED_AT="$PR786_MERGED_AT" \
RELEASE_BODY_SHA256="$RELEASE_BODY_SHA256" \
RELEASE_BODY_BYTES="$RELEASE_BODY_BYTES" \
RELEASE_NOTES_PATH="$RELEASE_NOTES_PATH" \
MANIFEST_PATH="$MANIFEST_PATH" \
REPORT_PATH="$REPORT_PATH" \
EVIDENCE_PATH="$EVIDENCE_PATH" \
TASK_PATH="$TASK_PATH" \
CLOSEOUT_EVIDENCE_PATH="$CLOSEOUT_EVIDENCE_PATH" \
python3 <<'PY'
import hashlib
import json
import os
import subprocess
from pathlib import Path


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def git_bytes(*args: str) -> bytes:
    return subprocess.check_output(["git", *args])


def git_text(*args: str) -> str:
    return git_bytes(*args).decode("utf-8")


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def is_ancestor(ancestor: str, descendant: str) -> bool:
    return subprocess.run(
        ["git", "merge-base", "--is-ancestor", ancestor, descendant],
        check=False,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    ).returncode == 0


release = json.loads(os.environ["RELEASE_JSON"])
pr769 = json.loads(os.environ["PR769_JSON"])
pr786 = json.loads(os.environ["PR786_JSON"])
manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))

release_notes_path = os.environ["RELEASE_NOTES_PATH"]
tag_sha = os.environ["TAG_SHA"]
pr769_merge_sha = os.environ["PR769_MERGE_SHA"]
pr786_merge_sha = os.environ["PR786_MERGE_SHA"]

tag_notes = git_bytes("show", f"{tag_sha}:{release_notes_path}")
pr769_notes = git_bytes("show", f"{pr769_merge_sha}:{release_notes_path}")
current_notes = Path(release_notes_path).read_bytes()
release_body = release["body"].encode("utf-8")

require(release["tagName"] == os.environ["RELEASE_TAG"], "release tag mismatch")
require(release["name"] == os.environ["RELEASE_NAME"], "release name mismatch")
require(release["url"] == os.environ["RELEASE_URL"], "release URL mismatch")
require(release["publishedAt"] == os.environ["PUBLISHED_AT"], "release publishedAt mismatch")
require(release["isDraft"] is False, "release must not be draft")
require(release["isPrerelease"] is False, "release must not be prerelease")
require(release["targetCommitish"] == "main", "release targetCommitish mismatch")

require(sha256(tag_notes) == os.environ["TAG_NOTES_SHA256"], "tag notes hash mismatch")
require(len(tag_notes) == int(os.environ["TAG_NOTES_BYTES"]), "tag notes byte length mismatch")
require(sha256(pr769_notes) == os.environ["PR769_NOTES_SHA256"], "PR #769 notes hash mismatch")
require(len(pr769_notes) == int(os.environ["PR769_NOTES_BYTES"]), "PR #769 notes byte length mismatch")
require(sha256(current_notes) == os.environ["PR769_NOTES_SHA256"], "current release notes must match PR #769 notes hash")
require(sha256(release_body) == os.environ["RELEASE_BODY_SHA256"], "GitHub Release body hash mismatch")
require(len(release_body) == int(os.environ["RELEASE_BODY_BYTES"]), "GitHub Release body byte length mismatch")
require(current_notes == release_body, "current release notes must exactly equal GitHub Release body")
require(pr769_notes == release_body, "PR #769 notes must exactly equal GitHub Release body")
require(tag_notes != pr769_notes, "tag notes must differ from PR #769 notes for the documented drift")

tag_tree = git_text("show", "-s", "--format=%T", tag_sha).strip()
require(tag_tree == os.environ["TAG_TREE"], "tag tree mismatch")

changed_from_tag = git_text("diff", "--name-only", tag_sha, pr769_merge_sha).splitlines()
require(changed_from_tag == [release_notes_path], f"unexpected tag-to-PR769 files: {changed_from_tag}")

numstat = git_text("diff", "--numstat", tag_sha, pr769_merge_sha, "--", release_notes_path).strip()
require(numstat == f"1\t2\t{release_notes_path}", f"unexpected PR #769 numstat: {numstat}")

diff_text = git_text("diff", tag_sha, pr769_merge_sha, "--", release_notes_path)
require("-  order-ticket controls;" in diff_text, "missing removed wrapped order-ticket line")
require(
    "+- Dashboard order, approval, cancel, retry, submit, replace, amend, flatten, or order-ticket controls;"
    in diff_text,
    "missing added single-line Dashboard disabled-control sentence",
)

later_notes_diff = git_text("diff", "--name-only", pr769_merge_sha, "HEAD", "--", release_notes_path).splitlines()
require(later_notes_diff == [], f"release notes changed after PR #769: {later_notes_diff}")

require(is_ancestor(tag_sha, pr769_merge_sha), "tag must be ancestor of PR #769 merge")
require(is_ancestor(pr769_merge_sha, pr786_merge_sha), "PR #769 merge must be ancestor of V241-001 closeout")
require(is_ancestor(pr786_merge_sha, "HEAD"), "V241-001 closeout must be ancestor of current HEAD")

require(pr769["number"] == int(os.environ.get("PR769_NUMBER", "769")), "PR #769 number mismatch")
require(pr769["state"] == "MERGED", "PR #769 must be merged")
require(pr769["headRefOid"] == os.environ["PR769_HEAD_SHA"], "PR #769 head SHA mismatch")
require(pr769["mergeCommit"]["oid"] == pr769_merge_sha, "PR #769 merge SHA mismatch")
require(pr769["mergedAt"] == os.environ["PR769_MERGED_AT"], "PR #769 mergedAt mismatch")
require(pr769["changedFiles"] == 1, "PR #769 changed file count mismatch")
require(pr769["additions"] == 1, "PR #769 addition count mismatch")
require(pr769["deletions"] == 2, "PR #769 deletion count mismatch")
require(pr769["files"] == [{"path": release_notes_path, "additions": 1, "deletions": 2}], "PR #769 file list mismatch")

require(pr786["number"] == int(os.environ.get("PR786_NUMBER", "786")), "PR #786 number mismatch")
require(pr786["state"] == "MERGED", "PR #786 must be merged")
require(pr786["headRefOid"] == os.environ["PR786_HEAD_SHA"], "PR #786 head SHA mismatch")
require(pr786["mergeCommit"]["oid"] == pr786_merge_sha, "PR #786 merge SHA mismatch")
require(pr786["mergedAt"] == os.environ["PR786_MERGED_AT"], "PR #786 mergedAt mismatch")

reconciliation = manifest.get("post_release_provenance_reconciliation") or {}
require(reconciliation.get("task_id") == "V241-002", "manifest reconciliation task mismatch")
require(reconciliation.get("issue") == 771, "manifest reconciliation issue mismatch")
require(reconciliation.get("report_path") == os.environ["REPORT_PATH"], "manifest report path mismatch")
require(reconciliation.get("evidence_path") == os.environ["EVIDENCE_PATH"], "manifest evidence path mismatch")
require(reconciliation.get("task_path") == os.environ["TASK_PATH"], "manifest task path mismatch")
require(reconciliation.get("strategy") == "patch_closeout_record_not_retag", "manifest strategy mismatch")
require(reconciliation.get("retag_required") is False, "manifest retag_required must be false")
require(reconciliation.get("retag_performed") is False, "manifest retag_performed must be false")
require(reconciliation.get("retag_escalation_required") is True, "manifest retag escalation flag mismatch")

tag = reconciliation.get("tag_source") or {}
require(tag.get("tag") == os.environ["RELEASE_TAG"], "manifest tag name mismatch")
require(tag.get("commit") == tag_sha, "manifest tag commit mismatch")
require(tag.get("tree") == os.environ["TAG_TREE"], "manifest tag tree mismatch")
require(tag.get("release_notes_sha256") == os.environ["TAG_NOTES_SHA256"], "manifest tag notes hash mismatch")
require(tag.get("release_notes_bytes") == int(os.environ["TAG_NOTES_BYTES"]), "manifest tag notes bytes mismatch")

source = reconciliation.get("main_source") or {}
require(source.get("pr_769_merge_commit") == pr769_merge_sha, "manifest PR #769 merge mismatch")
require(source.get("pr_769_head_commit") == os.environ["PR769_HEAD_SHA"], "manifest PR #769 head mismatch")
require(source.get("pr_769_release_notes_sha256") == os.environ["PR769_NOTES_SHA256"], "manifest PR #769 notes hash mismatch")
require(source.get("pr_769_changed_files") == [release_notes_path], "manifest PR #769 files mismatch")
require(source.get("v241_001_closeout_merge_commit") == pr786_merge_sha, "manifest V241-001 closeout mismatch")
require(source.get("v241_001_closeout_evidence_path") == os.environ["CLOSEOUT_EVIDENCE_PATH"], "manifest closeout path mismatch")
require(source.get("release_notes_changed_after_pr_769") is False, "manifest release-notes-after-PR769 flag mismatch")

body = reconciliation.get("release_body_source") or {}
require(body.get("github_release_url") == os.environ["RELEASE_URL"], "manifest release URL mismatch")
require(body.get("published_at") == os.environ["PUBLISHED_AT"], "manifest publishedAt mismatch")
require(body.get("body_sha256") == os.environ["RELEASE_BODY_SHA256"], "manifest body hash mismatch")
require(body.get("body_bytes") == int(os.environ["RELEASE_BODY_BYTES"]), "manifest body byte mismatch")
require(body.get("matches_pr_769_release_notes") is True, "manifest body/PR769 match flag mismatch")
require(body.get("matches_current_source_tree_release_notes") is True, "manifest body/current notes match flag mismatch")

drift = reconciliation.get("drift_classification") or {}
require(drift.get("tag_to_pr_769") == "explained_doc_only_release_body_sync", "manifest tag-to-PR769 drift mismatch")
require(drift.get("pr_769_to_v241_001") == "explained_closeout_evidence_only", "manifest PR769-to-closeout drift mismatch")
require(drift.get("runtime_behavior_changed") is False, "manifest runtime behavior flag mismatch")
require(drift.get("trading_behavior_changed") is False, "manifest trading behavior flag mismatch")
require(drift.get("public_api_changed") is False, "manifest public API flag mismatch")

for key, value in (reconciliation.get("boundary") or {}).items():
    require(value is False, f"manifest boundary must be false: {key}")

print(
    "v24_1_provenance_reconciliation status=ok "
    f"tag_sha={tag_sha} "
    f"tag_notes_sha256={os.environ['TAG_NOTES_SHA256']} "
    f"pr769_merge={pr769_merge_sha} "
    f"release_body_sha256={os.environ['RELEASE_BODY_SHA256']} "
    f"v241001_merge={pr786_merge_sha} "
    "strategy=patch_closeout_record_not_retag"
)
PY
