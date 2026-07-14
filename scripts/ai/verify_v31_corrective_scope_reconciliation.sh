#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

MODE="${1:-source}"
REPO="${NTPRO_V311_CORRECTIVE_REPO:-atxinbao/NTPRO}"
MANIFEST_PATH="${NTPRO_V311_CORRECTIVE_MANIFEST:-docs/rust-cutover/release/v0_31_0_release_manifest.json}"

fail() {
  echo "v31 corrective scope reconciliation failed: $*" >&2
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

source_files=(
  docs/rust-cutover/release/v0_31_0_release_manifest.json
  docs/rust-cutover/release/v0_31_0_release_notes.md
  docs/rust-cutover/release/v0_31_0_readiness_report.md
  docs/rust-cutover/release/v0_31_0_release_closeout_evidence.md
  docs/rust-cutover/release/README.md
  docs/rust-cutover/tasks/V310-009.md
  docs/rust-cutover/tasks/V310-010.md
  docs/rust-cutover/evidence/V310-009.md
  docs/rust-cutover/evidence/V310-010.md
  docs/rust-cutover/tasks/V311-004.md
  docs/rust-cutover/evidence/V311-004.md
  scripts/ai/verify_v31_release_gates.sh
  scripts/ai/check_github_release_published.sh
  scripts/ai/verify_v31_corrective_scope_reconciliation.sh
)

for path in "${source_files[@]}"; do
  require_file "$path"
done

run_source_validation() {
  SOURCE_FILES="$(printf '%s\n' "${source_files[@]}")" \
  MANIFEST_PATH="$MANIFEST_PATH" \
  python3 <<'PY'
import copy
import json
import os
import re
from pathlib import Path


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def require_contains(text: str, marker: str, label: str) -> None:
    require(marker in text, f"{label} missing marker: {marker}")


expected = [1006, 1007, 1008, 1009, 1010, 1011, 1012, 1013, 1014, 1015, 1033]
manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))
files = [Path(line) for line in os.environ["SOURCE_FILES"].splitlines() if line]


def validate_manifest(candidate: dict) -> None:
    scope = candidate.get("release_scope") or {}
    require(scope.get("milestone_issue_count") == 11, "milestone issue count mismatch")
    require(scope.get("exact_milestone_issue_numbers") == expected, "exact milestone issue numbers mismatch")
    require(scope.get("exact_milestone_issue_set") == "#1006-#1015 plus #1033", "exact milestone issue set mismatch")
    require(scope.get("final_release_scope_issue_count") == 11, "final release scope issue count mismatch")
    require(scope.get("final_release_scope_evidence_count") == 11, "final release scope evidence count mismatch")
    require(scope.get("registered_corrective_scope_exception_count") == 1, "corrective exception count mismatch")
    require(scope.get("registered_corrective_scope_exception_issue_numbers") == [1033], "corrective exception issue mismatch")
    require(scope.get("unregistered_corrective_milestone_issues_fail_closed") is True, "unregistered corrective fail-closed missing")

    evidence = candidate.get("v310_evidence") or []
    require(len(evidence) == 11, "v310 evidence count mismatch")
    by_issue = {item.get("issue"): item.get("task_id") for item in evidence}
    for issue, task in zip(expected, [f"V310-{i:03d}" for i in range(10)] + ["V310-010"]):
        require(by_issue.get(issue) == task, f"issue/task mapping mismatch: {issue}")


validate_manifest(manifest)

for bad_name, mutate in {
    "append_unregistered_corrective_issue": lambda m: m["release_scope"]["exact_milestone_issue_numbers"].append(1016),
    "replace_corrective_issue": lambda m: m["release_scope"].update({"registered_corrective_scope_exception_issue_numbers": [1016]}),
    "increase_corrective_count": lambda m: m["release_scope"].update({"registered_corrective_scope_exception_count": 2}),
    "disable_unregistered_fail_closed": lambda m: m["release_scope"].update({"unregistered_corrective_milestone_issues_fail_closed": False}),
    "scope_count_drift": lambda m: m["release_scope"].update({"final_release_scope_issue_count": 10}),
}.items():
    candidate = copy.deepcopy(manifest)
    mutate(candidate)
    try:
        validate_manifest(candidate)
    except AssertionError:
        continue
    raise AssertionError(f"negative self-test unexpectedly passed: {bad_name}")

required_markers = [
    "V310 final release scope issue count = 11",
    "V310 final release scope evidence count = 11",
    "V310 exact milestone issue set = #1006-#1015 plus #1033",
    "V310 registered corrective-scope exception count = 1",
    "V310 registered corrective-scope exception issues = #1033",
]
for path in [
    Path("docs/rust-cutover/release/v0_31_0_release_notes.md"),
    Path("docs/rust-cutover/release/v0_31_0_readiness_report.md"),
]:
    text = path.read_text(encoding="utf-8")
    for marker in required_markers:
        require_contains(text, marker, str(path))

