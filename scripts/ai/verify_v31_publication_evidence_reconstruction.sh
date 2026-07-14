#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

MODE="${1:-source}"
REPO="${NTPRO_V311_PUBLICATION_RECONSTRUCTION_REPO:-atxinbao/NTPRO}"
RELEASE_VERSION="${NTPRO_V311_PUBLICATION_RECONSTRUCTION_VERSION:-v0.31.0}"
RELEASE_TAG="${NTPRO_V311_PUBLICATION_RECONSTRUCTION_TAG:-ntpro-rust-only-v0.31.0}"
RELEASE_NAME="${NTPRO_V311_PUBLICATION_RECONSTRUCTION_NAME:-NTPRO Rust-only v0.31.0}"
RELEASE_URL="${NTPRO_V311_PUBLICATION_RECONSTRUCTION_URL:-https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.31.0}"
TAG_OBJECT="${NTPRO_V311_PUBLICATION_RECONSTRUCTION_TAG_OBJECT:-8c0d71f6e6ef2a890daf1e07299c658fa187a262}"
TAG_SHA="${NTPRO_V311_PUBLICATION_RECONSTRUCTION_TAG_SHA:-14e582868cb9c18b8b26a1fd50ab21cb56f8f1a1}"
TAG_TREE="${NTPRO_V311_PUBLICATION_RECONSTRUCTION_TAG_TREE:-7ace21b252c8a14b66eae9642baa2fd4ad3b895a}"
PUBLISHED_AT="${NTPRO_V311_PUBLICATION_RECONSTRUCTION_PUBLISHED_AT:-2026-07-13T22:42:06Z}"
GATE_RUN_ID="${NTPRO_V311_PUBLICATION_RECONSTRUCTION_GATE_RUN_ID:-29285960500}"
GATE_URL="${NTPRO_V311_PUBLICATION_RECONSTRUCTION_GATE_URL:-https://github.com/atxinbao/NTPRO/actions/runs/29285960500}"
GATE_COMPLETED_AT="${NTPRO_V311_PUBLICATION_RECONSTRUCTION_GATE_COMPLETED_AT:-2026-07-13T22:41:01Z}"
GATE_JOBS_TOTAL="${NTPRO_V311_PUBLICATION_RECONSTRUCTION_GATE_JOBS_TOTAL:-96}"
PUBLISH_RUN_ID="${NTPRO_V311_PUBLICATION_RECONSTRUCTION_PUBLISH_RUN_ID:-29290691138}"
PUBLISH_URL="${NTPRO_V311_PUBLICATION_RECONSTRUCTION_PUBLISH_URL:-https://github.com/atxinbao/NTPRO/actions/runs/29290691138}"
PUBLISH_COMPLETED_AT="${NTPRO_V311_PUBLICATION_RECONSTRUCTION_PUBLISH_COMPLETED_AT:-2026-07-13T22:42:11Z}"
ARTIFACT_NAME="${NTPRO_V311_PUBLICATION_RECONSTRUCTION_ARTIFACT_NAME:-release-publication-evidence-ntpro-rust-only-v0.31.0}"
ARTIFACT_FILE="${NTPRO_V311_PUBLICATION_RECONSTRUCTION_ARTIFACT_FILE:-ntpro-rust-only-v0.31.0.json}"
MILESTONE_NUMBER="${NTPRO_V311_PUBLICATION_RECONSTRUCTION_MILESTONE_NUMBER:-28}"
MILESTONE_TITLE="${NTPRO_V311_PUBLICATION_RECONSTRUCTION_MILESTONE_TITLE:-v0.31.0}"

MANIFEST_PATH="${NTPRO_V311_PUBLICATION_RECONSTRUCTION_MANIFEST:-docs/rust-cutover/release/v0_31_0_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V311_PUBLICATION_RECONSTRUCTION_NOTES:-docs/rust-cutover/release/v0_31_0_release_notes.md}"
CLOSEOUT_PATH="${NTPRO_V311_PUBLICATION_RECONSTRUCTION_CLOSEOUT:-docs/rust-cutover/release/v0_31_0_release_closeout_evidence.md}"
README_PATH="${NTPRO_V311_PUBLICATION_RECONSTRUCTION_README:-docs/rust-cutover/release/README.md}"
TASK_PATH="${NTPRO_V311_PUBLICATION_RECONSTRUCTION_TASK:-docs/rust-cutover/tasks/V311-005.md}"
EVIDENCE_PATH="${NTPRO_V311_PUBLICATION_RECONSTRUCTION_EVIDENCE:-docs/rust-cutover/evidence/V311-005.md}"
V31_GATE_PATH="${NTPRO_V311_PUBLICATION_RECONSTRUCTION_V31_GATE:-scripts/ai/verify_v31_release_gates.sh}"
VERIFIER_PATH="scripts/ai/verify_v31_publication_evidence_reconstruction.sh"

