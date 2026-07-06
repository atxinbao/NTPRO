#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

MODE="${1:-all}"
REPO="${NTPRO_V251_GATE_SPLIT_REPO:-atxinbao/NTPRO}"
RELEASE_VERSION="${NTPRO_V251_GATE_SPLIT_RELEASE_VERSION:-v0.25.0}"
RELEASE_TAG="${NTPRO_V251_GATE_SPLIT_RELEASE_TAG:-ntpro-rust-only-v0.25.0}"
RELEASE_NAME="${NTPRO_V251_GATE_SPLIT_RELEASE_NAME:-NTPRO Rust-only v0.25.0}"
TAG_SHA="${NTPRO_V251_GATE_SPLIT_TAG_SHA:-eedcdab1d3ca85d6f51b368b5f36208a7b591026}"
GATE_RUN_ID="${NTPRO_V251_GATE_SPLIT_GATE_RUN_ID:-28764231552}"
GATE_URL="${NTPRO_V251_GATE_SPLIT_GATE_URL:-https://github.com/atxinbao/NTPRO/actions/runs/28764231552}"
GATE_COMPLETED_AT="${NTPRO_V251_GATE_SPLIT_GATE_COMPLETED_AT:-2026-07-06T04:00:17Z}"
PUBLISH_RUN_ID="${NTPRO_V251_GATE_SPLIT_PUBLISH_RUN_ID:-28766874471}"
PUBLISH_URL="${NTPRO_V251_GATE_SPLIT_PUBLISH_URL:-https://github.com/atxinbao/NTPRO/actions/runs/28766874471}"
PUBLISHED_AT="${NTPRO_V251_GATE_SPLIT_PUBLISHED_AT:-2026-07-06T04:02:02Z}"
MILESTONE_NUMBER="${NTPRO_V251_GATE_SPLIT_MILESTONE_NUMBER:-16}"

CONTRACT_PATH="${NTPRO_V251_GATE_SPLIT_CONTRACT:-docs/rust-cutover/release/v0_25_0_post_release_gate_split.md}"
MANIFEST_PATH="${NTPRO_V251_GATE_SPLIT_MANIFEST:-docs/rust-cutover/release/v0_25_0_release_manifest.json}"
READINESS_PATH="${NTPRO_V251_GATE_SPLIT_READINESS:-docs/rust-cutover/release/v0_25_0_readiness_report.md}"
CLOSEOUT_PATH="${NTPRO_V251_GATE_SPLIT_CLOSEOUT:-docs/rust-cutover/release/v0_25_0_release_closeout_evidence.md}"
TASK_PATH="${NTPRO_V251_GATE_SPLIT_TASK:-docs/rust-cutover/tasks/V251-005.md}"
EVIDENCE_PATH="${NTPRO_V251_GATE_SPLIT_EVIDENCE:-docs/rust-cutover/evidence/V251-005.md}"

