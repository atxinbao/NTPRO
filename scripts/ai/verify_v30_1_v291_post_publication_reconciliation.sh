#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

MODE="${1:-source}"
REPO="${NTPRO_V301_V291_RECONCILIATION_REPO:-atxinbao/NTPRO}"
RELEASE_TAG="${NTPRO_V301_V291_RECONCILIATION_TAG:-ntpro-rust-only-v0.29.1}"
RELEASE_NAME="${NTPRO_V301_V291_RECONCILIATION_NAME:-NTPRO Rust-only v0.29.1}"
RELEASE_URL="${NTPRO_V301_V291_RECONCILIATION_URL:-https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.29.1}"
TAG_OBJECT="${NTPRO_V301_V291_RECONCILIATION_TAG_OBJECT:-d3d398530835342dab4aafe355d1c842be0fdd47}"
TAG_SHA="${NTPRO_V301_V291_RECONCILIATION_TAG_SHA:-a831d802e4321f50ed6e10481aea35b15a74b01e}"
PUBLISHED_AT="${NTPRO_V301_V291_RECONCILIATION_PUBLISHED_AT:-2026-07-11T01:07:24Z}"
GATE_RUN_ID="${NTPRO_V301_V291_RECONCILIATION_GATE_RUN_ID:-29130876713}"
GATE_COMPLETED_AT="${NTPRO_V301_V291_RECONCILIATION_GATE_COMPLETED_AT:-2026-07-11T01:06:27Z}"
GATE_JOBS_TOTAL="${NTPRO_V301_V291_RECONCILIATION_GATE_JOBS_TOTAL:-90}"
GATE_JOBS_SUCCESS="${NTPRO_V301_V291_RECONCILIATION_GATE_JOBS_SUCCESS:-90}"
BODY_NORMALIZED_SHA="${NTPRO_V301_V291_RECONCILIATION_BODY_NORMALIZED_SHA:-611c6cfe89480054d5c3a4718215740701ee43536e3e92fa0ff458f7730b204b}"
BODY_RAW_SHA="${NTPRO_V301_V291_RECONCILIATION_BODY_RAW_SHA:-5d5b7c34ceb7bca1a389e8261d04cc7fd28cea0a9d1e48ffe609f449b22ef2d1}"

MANIFEST_PATH="${NTPRO_V301_V291_RECONCILIATION_MANIFEST:-docs/rust-cutover/release/v0_29_1_release_manifest.json}"
NOTES_PATH="${NTPRO_V301_V291_RECONCILIATION_NOTES:-docs/rust-cutover/release/v0_29_1_release_notes.md}"
READINESS_PATH="${NTPRO_V301_V291_RECONCILIATION_READINESS:-docs/rust-cutover/release/v0_29_1_readiness_report.md}"
CLOSEOUT_PATH="${NTPRO_V301_V291_RECONCILIATION_CLOSEOUT:-docs/rust-cutover/release/v0_29_1_release_closeout_evidence.md}"
INTAKE_PATH="${NTPRO_V301_V291_RECONCILIATION_INTAKE:-docs/rust-cutover/release/v0_30_0_intake_gate.md}"
V300_EVIDENCE_PATH="${NTPRO_V301_V291_RECONCILIATION_V300_EVIDENCE:-docs/rust-cutover/evidence/V300-000.md}"
TASK_PATH="${NTPRO_V301_V291_RECONCILIATION_TASK:-docs/rust-cutover/tasks/V301-005.md}"
EVIDENCE_PATH="${NTPRO_V301_V291_RECONCILIATION_EVIDENCE:-docs/rust-cutover/evidence/V301-005.md}"

fail() {
  echo "v30.1 v291 post-publication reconciliation failed: $*" >&2
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
  "$NOTES_PATH" \
  "$READINESS_PATH" \
  "$CLOSEOUT_PATH" \
  "$INTAKE_PATH" \
  "$V300_EVIDENCE_PATH" \
  "$TASK_PATH" \
  "$EVIDENCE_PATH"; do
  require_file "$path"
done

run_source_validation() {
  MANIFEST_PATH="$MANIFEST_PATH" \
  NOTES_PATH="$NOTES_PATH" \
  READINESS_PATH="$READINESS_PATH" \
  CLOSEOUT_PATH="$CLOSEOUT_PATH" \
  INTAKE_PATH="$INTAKE_PATH" \
  V300_EVIDENCE_PATH="$V300_EVIDENCE_PATH" \
  TASK_PATH="$TASK_PATH" \
  EVIDENCE_PATH="$EVIDENCE_PATH" \
  RELEASE_TAG="$RELEASE_TAG" \
  RELEASE_NAME="$RELEASE_NAME" \
  RELEASE_URL="$RELEASE_URL" \
  TAG_OBJECT="$TAG_OBJECT" \
  TAG_SHA="$TAG_SHA" \
  PUBLISHED_AT="$PUBLISHED_AT" \
  GATE_RUN_ID="$GATE_RUN_ID" \
  GATE_COMPLETED_AT="$GATE_COMPLETED_AT" \
  BODY_NORMALIZED_SHA="$BODY_NORMALIZED_SHA" \
  BODY_RAW_SHA="$BODY_RAW_SHA" \
  python3 <<'PY'
import hashlib
import json
import os
from pathlib import Path


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def require_contains(text: str, marker: str, label: str) -> None:
    require(marker in text, f"{label} missing marker: {marker}")


def normalize(value: str) -> str:
    return "\n".join(line.rstrip() for line in value.splitlines()).strip()


manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))
notes = Path(os.environ["NOTES_PATH"]).read_text(encoding="utf-8")
readiness = Path(os.environ["READINESS_PATH"]).read_text(encoding="utf-8")
closeout = Path(os.environ["CLOSEOUT_PATH"]).read_text(encoding="utf-8")
intake = Path(os.environ["INTAKE_PATH"]).read_text(encoding="utf-8")
v300_evidence = Path(os.environ["V300_EVIDENCE_PATH"]).read_text(encoding="utf-8")
task = Path(os.environ["TASK_PATH"]).read_text(encoding="utf-8")
evidence = Path(os.environ["EVIDENCE_PATH"]).read_text(encoding="utf-8")

