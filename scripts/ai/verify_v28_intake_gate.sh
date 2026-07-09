#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V28_INTAKE_REPO:-atxinbao/NTPRO}"
V271_RELEASE_VERSION="${NTPRO_V28_INTAKE_V271_RELEASE_VERSION:-v0.27.1}"
V271_RELEASE_TAG="${NTPRO_V28_INTAKE_V271_RELEASE_TAG:-ntpro-rust-only-v0.27.1}"
V271_RELEASE_NAME="${NTPRO_V28_INTAKE_V271_RELEASE_NAME:-NTPRO Rust-only v0.27.1}"
V271_RELEASE_URL="${NTPRO_V28_INTAKE_V271_RELEASE_URL:-https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.27.1}"
V271_MANIFEST_PATH="${NTPRO_V28_INTAKE_V271_MANIFEST:-docs/rust-cutover/release/v0_27_1_release_manifest.json}"
V271_READINESS_PATH="${NTPRO_V28_INTAKE_V271_READINESS:-docs/rust-cutover/release/v0_27_1_readiness_report.md}"
V271_RELEASE_NOTES_PATH="${NTPRO_V28_INTAKE_V271_NOTES:-docs/rust-cutover/release/v0_27_1_release_notes.md}"
V271_CLOSEOUT_PATH="${NTPRO_V28_INTAKE_V271_CLOSEOUT:-docs/rust-cutover/release/v0_27_1_release_closeout_evidence.md}"
V280_INTAKE_PATH="${NTPRO_V28_INTAKE_REPORT:-docs/rust-cutover/release/v0_28_0_intake_gate.md}"
V271_MILESTONE_NUMBER="${NTPRO_V28_INTAKE_V271_MILESTONE_NUMBER:-21}"
V271_MILESTONE_TITLE="${NTPRO_V28_INTAKE_V271_MILESTONE_TITLE:-v0.27.1}"
V280_MILESTONE_TITLE="${NTPRO_V28_INTAKE_V280_MILESTONE_TITLE:-v0.28.0}"