fail() {
  echo "v31 publication evidence reconstruction failed: $*" >&2
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
  "$CLOSEOUT_PATH" \
  "$README_PATH" \
  "$TASK_PATH" \
  "$EVIDENCE_PATH" \
  "$V31_GATE_PATH"; do
  require_file "$path"
done

run_source_validation() {
  MANIFEST_PATH="$MANIFEST_PATH" \
  RELEASE_NOTES_PATH="$RELEASE_NOTES_PATH" \
  CLOSEOUT_PATH="$CLOSEOUT_PATH" \
  README_PATH="$README_PATH" \
  TASK_PATH="$TASK_PATH" \
  EVIDENCE_PATH="$EVIDENCE_PATH" \
  V31_GATE_PATH="$V31_GATE_PATH" \
  VERIFIER_PATH="$VERIFIER_PATH" \
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
  PUBLISH_RUN_ID="$PUBLISH_RUN_ID" \
  PUBLISH_URL="$PUBLISH_URL" \
  PUBLISH_COMPLETED_AT="$PUBLISH_COMPLETED_AT" \
  ARTIFACT_NAME="$ARTIFACT_NAME" \
  ARTIFACT_FILE="$ARTIFACT_FILE" \
  MILESTONE_NUMBER="$MILESTONE_NUMBER" \
  MILESTONE_TITLE="$MILESTONE_TITLE" \
  python3 <<'PY'
import copy
import hashlib
import json
import os
from datetime import datetime, timezone
from pathlib import Path


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def parse_ts(value: str) -> datetime:
    parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)


def normalize(value: str) -> str:
    return "\n".join(line.rstrip() for line in value.splitlines()).strip()


def require_contains(text: str, marker: str, label: str) -> None:
    require(marker in text, f"{label} missing marker: {marker}")


manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))
notes = Path(os.environ["RELEASE_NOTES_PATH"]).read_text(encoding="utf-8")
closeout = Path(os.environ["CLOSEOUT_PATH"]).read_text(encoding="utf-8")
readme = Path(os.environ["README_PATH"]).read_text(encoding="utf-8")
task = Path(os.environ["TASK_PATH"]).read_text(encoding="utf-8")
evidence = Path(os.environ["EVIDENCE_PATH"]).read_text(encoding="utf-8")
v31_gate = Path(os.environ["V31_GATE_PATH"]).read_text(encoding="utf-8")

expected_issues = [1006, 1007, 1008, 1009, 1010, 1011, 1012, 1013, 1014, 1015, 1033]