closeout = Path("docs/rust-cutover/release/v0_31_0_release_closeout_evidence.md").read_text(encoding="utf-8")
for marker in [
    "V310 final release issue set = 11/11 closed",
    "V310 exact milestone issue set = #1006-#1015 plus #1033",
    "#1033 V310-010 = closed",
    "V310 registered corrective-scope exception count = 1",
    "corrective release-publication scope changes runtime behavior = false",
    "corrective release-publication scope changes trading behavior = false",
]:
    require_contains(closeout, marker, "v31 closeout evidence")

readme = Path("docs/rust-cutover/release/README.md").read_text(encoding="utf-8")
require_contains(readme, "exact V310 issue set `#1006-#1015 plus #1033`", "release index")
require_contains(readme, "V310-010 hosted v31 release gate ARG_MAX", "release index")

v31010 = Path("docs/rust-cutover/evidence/V310-010.md").read_text(encoding="utf-8")
require_contains(v31010, "historical failed-run output only; it is not", "V310-010 evidence")

stale_issue_marker = "#" + "1016"
stale_range_marker = "#1006-" + stale_issue_marker
stale_numeric_range = "1006-" + "1016"
for path in files:
    text = path.read_text(encoding="utf-8")
    if (
        stale_issue_marker in text
        or stale_range_marker in text
        or stale_numeric_range in text
    ):
        raise AssertionError(f"stale post-#1015 v31 scope reference in {path}")
    stale_count_marker = "V310 final release scope issue count = " + "10"
    if stale_count_marker in text and path != Path("docs/rust-cutover/evidence/V310-010.md"):
        raise AssertionError(f"stale V310 count=10 marker in {path}")

for path in [
    Path("docs/rust-cutover/tasks/V311-004.md"),
    Path("docs/rust-cutover/evidence/V311-004.md"),
]:
    text = path.read_text(encoding="utf-8")
    require_contains(text, "#1039", str(path))
    require_contains(text, "#1006-#1015 plus #1033", str(path))
PY
}

run_live_validation() {
  command -v gh >/dev/null 2>&1 || fail "gh_unavailable"
  gh auth status >/dev/null 2>&1 || fail "gh_auth_unavailable"

  milestone_issues="$(gh_with_retry issue list --repo "$REPO" --milestone v0.31.0 --state all --limit 100 --json number,state)"
  issue_json="$(gh_with_retry issue view 1039 --repo "$REPO" --json number,title,body,comments,milestone)"
  milestone_json="$(gh_with_retry api "repos/$REPO/milestones/29" --jq '{number,title,description,state,open_issues,closed_issues}')"

  MILESTONE_ISSUES="$milestone_issues" \
  ISSUE_JSON="$issue_json" \
  V311_MILESTONE_JSON="$milestone_json" \
  python3 <<'PY'
import json
import os


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


expected_v310 = [1006, 1007, 1008, 1009, 1010, 1011, 1012, 1013, 1014, 1015, 1033]
issues = json.loads(os.environ["MILESTONE_ISSUES"])
observed = sorted(item["number"] for item in issues)
require(observed == expected_v310, f"live v0.31.0 issue set mismatch: {observed}")
for item in issues:
    require(item["state"] == "CLOSED", f"live V310 issue must be closed: {item}")

issue = json.loads(os.environ["ISSUE_JSON"])
body = issue.get("body") or ""
comments = "\n".join((comment.get("body") or "") for comment in issue.get("comments", []))
combined = body + "\n" + comments
require(issue.get("number") == 1039, "live issue mismatch")
require("#1036 #1037 #1038 #1039 #1040 #1041" in combined, "V311 exact issue set missing from GitHub-visible wording")
require("#1042 #1043 #1044 #1045 #1046 #1047 #1048 #1049 #1050 #1051" in combined, "V320 exact issue set missing from GitHub-visible wording")
require("ntpro-rust-only-v0.31.1 release evidence" in combined, "v31.1 release evidence dependency missing")
require("does not authorize backend production closeout" in combined, "backend closeout non-authorization wording missing")

milestone = json.loads(os.environ["V311_MILESTONE_JSON"])
description = milestone.get("description") or ""
require(milestone.get("title") == "v0.31.1", "V311 milestone title mismatch")
require("#1036 #1037 #1038 #1039 #1040 #1041" in description, "milestone V311 exact set missing")
require("Blocks v0.32.0 exact issue set #1042 #1043 #1044 #1045 #1046 #1047 #1048 #1049 #1050 #1051" in description, "milestone V320 blocker missing")
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

echo "v31_corrective_scope_reconciliation status=ok mode=$MODE final_scope_issues=11 corrective_exception=#1033 negative_selftest=5"
