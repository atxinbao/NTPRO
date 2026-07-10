#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

MODE="${1:-source}"
REPO="${NTPRO_V291_POST_PUBLICATION_REPO:-atxinbao/NTPRO}"
RELEASE_VERSION="${NTPRO_V291_POST_PUBLICATION_VERSION:-v0.29.0}"
RELEASE_TAG="${NTPRO_V291_POST_PUBLICATION_TAG:-ntpro-rust-only-v0.29.0}"
RELEASE_NAME="${NTPRO_V291_POST_PUBLICATION_NAME:-NTPRO Rust-only v0.29.0}"
RELEASE_URL="${NTPRO_V291_POST_PUBLICATION_URL:-https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.29.0}"
TAG_SHA="${NTPRO_V291_POST_PUBLICATION_TAG_SHA:-85110d29867763f8d3b6395f4ff8154378b475b9}"
TAG_OBJECT="${NTPRO_V291_POST_PUBLICATION_TAG_OBJECT:-25cccef7a99c6f231dac7f915f24abe882ad7f2c}"
PUBLISHED_AT="${NTPRO_V291_POST_PUBLICATION_PUBLISHED_AT:-2026-07-10T13:44:23Z}"
GATE_RUN_ID="${NTPRO_V291_POST_PUBLICATION_GATE_RUN_ID:-29091765148}"
GATE_URL="${NTPRO_V291_POST_PUBLICATION_GATE_URL:-https://github.com/atxinbao/NTPRO/actions/runs/29091765148}"
GATE_COMPLETED_AT="${NTPRO_V291_POST_PUBLICATION_GATE_COMPLETED_AT:-2026-07-10T13:43:15Z}"
GATE_JOBS_TOTAL="${NTPRO_V291_POST_PUBLICATION_GATE_JOBS_TOTAL:-88}"
GATE_JOBS_SUCCESS="${NTPRO_V291_POST_PUBLICATION_GATE_JOBS_SUCCESS:-88}"
MILESTONE_NUMBER="${NTPRO_V291_POST_PUBLICATION_MILESTONE_NUMBER:-24}"
MILESTONE_TITLE="${NTPRO_V291_POST_PUBLICATION_MILESTONE_TITLE:-v0.29.0}"

MANIFEST_PATH="${NTPRO_V291_POST_PUBLICATION_MANIFEST:-docs/rust-cutover/release/v0_29_0_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V291_POST_PUBLICATION_NOTES:-docs/rust-cutover/release/v0_29_0_release_notes.md}"
READINESS_PATH="${NTPRO_V291_POST_PUBLICATION_READINESS:-docs/rust-cutover/release/v0_29_0_readiness_report.md}"
CLOSEOUT_PATH="${NTPRO_V291_POST_PUBLICATION_CLOSEOUT:-docs/rust-cutover/release/v0_29_0_release_closeout_evidence.md}"
TASK_PATH="${NTPRO_V291_POST_PUBLICATION_TASK:-docs/rust-cutover/tasks/V291-004.md}"
EVIDENCE_PATH="${NTPRO_V291_POST_PUBLICATION_EVIDENCE:-docs/rust-cutover/evidence/V291-004.md}"

fail() {
  echo "v29.1 post-publication closeout gate failed: $*" >&2
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
  PUBLISHED_AT="$PUBLISHED_AT" \
  GATE_RUN_ID="$GATE_RUN_ID" \
  GATE_URL="$GATE_URL" \
  GATE_COMPLETED_AT="$GATE_COMPLETED_AT" \
  GATE_JOBS_TOTAL="$GATE_JOBS_TOTAL" \
  GATE_JOBS_SUCCESS="$GATE_JOBS_SUCCESS" \
  MILESTONE_NUMBER="$MILESTONE_NUMBER" \
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


def require_contains(text: str, marker: str, label: str) -> None:
    require(marker in text, f"{label} missing marker: {marker}")


def normalize(text: str) -> str:
    return "\n".join(line.rstrip() for line in text.splitlines()).strip()


def parse_ts(value: str) -> datetime:
    parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)


manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))
notes = Path(os.environ["RELEASE_NOTES_PATH"]).read_text(encoding="utf-8")
readiness = Path(os.environ["READINESS_PATH"]).read_text(encoding="utf-8")
closeout_text = Path(os.environ["CLOSEOUT_PATH"]).read_text(encoding="utf-8")
task_text = Path(os.environ["TASK_PATH"]).read_text(encoding="utf-8")
evidence_text = Path(os.environ["EVIDENCE_PATH"]).read_text(encoding="utf-8")