def validate(candidate: dict) -> None:
    require(candidate.get("schema_version") == "ntpro.v310_release_manifest.v1", "manifest schema mismatch")
    require(candidate.get("product_version") == os.environ["RELEASE_VERSION"], "manifest release version mismatch")
    require(candidate.get("release_status") == "released", "manifest must be released")

    published = candidate.get("published_release") or {}
    closeout_block = candidate.get("post_publication_closeout") or {}
    requirements = candidate.get("post_publication_requirements") or {}
    reconstruction = candidate.get("publication_evidence_reconstruction") or {}
    release_inputs = candidate.get("release_inputs") or {}
    scope = candidate.get("release_scope") or {}

    require(published.get("tag") == os.environ["RELEASE_TAG"], "published tag mismatch")
    require(published.get("name") == os.environ["RELEASE_NAME"], "published name mismatch")
    require(published.get("github_release_url") == os.environ["RELEASE_URL"], "published URL mismatch")
    require(published.get("draft") is False, "published release must not be draft")
    require(published.get("prerelease") is False, "published release must not be prerelease")
    require(published.get("target_commitish") == "main", "published target mismatch")
    require(parse_ts(published["published_at"]) >= parse_ts(published["release_gate_completed_at"]), "release was published before gate success")
    require(parse_ts(published["publish_workflow_completed_at"]) >= parse_ts(published["published_at"]), "publish workflow completed before release publication")
    require(published.get("published_at") == os.environ["PUBLISHED_AT"], "published_at mismatch")
    require(published.get("tag_object_sha") == os.environ["TAG_OBJECT"], "tag object mismatch")
    require(published.get("tag_sha") == os.environ["TAG_SHA"], "tag commit mismatch")
    require(published.get("tag_tree_sha") == os.environ["TAG_TREE"], "tag tree mismatch")
    require(published.get("release_gate_run_id") == int(os.environ["GATE_RUN_ID"]), "gate run mismatch")
    require(published.get("release_gate_url") == os.environ["GATE_URL"], "gate URL mismatch")
    require(published.get("release_gate_completed_at") == os.environ["GATE_COMPLETED_AT"], "gate completed mismatch")
    require(published.get("release_gate_conclusion") == "success", "gate conclusion mismatch")
    require(published.get("release_gate_jobs_success") == int(os.environ["GATE_JOBS_TOTAL"]), "gate success count mismatch")
    require(published.get("release_gate_jobs_total") == int(os.environ["GATE_JOBS_TOTAL"]), "gate total count mismatch")
    require(published.get("publish_workflow_run_id") == int(os.environ["PUBLISH_RUN_ID"]), "publish run mismatch")
    require(published.get("publish_workflow_url") == os.environ["PUBLISH_URL"], "publish URL mismatch")
    require(published.get("publish_workflow_completed_at") == os.environ["PUBLISH_COMPLETED_AT"], "publish completed mismatch")
    require(published.get("publish_workflow_conclusion") == "success", "publish conclusion mismatch")
    require(published.get("publish_workflow_jobs_success") == 1, "publish success count mismatch")
    require(published.get("publish_workflow_jobs_total") == 1, "publish total count mismatch")
    require(published.get("published_after_hosted_gate") is True, "published-after-gate marker missing")

    require(closeout_block.get("release_gate_head_sha") == published.get("tag_sha"), "gate head SHA must match tag")
    require(closeout_block.get("publish_workflow_head_sha") == published.get("tag_sha"), "publish head SHA must match tag")
    require(closeout_block.get("release_gate_run_id") == published.get("release_gate_run_id"), "closeout gate run mismatch")
    require(closeout_block.get("publish_workflow_run_id") == published.get("publish_workflow_run_id"), "closeout publish run mismatch")
    require(closeout_block.get("published_after_hosted_gate") is True, "closeout published-after-gate marker missing")
    require(closeout_block.get("generated_publication_evidence_sole_proof_allowed") is False, "generated-only closeout proof must be false")

    for key in [
        "hosted_release_gate_success_required",
        "publish_workflow_success_required",
        "publication_after_hosted_gate_required",
        "same_tag_commit_required",
        "release_body_hash_record_required",
        "source_controlled_closeout_evidence_required",
    ]:
        require(requirements.get(key) is True, f"post-publication requirement missing: {key}")
    require(requirements.get("generated_publication_evidence_sole_proof_allowed") is False, "generated-only requirement must be false")
    require(requirements.get("v0_32_start_gate_fails_without_v311_release_evidence") is True, "v32 V311 blocker missing")

    require(scope.get("exact_milestone_issue_numbers") == expected_issues, "exact issue scope mismatch")
    require(closeout_block.get("exact_issue_numbers") == expected_issues, "closeout exact issue scope mismatch")
    require(closeout_block.get("milestone_number") == int(os.environ["MILESTONE_NUMBER"]), "milestone number mismatch")
    require(closeout_block.get("milestone_title") == os.environ["MILESTONE_TITLE"], "milestone title mismatch")
    require(closeout_block.get("milestone_state") == "closed", "milestone must be closed")
    require(closeout_block.get("milestone_open_issues") == 0, "milestone open issues mismatch")
    require(closeout_block.get("milestone_closed_issues") == 11, "milestone closed issues mismatch")

    notes_normalized_sha = hashlib.sha256(normalize(notes).encode("utf-8")).hexdigest()
    notes_raw_sha = hashlib.sha256(notes.encode("utf-8")).hexdigest()
    require(published.get("current_release_body_normalized_sha256") == notes_normalized_sha, "current normalized notes hash mismatch")
    require(published.get("current_release_body_raw_sha256") == notes_raw_sha, "current raw notes hash mismatch")
    require(published.get("current_release_body_matches_tracked_release_notes") is True, "current normalized body match missing")
    require(published.get("current_release_body_raw_matches_tracked_release_notes") is True, "current raw body match missing")

    require(release_inputs.get("v311_publication_reconstruction_gate_path") == os.environ["VERIFIER_PATH"], "reconstruction gate path missing")
    require(release_inputs.get("v311_publication_reconstruction_evidence_path") == os.environ["EVIDENCE_PATH"], "reconstruction evidence path missing")
    require(reconstruction.get("task_id") == "V311-005", "reconstruction task mismatch")
    require(reconstruction.get("issue") == 1040, "reconstruction issue mismatch")
    require(reconstruction.get("status") == "source_tree_plus_github_remote_reconstructable", "reconstruction status mismatch")
    require(reconstruction.get("verifier_path") == os.environ["VERIFIER_PATH"], "reconstruction verifier path mismatch")
    require(reconstruction.get("closeout_audit_path") == os.environ["CLOSEOUT_PATH"], "reconstruction closeout path mismatch")
    require(reconstruction.get("source_controlled_closeout_evidence_required") is True, "source closeout requirement missing")
    require(reconstruction.get("github_remote_state_required") is True, "remote state requirement missing")
    require(reconstruction.get("release_publish_workflow_artifact_name") == os.environ["ARTIFACT_NAME"], "artifact name mismatch")
    require(reconstruction.get("release_publish_workflow_artifact_file") == os.environ["ARTIFACT_FILE"], "artifact file mismatch")
    require(reconstruction.get("release_publish_workflow_artifact_required_for_live_reconstruction") is True, "artifact live requirement missing")
    require(reconstruction.get("release_publish_workflow_artifact_sole_source_of_truth") is False, "artifact sole source must be false")
    require(reconstruction.get("local_generated_artifact_required_in_source_tree") is False, "local generated artifact must not be required")
    require(reconstruction.get("local_generated_artifact_sole_source_of_truth") is False, "local generated artifact must not be sole source")
    require(reconstruction.get("release_publish_after_gate_ordering_required") is True, "publish-after-gate requirement missing")
    require(reconstruction.get("same_tag_commit_required") is True, "same tag commit requirement missing")
    require(reconstruction.get("release_gate_head_sha_matches_tag") is True, "gate SHA match marker missing")
    require(reconstruction.get("publish_workflow_head_sha_matches_tag") is True, "publish SHA match marker missing")
    require(reconstruction.get("remote_tag_object_matches_manifest") is True, "remote tag object marker missing")
    require(reconstruction.get("remote_tag_peeled_commit_matches_manifest") is True, "remote peeled commit marker missing")
    require(reconstruction.get("closeout_audit_path_in_v31_release_gates") is True, "v31 release gate inclusion missing")
    require(reconstruction.get("negative_selftest_cases") == [
        "publication_before_gate",
        "release_gate_head_sha_mismatch",
        "publish_workflow_head_sha_mismatch",
        "generated_artifact_as_sole_proof",
    ], "negative self-test cases mismatch")


