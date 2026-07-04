#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V231_PUBLICATION_REPO:-atxinbao/NTPRO}"
RELEASE_TAG="${NTPRO_V231_PUBLICATION_TAG:-ntpro-rust-only-v0.23.0}"
RELEASE_NAME="${NTPRO_V231_PUBLICATION_NAME:-NTPRO Rust-only v0.23.0}"
RELEASE_URL="${NTPRO_V231_PUBLICATION_URL:-https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.23.0}"
PUBLISHED_AT="${NTPRO_V231_PUBLICATION_PUBLISHED_AT:-2026-07-03T18:34:39Z}"
TAG_SHA="${NTPRO_V231_PUBLICATION_TAG_SHA:-783b024621116d50feaf418f12cb95fb95f87575}"
GATE_RUN_ID="${NTPRO_V231_PUBLICATION_GATE_RUN_ID:-28673868094}"
GATE_URL="${NTPRO_V231_PUBLICATION_GATE_URL:-https://github.com/atxinbao/NTPRO/actions/runs/28673868094}"
GATE_WORKFLOW="${NTPRO_V231_PUBLICATION_GATE_WORKFLOW:-Rust Cutover Release Gate}"
GATE_COMPLETED_AT="${NTPRO_V231_PUBLICATION_GATE_COMPLETED_AT:-2026-07-03T18:29:30Z}"
GATE_JOBS_TOTAL="${NTPRO_V231_PUBLICATION_GATE_JOBS_TOTAL:-66}"
GATE_JOBS_SUCCESS="${NTPRO_V231_PUBLICATION_GATE_JOBS_SUCCESS:-66}"

AUDIT_PATH="${NTPRO_V231_PUBLICATION_AUDIT_PATH:-docs/rust-cutover/release/v0_23_0_publication_evidence_audit_path.md}"
MANIFEST_PATH="${NTPRO_V231_PUBLICATION_MANIFEST:-docs/rust-cutover/release/v0_23_0_release_manifest.json}"
CLOSEOUT_PATH="${NTPRO_V231_PUBLICATION_CLOSEOUT:-docs/rust-cutover/release/v0_23_0_release_closeout_evidence.md}"
RELEASE_NOTES_PATH="${NTPRO_V231_PUBLICATION_RELEASE_NOTES:-docs/rust-cutover/release/v0_23_0_release_notes.md}"
READINESS_PATH="${NTPRO_V231_PUBLICATION_READINESS:-docs/rust-cutover/release/v0_23_0_readiness_report.md}"
PUBLISH_SCRIPT="${NTPRO_V231_PUBLICATION_PUBLISH_SCRIPT:-scripts/ai/publish_ntpro_release_after_gate.sh}"
CHECK_SCRIPT="${NTPRO_V231_PUBLICATION_CHECK_SCRIPT:-scripts/ai/check_github_release_published.sh}"