require(hashlib.sha256(normalize(notes).encode()).hexdigest() == os.environ["BODY_NORMALIZED_SHA"], "tracked notes normalized hash mismatch")
require(hashlib.sha256(notes.encode()).hexdigest() == os.environ["BODY_RAW_SHA"], "tracked notes raw hash mismatch")

for label, text in {"release notes": notes, "readiness": readiness}.items():
    require_contains(text, "Status: RELEASED", label)
    require("Status: RELEASE GATE READY" not in text, f"{label} still exposes release gate ready as final status")
    require_contains(text, "post-publication predecessor closeout contract = authoritative", label)
    require_contains(text, "published release status = published_after_gate", label)
    require_contains(text, f"hosted release gate run = {os.environ['GATE_RUN_ID']}", label)
    require_contains(text, "v0.30.0 intake predecessor contract = v0_29_1_authoritative_closeout_contract", label)

require(manifest.get("release_status") == "released", "manifest release_status must be released")
published = manifest.get("published_release") or {}
require(published.get("tag") == os.environ["RELEASE_TAG"], "published tag mismatch")
require(published.get("name") == os.environ["RELEASE_NAME"], "published name mismatch")
require(published.get("github_release_url") == os.environ["RELEASE_URL"], "published URL mismatch")
require(published.get("tag_object_sha") == os.environ["TAG_OBJECT"], "published tag object mismatch")
require(published.get("tag_sha") == os.environ["TAG_SHA"], "published tag sha mismatch")
require(published.get("published_at") == os.environ["PUBLISHED_AT"], "published_at mismatch")
post = manifest.get("post_publication_closeout") or {}
require(post.get("status") == "source_controlled_closeout_recorded", "post closeout status mismatch")
require(post.get("release_gate_run_id") == int(os.environ["GATE_RUN_ID"]), "post closeout gate run mismatch")
require(post.get("release_gate_head_sha") == os.environ["TAG_SHA"], "post closeout gate sha mismatch")
require(post.get("published_after_hosted_gate") is True, "published-after-gate missing")
require(post.get("generated_evidence_is_sole_proof") is False, "generated-only proof must be false")
contract = manifest.get("authoritative_predecessor_closeout_contract") or {}
require(contract.get("contract_id") == "v0_29_1_authoritative_closeout_contract", "contract id mismatch")
require(contract.get("release_status") == "released", "contract release status mismatch")
require(contract.get("source_of_truth") == "source_controlled_closeout_evidence_plus_manifest_published_release", "contract source mismatch")
require(contract.get("v30_intake_consumes_contract") is True, "v30 intake contract marker missing")
require(contract.get("contradictory_release_gate_ready_final_state_allowed") is False, "contradictory final state must be rejected")
require(contract.get("release_body_normalized_sha256") == os.environ["BODY_NORMALIZED_SHA"], "contract normalized hash mismatch")
require(contract.get("release_body_raw_sha256") == os.environ["BODY_RAW_SHA"], "contract raw hash mismatch")

for marker in [
    "Status: RELEASED",
    "authoritative predecessor closeout contract = v0_29_1_authoritative_closeout_contract",
    "manifest release_status = released",
    "manifest published_release populated = true",
    "manifest post_publication_closeout populated = true",
    f"release body normalized sha256 = {os.environ['BODY_NORMALIZED_SHA']}",
    f"release body raw sha256 = {os.environ['BODY_RAW_SHA']}",
]:
    require_contains(closeout, marker, "closeout evidence")

for label, text in {"v30 intake": intake, "V300-000 evidence": v300_evidence}.items():
    require_contains(text, "v0.29.1 authoritative predecessor closeout contract = v0_29_1_authoritative_closeout_contract", label)
    require_contains(text, f"v0.29.1 GitHub Release body normalized sha256 = {os.environ['BODY_NORMALIZED_SHA']}", label)

for marker in [
    "GitHub issue: `#1003`",
    "Resolve the v0.29.1 post-publication semantics mismatch",
    "backend go-live;",
    "product-grade live trading claim.",
]:
    require_contains(task, marker, "V301-005 task")