validate(manifest)

for label, candidate, expected in [
    ("publication-before-gate", copy.deepcopy(manifest), "release was published before gate success"),
    ("gate-sha-mismatch", copy.deepcopy(manifest), "gate head SHA must match tag"),
    ("publish-sha-mismatch", copy.deepcopy(manifest), "publish head SHA must match tag"),
    ("generated-sole-proof", copy.deepcopy(manifest), "local generated artifact must not be sole source"),
]:
    if label == "publication-before-gate":
        candidate["published_release"]["published_at"] = "2026-07-13T22:40:00Z"
    elif label == "gate-sha-mismatch":
        candidate["post_publication_closeout"]["release_gate_head_sha"] = "0" * 40
    elif label == "publish-sha-mismatch":
        candidate["post_publication_closeout"]["publish_workflow_head_sha"] = "0" * 40
    elif label == "generated-sole-proof":
        candidate["publication_evidence_reconstruction"]["local_generated_artifact_sole_source_of_truth"] = True
    try:
        validate(candidate)
    except AssertionError as exc:
        require(expected in str(exc), f"wrong negative self-test failure for {label}: {exc}")
    else:
        raise AssertionError(f"negative self-test unexpectedly passed: {label}")

for marker in [
    "publication evidence reconstruction verifier = scripts/ai/verify_v31_publication_evidence_reconstruction.sh",
    "workflow artifact evidence agrees with source closeout = true",
    "release-publish-after-gate ordering is reconstructable = true",
    "tag commit mismatch fails closed = true",
    "local generated publication artifact required in source tree = false",
    "generated publication artifact sole source of truth = false",
]:
    require_contains(closeout, marker, "closeout evidence")

