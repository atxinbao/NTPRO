#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V261_POST_PUBLICATION_REPO:-atxinbao/NTPRO}"
RELEASE_VERSION="${NTPRO_V261_POST_PUBLICATION_VERSION:-v0.26.0}"
RELEASE_TAG="${NTPRO_V261_POST_PUBLICATION_TAG:-ntpro-rust-only-v0.26.0}"
RELEASE_NAME="${NTPRO_V261_POST_PUBLICATION_NAME:-NTPRO Rust-only v0.26.0}"
RELEASE_URL="${NTPRO_V261_POST_PUBLICATION_URL:-https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.26.0}"
RELEASE_NOTES_PATH="${NTPRO_V261_POST_PUBLICATION_NOTES:-docs/rust-cutover/release/v0_26_0_release_notes.md}"
READINESS_PATH="${NTPRO_V261_POST_PUBLICATION_READINESS:-docs/rust-cutover/release/v0_26_0_readiness_report.md}"
CLOSEOUT_PATH="${NTPRO_V261_POST_PUBLICATION_CLOSEOUT:-docs/rust-cutover/release/v0_26_0_release_closeout_evidence.md}"
MANIFEST_PATH="${NTPRO_V261_POST_PUBLICATION_MANIFEST:-docs/rust-cutover/release/v0_26_0_release_manifest.json}"
HOSTED_GATE_RUN_ID="${NTPRO_V261_POST_PUBLICATION_GATE_RUN_ID:-28853960135}"
PUBLISH_RUN_ID="${NTPRO_V261_POST_PUBLICATION_PUBLISH_RUN_ID:-28867689146}"
MILESTONE_NUMBER="${NTPRO_V261_POST_PUBLICATION_MILESTONE_NUMBER:-18}"
EXPECTED_TAG_COMMIT="${NTPRO_V261_POST_PUBLICATION_TAG_COMMIT:-b09ec3a9f96ac718d6660b345a74cb4b7790f19a}"
EXPECTED_RELEASE_BODY_SHA256="${NTPRO_V261_POST_PUBLICATION_BODY_SHA256:-ab2ed2be9b10371e4aabea74c7314c1ebae791ffd4e3d129d0f4c208b15a985e}"
EXPECTED_RELEASE_PUBLISHED_AT="${NTPRO_V261_POST_PUBLICATION_PUBLISHED_AT:-2026-07-07T05:29:16Z}"
EXPECTED_RELEASE_UPDATED_AT="${NTPRO_V261_POST_PUBLICATION_UPDATED_AT:-2026-07-07T12:54:42Z}"
EXPECTED_PUBLISH_HEAD_SHA="${NTPRO_V261_POST_PUBLICATION_PUBLISH_HEAD_SHA:-a7f5de3086ae1624d9b4870cfda5ce47f5f4dd5c}"
GH_BIN="${NTPRO_V261_POST_PUBLICATION_GH_BIN:-gh}"

fail() {
  echo "v26.1 post-publication strict gate failed: $*" >&2
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
    if "$GH_BIN" "$@"; then
      return 0
    fi
    if (( attempt >= max_attempts )); then
      return 1
    fi
    sleep "$((attempt * 2))"
    attempt=$((attempt + 1))
  done
}

ensure_origin_main_ref() {
  if git rev-parse -q --verify origin/main^{commit} >/dev/null; then
    return 0
  fi
  git fetch --no-tags --depth=1 origin +refs/heads/main:refs/remotes/origin/main >/dev/null 2>&1 || true
}

for path in "$RELEASE_NOTES_PATH" "$READINESS_PATH" "$CLOSEOUT_PATH" "$MANIFEST_PATH"; do
  require_file "$path"
done

command -v "$GH_BIN" >/dev/null 2>&1 || fail "gh command not found: $GH_BIN"
gh_with_retry auth status >/dev/null 2>&1 || fail "gh auth is required"

git rev-parse -q --verify "${RELEASE_TAG}^{commit}" >/dev/null || fail "missing local release tag: $RELEASE_TAG"
tag_commit="$(git rev-list -n 1 "$RELEASE_TAG")"
[[ "$tag_commit" == "$EXPECTED_TAG_COMMIT" ]] || fail "local tag commit mismatch: $tag_commit"

ensure_origin_main_ref
origin_main_sha="$(git rev-parse origin/main)"
git merge-base --is-ancestor "$tag_commit" "$origin_main_sha" || fail "release tag commit is not ancestor of origin/main"