fail() {
  echo "v23.1 publication evidence audit path failed: $*" >&2
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
  "$AUDIT_PATH" \
  "$MANIFEST_PATH" \
  "$CLOSEOUT_PATH" \
  "$RELEASE_NOTES_PATH" \
  "$READINESS_PATH" \
  "$PUBLISH_SCRIPT" \
  "$CHECK_SCRIPT" \
  docs/rust-cutover/tasks/V231-005.md \
  docs/rust-cutover/evidence/V231-005.md; do
  require_file "$path"
done

for marker in \
  "publication evidence strategy = source_tree_plus_github_remote" \
  "tracked audit path = $AUDIT_PATH" \
  "local generated evidence path = release-publication-evidence/$RELEASE_TAG.json" \
  "local generated evidence required in source tree = false" \
  "remote reconstruction required = true" \
  "secret / token / credential / raw sensitive material = forbidden" \
  "release tag = $RELEASE_TAG" \
  "release URL = $RELEASE_URL" \
  "tag commit = $TAG_SHA" \
  "hosted release gate = $GATE_URL" \
  "hosted release gate workflow = $GATE_WORKFLOW" \
  "hosted release gate conclusion = success" \
  "hosted release gate jobs = ${GATE_JOBS_SUCCESS}/${GATE_JOBS_TOTAL} success"; do
  require_contains "$AUDIT_PATH" "$marker"
done

for marker in \
  "publication evidence audit path = $AUDIT_PATH" \
  "publication evidence strategy = source_tree_plus_github_remote" \
  "local generated publication evidence required in source tree = false" \
  "release-publication-evidence/$RELEASE_TAG.json = generated artifact, not sole proof"; do
  require_contains "$CLOSEOUT_PATH" "$marker"
done

for marker in \
  "v23.1 publication evidence audit path = required" \
  "publication evidence strategy = source_tree_plus_github_remote" \
  "publication evidence audit path = $AUDIT_PATH"; do
  require_contains "$READINESS_PATH" "$marker"
done

for marker in \
  "scripts/ai/verify_release.sh v23.1-publication-evidence-audit-path" \
  "$AUDIT_PATH"; do
  require_contains "$RELEASE_NOTES_PATH" "$marker"
done

for marker in \
  "publication_evidence_strategy=source_tree_plus_github_remote" \
  "local_evidence_path_is_generated_artifact=true" \
  "local_evidence_path_required_in_source_tree=false" \
  "remote_reconstruction_required=true"; do
  require_contains "$PUBLISH_SCRIPT" "$marker"
  require_contains "$CHECK_SCRIPT" "$marker"
done

if ! command -v gh >/dev/null 2>&1; then
  fail "gh is required for publication evidence remote proof"
fi
gh auth status >/dev/null 2>&1 || fail "gh authentication is required for publication evidence remote proof"

release_json="$(gh_with_retry release view "$RELEASE_TAG" --repo "$REPO" --json tagName,name,isDraft,isPrerelease,url,publishedAt,targetCommitish)"
run_json="$(gh_with_retry run view "$GATE_RUN_ID" --repo "$REPO" --json status,conclusion,workflowName,headSha,url,createdAt,updatedAt,jobs)"
tag_json="$(gh_with_retry api "repos/$REPO/git/ref/tags/$RELEASE_TAG")"

MANIFEST_PATH="$MANIFEST_PATH" \
AUDIT_PATH="$AUDIT_PATH" \
RELEASE_JSON="$release_json" \
RUN_JSON="$run_json" \
TAG_JSON="$tag_json" \
RELEASE_TAG="$RELEASE_TAG" \
RELEASE_NAME="$RELEASE_NAME" \
RELEASE_URL="$RELEASE_URL" \
PUBLISHED_AT="$PUBLISHED_AT" \
TAG_SHA="$TAG_SHA" \
GATE_URL="$GATE_URL" \
GATE_WORKFLOW="$GATE_WORKFLOW" \
GATE_COMPLETED_AT="$GATE_COMPLETED_AT" \
GATE_JOBS_TOTAL="$GATE_JOBS_TOTAL" \
GATE_JOBS_SUCCESS="$GATE_JOBS_SUCCESS" \
python3 <<'PY'
import copy
import json
import os
from datetime import datetime, timezone
from pathlib import Path

manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))
release = json.loads(os.environ["RELEASE_JSON"])
run = json.loads(os.environ["RUN_JSON"])
tag = json.loads(os.environ["TAG_JSON"])


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def parse_ts(value: str) -> datetime:
    parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)


def validate_manifest(candidate: dict) -> None:
    closeout = candidate.get("post_release_closeout") or {}
    evidence = closeout.get("publication_evidence") or {}
    require(evidence.get("status") == "published_after_gate", "publication evidence status mismatch")
    require(evidence.get("release_publication_after_gate") == "pass", "publication after gate mismatch")
    require(evidence.get("audit_source") == "source_tree_plus_github_remote", "audit source mismatch")
    require(evidence.get("tracked_audit_path") == os.environ["AUDIT_PATH"], "tracked audit path mismatch")
    require(
        evidence.get("local_generated_evidence_path")
        == f"release-publication-evidence/{os.environ['RELEASE_TAG']}.json",
        "local generated evidence path mismatch",
    )
    require(evidence.get("local_generated_evidence_required_in_source_tree") is False, "local generated evidence must not be required in source tree")
    require(evidence.get("remote_reconstruction_required") is True, "remote reconstruction must be required")
    require(evidence.get("secret_material_allowed") is False, "secret material must not be allowed")

    inputs = candidate.get("release_inputs") or {}
    require(inputs.get("publication_evidence_audit_path") == os.environ["AUDIT_PATH"], "release input audit path mismatch")
    require(Path(inputs.get("publication_evidence_audit_path", "")).is_file(), "release input audit file missing")

    commands = {
        gate.get("command")
        for gate in candidate.get("release_gates", [])
        if gate.get("required") is True
    }
    require(
        "scripts/ai/verify_release.sh v23.1-publication-evidence-audit-path" in commands,
        "publication evidence audit path gate missing",
    )