for marker in [
    "V311-005",
    "publication evidence reconstruction",
    "scripts/ai/verify_v31_publication_evidence_reconstruction.sh",
    "../evidence/V311-005.md",
]:
    require_contains(readme, marker, "release README")

for label, text in {"task": task, "evidence": evidence}.items():
    require_contains(text, "V311-005", label)
    require_contains(text, "GitHub issue", label)
    require_contains(text, "source_tree_plus_github_remote", label)
    require_contains(text, "release-publication-evidence-ntpro-rust-only-v0.31.0", label)
    require_contains(text, "generated publication evidence sole proof allowed = false", label)
    require_contains(text, "v0.32.0 backend production closeout", label)

require_contains(v31_gate, "scripts/ai/verify_v31_publication_evidence_reconstruction.sh source", "v31 release gate")
PY
}

run_live_validation() {
  command -v gh >/dev/null 2>&1 || fail "gh_unavailable"
  gh auth status >/dev/null 2>&1 || fail "gh_auth_unavailable"

  local tmp_dir
  tmp_dir="$(mktemp -d)"
  trap 'rm -rf "$tmp_dir"' RETURN

  release_json="$(gh_with_retry release view "$RELEASE_TAG" --repo "$REPO" --json tagName,name,isDraft,isPrerelease,url,publishedAt,createdAt,targetCommitish,author,body,databaseId,id)"
  gate_json="$(gh_with_retry run view "$GATE_RUN_ID" --repo "$REPO" --json status,conclusion,headSha,url,createdAt,updatedAt,workflowName,jobs --jq '{workflowName,status,conclusion,headSha,url,createdAt,updatedAt,jobs_success:([.jobs[] | select(.conclusion=="success")] | length),jobs_total:(.jobs | length)}')"
  publish_json="$(gh_with_retry run view "$PUBLISH_RUN_ID" --repo "$REPO" --json status,conclusion,headSha,url,createdAt,updatedAt,workflowName,jobs --jq '{workflowName,status,conclusion,headSha,url,createdAt,updatedAt,jobs_success:([.jobs[] | select(.conclusion=="success")] | length),jobs_total:(.jobs | length)}')"
  artifacts_json="$(gh_with_retry api "repos/$REPO/actions/runs/$PUBLISH_RUN_ID/artifacts")"
  milestone_json="$(gh_with_retry api "repos/$REPO/milestones/$MILESTONE_NUMBER" --jq '{number,title,state,open_issues,closed_issues,closed_at}')"
  issues_json="$(gh_with_retry issue list --repo "$REPO" --milestone "$MILESTONE_TITLE" --state all --limit 100 --json number,state)"
  remote_tags="$(git ls-remote --tags origin "refs/tags/$RELEASE_TAG" "refs/tags/$RELEASE_TAG^{}")"

  gh_with_retry run download "$PUBLISH_RUN_ID" \
    --repo "$REPO" \
    --name "$ARTIFACT_NAME" \
    --dir "$tmp_dir" >/dev/null
  local artifact_path="$tmp_dir/$ARTIFACT_FILE"
  [[ -f "$artifact_path" ]] || fail "missing downloaded artifact file: $ARTIFACT_FILE"

  MANIFEST_PATH="$MANIFEST_PATH" \
  RELEASE_NOTES_PATH="$RELEASE_NOTES_PATH" \
  RELEASE_JSON="$release_json" \
  GATE_JSON="$gate_json" \
  PUBLISH_JSON="$publish_json" \
  ARTIFACTS_JSON="$artifacts_json" \
  ARTIFACT_JSON_PATH="$artifact_path" \
  MILESTONE_JSON="$milestone_json" \
  ISSUES_JSON="$issues_json" \
  REMOTE_TAGS="$remote_tags" \
  REPO="$REPO" \
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
  PUBLISH_RUN_ID="$PUBLISH_RUN_ID" \
  PUBLISH_URL="$PUBLISH_URL" \
  PUBLISH_COMPLETED_AT="$PUBLISH_COMPLETED_AT" \
  ARTIFACT_NAME="$ARTIFACT_NAME" \
  MILESTONE_NUMBER="$MILESTONE_NUMBER" \
  MILESTONE_TITLE="$MILESTONE_TITLE" \
  python3 <<'PY'
import hashlib
import json
import os
from datetime import datetime, timezone
from pathlib import Path


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def parse_ts(value: str) -> datetime:
    parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)