remote_tag_object="$(git ls-remote --tags origin "refs/tags/$RELEASE_TAG" | awk '{print $1}')"
remote_tag_commit="$(git ls-remote --tags origin "refs/tags/$RELEASE_TAG^{}" | awk '{print $1}')"
[[ -n "$remote_tag_object" ]] || fail "missing remote release tag object"
[[ "$remote_tag_commit" == "$tag_commit" ]] || fail "remote peeled tag commit mismatch: $remote_tag_commit"

release_json="$(gh_with_retry api "/repos/$REPO/releases/tags/$RELEASE_TAG")" || fail "could not read GitHub Release"
gate_json="$(gh_with_retry run view "$HOSTED_GATE_RUN_ID" --repo "$REPO" --json status,conclusion,headSha,url,updatedAt,workflowName)" || fail "could not read hosted release gate run"
publish_json="$(gh_with_retry run view "$PUBLISH_RUN_ID" --repo "$REPO" --json status,conclusion,headSha,url,updatedAt,workflowName)" || fail "could not read publish workflow run"
milestone_json="$(gh_with_retry api "/repos/$REPO/milestones/$MILESTONE_NUMBER")" || fail "could not read v0.26.0 milestone"

RELEASE_JSON="$release_json" \
GATE_JSON="$gate_json" \
PUBLISH_JSON="$publish_json" \
MILESTONE_JSON="$milestone_json" \
RELEASE_VERSION="$RELEASE_VERSION" \
RELEASE_TAG="$RELEASE_TAG" \
RELEASE_NAME="$RELEASE_NAME" \
RELEASE_URL="$RELEASE_URL" \
RELEASE_NOTES_PATH="$RELEASE_NOTES_PATH" \
READINESS_PATH="$READINESS_PATH" \
CLOSEOUT_PATH="$CLOSEOUT_PATH" \
MANIFEST_PATH="$MANIFEST_PATH" \
HOSTED_GATE_RUN_ID="$HOSTED_GATE_RUN_ID" \
PUBLISH_RUN_ID="$PUBLISH_RUN_ID" \
MILESTONE_NUMBER="$MILESTONE_NUMBER" \
EXPECTED_TAG_COMMIT="$EXPECTED_TAG_COMMIT" \
EXPECTED_RELEASE_BODY_SHA256="$EXPECTED_RELEASE_BODY_SHA256" \
EXPECTED_RELEASE_PUBLISHED_AT="$EXPECTED_RELEASE_PUBLISHED_AT" \
EXPECTED_RELEASE_UPDATED_AT="$EXPECTED_RELEASE_UPDATED_AT" \
EXPECTED_PUBLISH_HEAD_SHA="$EXPECTED_PUBLISH_HEAD_SHA" \
REMOTE_TAG_OBJECT="$remote_tag_object" \
REMOTE_TAG_COMMIT="$remote_tag_commit" \
ORIGIN_MAIN_SHA="$origin_main_sha" \
python3 <<'PY'
import copy
import hashlib
import json
import os
from datetime import datetime, timezone
from pathlib import Path

release = json.loads(os.environ["RELEASE_JSON"])
gate = json.loads(os.environ["GATE_JSON"])
publish = json.loads(os.environ["PUBLISH_JSON"])
milestone = json.loads(os.environ["MILESTONE_JSON"])
manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))
notes = Path(os.environ["RELEASE_NOTES_PATH"]).read_text(encoding="utf-8")
readiness = Path(os.environ["READINESS_PATH"]).read_text(encoding="utf-8")
closeout = Path(os.environ["CLOSEOUT_PATH"]).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def normalized(text: str) -> str:
    return "\n".join(line.rstrip() for line in text.splitlines()).strip()


def sha256(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def parse_time(value: str) -> datetime:
    require(bool(value), "timestamp is empty")
    parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)


expected = {
    "version": os.environ["RELEASE_VERSION"],
    "tag": os.environ["RELEASE_TAG"],
    "name": os.environ["RELEASE_NAME"],
    "url": os.environ["RELEASE_URL"],
    "tag_commit": os.environ["EXPECTED_TAG_COMMIT"],
    "body_sha256": os.environ["EXPECTED_RELEASE_BODY_SHA256"],
    "published_at": os.environ["EXPECTED_RELEASE_PUBLISHED_AT"],
    "updated_at": os.environ["EXPECTED_RELEASE_UPDATED_AT"],
    "publish_head_sha": os.environ["EXPECTED_PUBLISH_HEAD_SHA"],
    "hosted_gate_run_id": os.environ["HOSTED_GATE_RUN_ID"],
    "publish_run_id": os.environ["PUBLISH_RUN_ID"],
    "milestone_number": int(os.environ["MILESTONE_NUMBER"]),
}