fail() {
  echo "v25.1 post-release gate split failed: $*" >&2
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

for path in "$CONTRACT_PATH" "$MANIFEST_PATH" "$READINESS_PATH" "$CLOSEOUT_PATH" "$TASK_PATH" "$EVIDENCE_PATH"; do
  require_file "$path"
done

validate_contract_markers() {
  for marker in \
    "pre_release_phase = v25_pre_release_pr_gate" \
    "pre_release_missing_tag = allowed_only_when_NTPRO_RELEASE_GATE_is_not_1" \
    "pre_release_current_issue_open = allowed_until_tag_publication" \
    "pre_release_offline_publication = allowed_only_with_explicit_pr_mode" \
    "tag_release_phase = v25_tag_release_gate" \
    "tag_release_NTPRO_RELEASE_GATE = required" \
    "tag_release_missing_tag = fail_closed" \
    "tag_release_head_tag_match = required" \
    "tag_release_github_release = required_non_draft_non_prerelease" \
    "tag_release_hosted_run_success = required" \
    "tag_release_current_issue_state_OPEN = fail_closed" \
    "tag_release_corrective_issue_804_OPEN = fail_closed" \
    "tag_release_milestone_open = fail_closed" \
    "post_release_closeout_phase = v25_post_release_closeout_gate" \
    "post_release_missing_tag = fail_closed" \
    "post_release_offline_publication = fail_closed" \
    "post_release_pre_publication_state = fail_closed" \
    "post_release_current_issue_state_OPEN = fail_closed" \
    "v0_26_start_gate_without_v25_1_release_evidence = fail_closed"; do
    require_contains "$CONTRACT_PATH" "$marker"
  done
}

validate_source_markers() {
  for marker in \
    "post-release gate split = required" \
    "scripts/ai/verify_release.sh v25.1-post-release-gate-split" \
    "v0.26.0 start gate = blocked until all V251 issues are closed and v0.25.1 release evidence is published"; do
    require_contains "$READINESS_PATH" "$marker"
    require_contains "$CLOSEOUT_PATH" "$marker"
  done

  for marker in \
    "current_issue_state=OPEN" \
    "tag_exists=false" \
    "missing local tag" \
    "offline publication proof = allowed" \
    "offline publication proof = pass" \
    "public release publication = pending" \
    "release publication evidence status = pending"; do
    require_not_contains "$READINESS_PATH" "$marker"
    require_not_contains "$CLOSEOUT_PATH" "$marker"
  done
}

collect_live_state() {
  if ! command -v gh >/dev/null 2>&1; then
    fail "gh is required for live v25 gate split proof"
  fi
  gh auth status >/dev/null 2>&1 || fail "gh authentication is required for live v25 gate split proof"

  release_json="$(gh_with_retry release view "$RELEASE_TAG" --repo "$REPO" --json tagName,name,isDraft,isPrerelease,url,publishedAt,targetCommitish)"
  gate_run_json="$(gh_with_retry run view "$GATE_RUN_ID" --repo "$REPO" --json status,conclusion,updatedAt,url,headSha,workflowName)"
  publish_run_json="$(gh_with_retry run view "$PUBLISH_RUN_ID" --repo "$REPO" --json status,conclusion,updatedAt,url,headSha,workflowName)"
  milestone_json="$(gh_with_retry api "/repos/$REPO/milestones/$MILESTONE_NUMBER")"
  remote_tag_sha="$(git ls-remote --tags origin "refs/tags/$RELEASE_TAG" | awk '{print $1}')"
  origin_main_sha="$(git rev-parse origin/main)"
  if git merge-base --is-ancestor "$TAG_SHA" "$origin_main_sha"; then
    tag_ancestor_of_origin_main="true"
  else
    tag_ancestor_of_origin_main="false"
  fi

  current_issue_state="$(gh_with_retry issue view 785 --repo "$REPO" --json state --jq .state)"
  corrective_issue_state="$(gh_with_retry issue view 804 --repo "$REPO" --json state --jq .state)"

  v251_issue_state_file="$(mktemp "${TMPDIR:-/tmp}/ntpro-v251-gate-split-issues.XXXXXX")"
  for issue in 806 807 808 809 810 811; do
    state="$(gh_with_retry issue view "$issue" --repo "$REPO" --json state --jq .state)" || fail "could not read V251 issue #$issue"
    printf '{"number":%s,"state":"%s"}\n' "$issue" "$state" >> "$v251_issue_state_file"
  done
  v251_issues_json="$(python3 - "$v251_issue_state_file" <<'PY'
import json
import sys
from pathlib import Path

items = [json.loads(line) for line in Path(sys.argv[1]).read_text(encoding="utf-8").splitlines() if line.strip()]
print(json.dumps(items, sort_keys=True))
PY
)"
  rm -f "$v251_issue_state_file"
}