def validate(candidate: dict, notes_text: str, readiness_text: str, closeout: str) -> None:
    require(candidate.get("schema_version") == "ntpro.v290_release_manifest.v1", "manifest schema mismatch")
    require(candidate.get("product_version") == os.environ["RELEASE_VERSION"], "manifest version mismatch")
    require(candidate.get("release_status") == "released", "manifest must be released after publication")

    for label, text in {"release notes": notes_text, "readiness report": readiness_text}.items():
        require_contains(text, "Status: RELEASED", label)
        require("Status: RELEASE GATE READY" not in text, f"{label} still uses release-gate-ready status")
        require_contains(text, "post-publication closeout gate = required", label)
        require_contains(text, "publication evidence strategy = source_tree_plus_github_remote", label)
        require_contains(text, "generated publication evidence sole proof allowed = false", label)
        require_contains(text, "published release closeout evidence = docs/rust-cutover/release/v0_29_0_release_closeout_evidence.md", label)
        require_contains(text, "published release status = published_after_gate", label)
        require_contains(text, f"hosted release gate run = {os.environ['GATE_RUN_ID']}", label)
        require_contains(text, "release body hash semantics = normalized_sha256", label)

    post = candidate.get("post_release_closeout") or {}
    require(post.get("closeout_evidence_path") == os.environ["CLOSEOUT_PATH"], "post-release closeout path mismatch")
    release = post.get("github_release") or {}
    require(release.get("tag") == os.environ["RELEASE_TAG"], "closeout release tag mismatch")
    require(release.get("name") == os.environ["RELEASE_NAME"], "closeout release name mismatch")
    require(release.get("url") == os.environ["RELEASE_URL"], "closeout release URL mismatch")
    require(release.get("draft") is False, "release must not be draft")
    require(release.get("prerelease") is False, "release must not be prerelease")
    require(release.get("published_at") == os.environ["PUBLISHED_AT"], "published_at mismatch")

    tag = post.get("tag") or {}
    require(tag.get("object") == os.environ["TAG_OBJECT"], "tag object mismatch")
    require(tag.get("peeled_commit") == os.environ["TAG_SHA"], "tag commit mismatch")

    gate = post.get("hosted_release_gate") or {}
    require(gate.get("run_id") == int(os.environ["GATE_RUN_ID"]), "gate run mismatch")
    require(gate.get("url") == os.environ["GATE_URL"], "gate URL mismatch")
    require(gate.get("workflow") == "Rust Cutover Release Gate", "gate workflow mismatch")
    require(gate.get("status") == "completed", "gate status mismatch")
    require(gate.get("conclusion") == "success", "gate conclusion mismatch")
    require(gate.get("head_sha") == os.environ["TAG_SHA"], "gate head SHA mismatch")
    require(gate.get("completed_at") == os.environ["GATE_COMPLETED_AT"], "gate completed_at mismatch")
    require(gate.get("jobs_total") == int(os.environ["GATE_JOBS_TOTAL"]), "gate jobs_total mismatch")
    require(gate.get("jobs_success") == int(os.environ["GATE_JOBS_SUCCESS"]), "gate jobs_success mismatch")
    require(parse_ts(release["published_at"]) >= parse_ts(gate["completed_at"]), "release must publish after gate")

    body = post.get("release_body") or {}
    normalized = normalize(notes_text)
    raw = notes_text
    normalized_sha = hashlib.sha256(normalized.encode("utf-8")).hexdigest()
    raw_sha = hashlib.sha256(raw.encode("utf-8")).hexdigest()
    require(body.get("hash_semantics") == "normalized_sha256", "hash semantics mismatch")
    require(body.get("normalized_sha256") == normalized_sha, "normalized release body hash mismatch")
    require(body.get("tracked_notes_normalized_sha256") == normalized_sha, "tracked normalized hash mismatch")
    require(body.get("raw_sha256") == raw_sha, "raw release body hash mismatch")
    require(body.get("tracked_notes_raw_sha256") == raw_sha, "tracked raw hash mismatch")
    require(body.get("normalized_line_count") == len(normalized.splitlines()), "normalized line count mismatch")
    require(body.get("tracked_notes_normalized_line_count") == len(normalized.splitlines()), "tracked line count mismatch")

    milestone = post.get("milestone_closeout") or {}
    require(milestone.get("number") == int(os.environ["MILESTONE_NUMBER"]), "milestone number mismatch")
    require(milestone.get("title") == "v0.29.0", "milestone title mismatch")
    require(milestone.get("state") == "closed", "milestone state mismatch")
    require(milestone.get("open_issues") == 0, "milestone open issue count mismatch")
    require(milestone.get("closed_issues") == 12, "milestone closed issue count mismatch")
    require(milestone.get("exact_issue_numbers") == list(range(926, 937)) + [961], "exact issue numbers mismatch")

    publication = post.get("publication_evidence") or {}
    require(publication.get("status") == "published_after_gate", "publication status mismatch")
    require(publication.get("release_publication_after_gate") == "pass", "publication-after-gate mismatch")
    require(publication.get("current_release_publish_after_gate_binding") == "pass", "current binding mismatch")
    require(publication.get("generated_publication_evidence_sole_proof_allowed") is False, "generated-only proof must be false")
    require(publication.get("historical_fixture_only_current_release_proof_allowed") is False, "historical fixture-only proof must be false")
    require(publication.get("local_generated_evidence_required_in_source_tree") is False, "local generated evidence must not be required")
    require(publication.get("remote_reconstruction_required") is True, "remote reconstruction must be required")
    require(publication.get("audit_source") == "source_tree_plus_github_remote", "audit source mismatch")

    requirements = candidate.get("post_publication_requirements") or {}
    for key in [
        "github_release_published_required",
        "hosted_release_gate_success_required",
        "strict_release_body_match_required",
        "publication_after_hosted_gate_required",
        "source_controlled_closeout_evidence_required",
        "remote_reconstruction_required",
    ]:
        require(requirements.get(key) is True, f"post-publication requirement missing: {key}")
    require(requirements.get("generated_publication_evidence_sole_proof_allowed") is False, "generated-only requirement mismatch")

    cleanup = candidate.get("post_release_stale_v290_evidence_cleanup") or {}
    require(cleanup.get("task_id") == "V291-003", "stale cleanup dependency missing")
    require(cleanup.get("issue") == 965, "stale cleanup issue mismatch")
    require(cleanup.get("final_strict_provenance", {}).get("tag_exists") is True, "final tag_exists proof missing")
    require(cleanup.get("final_strict_provenance", {}).get("source_dirty") is False, "final source_dirty proof missing")

    closeout_gate = candidate.get("post_publication_closeout_gate") or {}
    require(closeout_gate.get("task_id") == "V291-004", "post-publication gate task mismatch")
    require(closeout_gate.get("issue") == 966, "post-publication gate issue mismatch")
    require(closeout_gate.get("gate") == "scripts/ai/verify_v29_1_post_publication_closeout_gate.sh source", "gate command mismatch")
    require(closeout_gate.get("required_release_status") == "released", "required release status mismatch")
    require(closeout_gate.get("rejects_release_gate_ready_only") is True, "release_gate_ready rejection missing")
    require(closeout_gate.get("requires_source_tree_plus_github_remote") is True, "source_tree_plus_github_remote requirement missing")
    require(closeout_gate.get("generated_evidence_only_allowed") is False, "generated-only proof must be rejected")
    require(closeout_gate.get("validates_milestone_closeout") is True, "milestone validation missing")
    require(closeout_gate.get("validates_exact_issue_scope") is True, "issue scope validation missing")
    require(closeout_gate.get("validates_final_release_gate_run") == int(os.environ["GATE_RUN_ID"]), "gate run validation mismatch")
    require(closeout_gate.get("validates_release_body_hash_semantics") == "normalized_sha256", "hash semantic validation mismatch")
    require(closeout_gate.get("validates_tag_commit") == os.environ["TAG_SHA"], "tag commit validation mismatch")
    require(closeout_gate.get("validates_publication_after_gate_ordering") is True, "publication ordering validation missing")
    require(closeout_gate.get("preserves_raw_hash_as_diagnostic_only") is True, "raw hash diagnostic flag missing")
    require(closeout_gate.get("runtime_behavior_changed") is False, "runtime behavior must not change")
    require(closeout_gate.get("trading_behavior_changed") is False, "trading behavior must not change")

    for marker in [
        "post-publication closeout gate = required",
        "release_gate_ready-only artifacts after publication accepted = false",
        "source_tree_plus_github_remote reconstruction accepted = true",
        "generated-evidence-only proof accepted = false",
        "release body hash semantics = normalized_sha256",
    ]:
        require_contains(closeout, marker, "closeout evidence")