def normalize(value: str) -> str:
    return "\n".join(line.rstrip() for line in value.splitlines()).strip()


manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))
notes = Path(os.environ["RELEASE_NOTES_PATH"]).read_text(encoding="utf-8")
release = json.loads(os.environ["RELEASE_JSON"])
gate = json.loads(os.environ["GATE_JSON"])
publish = json.loads(os.environ["PUBLISH_JSON"])
artifacts = json.loads(os.environ["ARTIFACTS_JSON"])
artifact = json.loads(Path(os.environ["ARTIFACT_JSON_PATH"]).read_text(encoding="utf-8"))
milestone = json.loads(os.environ["MILESTONE_JSON"])
issues = json.loads(os.environ["ISSUES_JSON"])
remote_tags = os.environ["REMOTE_TAGS"]
published = manifest["published_release"]
closeout = manifest["post_publication_closeout"]
expected_issues = [1006, 1007, 1008, 1009, 1010, 1011, 1012, 1013, 1014, 1015, 1033]

require(release["tagName"] == os.environ["RELEASE_TAG"], "live release tag mismatch")
require(release["name"] == os.environ["RELEASE_NAME"], "live release name mismatch")
require(release["url"] == os.environ["RELEASE_URL"], "live release URL mismatch")
require(release["isDraft"] is False, "live release must not be draft")
require(release["isPrerelease"] is False, "live release must not be prerelease")
require(release["publishedAt"] == os.environ["PUBLISHED_AT"], "live publishedAt mismatch")
require(release["targetCommitish"] == "main", "live target commitish mismatch")
require(release["databaseId"] == published["github_release_id"], "live release database id mismatch")
require(release["id"] == published["github_release_node_id"], "live release node id mismatch")

body = release.get("body") or ""
body_normalized_sha = hashlib.sha256(normalize(body).encode("utf-8")).hexdigest()
body_raw_sha = hashlib.sha256(body.encode("utf-8")).hexdigest()
notes_normalized_sha = hashlib.sha256(normalize(notes).encode("utf-8")).hexdigest()
notes_raw_sha = hashlib.sha256(notes.encode("utf-8")).hexdigest()
require(body_normalized_sha == notes_normalized_sha, "live normalized release body does not match tracked notes")
require(body_raw_sha == notes_raw_sha, "live raw release body does not match tracked notes")
require(body_normalized_sha == published["current_release_body_normalized_sha256"], "live normalized body hash mismatch")
require(body_raw_sha == published["current_release_body_raw_sha256"], "live raw body hash mismatch")

require(gate["workflowName"] == "Rust Cutover Release Gate", "live gate workflow mismatch")
require(gate["status"] == "completed", "live gate status mismatch")
require(gate["conclusion"] == "success", "live gate conclusion mismatch")
require(gate["headSha"] == os.environ["TAG_SHA"], "live gate head SHA mismatch")
require(gate["url"] == os.environ["GATE_URL"], "live gate URL mismatch")
require(gate["updatedAt"] == os.environ["GATE_COMPLETED_AT"], "live gate completed_at mismatch")
require(gate["jobs_success"] == int(os.environ["GATE_JOBS_TOTAL"]), "live gate jobs_success mismatch")
require(gate["jobs_total"] == int(os.environ["GATE_JOBS_TOTAL"]), "live gate jobs_total mismatch")

