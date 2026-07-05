#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V241_STALE_REPO:-atxinbao/NTPRO}"
RELEASE_TAG="${NTPRO_V241_STALE_RELEASE_TAG:-ntpro-rust-only-v0.24.0}"
ISSUE_NUMBER="${NTPRO_V241_STALE_ISSUE_NUMBER:-752}"
MILESTONE_NUMBER="${NTPRO_V241_STALE_MILESTONE_NUMBER:-14}"

V240_EVIDENCE_PATH="${NTPRO_V241_STALE_V240_EVIDENCE:-docs/rust-cutover/evidence/V240-009.md}"
READINESS_PATH="${NTPRO_V241_STALE_READINESS:-docs/rust-cutover/release/v0_24_0_readiness_report.md}"
RELEASE_NOTES_PATH="${NTPRO_V241_STALE_RELEASE_NOTES:-docs/rust-cutover/release/v0_24_0_release_notes.md}"
CLOSEOUT_EVIDENCE_PATH="${NTPRO_V241_STALE_CLOSEOUT:-docs/rust-cutover/release/v0_24_0_release_closeout_evidence.md}"
PROVENANCE_RECONCILIATION_PATH="${NTPRO_V241_STALE_PROVENANCE:-docs/rust-cutover/release/v0_24_0_provenance_reconciliation.md}"
V241_001_EVIDENCE_PATH="${NTPRO_V241_STALE_V241_001_EVIDENCE:-docs/rust-cutover/evidence/V241-001.md}"
V241_002_EVIDENCE_PATH="${NTPRO_V241_STALE_V241_002_EVIDENCE:-docs/rust-cutover/evidence/V241-002.md}"
V241_003_EVIDENCE_PATH="${NTPRO_V241_STALE_V241_003_EVIDENCE:-docs/rust-cutover/evidence/V241-003.md}"
V241_003_TASK_PATH="${NTPRO_V241_STALE_V241_003_TASK:-docs/rust-cutover/tasks/V241-003.md}"
MANIFEST_PATH="${NTPRO_V241_STALE_MANIFEST:-docs/rust-cutover/release/v0_24_0_release_manifest.json}"
VERIFICATION_PATH="${NTPRO_V241_STALE_VERIFICATION:-verification.md}"

fail() {
  echo "v24.1 stale pre-tag cleanup failed: $*" >&2
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
  "$V240_EVIDENCE_PATH" \
  "$READINESS_PATH" \
  "$RELEASE_NOTES_PATH" \
  "$CLOSEOUT_EVIDENCE_PATH" \
  "$PROVENANCE_RECONCILIATION_PATH" \
  "$V241_001_EVIDENCE_PATH" \
  "$V241_002_EVIDENCE_PATH" \
  "$V241_003_EVIDENCE_PATH" \
  "$V241_003_TASK_PATH" \
  "$MANIFEST_PATH" \
  "$VERIFICATION_PATH"; do
  require_file "$path"
done

strict_scan_files=(
  "$V240_EVIDENCE_PATH"
  "$READINESS_PATH"
  "$RELEASE_NOTES_PATH"
  "$CLOSEOUT_EVIDENCE_PATH"
  "$PROVENANCE_RECONCILIATION_PATH"
  "$V241_001_EVIDENCE_PATH"
  "$V241_002_EVIDENCE_PATH"
  "$V241_003_EVIDENCE_PATH"
)

if [[ -n "${NTPRO_V241_STALE_EXTRA_SCAN_FILE:-}" ]]; then
  require_file "$NTPRO_V241_STALE_EXTRA_SCAN_FILE"
  strict_scan_files+=("$NTPRO_V241_STALE_EXTRA_SCAN_FILE")
fi

stale_markers=(
  "tag_exists=false"
  "current_issue_state=OPEN"
  "stays open until"
  "offline_skip missing_local_git_tag:$RELEASE_TAG"
  "missing_local_git_tag:$RELEASE_TAG"
  "public release publication = pending"
  "tag gate run = pending"
  "tag gate result = pending"
  "RELEASE GATE CORRECTIVE FIX IN PROGRESS"
  "corrective fix in progress"
  "tag is intentionally absent"
)

for path in "${strict_scan_files[@]}"; do
  for marker in "${stale_markers[@]}"; do
    require_not_contains "$path" "$marker"
  done
done

verification_stale_markers=(
  "offline_skip missing_local_git_tag:$RELEASE_TAG"
  "scripts/ai/verify_release.sh v24-strict-provenance = PASS, tag_exists=false"
  "scripts/ai/verify_release.sh v24-release-gates = PASS, current_issue_state=OPEN"
  "The v0.24.0 tag is intentionally absent"
)

for marker in "${verification_stale_markers[@]}"; do
  require_not_contains "$VERIFICATION_PATH" "$marker"
done

for marker in \
  "Status: RELEASED / POST-RELEASE CLOSEOUT RECORDED" \
  "pre-release validation snapshot = historical, not post-release state" \
  "#752 V240-009 = closed" \
  "V240 issue set = 10/10 closed" \
  "v0.24.0 milestone = closed" \
  "$CLOSEOUT_EVIDENCE_PATH" \
  "$PROVENANCE_RECONCILIATION_PATH"; do
  require_contains "$V240_EVIDENCE_PATH" "$marker"
done

for marker in \
  "#752 V240-009 = closed after tag, hosted gate, public release, strict provenance, and publication evidence were recorded" \
  "V240 issue set = 10/10 closed" \
  "v0.24.0 milestone = closed"; do
  require_contains "$READINESS_PATH" "$marker"
done

for marker in \
  "V241-003 Evidence - V240-009 stale pre-tag evidence cleanup" \
  "pre-release validation snapshot = historical, not post-release state" \
  "issue #752 = closed" \
  "v0.24.0 milestone = closed"; do
  require_contains "$V241_003_EVIDENCE_PATH" "$marker"
done

if ! command -v gh >/dev/null 2>&1; then
  fail "gh is required for live stale cleanup proof"
fi
gh auth status >/dev/null 2>&1 || fail "gh authentication is required for live stale cleanup proof"

issue_json="$(gh_with_retry issue view "$ISSUE_NUMBER" --repo "$REPO" --json number,state,closedAt,url)"
milestone_json="$(gh_with_retry api "/repos/$REPO/milestones/$MILESTONE_NUMBER")"
release_json="$(gh_with_retry release view "$RELEASE_TAG" --repo "$REPO" --json tagName,isDraft,isPrerelease,url,publishedAt)"

ISSUE_JSON="$issue_json" \
MILESTONE_JSON="$milestone_json" \
RELEASE_JSON="$release_json" \
RELEASE_TAG="$RELEASE_TAG" \
MANIFEST_PATH="$MANIFEST_PATH" \
V241_003_EVIDENCE_PATH="$V241_003_EVIDENCE_PATH" \
V241_003_TASK_PATH="$V241_003_TASK_PATH" \
V240_EVIDENCE_PATH="$V240_EVIDENCE_PATH" \
python3 <<'PY'
import json
import os
from pathlib import Path


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


issue = json.loads(os.environ["ISSUE_JSON"])
milestone = json.loads(os.environ["MILESTONE_JSON"])
release = json.loads(os.environ["RELEASE_JSON"])
manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))