validate(manifest, notes, readiness, closeout_text)

for marker in [
    "GitHub issue: `#966`",
    "Add a post-publication closeout gate",
    "backend go-live;",
    "product-grade live trading claim.",
]:
    require_contains(task_text, marker, "V291-004 task")

for marker in [
    "Task: `V291-004` / GitHub issue `#966`",
    "Status: LOCAL AND LIVE VALIDATION PASS",
    "release_gate_ready-only artifacts rejected = true",
    "generated-evidence-only proof rejected = true",
    "source_tree_plus_github_remote reconstruction required = true",
]:
    require_contains(evidence_text, marker, "V291-004 evidence")

if os.environ.get("NTPRO_V291_POST_PUBLICATION_SELFTEST", "1") == "1":
    manifest_mutations = {
        "release_gate_ready_only": lambda m: (m.update({"release_status": "release_gate_ready"}), m.pop("post_release_closeout", None)),
        "generated_evidence_only": lambda m: m["post_release_closeout"]["publication_evidence"].update({"audit_source": "generated_artifact_only", "generated_publication_evidence_sole_proof_allowed": True}),
        "missing_closeout_gate": lambda m: m.pop("post_publication_closeout_gate", None),
        "failed_hosted_gate": lambda m: m["post_release_closeout"]["hosted_release_gate"].update({"conclusion": "failure"}),
        "published_before_gate": lambda m: m["post_release_closeout"]["github_release"].update({"published_at": "2026-07-10T13:00:00Z"}),
        "hash_semantics_drift": lambda m: m["post_release_closeout"]["release_body"].update({"hash_semantics": "raw_sha256"}),
    }
    for name, mutate in manifest_mutations.items():
        candidate = copy.deepcopy(manifest)
        mutate(candidate)
        try:
            validate(candidate, notes, readiness, closeout_text)
        except AssertionError:
            continue
        raise AssertionError(f"manifest negative self-test unexpectedly passed: {name}")

    for label, bad_notes, bad_readiness in [
        ("notes_release_gate_ready", notes.replace("Status: RELEASED", "Status: RELEASE GATE READY", 1), readiness),
        ("readiness_release_gate_ready", notes, readiness.replace("Status: RELEASED", "Status: RELEASE GATE READY", 1)),
    ]:
        try:
            validate(manifest, bad_notes, bad_readiness, closeout_text)
        except AssertionError:
            continue
        raise AssertionError(f"text negative self-test unexpectedly passed: {label}")