require(publish["workflowName"] == "Rust Cutover Publish Release", "live publish workflow mismatch")
require(publish["status"] == "completed", "live publish status mismatch")
require(publish["conclusion"] == "success", "live publish conclusion mismatch")
require(publish["headSha"] == os.environ["TAG_SHA"], "live publish head SHA mismatch")
require(publish["url"] == os.environ["PUBLISH_URL"], "live publish URL mismatch")
require(publish["updatedAt"] == os.environ["PUBLISH_COMPLETED_AT"], "live publish completed_at mismatch")
require(publish["jobs_success"] == 1 and publish["jobs_total"] == 1, "live publish job count mismatch")

require(parse_ts(release["publishedAt"]) >= parse_ts(gate["updatedAt"]), "live release published before hosted gate")
require(parse_ts(publish["createdAt"]) >= parse_ts(gate["updatedAt"]), "live publish workflow started before hosted gate completed")
require(parse_ts(publish["updatedAt"]) >= parse_ts(release["publishedAt"]), "live publish workflow completed before release publication")

artifact_matches = [
    item for item in artifacts.get("artifacts", [])
    if item.get("name") == os.environ["ARTIFACT_NAME"]
]
require(artifacts.get("total_count", 0) >= 1, "publish run artifact count mismatch")
require(len(artifact_matches) == 1, "publication evidence artifact missing or duplicated")
artifact_meta = artifact_matches[0]
require(artifact_meta.get("expired") is False, "publication evidence artifact expired")
require(artifact_meta.get("size_in_bytes") == 472, "publication evidence artifact size mismatch")

require(artifact.get("status") == "published_after_gate", "artifact publication status mismatch")
require(artifact.get("repository") == os.environ["REPO"], "artifact repository mismatch")
require(artifact.get("tag_name") == os.environ["RELEASE_TAG"], "artifact tag mismatch")
require(artifact.get("release_version") == os.environ["RELEASE_VERSION"], "artifact version mismatch")
require(artifact.get("release_name") == os.environ["RELEASE_NAME"], "artifact release name mismatch")
require(artifact.get("release_notes") == os.environ["RELEASE_NOTES_PATH"], "artifact release notes path mismatch")
require(artifact.get("release_gate_run_id") == os.environ["GATE_RUN_ID"], "artifact gate run mismatch")
require(artifact.get("release_gate_url") == os.environ["GATE_URL"], "artifact gate URL mismatch")
require(artifact.get("release_gate_completed_at") == os.environ["GATE_COMPLETED_AT"], "artifact gate completed mismatch")
require(artifact.get("release_url") == os.environ["RELEASE_URL"], "artifact release URL mismatch")
require(artifact.get("published_at") == os.environ["PUBLISHED_AT"], "artifact published_at mismatch")
require(artifact.get("tag_sha") == os.environ["TAG_SHA"], "artifact tag SHA mismatch")
require(parse_ts(artifact["published_at"]) >= parse_ts(artifact["release_gate_completed_at"]), "artifact publication precedes gate")

remote_tag_map = {}
for line in remote_tags.splitlines():
    parts = line.split()
    if len(parts) == 2:
        remote_tag_map[parts[1]] = parts[0]
require(remote_tag_map.get(f"refs/tags/{os.environ['RELEASE_TAG']}") == os.environ["TAG_OBJECT"], "remote tag object mismatch")
require(remote_tag_map.get(f"refs/tags/{os.environ['RELEASE_TAG']}^{{}}") == os.environ["TAG_SHA"], "remote peeled tag mismatch")

require(closeout["release_gate_head_sha"] == os.environ["TAG_SHA"], "source closeout gate SHA mismatch")
require(closeout["publish_workflow_head_sha"] == os.environ["TAG_SHA"], "source closeout publish SHA mismatch")
require(closeout["release_gate_run_id"] == int(os.environ["GATE_RUN_ID"]), "source closeout gate run mismatch")
require(closeout["publish_workflow_run_id"] == int(os.environ["PUBLISH_RUN_ID"]), "source closeout publish run mismatch")
require(closeout["release_gate_completed_at"] == os.environ["GATE_COMPLETED_AT"], "source closeout gate time mismatch")
require(closeout["publish_workflow_completed_at"] == os.environ["PUBLISH_COMPLETED_AT"], "source closeout publish time mismatch")

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

echo "v31_publication_evidence_reconstruction status=ok mode=$MODE release_tag=$RELEASE_TAG release_gate_run=$GATE_RUN_ID publish_run=$PUBLISH_RUN_ID artifact=$ARTIFACT_NAME negative_selftest=4"