require(issue["number"] == 752, issue)
require(issue["state"] == "CLOSED", issue)
require(issue["closedAt"] == "2026-07-05T04:04:48Z", issue)

require(milestone["title"] == "v0.24.0", milestone)
require(milestone["state"] == "closed", milestone)
require(milestone["open_issues"] == 0, milestone)
require(milestone["closed_issues"] == 10, milestone)

require(release["tagName"] == os.environ["RELEASE_TAG"], release)
require(release["isDraft"] is False, release)
require(release["isPrerelease"] is False, release)

cleanup = manifest.get("post_release_stale_pretag_cleanup") or {}
require(cleanup.get("task_id") == "V241-003", cleanup)
require(cleanup.get("issue") == 772, cleanup)
require(cleanup.get("evidence_path") == os.environ["V241_003_EVIDENCE_PATH"], cleanup)
require(cleanup.get("task_path") == os.environ["V241_003_TASK_PATH"], cleanup)
require(cleanup.get("v240_evidence_path") == os.environ["V240_EVIDENCE_PATH"], cleanup)
require(cleanup.get("release_tag") == os.environ["RELEASE_TAG"], cleanup)
require(cleanup.get("issue_752_state") == "closed", cleanup)
require(cleanup.get("v0_24_0_milestone_state") == "closed", cleanup)
require(cleanup.get("pre_release_validation_snapshot") == "historical_not_post_release_state", cleanup)
require(cleanup.get("retag_required") is False, cleanup)
PY

if [[ "${NTPRO_V241_STALE_SELFTEST:-1}" == "1" ]]; then
  tmp_file="$(mktemp "${TMPDIR:-/tmp}/ntpro-v241-stale.XXXXXX.md")"
  cp "$V240_EVIDENCE_PATH" "$tmp_file"
  printf '\ntag_exists=false\n' >>"$tmp_file"
  if NTPRO_V241_STALE_SELFTEST=0 NTPRO_V241_STALE_EXTRA_SCAN_FILE="$tmp_file" "$0" >/tmp/ntpro-v241-stale-negative.log 2>&1; then
    rm -f "$tmp_file"
    fail "negative stale marker self-test unexpectedly passed"
  fi
  rm -f "$tmp_file"
fi

echo "v24_1_stale_pretag_cleanup status=ok release_tag=$RELEASE_TAG issue_752=closed milestone=v0.24.0:closed stale_selftest=${NTPRO_V241_STALE_SELFTEST:-1}"