PY
}

run_live_validation() {
  command -v gh >/dev/null 2>&1 || fail "gh is required for live post-publication closeout proof"
  gh_with_retry auth status >/dev/null 2>&1 || fail "gh authentication is required for live post-publication closeout proof"

  tmp_dir="$(mktemp -d)"
  trap 'rm -rf "$tmp_dir"' RETURN
  gh_with_retry release view "$RELEASE_TAG" --repo "$REPO" --json tagName,name,isDraft,isPrerelease,url,publishedAt,targetCommitish,body >"$tmp_dir/release.json"
  gh_with_retry run view "$GATE_RUN_ID" --repo "$REPO" --json status,conclusion,headSha,url,updatedAt,workflowName,jobs >"$tmp_dir/run.json"
  gh_with_retry issue list --repo "$REPO" --milestone "$MILESTONE_TITLE" --state all --limit 100 --json number,state >"$tmp_dir/issues.json"
  gh_with_retry api "/repos/$REPO/milestones/$MILESTONE_NUMBER" >"$tmp_dir/milestone.json"
  git ls-remote --tags origin "refs/tags/$RELEASE_TAG" "refs/tags/$RELEASE_TAG^{}" >"$tmp_dir/tags.txt"

  RELEASE_JSON_PATH="$tmp_dir/release.json" \
  RUN_JSON_PATH="$tmp_dir/run.json" \
  ISSUES_JSON_PATH="$tmp_dir/issues.json" \
  MILESTONE_JSON_PATH="$tmp_dir/milestone.json" \
  TAGS_PATH="$tmp_dir/tags.txt" \
  RELEASE_NOTES_PATH="$RELEASE_NOTES_PATH" \
  RELEASE_TAG="$RELEASE_TAG" \
  RELEASE_NAME="$RELEASE_NAME" \
  RELEASE_URL="$RELEASE_URL" \
  TAG_SHA="$TAG_SHA" \
  TAG_OBJECT="$TAG_OBJECT" \
  PUBLISHED_AT="$PUBLISHED_AT" \
  GATE_RUN_ID="$GATE_RUN_ID" \
  GATE_URL="$GATE_URL" \
  GATE_COMPLETED_AT="$GATE_COMPLETED_AT" \
  GATE_JOBS_TOTAL="$GATE_JOBS_TOTAL" \
  GATE_JOBS_SUCCESS="$GATE_JOBS_SUCCESS" \
  python3 <<'PY'
import hashlib
import json
import os
from datetime import datetime, timezone
from pathlib import Path


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def normalize(text: str) -> str:
    return "\n".join(line.rstrip() for line in text.splitlines()).strip()


def parse_ts(value: str):
    parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)