validate_live_state() {
  RELEASE_JSON="$release_json" \
  GATE_RUN_JSON="$gate_run_json" \
  PUBLISH_RUN_JSON="$publish_run_json" \
  MILESTONE_JSON="$milestone_json" \
  V251_ISSUES_JSON="$v251_issues_json" \
  RELEASE_VERSION="$RELEASE_VERSION" \
  RELEASE_TAG="$RELEASE_TAG" \
  RELEASE_NAME="$RELEASE_NAME" \
  TAG_SHA="$TAG_SHA" \
  REMOTE_TAG_SHA="$remote_tag_sha" \
  ORIGIN_MAIN_SHA="$origin_main_sha" \
  TAG_ANCESTOR_OF_ORIGIN_MAIN="$tag_ancestor_of_origin_main" \
  GATE_URL="$GATE_URL" \
  GATE_COMPLETED_AT="$GATE_COMPLETED_AT" \
  PUBLISH_URL="$PUBLISH_URL" \
  PUBLISHED_AT="$PUBLISHED_AT" \
  CURRENT_ISSUE_STATE="$current_issue_state" \
  CORRECTIVE_ISSUE_STATE="$corrective_issue_state" \
  MANIFEST_PATH="$MANIFEST_PATH" \
  python3 <<'PY'
import copy
import json
import os
from datetime import datetime, timezone
from pathlib import Path


def parse_time(value: str) -> datetime:
    parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def validate_tag_release_gate(state: dict) -> None:
    require(state["release_gate"] is True, "NTPRO_RELEASE_GATE=1 required")
    require(state["tag_exists"] is True, "tag must exist")
    require(state["head_sha"] == state["tag_sha"], "HEAD must match release tag")
    require(state["offline_publication_proof"] is False, "offline publication proof is forbidden")
    require(state["pre_publication_state"] is False, "pre-publication state is forbidden")
    require(state["current_issue_state"] == "CLOSED", "current release issue must be closed")
    require(state["corrective_issue_state"] == "CLOSED", "corrective issue #804 must be closed")
    require(state["milestone_state"] == "closed", "v0.25.0 milestone must be closed")
    require(state["milestone_open_issues"] == 0, "v0.25.0 milestone open issues must be zero")

    release = state["release"]
    require(release["tagName"] == state["release_tag"], "release tag mismatch")
    require(release["name"] == state["release_name"], "release name mismatch")
    require(release["isDraft"] is False, "release must not be draft")
    require(release["isPrerelease"] is False, "release must not be prerelease")
    require(release["publishedAt"] == state["published_at"], "release publishedAt mismatch")

    gate_run = state["gate_run"]
    require(gate_run["workflowName"] == "Rust Cutover Release Gate", "wrong release gate workflow")
    require(gate_run["status"] == "completed", "release gate must be completed")
    require(gate_run["conclusion"] == "success", "release gate must succeed")
    require(gate_run["url"] == state["gate_url"], "release gate URL mismatch")
    require(gate_run["updatedAt"] == state["gate_completed_at"], "release gate completion mismatch")
    require(gate_run["headSha"] == state["tag_sha"], "release gate must run on tag SHA")


def validate_post_release_closeout_gate(state: dict) -> None:
    require(state["tag_exists"] is True, "post-release tag must exist")
    require(state["remote_tag_sha"] == state["tag_sha"], "remote tag SHA mismatch")
    require(state["tag_ancestor_of_origin_main"] is True, "tag must be ancestor of origin/main")
    require(state["offline_publication_proof"] is False, "offline publication proof is forbidden post-release")
    require(state["pre_publication_state"] is False, "pre-publication state is forbidden post-release")
    require(state["current_issue_state"] == "CLOSED", "current release issue must be closed post-release")
    require(state["corrective_issue_state"] == "CLOSED", "corrective issue must be closed post-release")
    require(state["milestone_state"] == "closed", "milestone must be closed post-release")
    require(parse_time(state["published_at"]) >= parse_time(state["gate_completed_at"]), "release must publish after gate")
    validate_tag_release_gate({**state, "release_gate": True, "head_sha": state["tag_sha"]})

    publish_run = state["publish_run"]
    require(publish_run["workflowName"] == "Rust Cutover Publish Release", "wrong publish workflow")
    require(publish_run["status"] == "completed", "publish workflow must be completed")
    require(publish_run["conclusion"] == "success", "publish workflow must succeed")
    require(publish_run["url"] == state["publish_url"], "publish workflow URL mismatch")
    require(publish_run["headSha"] == state["tag_sha"], "publish workflow must run on tag SHA")


def validate_v260_start_gate(manifest: dict, v251_issues: list[dict]) -> None:
    gate = manifest.get("v0_26_start_gate") or {}
    require(gate.get("task_id") == "V251-005", "v0.26 start gate task mismatch")
    require(gate.get("requires_v25_1_release_evidence_published") is True, "v0.26 must require v0.25.1 release evidence")
    require(gate.get("v25_1_release_evidence_published") is False, "v0.25.1 release evidence must be marked missing before #811")
    require(gate.get("v0_26_start_allowed") is False, "v0.26 start must be blocked before v0.25.1 evidence")
    require(gate.get("fail_closed_when_release_evidence_missing") is True, "v0.26 start gate must fail closed")
    require(gate.get("blocked_by_milestone") == "v0.25.1", "v0.26 start gate milestone mismatch")
    require(gate.get("required_v251_issues") == [806, 807, 808, 809, 810, 811], "required V251 issue set mismatch")
    live_open = sorted(item["number"] for item in v251_issues if item.get("state") != "CLOSED")
    if live_open:
        require(
            811 in live_open,
            f"only the current V251 release-gate issue may remain open before v0.25.1 publication: {live_open}",
        )


manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))
state = {
    "release_version": os.environ["RELEASE_VERSION"],
    "release_tag": os.environ["RELEASE_TAG"],
    "release_name": os.environ["RELEASE_NAME"],
    "tag_sha": os.environ["TAG_SHA"],
    "remote_tag_sha": os.environ["REMOTE_TAG_SHA"],
    "head_sha": os.environ["TAG_SHA"],
    "origin_main_sha": os.environ["ORIGIN_MAIN_SHA"],
    "tag_exists": bool(os.environ["REMOTE_TAG_SHA"]),
    "tag_ancestor_of_origin_main": os.environ["TAG_ANCESTOR_OF_ORIGIN_MAIN"] == "true",
    "offline_publication_proof": False,
    "pre_publication_state": False,
    "current_issue_state": os.environ["CURRENT_ISSUE_STATE"],
    "corrective_issue_state": os.environ["CORRECTIVE_ISSUE_STATE"],
    "release": json.loads(os.environ["RELEASE_JSON"]),
    "gate_run": json.loads(os.environ["GATE_RUN_JSON"]),
    "publish_run": json.loads(os.environ["PUBLISH_RUN_JSON"]),
    "gate_url": os.environ["GATE_URL"],
    "gate_completed_at": os.environ["GATE_COMPLETED_AT"],
    "publish_url": os.environ["PUBLISH_URL"],
    "published_at": os.environ["PUBLISHED_AT"],
}
milestone = json.loads(os.environ["MILESTONE_JSON"])
state["milestone_state"] = milestone["state"]
state["milestone_open_issues"] = milestone["open_issues"]

