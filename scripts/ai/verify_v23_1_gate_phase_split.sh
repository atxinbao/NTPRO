#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

MODE="${1:-all}"
RELEASE_TAG="${NTPRO_V231_PHASE_RELEASE_TAG:-ntpro-rust-only-v0.23.0}"
RELEASE_NAME="${NTPRO_V231_PHASE_RELEASE_NAME:-NTPRO Rust-only v0.23.0}"
RELEASE_URL="${NTPRO_V231_PHASE_RELEASE_URL:-https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.23.0}"
PUBLISHED_AT="${NTPRO_V231_PHASE_PUBLISHED_AT:-2026-07-03T18:34:39Z}"
TAG_SHA="${NTPRO_V231_PHASE_TAG_SHA:-783b024621116d50feaf418f12cb95fb95f87575}"
GATE_RUN_ID="${NTPRO_V231_PHASE_GATE_RUN_ID:-28673868094}"
GATE_URL="${NTPRO_V231_PHASE_GATE_URL:-https://github.com/atxinbao/NTPRO/actions/runs/28673868094}"
GATE_COMPLETED_AT="${NTPRO_V231_PHASE_GATE_COMPLETED_AT:-2026-07-03T18:29:30Z}"
GATE_JOBS_TOTAL="${NTPRO_V231_PHASE_GATE_JOBS_TOTAL:-66}"
GATE_JOBS_SUCCESS="${NTPRO_V231_PHASE_GATE_JOBS_SUCCESS:-66}"
MILESTONE_NUMBER="${NTPRO_V231_PHASE_MILESTONE_NUMBER:-11}"

PHASE_CONTRACT_PATH="${NTPRO_V231_PHASE_CONTRACT:-docs/rust-cutover/release/v0_23_0_gate_phase_split.md}"
READINESS_PATH="${NTPRO_V231_PHASE_READINESS:-docs/rust-cutover/release/v0_23_0_readiness_report.md}"
RELEASE_NOTES_PATH="${NTPRO_V231_PHASE_RELEASE_NOTES:-docs/rust-cutover/release/v0_23_0_release_notes.md}"
MANIFEST_PATH="${NTPRO_V231_PHASE_MANIFEST:-docs/rust-cutover/release/v0_23_0_release_manifest.json}"

fail() {
  echo "v23.1 gate phase split failed: $*" >&2
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

validate_pre_release_contract() {
  for marker in \
    "pre_release_phase = v23_pre_release_gate" \
    "pre_release_issue_718_state = open_allowed_until_publication" \
    "pre_release_milestone_state = open_allowed_until_publication" \
    "pre_release_github_release = not_required_before_publication" \
    "pre_release_hosted_run_success = not_required_before_tag_gate" \
    "pre_release_publication_evidence = not_required_before_publication" \
    "pre_release_output = waiting_for_tag_hosted_gate_public_release_publication_evidence"; do
    require_contains "$PHASE_CONTRACT_PATH" "$marker"
  done
}

validate_post_release_contract() {
  for marker in \
    "post_release_phase = v23_post_release_closeout_gate" \
    "post_release_issue_718_state = closed_required" \
    "post_release_milestone_state = closed_required" \
    "post_release_github_release = required_non_draft_non_prerelease" \
    "post_release_hosted_run_success = required" \
    "post_release_publication_evidence = required_published_after_gate" \
    "post_release_output = released_closeout_verified"; do
    require_contains "$PHASE_CONTRACT_PATH" "$marker"
  done
}

collect_live_post_release_state() {
  if ! command -v gh >/dev/null 2>&1; then
    fail "gh is required for post-release live closeout proof"
  fi
  gh auth status >/dev/null 2>&1 || fail "gh authentication is required for post-release live closeout proof"

  release_json="$(gh_with_retry release view "$RELEASE_TAG" --json tagName,name,isDraft,isPrerelease,url,publishedAt,targetCommitish)"
  run_json="$(gh_with_retry run view "$GATE_RUN_ID" --json status,conclusion,updatedAt,url,headSha,jobs)"
  milestone_json="$(gh_with_retry api "repos/atxinbao/NTPRO/milestones/$MILESTONE_NUMBER")"

  issue_state_file="$(mktemp "${TMPDIR:-/tmp}/ntpro-v231-phase-issues.XXXXXX")"
  for issue in 711 712 713 714 715 716 717 718; do
    state="$(gh_with_retry issue view "$issue" --json state --jq .state)" || fail "could not read issue #$issue"
    printf '{"number":%s,"state":"%s"}\n' "$issue" "$state" >> "$issue_state_file"
  done
  issues_json="$(python3 - "$issue_state_file" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
items = [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]
print(json.dumps(items, sort_keys=True))
PY
)"
  rm -f "$issue_state_file"
}