def validate(
    release_payload: dict,
    gate_payload: dict,
    publish_payload: dict,
    milestone_payload: dict,
    manifest_payload: dict,
    notes_text: str,
    readiness_text: str,
    closeout_text: str,
) -> None:
    normalized_notes = normalized(notes_text)
    normalized_body = normalized(release_payload.get("body") or "")
    notes_sha = sha256(normalized_notes)
    body_sha = sha256(normalized_body)

    require(release_payload.get("tag_name") == expected["tag"], "release tag mismatch")
    require(release_payload.get("name") == expected["name"], "release name mismatch")
    require(release_payload.get("html_url") == expected["url"], "release URL mismatch")
    require(release_payload.get("draft") is False, "release must not be draft")
    require(release_payload.get("prerelease") is False, "release must not be prerelease")
    require(release_payload.get("target_commitish") == "main", "release target_commitish must be main")
    require(release_payload.get("published_at") == expected["published_at"], "release published_at mismatch")
    require(release_payload.get("updated_at") == expected["updated_at"], "release updated_at mismatch")
    require(normalized_body == normalized_notes, "release body must match tracked release notes")
    require(body_sha == expected["body_sha256"], "release body sha256 mismatch")
    require(notes_sha == expected["body_sha256"], "tracked release notes sha256 mismatch")

    require(gate_payload.get("workflowName") == "Rust Cutover Release Gate", "hosted gate workflow mismatch")
    require(gate_payload.get("status") == "completed", "hosted gate status mismatch")
    require(gate_payload.get("conclusion") == "success", "hosted gate conclusion mismatch")
    require(gate_payload.get("headSha") == expected["tag_commit"], "hosted gate head SHA mismatch")
    require(gate_payload.get("url", "").endswith(f"/{expected['hosted_gate_run_id']}"), "hosted gate URL/run mismatch")

    require(publish_payload.get("workflowName") == "Rust Cutover Publish Release", "publish workflow name mismatch")
    require(publish_payload.get("status") == "completed", "publish workflow status mismatch")
    require(publish_payload.get("conclusion") == "success", "publish workflow conclusion mismatch")
    require(publish_payload.get("headSha") == expected["publish_head_sha"], "publish workflow head SHA mismatch")
    require(publish_payload.get("url", "").endswith(f"/{expected['publish_run_id']}"), "publish workflow URL/run mismatch")

    gate_updated = parse_time(gate_payload.get("updatedAt", ""))
    publish_updated = parse_time(publish_payload.get("updatedAt", ""))
    release_updated = parse_time(release_payload.get("updated_at", ""))
    require(release_updated >= gate_updated, "release updated_at must be after hosted gate completion")
    require(publish_updated >= release_updated, "publish workflow completion must be after release update")

    require(milestone_payload.get("number") == expected["milestone_number"], "milestone number mismatch")
    require(milestone_payload.get("title") == "v0.26.0", "milestone title mismatch")
    require(milestone_payload.get("state") == "closed", "milestone must be closed")
    require(milestone_payload.get("open_issues") == 0, "milestone open issue count must be 0")
    require(milestone_payload.get("closed_issues") == 14, "milestone closed issue count must be 14")

    proof = manifest_payload.get("post_publication_strict_proof") or {}
    require(proof.get("source_closeout_path") == os.environ["CLOSEOUT_PATH"], "manifest source closeout path mismatch")
    require(proof.get("release_tag") == expected["tag"], "manifest proof release tag mismatch")
    require(proof.get("tag_commit") == expected["tag_commit"], "manifest proof tag commit mismatch")
    require(proof.get("remote_tag_object") == os.environ["REMOTE_TAG_OBJECT"], "manifest proof remote tag object mismatch")
    require(proof.get("release_body_sha256") == expected["body_sha256"], "manifest proof release body hash mismatch")
    require(proof.get("tracked_release_notes_sha256") == expected["body_sha256"], "manifest proof notes hash mismatch")
    require(proof.get("release_body_matches_tracked_release_notes") is True, "manifest proof strict body match missing")
    require(proof.get("hosted_release_gate_run_id") == int(expected["hosted_gate_run_id"]), "manifest proof hosted gate run mismatch")
    require(proof.get("publish_workflow_run_id") == int(expected["publish_run_id"]), "manifest proof publish run mismatch")
    require(proof.get("publish_workflow_head_sha") == expected["publish_head_sha"], "manifest proof publish workflow head mismatch")
    require(proof.get("generated_publication_evidence_authoritative") is False, "generated publication evidence must be non-authoritative")

    manifest_requirements = manifest_payload.get("post_publication_requirements") or {}
    require(manifest_requirements.get("github_release_published_required") is True, "manifest GitHub release requirement missing")
    require(manifest_requirements.get("hosted_release_gate_success_required") is True, "manifest hosted gate requirement missing")
    require(manifest_requirements.get("strict_release_body_match_required") is True, "manifest strict body requirement missing")
    require(manifest_requirements.get("publication_after_hosted_gate_required") is True, "manifest after-gate requirement missing")

    required_closeout_lines = (
        f"release tag = {expected['tag']}",
        f"release URL = {expected['url']}",
        f"published at = {expected['published_at']}",
        f"GitHub Release updated at = {expected['updated_at']}",
        f"annotated tag peeled commit = {expected['tag_commit']}",
        f"hosted release gate = https://github.com/atxinbao/NTPRO/actions/runs/{expected['hosted_gate_run_id']}",
        "hosted release gate workflow = Rust Cutover Release Gate",
        "hosted release gate conclusion = success",
        f"hosted release gate head SHA = {expected['tag_commit']}",
        f"publish workflow = https://github.com/atxinbao/NTPRO/actions/runs/{expected['publish_run_id']}",
        "publish workflow name = Rust Cutover Publish Release",
        "publish workflow conclusion = success",
        f"publish workflow head SHA = {expected['publish_head_sha']}",
        f"release body sha256 = {expected['body_sha256']}",
        f"tracked release notes sha256 = {expected['body_sha256']}",
        "release body matches tracked release notes = true",
        "v0.26.0 milestone state = closed",
        "v0.26.0 open_issues = 0",
        "v0.26.0 closed_issues = 14",
        "release-publication-evidence/ = generated artifact, not sole proof",
    )
    for line in required_closeout_lines:
        require(line in closeout_text, f"source closeout missing line: {line}")

    for text, label in ((readiness_text, "readiness"), (closeout_text, "closeout")):
        require(f"release body sha256 = {expected['body_sha256']}" in text, f"{label} release body hash mismatch")
        require("release body matches tracked release notes = true" in text, f"{label} strict body marker missing")
        require(f"hosted release gate = https://github.com/atxinbao/NTPRO/actions/runs/{expected['hosted_gate_run_id']}" in text, f"{label} hosted gate marker missing")
        require(f"publish workflow = https://github.com/atxinbao/NTPRO/actions/runs/{expected['publish_run_id']}" in text, f"{label} publish run marker missing")