for marker in [
    "Task: `V301-005` / GitHub issue `#1003`",
    "authoritative predecessor closeout contract = v0_29_1_authoritative_closeout_contract",
    f"release body normalized sha256 = {os.environ['BODY_NORMALIZED_SHA']}",
]:
    require_contains(evidence, marker, "V301-005 evidence")
PY
}

run_live_validation() {
  command -v gh >/dev/null 2>&1 || fail "gh is required for live v291 reconciliation proof"
  gh_with_retry auth status >/dev/null 2>&1 || fail "gh authentication is required for live v291 reconciliation proof"
  tmp_dir="$(mktemp -d)"
  trap 'rm -rf "$tmp_dir"' RETURN
  gh_with_retry release view "$RELEASE_TAG" --repo "$REPO" --json tagName,name,isDraft,isPrerelease,url,publishedAt,targetCommitish,body >"$tmp_dir/release.json"
  gh_with_retry run view "$GATE_RUN_ID" --repo "$REPO" --json status,conclusion,headSha,updatedAt,workflowName,jobs >"$tmp_dir/run.json"
  git ls-remote --tags origin "refs/tags/$RELEASE_TAG" "refs/tags/$RELEASE_TAG^{}" >"$tmp_dir/tags.txt"
  RELEASE_JSON_PATH="$tmp_dir/release.json" \
  RUN_JSON_PATH="$tmp_dir/run.json" \
  TAGS_PATH="$tmp_dir/tags.txt" \
  NOTES_PATH="$NOTES_PATH" \
  RELEASE_TAG="$RELEASE_TAG" \
  RELEASE_NAME="$RELEASE_NAME" \
  RELEASE_URL="$RELEASE_URL" \
  TAG_OBJECT="$TAG_OBJECT" \
  TAG_SHA="$TAG_SHA" \
  PUBLISHED_AT="$PUBLISHED_AT" \
  GATE_COMPLETED_AT="$GATE_COMPLETED_AT" \
  GATE_JOBS_TOTAL="$GATE_JOBS_TOTAL" \
  GATE_JOBS_SUCCESS="$GATE_JOBS_SUCCESS" \
  BODY_NORMALIZED_SHA="$BODY_NORMALIZED_SHA" \
  BODY_RAW_SHA="$BODY_RAW_SHA" \
  python3 <<'PY'
import hashlib
import json
import os
from pathlib import Path


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def normalize(value: str) -> str:
    return "\n".join(line.rstrip() for line in value.splitlines()).strip()


release = json.loads(Path(os.environ["RELEASE_JSON_PATH"]).read_text(encoding="utf-8"))
run = json.loads(Path(os.environ["RUN_JSON_PATH"]).read_text(encoding="utf-8"))
tags = Path(os.environ["TAGS_PATH"]).read_text(encoding="utf-8")
notes = Path(os.environ["NOTES_PATH"]).read_text(encoding="utf-8")
body = release.get("body") or ""

require(release.get("tagName") == os.environ["RELEASE_TAG"], "release tag mismatch")
require(release.get("name") == os.environ["RELEASE_NAME"], "release name mismatch")
require(release.get("url") == os.environ["RELEASE_URL"], "release URL mismatch")
require(release.get("isDraft") is False, "release must not be draft")
require(release.get("isPrerelease") is False, "release must not be prerelease")
require(release.get("targetCommitish") == "main", "target commitish mismatch")
require(release.get("publishedAt") == os.environ["PUBLISHED_AT"], "publishedAt mismatch")
require(hashlib.sha256(normalize(body).encode()).hexdigest() == os.environ["BODY_NORMALIZED_SHA"], "live normalized body hash mismatch")
require(hashlib.sha256(body.encode()).hexdigest() == os.environ["BODY_RAW_SHA"], "live raw body hash mismatch")
require(normalize(body) == normalize(notes), "live body must match tracked notes")
require(run.get("status") == "completed", "gate status mismatch")
require(run.get("conclusion") == "success", "gate conclusion mismatch")
require(run.get("workflowName") == "Rust Cutover Release Gate", "gate workflow mismatch")
require(run.get("headSha") == os.environ["TAG_SHA"], "gate head sha mismatch")
require(run.get("updatedAt") == os.environ["GATE_COMPLETED_AT"], "gate completion mismatch")
jobs = run.get("jobs") or []
require(len(jobs) == int(os.environ["GATE_JOBS_TOTAL"]), "gate job count mismatch")
require(sum(1 for item in jobs if item.get("conclusion") == "success") == int(os.environ["GATE_JOBS_SUCCESS"]), "gate success count mismatch")
require(os.environ["TAG_OBJECT"] in tags, "tag object missing")
require(os.environ["TAG_SHA"] in tags, "tag commit missing")
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

echo "v30_1_v291_post_publication_reconciliation status=ok mode=$MODE release_tag=$RELEASE_TAG authoritative_contract=v0_29_1_authoritative_closeout_contract release_body_normalized_sha256=$BODY_NORMALIZED_SHA"