validate_post_release_state() {
  RELEASE_JSON="$release_json" \
  RUN_JSON="$run_json" \
  ISSUES_JSON="$issues_json" \
  MILESTONE_JSON="$milestone_json" \
  RELEASE_TAG="$RELEASE_TAG" \
  RELEASE_NAME="$RELEASE_NAME" \
  RELEASE_URL="$RELEASE_URL" \
  PUBLISHED_AT="$PUBLISHED_AT" \
  TAG_SHA="$TAG_SHA" \
  GATE_RUN_ID="$GATE_RUN_ID" \
  GATE_URL="$GATE_URL" \
  GATE_COMPLETED_AT="$GATE_COMPLETED_AT" \
  GATE_JOBS_TOTAL="$GATE_JOBS_TOTAL" \
  GATE_JOBS_SUCCESS="$GATE_JOBS_SUCCESS" \
  python3 <<'PY'
import copy
import json
import os


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def validate_post_release(state: dict) -> None:
    release = state.get("release")
    require(isinstance(release, dict), "missing GitHub release")
    require(release.get("tagName") == os.environ["RELEASE_TAG"], "release tag mismatch")
    require(release.get("name") == os.environ["RELEASE_NAME"], "release name mismatch")
    require(release.get("url") == os.environ["RELEASE_URL"], "release URL mismatch")
    require(release.get("publishedAt") == os.environ["PUBLISHED_AT"], "release publishedAt mismatch")
    require(release.get("isDraft") is False, "release must not be draft")
    require(release.get("isPrerelease") is False, "release must not be prerelease")
    require(release.get("targetCommitish") == "main", "release targetCommitish mismatch")

    run = state.get("run")
    require(isinstance(run, dict), "missing hosted release gate run")
    require(str(run.get("databaseId", os.environ["GATE_RUN_ID"])) == os.environ["GATE_RUN_ID"], "gate run id mismatch")
    require(run.get("status") == "completed", "hosted gate must be completed")
    require(run.get("conclusion") == "success", "hosted gate must succeed")
    require(run.get("url") == os.environ["GATE_URL"], "hosted gate URL mismatch")
    require(run.get("updatedAt") == os.environ["GATE_COMPLETED_AT"], "hosted gate completion mismatch")
    require(run.get("headSha") == os.environ["TAG_SHA"], "hosted gate SHA mismatch")
    jobs = run.get("jobs") or []
    require(len(jobs) == int(os.environ["GATE_JOBS_TOTAL"]), "hosted gate job count mismatch")
    require(sum(1 for job in jobs if job.get("conclusion") == "success") == int(os.environ["GATE_JOBS_SUCCESS"]), "hosted gate success count mismatch")

    issues = {item.get("number"): item.get("state") for item in state.get("issues") or []}
    require(set(issues) == {711, 712, 713, 714, 715, 716, 717, 718}, "issue closeout set mismatch")
    for issue, status in issues.items():
        require(status == "CLOSED", f"issue #{issue} must be CLOSED, got {status}")

    milestone = state.get("milestone")
    require(isinstance(milestone, dict), "missing milestone")
    require(milestone.get("title") == "v0.23.0", "milestone title mismatch")
    require(milestone.get("state") == "closed", "milestone must be closed")
    require(milestone.get("open_issues") == 0, "milestone open issue count must be zero")
    require(milestone.get("closed_issues") == 8, "milestone closed issue count mismatch")


state = {
    "release": json.loads(os.environ["RELEASE_JSON"]),
    "run": json.loads(os.environ["RUN_JSON"]),
    "issues": json.loads(os.environ["ISSUES_JSON"]),
    "milestone": json.loads(os.environ["MILESTONE_JSON"]),
}
state["run"]["databaseId"] = int(os.environ["GATE_RUN_ID"])
validate_post_release(state)

if os.environ.get("NTPRO_V231_PHASE_SELFTEST", "1") == "1":
    mutations = {
        "missing_release": lambda s: s.update({"release": None}),
        "failed_hosted_run": lambda s: s["run"].update({"conclusion": "failure"}),
        "open_issue_718": lambda s: [item.update({"state": "OPEN"}) for item in s["issues"] if item.get("number") == 718],
        "open_milestone": lambda s: s["milestone"].update({"state": "open", "open_issues": 1}),
    }
    for name, mutate in mutations.items():
        candidate = copy.deepcopy(state)
        mutate(candidate)
        try:
            validate_post_release(candidate)
        except AssertionError:
            continue
        raise AssertionError(f"negative self-test unexpectedly passed: {name}")
PY
}

require_file "$PHASE_CONTRACT_PATH"
require_file "$READINESS_PATH"
require_file "$RELEASE_NOTES_PATH"
require_file "$MANIFEST_PATH"

case "$MODE" in
  all)
    validate_pre_release_contract
    validate_post_release_contract
    collect_live_post_release_state
    validate_post_release_state
    ;;
  pre-release-contract)
    validate_pre_release_contract
    ;;
  post-release-live)
    validate_post_release_contract
    collect_live_post_release_state
    validate_post_release_state
    ;;
  *)
    fail "unknown mode: $MODE"
    ;;
esac

for marker in \
  "v23.1 gate phase split = required" \
  "release closeout evidence = docs/rust-cutover/release/v0_23_0_release_closeout_evidence.md"; do
  require_contains "$READINESS_PATH" "$marker"
done

require_contains "$RELEASE_NOTES_PATH" "scripts/ai/verify_release.sh v23.1-gate-phase-split"
require_contains "$MANIFEST_PATH" "v23.1-gate-phase-split"
require_contains "$MANIFEST_PATH" "gate_phase_split_path"

echo "v23_1_gate_phase_split status=ok mode=$MODE release_tag=$RELEASE_TAG gate_run=$GATE_RUN_ID issue_718=closed milestone=v0.23.0:closed negative_selftest=${NTPRO_V231_PHASE_SELFTEST:-1}"