validate(release, gate, publish, milestone, manifest, notes, readiness, closeout)

if os.environ.get("NTPRO_V261_POST_PUBLICATION_SELFTEST", "1") == "1":
    body_drift = copy.deepcopy(release)
    body_drift["body"] = "# stale release body\n"
    try:
        validate(body_drift, gate, publish, milestone, manifest, notes, readiness, closeout)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: release body drift")

    tag_drift = copy.deepcopy(gate)
    tag_drift["headSha"] = "0" * 40
    try:
        validate(release, tag_drift, publish, milestone, manifest, notes, readiness, closeout)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: hosted gate tag drift")

    missing_publish = copy.deepcopy(publish)
    missing_publish["conclusion"] = ""
    try:
        validate(release, gate, missing_publish, milestone, manifest, notes, readiness, closeout)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: missing publish run")

    missing_gate = copy.deepcopy(gate)
    missing_gate["conclusion"] = ""
    try:
        validate(release, missing_gate, publish, milestone, manifest, notes, readiness, closeout)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: missing hosted gate")

    missing_closeout = closeout.replace(f"release body sha256 = {expected['body_sha256']}", "release body sha256 = missing", 1)
    try:
        validate(release, gate, publish, milestone, manifest, notes, readiness, missing_closeout)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: missing source closeout body hash")

    manifest_missing_proof = copy.deepcopy(manifest)
    manifest_missing_proof.pop("post_publication_strict_proof", None)
    try:
        validate(release, gate, publish, milestone, manifest_missing_proof, notes, readiness, closeout)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: missing manifest strict proof")
PY

echo "v26_1_post_publication_strict_gate status=ok release_body_sha256=$EXPECTED_RELEASE_BODY_SHA256 tag_commit=$EXPECTED_TAG_COMMIT gate_run=$HOSTED_GATE_RUN_ID publish_run=$PUBLISH_RUN_ID milestone=$MILESTONE_NUMBER:closed negative_selftest=${NTPRO_V261_POST_PUBLICATION_SELFTEST:-1}"