fail() {
  echo "v28 intake gate failed: $*" >&2
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

gh_auth_status_with_retry() {
  local attempt=1
  local max_attempts=4
  while true; do
    if gh auth status >/dev/null 2>&1; then
      return 0
    fi
    if (( attempt >= max_attempts )); then
      return 1
    fi
    sleep "$((attempt * 2))"
    attempt=$((attempt + 1))
  done
}

git_ls_remote_with_retry() {
  local attempt=1
  local max_attempts=4
  while true; do
    if git ls-remote "$@"; then
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
  "$V271_MANIFEST_PATH" \
  "$V271_READINESS_PATH" \
  "$V271_RELEASE_NOTES_PATH" \
  "$V271_CLOSEOUT_PATH" \
  "$V280_INTAKE_PATH" \
  scripts/ai/verify_v27_1_release_gates.sh \
  scripts/ai/verify_v27_1_strict_provenance.sh; do
  require_file "$path"
done

for task_id in V271-001 V271-002 V271-003 V271-004 V271-005 V271-006; do
  require_file "docs/rust-cutover/evidence/${task_id}.md"
  require_file "docs/rust-cutover/tasks/${task_id}.md"
done

V271_RELEASE_VERSION="$V271_RELEASE_VERSION" \
V271_RELEASE_TAG="$V271_RELEASE_TAG" \
V271_RELEASE_NAME="$V271_RELEASE_NAME" \
V271_RELEASE_URL="$V271_RELEASE_URL" \
V271_MANIFEST_PATH="$V271_MANIFEST_PATH" \
V271_READINESS_PATH="$V271_READINESS_PATH" \
V271_RELEASE_NOTES_PATH="$V271_RELEASE_NOTES_PATH" \
V271_CLOSEOUT_PATH="$V271_CLOSEOUT_PATH" \
V280_INTAKE_PATH="$V280_INTAKE_PATH" \
python3 <<'PY'
import json
import os
from pathlib import Path

manifest = json.loads(Path(os.environ["V271_MANIFEST_PATH"]).read_text(encoding="utf-8"))
readiness = Path(os.environ["V271_READINESS_PATH"]).read_text(encoding="utf-8")
notes = Path(os.environ["V271_RELEASE_NOTES_PATH"]).read_text(encoding="utf-8")
closeout_evidence = Path(os.environ["V271_CLOSEOUT_PATH"]).read_text(encoding="utf-8")
intake = Path(os.environ["V280_INTAKE_PATH"]).read_text(encoding="utf-8")
EXPECTED_V271 = {
    "V271-001": 887,
    "V271-002": 888,
    "V271-003": 889,
    "V271-004": 890,
    "V271-005": 891,
    "V271-006": 892,
}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


require(manifest.get("schema_version") == "ntpro.v271_patch_release_manifest.v1", "manifest schema mismatch")
require(manifest.get("task_id") == "V271-006", "manifest task mismatch")
require(manifest.get("product_version") == os.environ["V271_RELEASE_VERSION"], "manifest product version mismatch")
require(manifest.get("release_status") == "released", "manifest release status must be released")
planned = manifest.get("planned_release") or {}
require(planned.get("tag") == os.environ["V271_RELEASE_TAG"], "planned release tag mismatch")
require(planned.get("name") == os.environ["V271_RELEASE_NAME"], "planned release name mismatch")
require(planned.get("github_release_url") == os.environ["V271_RELEASE_URL"], "planned release URL mismatch")
release_inputs = manifest.get("release_inputs") or {}
require(release_inputs.get("release_closeout_evidence_path") == os.environ["V271_CLOSEOUT_PATH"], "closeout evidence input missing")
published = manifest.get("published_release") or {}
require(published.get("tag") == os.environ["V271_RELEASE_TAG"], "published release tag mismatch")
require(published.get("tag_sha") == "0fdc11dc983bbfb9fe124a3f171a58fb1e7ccf19", "published release tag SHA mismatch")
require(published.get("release_body_matches_tracked_release_notes") is True, "published release body/source match missing")
evidence = manifest.get("v271_evidence") or []
require(len(evidence) == 6, "V271 evidence count mismatch")
for item in evidence:
    task_id = item.get("task_id")
    require(EXPECTED_V271.get(task_id) == item.get("issue"), f"V271 issue mismatch: {task_id}")
    require(Path(item.get("path", "")).is_file(), f"missing V271 evidence: {item}")
scope = manifest.get("release_scope") or {}
require(scope.get("final_release_scope_issue_count") == 6, "V271 final issue count mismatch")
require(scope.get("final_release_scope_evidence_count") == 6, "V271 final evidence count mismatch")
require(scope.get("exact_milestone_issue_numbers") == [887, 888, 889, 890, 891, 892], "V271 exact issue numbers mismatch")
require(scope.get("exact_milestone_issue_set") == "#887-#892", "V271 exact issue set mismatch")
require(scope.get("registered_corrective_scope_exception_count") == 0, "V271 corrective exception count mismatch")
require(scope.get("unregistered_corrective_milestone_issues_fail_closed") is True, "V271 unregistered corrective fail-closed rule missing")
requirements = manifest.get("post_publication_requirements") or {}
require(requirements.get("all_v271_issues_closed_required") is True, "V271 closeout requirement missing")
require(requirements.get("exact_milestone_issue_set_required") is True, "V271 exact issue-set requirement missing")
require(requirements.get("github_release_published_required") is True, "GitHub release requirement missing")
require(requirements.get("hosted_release_gate_success_required") is True, "hosted gate requirement missing")
require(requirements.get("strict_release_body_match_required") is True, "strict release body requirement missing")
require(requirements.get("publication_after_hosted_gate_required") is True, "publication ordering requirement missing")
require(requirements.get("source_controlled_closeout_evidence_required") is True, "source closeout requirement missing")
require(requirements.get("v0_28_start_gate_fails_without_v271_release_evidence") is True, "v28 hard-block requirement missing")
closeout = manifest.get("post_publication_closeout") or {}
require(closeout.get("source_controlled_closeout_evidence") is True, "source-controlled closeout proof missing")
require(closeout.get("source_controlled_closeout_evidence_path") == os.environ["V271_CLOSEOUT_PATH"], "source-controlled closeout path mismatch")
gate = closeout.get("hosted_release_gate") or {}
require(gate.get("run_id") == 28940442369, "closeout hosted gate run mismatch")
require(gate.get("jobs_success") == 82 and gate.get("jobs_failed") == 0, "closeout hosted gate jobs mismatch")
next_tracks = manifest.get("next_tracks") or {}
require(next_tracks.get("capability") == "v0.28.0", "next capability mismatch")
require(next_tracks.get("start_gate") == "blocked_until_v271_release_evidence_published", "next start gate mismatch")
require(next_tracks.get("implementation_started") is False, "v28 implementation must not start from v27.1")
for marker in (
    "V271 final release scope issue count = 6",
    "V271 final release scope evidence count = 6",
    "V271 exact milestone issue set = #887-#892",
    "v0.28.0 start gate = blocked until v0.27.1 release evidence is published",
):
    require(marker in readiness, f"missing v27.1 readiness marker: {marker}")
require("v28 intake gate = hard-blocked until v0.27.1 publication evidence exists" in notes, "v27.1 notes hard-block marker missing")
for marker in (
    "start_gate_status = satisfied",
    "V271 issues closed = 6/6",
    "V271 milestone closed issues = 6",
    "V271 exact milestone issue set = #887-#892",
    "v0.27.1 milestone = closed",
    "v0.27.1 milestone open issues = 0",
    "v0.27.1 release evidence = published",
    "v0.27.1 hosted release gate = success",
    "v0.27.1 hosted release gate jobs = 82/82 success",
    "v0.27.1 release tag = ntpro-rust-only-v0.27.1",
    "v0.27.1 release closeout evidence = docs/rust-cutover/release/v0_27_1_release_closeout_evidence.md",
    "v0.27.1 release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.27.1",
    "v0.27.1 hosted release gate URL = https://github.com/atxinbao/NTPRO/actions/runs/28940442369",
    "v0.27.1 tag SHA = 0fdc11dc983bbfb9fe124a3f171a58fb1e7ccf19",
    "v0.27.1 tag is ancestor of origin/main = true",
    "v0.27.1 GitHub Release published at = 2026-07-08T13:18:35Z",
    "v0.27.1 hosted release gate completed at = 2026-07-08T13:17:36Z",
    "v0.27.1 GitHub Release published after hosted gate = true",
    "v0.27.1 GitHub Release body sha256 = 74bbc4d42d8e6f70d93a63fa4b42ae684cba4ccf0ce7c06e60490d4ad3a0f3f0",
    "v0.28.0 capability track = backend_closure_product_operations_runtime_finalization_only",
    "v0.28.0 runtime capability inherited from v0.27.1 = false",
    "v0.28.0 trading controls inherited from v0.27.0 = false",
    "v0.28.0 trading controls inherited from v0.27.1 = false",
    "v0.28.0 milestone issue set = #893-#902",
):
    require(marker in intake, f"missing v28 intake marker: {marker}")
for marker in (
    "release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.27.1",
    "hosted release gate jobs = 82/82 success",
    "release body matches tracked release notes = true",
    "source-controlled closeout evidence = docs/rust-cutover/release/v0_27_1_release_closeout_evidence.md",
):
    require(marker in closeout_evidence, f"missing v27.1 closeout marker: {marker}")
for key in (
    "new_submit_capability",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "execution_adapter_call_allowed",
    "adapter_send_allowed",
    "live_exchange_request_allowed",
    "network_attempted",
    "retry_scheduler_enabled",
    "automatic_remediation_allowed",
    "automatic_operation_action_allowed",
    "dashboard_operation_controls_enabled",
    "dashboard_trading_controls_enabled",
    "admin_workbench_operation_controls_enabled",
    "admin_workbench_trading_controls_enabled",
    "trader_terminal_order_ticket_enabled",
    "manual_operation_submit_allowed",
    "product_grade_trading_terminal_claim",
):
    require((manifest.get("boundary_flags") or {}).get(key) is False, f"v27.1 boundary must remain false: {key}")
    require(f"{key} = false" in intake, f"missing v28 intake boundary marker: {key}")
PY

command -v gh >/dev/null 2>&1 || fail "gh is required for live intake proof"
gh_auth_status_with_retry || fail "gh authentication is required for live intake proof"

remote_tag_commit="$(git_ls_remote_with_retry --tags origin "refs/tags/$V271_RELEASE_TAG^{}" | awk '{print $1}')"
if [[ -z "$remote_tag_commit" ]]; then
  remote_tag_commit="$(git_ls_remote_with_retry --tags origin "refs/tags/$V271_RELEASE_TAG" | awk '{print $1}')"
fi
[[ -n "$remote_tag_commit" ]] || fail "missing remote release tag: $V271_RELEASE_TAG"

if git rev-parse -q --verify origin/main^{commit} >/dev/null; then
  origin_main_sha="$(git rev-parse origin/main)"
else
  git fetch --no-tags --depth=1 origin +refs/heads/main:refs/remotes/origin/main >/dev/null 2>&1 || true
  origin_main_sha="$(git rev-parse origin/main)"
fi
git merge-base --is-ancestor "$remote_tag_commit" "$origin_main_sha" || fail "v0.27.1 tag is not ancestor of origin/main: tag=$remote_tag_commit origin_main=$origin_main_sha"

release_json="$(gh_with_retry api "/repos/$REPO/releases/tags/$V271_RELEASE_TAG")" || fail "missing GitHub Release for $V271_RELEASE_TAG"
milestone_json="$(gh_with_retry api "/repos/$REPO/milestones/$V271_MILESTONE_NUMBER")" || fail "could not read v0.27.1 milestone"
milestone_issues_json="$(gh_with_retry issue list --repo "$REPO" --milestone "$V271_MILESTONE_TITLE" --state all --limit 100 --json number,state,title)" || fail "could not read v0.27.1 milestone issues"
v280_milestone_issues_json="$(gh_with_retry issue list --repo "$REPO" --milestone "$V280_MILESTONE_TITLE" --state all --limit 100 --json number,state,title)" || fail "could not read v0.28.0 milestone issues"
run_json="$(gh_with_retry run list --repo "$REPO" --workflow release-tag.yml --commit "$remote_tag_commit" --limit 20 --json databaseId,status,conclusion,headSha,url,updatedAt,workflowName)" || fail "could not read v0.27.1 hosted gate runs"

RELEASE_JSON="$release_json" \
MILESTONE_JSON="$milestone_json" \
MILESTONE_ISSUES_JSON="$milestone_issues_json" \
V280_MILESTONE_ISSUES_JSON="$v280_milestone_issues_json" \
RUN_JSON="$run_json" \
V271_RELEASE_TAG="$V271_RELEASE_TAG" \
V271_RELEASE_NAME="$V271_RELEASE_NAME" \
V271_RELEASE_URL="$V271_RELEASE_URL" \
V271_RELEASE_NOTES_PATH="$V271_RELEASE_NOTES_PATH" \
REMOTE_TAG_COMMIT="$remote_tag_commit" \
python3 <<'PY'
import hashlib
import json
import os
from datetime import datetime, timezone
from pathlib import Path

release = json.loads(os.environ["RELEASE_JSON"])
milestone = json.loads(os.environ["MILESTONE_JSON"])
milestone_issues = json.loads(os.environ["MILESTONE_ISSUES_JSON"])
v280_milestone_issues = json.loads(os.environ["V280_MILESTONE_ISSUES_JSON"])
runs = json.loads(os.environ["RUN_JSON"])
notes = Path(os.environ["V271_RELEASE_NOTES_PATH"]).read_text(encoding="utf-8")
expected_v271 = {887, 888, 889, 890, 891, 892}
expected_v280 = {893, 894, 895, 896, 897, 898, 899, 900, 901, 902}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def normalized(text: str) -> str:
    return "\n".join(line.rstrip() for line in text.splitlines()).strip()


def parse_time(value: str) -> datetime:
    parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)


require(release.get("tag_name") == os.environ["V271_RELEASE_TAG"], "release tag mismatch")
require(release.get("name") == os.environ["V271_RELEASE_NAME"], "release name mismatch")
require(release.get("html_url") == os.environ["V271_RELEASE_URL"], "release URL mismatch")
require(release.get("draft") is False, "release must not be draft")
require(release.get("prerelease") is False, "release must not be prerelease")
require(normalized(release.get("body") or "") == normalized(notes), "GitHub Release body does not strictly match v0.27.1 release notes")
body_sha = hashlib.sha256(normalized(release.get("body") or "").encode("utf-8")).hexdigest()
notes_sha = hashlib.sha256(normalized(notes).encode("utf-8")).hexdigest()
require(body_sha == notes_sha == "74bbc4d42d8e6f70d93a63fa4b42ae684cba4ccf0ce7c06e60490d4ad3a0f3f0", "release body hash mismatch")

require(milestone.get("title") == "v0.27.1", "milestone title mismatch")
require(milestone.get("state") == "closed", "v0.27.1 milestone must be closed")
require(milestone.get("open_issues") == 0, "v0.27.1 milestone open issue count must be 0")
require(milestone.get("closed_issues") == len(expected_v271), "v0.27.1 milestone closed issue count mismatch")
numbers = {issue.get("number") for issue in milestone_issues}
require(numbers == expected_v271, f"v0.27.1 milestone issue set mismatch: {sorted(numbers)}")
for issue in milestone_issues:
    require(issue.get("state") == "CLOSED", f"v0.27.1 milestone issue must be closed: #{issue.get('number')}")

v280_numbers = {issue.get("number") for issue in v280_milestone_issues}
require(v280_numbers == expected_v280, f"v0.28.0 milestone issue set mismatch: {sorted(v280_numbers)}")

matching = [
    run
    for run in runs
    if run.get("headSha") == os.environ["REMOTE_TAG_COMMIT"]
    and run.get("status") == "completed"
    and run.get("conclusion") == "success"
]
require(matching, "missing successful hosted release gate for v0.27.1 tag commit")
matching.sort(key=lambda run: run.get("updatedAt") or "", reverse=True)
gate = matching[0]
published_at = parse_time(release.get("published_at", ""))
gate_completed = parse_time(gate.get("updatedAt", ""))
require(published_at >= gate_completed, "v0.27.1 release must be published after hosted gate success")
print(f"gate_run={gate['databaseId']} gate_updated_at={gate['updatedAt']} body_sha256={body_sha}")
PY

for issue in 887 888 889 890 891 892; do
  state="$(gh_with_retry issue view "$issue" --repo "$REPO" --json state --jq .state)" || fail "could not read GitHub issue #$issue"
  [[ "$state" == "CLOSED" ]] || fail "GitHub issue #$issue must be CLOSED before v28 intake, got $state"
done

echo "v28_intake_gate=pass v271_release_tag=$V271_RELEASE_TAG tag_sha=$remote_tag_commit v271_issues=6/6_closed v271_milestone_issues=6/6_closed v280_milestone_issues=10 negative_selftest=${NTPRO_V28_INTAKE_SELFTEST:-1}"