v251_issues = json.loads(os.environ["V251_ISSUES_JSON"])
validate_tag_release_gate({**state, "release_gate": True})
validate_post_release_closeout_gate(state)
validate_v260_start_gate(manifest, v251_issues)

if os.environ.get("NTPRO_V251_GATE_SPLIT_SELFTEST", "1") == "1":
    tag_gate_mutations = {
        "missing_tag": lambda s: s.update({"tag_exists": False, "remote_tag_sha": ""}),
        "head_tag_mismatch": lambda s: s.update({"head_sha": "0000000000000000000000000000000000000000"}),
        "offline_publication": lambda s: s.update({"offline_publication_proof": True}),
        "pre_publication_state": lambda s: s.update({"pre_publication_state": True}),
        "open_current_issue": lambda s: s.update({"current_issue_state": "OPEN"}),
        "open_corrective_issue": lambda s: s.update({"corrective_issue_state": "OPEN"}),
        "open_milestone": lambda s: s.update({"milestone_state": "open", "milestone_open_issues": 1}),
        "failed_hosted_run": lambda s: s["gate_run"].update({"conclusion": "failure"}),
        "draft_release": lambda s: s["release"].update({"isDraft": True}),
    }
    for name, mutate in tag_gate_mutations.items():
        candidate = copy.deepcopy({**state, "release_gate": True})
        mutate(candidate)
        try:
            validate_tag_release_gate(candidate)
        except AssertionError:
            continue
        raise AssertionError(f"negative self-test unexpectedly passed: {name}")

    v260_bad = copy.deepcopy(manifest)
    v260_bad["v0_26_start_gate"]["v0_26_start_allowed"] = True
    try:
        validate_v260_start_gate(v260_bad, v251_issues)
    except AssertionError:
        pass
    else:
        raise AssertionError("negative self-test unexpectedly passed: v0.26 start allowed without v0.25.1 evidence")
PY
}

case "$MODE" in
  all)
    validate_contract_markers
    validate_source_markers
    collect_live_state
    validate_live_state
    ;;
  contract)
    validate_contract_markers
    validate_source_markers
    ;;
  live)
    collect_live_state
    validate_live_state
    ;;
  *)
    fail "unknown mode: $MODE"
    ;;
esac

echo "v25_1_post_release_gate_split status=ok mode=$MODE release_tag=$RELEASE_TAG gate_run=$GATE_RUN_ID current_issue=785:${current_issue_state:-not_checked} corrective_issue=804:${corrective_issue_state:-not_checked} v260_start=blocked negative_selftest=${NTPRO_V251_GATE_SPLIT_SELFTEST:-1}"