def validate_remote(state: dict) -> None:
    release_state = state["release"]
    require(release_state.get("tagName") == os.environ["RELEASE_TAG"], "release tag mismatch")
    require(release_state.get("name") == os.environ["RELEASE_NAME"], "release name mismatch")
    require(release_state.get("url") == os.environ["RELEASE_URL"], "release URL mismatch")
    require(release_state.get("publishedAt") == os.environ["PUBLISHED_AT"], "release publishedAt mismatch")
    require(release_state.get("isDraft") is False, "release must not be draft")
    require(release_state.get("isPrerelease") is False, "release must not be prerelease")
    require(release_state.get("targetCommitish") == "main", "release targetCommitish mismatch")

    run_state = state["run"]
    require(run_state.get("status") == "completed", "gate run must be completed")
    require(run_state.get("conclusion") == "success", "gate run must be successful")
    require(run_state.get("workflowName") == os.environ["GATE_WORKFLOW"], "gate workflow mismatch")
    require(run_state.get("headSha") == os.environ["TAG_SHA"], "gate head SHA mismatch")
    require(run_state.get("url") == os.environ["GATE_URL"], "gate URL mismatch")
    require(run_state.get("updatedAt") == os.environ["GATE_COMPLETED_AT"], "gate completion timestamp mismatch")
    jobs = run_state.get("jobs") or []
    require(len(jobs) == int(os.environ["GATE_JOBS_TOTAL"]), "gate job count mismatch")
    require(sum(1 for job in jobs if job.get("conclusion") == "success") == int(os.environ["GATE_JOBS_SUCCESS"]), "gate success job count mismatch")

    tag_state = state["tag"]
    require(tag_state.get("ref") == f"refs/tags/{os.environ['RELEASE_TAG']}", "tag ref mismatch")
    tag_object = tag_state.get("object") or {}
    require(tag_object.get("type") == "commit", "tag object type mismatch")
    require(tag_object.get("sha") == os.environ["TAG_SHA"], "tag object sha mismatch")

    require(parse_ts(release_state["publishedAt"]) >= parse_ts(run_state["updatedAt"]), "release must be published after hosted gate completion")


validate_manifest(manifest)
state = {"release": release, "run": run, "tag": tag}
validate_remote(state)

if os.environ.get("NTPRO_V231_PUBLICATION_SELFTEST", "1") == "1":
    manifest_mutations = {
        "local_evidence_required": lambda m: m["post_release_closeout"]["publication_evidence"].update({"local_generated_evidence_required_in_source_tree": True}),
        "missing_tracked_audit": lambda m: m["post_release_closeout"]["publication_evidence"].pop("tracked_audit_path", None),
        "secret_material_allowed": lambda m: m["post_release_closeout"]["publication_evidence"].update({"secret_material_allowed": True}),
    }
    for name, mutate in manifest_mutations.items():
        candidate = copy.deepcopy(manifest)
        mutate(candidate)
        try:
            validate_manifest(candidate)
        except AssertionError:
            continue
        raise AssertionError(f"manifest negative self-test unexpectedly passed: {name}")

    remote_mutations = {
        "draft_release": lambda s: s["release"].update({"isDraft": True}),
        "failed_gate": lambda s: s["run"].update({"conclusion": "failure"}),
        "tag_sha_mismatch": lambda s: s["tag"]["object"].update({"sha": "0" * 40}),
    }
    for name, mutate in remote_mutations.items():
        candidate = copy.deepcopy(state)
        mutate(candidate)
        try:
            validate_remote(candidate)
        except AssertionError:
            continue
        raise AssertionError(f"remote negative self-test unexpectedly passed: {name}")
PY

echo "v23_1_publication_evidence_audit_path status=ok release_tag=$RELEASE_TAG gate_run=$GATE_RUN_ID audit_path=$AUDIT_PATH negative_selftest=${NTPRO_V231_PUBLICATION_SELFTEST:-1}"