release = json.loads(Path(os.environ["RELEASE_JSON_PATH"]).read_text(encoding="utf-8"))
run = json.loads(Path(os.environ["RUN_JSON_PATH"]).read_text(encoding="utf-8"))
issues = json.loads(Path(os.environ["ISSUES_JSON_PATH"]).read_text(encoding="utf-8"))
milestone = json.loads(Path(os.environ["MILESTONE_JSON_PATH"]).read_text(encoding="utf-8"))
tags = Path(os.environ["TAGS_PATH"]).read_text(encoding="utf-8")
notes = Path(os.environ["RELEASE_NOTES_PATH"]).read_text(encoding="utf-8")

require(release.get("tagName") == os.environ["RELEASE_TAG"], "release tag mismatch")
require(release.get("name") == os.environ["RELEASE_NAME"], "release name mismatch")
require(release.get("url") == os.environ["RELEASE_URL"], "release URL mismatch")
require(release.get("isDraft") is False, "release must not be draft")
require(release.get("isPrerelease") is False, "release must not be prerelease")
require(release.get("targetCommitish") == "main", "target commitish mismatch")
require(release.get("publishedAt") == os.environ["PUBLISHED_AT"], "publishedAt mismatch")
require(normalize(release.get("body") or "") == normalize(notes), "live release body must match tracked release notes")
require(hashlib.sha256(normalize(release.get("body") or "").encode()).hexdigest() == hashlib.sha256(normalize(notes).encode()).hexdigest(), "normalized body hash mismatch")

require(run.get("status") == "completed", "gate status mismatch")
require(run.get("conclusion") == "success", "gate conclusion mismatch")
require(run.get("workflowName") == "Rust Cutover Release Gate", "gate workflow mismatch")
require(run.get("headSha") == os.environ["TAG_SHA"], "gate head sha mismatch")
require(run.get("url") == os.environ["GATE_URL"], "gate URL mismatch")
require(run.get("updatedAt") == os.environ["GATE_COMPLETED_AT"], "gate completed time mismatch")
jobs = run.get("jobs") or []
require(len(jobs) == int(os.environ["GATE_JOBS_TOTAL"]), "gate job count mismatch")
require(sum(1 for job in jobs if job.get("conclusion") == "success") == int(os.environ["GATE_JOBS_SUCCESS"]), "gate success count mismatch")
require(parse_ts(release["publishedAt"]) >= parse_ts(run["updatedAt"]), "release must publish after gate completion")

expected = set(range(926, 937)) | {961}
issue_states = {int(item["number"]): item["state"] for item in issues}
require(set(issue_states) == expected, f"issue set mismatch: {sorted(issue_states)}")
require(all(state == "CLOSED" for state in issue_states.values()), f"issues not closed: {issue_states}")
require(milestone.get("title") == "v0.29.0", "milestone title mismatch")
require(milestone.get("state") == "closed", "milestone state mismatch")
require(milestone.get("open_issues") == 0, "milestone open issues mismatch")
require(milestone.get("closed_issues") == 12, "milestone closed issues mismatch")
require(os.environ["TAG_OBJECT"] in tags, "annotated tag object missing")
require(os.environ["TAG_SHA"] in tags, "peeled tag commit missing")
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
  all)
    run_source_validation
    run_live_validation
    ;;
  *)
    fail "unknown mode: $MODE"
    ;;
esac

echo "v29_1_post_publication_closeout_gate status=ok mode=$MODE release_tag=$RELEASE_TAG release_gate_run=$GATE_RUN_ID source_tree_plus_github_remote=true generated_evidence_only_rejected=true release_gate_ready_only_rejected=true"
