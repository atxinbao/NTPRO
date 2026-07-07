#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V26_INTAKE_REPO:-atxinbao/NTPRO}"
V251_RELEASE_VERSION="${NTPRO_V26_INTAKE_V251_RELEASE_VERSION:-v0.25.1}"
V251_RELEASE_TAG="${NTPRO_V26_INTAKE_V251_RELEASE_TAG:-ntpro-rust-only-v0.25.1}"
V251_RELEASE_NAME="${NTPRO_V26_INTAKE_V251_RELEASE_NAME:-NTPRO Rust-only v0.25.1}"
V251_RELEASE_URL="${NTPRO_V26_INTAKE_V251_RELEASE_URL:-https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.25.1}"
V251_TAG_SHA="${NTPRO_V26_INTAKE_V251_TAG_SHA:-a7f665ebd54ea542f3b7720c44a080a01b206eb8}"
V251_GATE_RUN_ID="${NTPRO_V26_INTAKE_V251_GATE_RUN_ID:-28803741873}"
V251_GATE_URL="${NTPRO_V26_INTAKE_V251_GATE_URL:-https://github.com/atxinbao/NTPRO/actions/runs/28803741873}"
V251_GATE_COMPLETED_AT="${NTPRO_V26_INTAKE_V251_GATE_COMPLETED_AT:-2026-07-06T17:18:35Z}"
V251_GATE_JOBS_TOTAL="${NTPRO_V26_INTAKE_V251_GATE_JOBS_TOTAL:-76}"
V251_GATE_JOBS_SUCCESS="${NTPRO_V26_INTAKE_V251_GATE_JOBS_SUCCESS:-76}"
V251_PUBLISHED_AT="${NTPRO_V26_INTAKE_V251_PUBLISHED_AT:-2026-07-06T17:20:06Z}"
V251_MILESTONE_NUMBER="${NTPRO_V26_INTAKE_V251_MILESTONE_NUMBER:-17}"

TASK_PATH="${NTPRO_V26_INTAKE_TASK_PATH:-docs/rust-cutover/tasks/V260-000.md}"
EVIDENCE_PATH="${NTPRO_V26_INTAKE_EVIDENCE_PATH:-docs/rust-cutover/evidence/V260-000.md}"
INTAKE_PATH="${NTPRO_V26_INTAKE_DOC_PATH:-docs/rust-cutover/release/v0_26_0_intake_gate.md}"
V251_MANIFEST_PATH="${NTPRO_V26_INTAKE_V251_MANIFEST:-docs/rust-cutover/release/v0_25_1_release_manifest.json}"
V251_READINESS_PATH="${NTPRO_V26_INTAKE_V251_READINESS:-docs/rust-cutover/release/v0_25_1_readiness_report.md}"
V251_RELEASE_NOTES_PATH="${NTPRO_V26_INTAKE_V251_NOTES:-docs/rust-cutover/release/v0_25_1_release_notes.md}"

fail() {
  echo "v26 intake gate failed: $*" >&2
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

json_field() {
  python3 - "$1" "$2" <<'PY'
import json
import sys

payload = json.loads(sys.argv[1])
value = payload.get(sys.argv[2])
if value is None:
    value = ""
print(value)
PY
}

timestamp_ge() {
  python3 - "$1" "$2" <<'PY'
from datetime import datetime, timezone
import sys


def parse(value: str) -> datetime:
    if not value:
        raise SystemExit(2)
    value = value.replace("Z", "+00:00")
    parsed = datetime.fromisoformat(value)
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)


published = parse(sys.argv[1])
gate_completed = parse(sys.argv[2])
raise SystemExit(0 if published >= gate_completed else 1)
PY
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

run_json_path="$(mktemp "${TMPDIR:-/tmp}/ntpro-v26-intake-run.XXXXXX.json")"
trap 'rm -f "$run_json_path"' EXIT

for path in \
  "$TASK_PATH" \
  "$EVIDENCE_PATH" \
  "$INTAKE_PATH" \
  "$V251_MANIFEST_PATH" \
  "$V251_READINESS_PATH" \
  "$V251_RELEASE_NOTES_PATH" \
  docs/rust-cutover/evidence/V251-001.md \
  docs/rust-cutover/evidence/V251-002.md \
  docs/rust-cutover/evidence/V251-003.md \
  docs/rust-cutover/evidence/V251-004.md \
  docs/rust-cutover/evidence/V251-005.md \
  docs/rust-cutover/evidence/V251-006.md; do
  require_file "$path"
done

for marker in \
  "Task: \`V260-000\` / GitHub issue \`#812\`" \
  "start_gate_status = satisfied" \
  "V251 issues closed = 6/6" \
  "v0.25.1 milestone = closed" \
  "v0.25.1 release evidence = published" \
  "v0.25.1 hosted release gate = success" \
  "v0.25.1 hosted release gate jobs = 76/76 success" \
  "v0.25.1 release tag = $V251_RELEASE_TAG" \
  "v0.25.1 release URL = $V251_RELEASE_URL" \
  "v0.25.1 hosted release gate URL = $V251_GATE_URL" \
  "v0.25.1 tag SHA = $V251_TAG_SHA" \
  "v0.25.1 GitHub Release published at = $V251_PUBLISHED_AT" \
  "v0.25.1 hosted release gate completed at = $V251_GATE_COMPLETED_AT" \
  "v0.26.0 capability track = product_hardening_foundation_only" \
  "v0.26.0 runtime capability inherited from v0.25.1 = false" \
  "new_submit_capability = false" \
  "production_order_mutation_allowed = false" \
  "execution_adapter_call_allowed = false" \
  "adapter_send_allowed = false" \
  "live_exchange_request_allowed = false" \
  "retry_scheduler_enabled = false" \
  "automatic_remediation_allowed = false" \
  "dashboard_operation_controls_enabled = false" \
  "dashboard_trading_controls_enabled = false" \
  "manual_operation_submit_allowed = false" \
  "product_grade_trading_terminal_claim = false"; do
  require_contains "$INTAKE_PATH" "$marker"
done

for marker in \
  "Status: LOCAL VALIDATION PASSED" \
  "Task: \`V260-000\` / GitHub issue \`#812\`" \
  "scripts/ai/verify_release.sh v26-intake-gate"; do
  require_contains "$TASK_PATH" "$marker"
  require_contains "$EVIDENCE_PATH" "$marker"
done

for path in "$TASK_PATH" "$EVIDENCE_PATH" "$INTAKE_PATH"; do
  for marker in \
    "new_submit_capability = true" \
    "production_order_submission_allowed = true" \
    "production_order_mutation_allowed = true" \
    "execution_adapter_call_allowed = true" \
    "adapter_send_allowed = true" \
    "live_exchange_request_allowed = true" \
    "retry_scheduler_enabled = true" \
    "automatic_remediation_allowed = true" \
    "dashboard_operation_controls_enabled = true" \
    "dashboard_trading_controls_enabled = true" \
    "trader_terminal_order_ticket_enabled = true" \
    "manual_operation_submit_allowed = true" \
    "product_grade_trading_terminal_claim = true"; do
    require_not_contains "$path" "$marker"
  done
done

V251_RELEASE_VERSION="$V251_RELEASE_VERSION" \
V251_RELEASE_TAG="$V251_RELEASE_TAG" \
V251_RELEASE_NAME="$V251_RELEASE_NAME" \
V251_RELEASE_URL="$V251_RELEASE_URL" \
V251_MANIFEST_PATH="$V251_MANIFEST_PATH" \
python3 <<'PY'
import copy
import json
import os
from pathlib import Path

manifest = json.loads(Path(os.environ["V251_MANIFEST_PATH"]).read_text(encoding="utf-8"))


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def require_false(mapping: dict, key: str) -> None:
    require(mapping.get(key) is False, f"boundary flag must be false: {key}")


def validate(candidate: dict) -> None:
    require(candidate.get("schema_version") == "ntpro.v251_patch_release_manifest.v1", "manifest schema mismatch")
    require(candidate.get("task_id") == "V251-006", "manifest task mismatch")
    require(candidate.get("product_version") == os.environ["V251_RELEASE_VERSION"], "manifest product version mismatch")
    require(candidate.get("release_status") == "released", "manifest release status mismatch")
    planned = candidate.get("planned_release") or {}
    require(planned.get("tag") == os.environ["V251_RELEASE_TAG"], "planned release tag mismatch")
    require(planned.get("name") == os.environ["V251_RELEASE_NAME"], "planned release name mismatch")
    require(planned.get("github_release_url") == os.environ["V251_RELEASE_URL"], "planned release URL mismatch")
    require(planned.get("target_commitish") == "main", "planned release target mismatch")
    require(planned.get("draft") is False, "planned release must not be draft")
    require(planned.get("prerelease") is False, "planned release must not be prerelease")

    expected_evidence = {
        "V251-001": 806,
        "V251-002": 807,
        "V251-003": 808,
        "V251-004": 809,
        "V251-005": 810,
        "V251-006": 811,
    }
    evidence = candidate.get("v251_evidence") or []
    require(len(evidence) == len(expected_evidence), "V251 evidence count mismatch")
    for item in evidence:
        task_id = item.get("task_id")
        require(expected_evidence.get(task_id) == item.get("issue"), f"V251 evidence issue mismatch: {task_id}")
        path = Path(item.get("path", ""))
        require(path.is_file(), f"V251 evidence file missing: {path}")

    gate_commands = {gate.get("command") for gate in candidate.get("release_gates", []) if gate.get("required") is True}
    for command in (
        "scripts/ai/verify_release.sh v25.1-release-gates",
        "scripts/ai/verify_release.sh v25.1-strict-provenance",
        "scripts/ai/verify_release.sh release-publication-guard",
        "scripts/ai/verify_release.sh release-publish-after-gate",
    ):
        require(command in gate_commands, f"missing required V251 gate command: {command}")

    publication = candidate.get("publication_governance") or {}
    require(publication.get("gate_before_publish") is True, "gate-before-publish requirement missing")
    require(publication.get("public_release_requires_successful_hosted_gate_for_same_tag_commit") is True, "hosted gate publication requirement missing")
    require(publication.get("release_gate_success_before_publication_required") is True, "gate-before-publication flag missing")
    require(publication.get("publication_evidence_strategy") == "source_tree_plus_github_remote", "publication strategy mismatch")

    post_publication = candidate.get("post_publication_requirements") or {}
    require(post_publication.get("release_tag") == os.environ["V251_RELEASE_TAG"], "post-publication tag mismatch")
    require(post_publication.get("milestone_number") == 17, "post-publication milestone mismatch")
    require(post_publication.get("all_v251_issues_closed_required") is True, "V251 closeout requirement missing")
    require(post_publication.get("github_release_published_required") is True, "release publication requirement missing")
    require(post_publication.get("hosted_release_gate_success_required") is True, "hosted gate requirement missing")
    require(post_publication.get("strict_release_body_match_required") is True, "strict body requirement missing")
    require(post_publication.get("v0_26_start_gate_fails_without_v251_release_evidence") is True, "V260 fail-closed requirement missing")

    next_tracks = candidate.get("next_tracks") or {}
    require(next_tracks.get("capability") == "v0.26.0", "next capability mismatch")
    require(next_tracks.get("capability_entry") == "product_hardening_boundary_track", "next capability entry mismatch")
    require(next_tracks.get("start_gate") == "blocked_until_v251_release_evidence_published", "next start gate mismatch")
    for key in (
        "implementation_started",
        "inherits_production_submit",
        "inherits_production_mutation",
        "inherits_adapter_send",
        "inherits_live_exchange_request",
        "inherits_retry_scheduler",
        "inherits_automatic_remediation",
        "inherits_dashboard_trading_controls",
    ):
        require_false(next_tracks, key)

    boundary = candidate.get("boundary_flags") or {}
    for key in (
        "new_submit_capability",
        "production_order_submission_allowed",
        "production_order_mutation_allowed",
        "execution_adapter_call_allowed",
        "adapter_send_allowed",
        "live_exchange_request_allowed",
        "retry_scheduler_enabled",
        "automatic_remediation_allowed",
        "dashboard_operation_controls_enabled",
        "dashboard_trading_controls_enabled",
        "trader_terminal_order_ticket_enabled",
        "manual_operation_submit_allowed",
        "product_grade_trading_terminal_claim",
    ):
        require_false(boundary, key)


validate(manifest)

negative = copy.deepcopy(manifest)
negative.setdefault("post_publication_requirements", {})["v0_26_start_gate_fails_without_v251_release_evidence"] = False
try:
    validate(negative)
except SystemExit:
    pass
else:
    raise SystemExit("negative selftest failed: V260 start gate accepted missing V251 release evidence")
PY

NTPRO_V251_RELEASE_REQUIRE_CLOSEOUT=1 \
NTPRO_V251_RELEASE_HISTORICAL_PREREQ=1 \
NTPRO_V251_RELEASE_SKIP_CURRENT_SURFACE_GUARD=1 \
  scripts/ai/verify_release.sh v25.1-release-gates
NTPRO_CURRENT_RELEASE_VERSION="$V251_RELEASE_VERSION" \
  NTPRO_CURRENT_RELEASE_TAG="$V251_RELEASE_TAG" \
  NTPRO_CURRENT_RELEASE_NAME="$V251_RELEASE_NAME" \
  NTPRO_CURRENT_RELEASE_URL="$V251_RELEASE_URL" \
  NTPRO_CURRENT_RELEASE_NOTES="$V251_RELEASE_NOTES_PATH" \
  NTPRO_RELEASE_PUBLICATION_STRICT_BODY=1 \
  scripts/ai/verify_release.sh release-publication-guard

if ! command -v gh >/dev/null 2>&1; then
  fail "gh is required for live intake proof"
fi
gh_auth_status_with_retry || fail "gh authentication is required for live intake proof"

for issue in 806 807 808 809 810 811; do
  issue_json="$(gh_with_retry issue view "$issue" --repo "$REPO" --json state,title,url)"
  state="$(json_field "$issue_json" state)"
  [[ "$state" == "CLOSED" ]] || fail "V251 dependency issue #$issue is not closed: $state"
done

milestone_json="$(gh_with_retry api "repos/$REPO/milestones/$V251_MILESTONE_NUMBER")"
MILESTONE_JSON="$milestone_json" python3 <<'PY'
import json
import os

milestone = json.loads(os.environ["MILESTONE_JSON"])
assert milestone["title"] == "v0.25.1", milestone
assert milestone["state"] == "closed", milestone
assert milestone["open_issues"] == 0, milestone
assert milestone["closed_issues"] == 6, milestone
PY

release_json="$(gh_with_retry release view "$V251_RELEASE_TAG" --repo "$REPO" --json tagName,name,isDraft,isPrerelease,url,body,publishedAt,targetCommitish)"
RELEASE_JSON="$release_json" \
V251_RELEASE_TAG="$V251_RELEASE_TAG" \
V251_RELEASE_NAME="$V251_RELEASE_NAME" \
V251_RELEASE_URL="$V251_RELEASE_URL" \
V251_PUBLISHED_AT="$V251_PUBLISHED_AT" \
V251_RELEASE_NOTES_PATH="$V251_RELEASE_NOTES_PATH" \
python3 <<'PY'
import json
import os
from pathlib import Path

release = json.loads(os.environ["RELEASE_JSON"])
assert release["tagName"] == os.environ["V251_RELEASE_TAG"], release
assert release["name"] == os.environ["V251_RELEASE_NAME"], release
assert release["url"] == os.environ["V251_RELEASE_URL"], release
assert release["publishedAt"] == os.environ["V251_PUBLISHED_AT"], release
assert release["isDraft"] is False, release
assert release["isPrerelease"] is False, release
assert release["targetCommitish"] == "main", release
notes = "\n".join(line.rstrip() for line in Path(os.environ["V251_RELEASE_NOTES_PATH"]).read_text(encoding="utf-8").splitlines()).strip()
body = "\n".join(line.rstrip() for line in release["body"].splitlines()).strip()
assert body == notes, "GitHub Release body does not strictly match v0.25.1 release notes"
PY

remote_tag_sha="$(git_ls_remote_with_retry --tags origin "refs/tags/$V251_RELEASE_TAG" | awk '{print $1}')"
[[ "$remote_tag_sha" == "$V251_TAG_SHA" ]] || fail "remote tag SHA mismatch: $remote_tag_sha"

origin_main_sha="$(git_ls_remote_with_retry origin refs/heads/main | awk '{print $1}')"
if ! git merge-base --is-ancestor "$V251_TAG_SHA" "$origin_main_sha"; then
  fail "v0.25.1 tag is not an ancestor of origin/main: tag=$V251_TAG_SHA origin_main=$origin_main_sha"
fi

gh_with_retry run view "$V251_GATE_RUN_ID" --repo "$REPO" --json status,conclusion,workflowName,headSha,url,updatedAt,jobs >"$run_json_path"
RUN_JSON_PATH="$run_json_path" \
V251_GATE_URL="$V251_GATE_URL" \
V251_GATE_COMPLETED_AT="$V251_GATE_COMPLETED_AT" \
V251_GATE_JOBS_TOTAL="$V251_GATE_JOBS_TOTAL" \
V251_GATE_JOBS_SUCCESS="$V251_GATE_JOBS_SUCCESS" \
V251_TAG_SHA="$V251_TAG_SHA" \
python3 <<'PY'
import json
import os
from pathlib import Path

run = json.loads(Path(os.environ["RUN_JSON_PATH"]).read_text(encoding="utf-8"))
assert run["status"] == "completed", run
assert run["conclusion"] == "success", run
assert run["workflowName"] == "Rust Cutover Release Gate", run
assert run["url"] == os.environ["V251_GATE_URL"], run
assert run["updatedAt"] == os.environ["V251_GATE_COMPLETED_AT"], run
assert run["headSha"] == os.environ["V251_TAG_SHA"], run
jobs = run.get("jobs") or []
assert len(jobs) == int(os.environ["V251_GATE_JOBS_TOTAL"]), len(jobs)
assert sum(1 for job in jobs if job.get("conclusion") == "success") == int(os.environ["V251_GATE_JOBS_SUCCESS"]), jobs
PY

timestamp_ge "$V251_PUBLISHED_AT" "$V251_GATE_COMPLETED_AT" || fail "v0.25.1 release was not published after hosted gate success"

echo "v26_intake_gate=pass"
echo "start_gate_status=satisfied"
echo "v251_issues_closed=6/6"
echo "v251_release_tag=$V251_RELEASE_TAG"
echo "v251_release_url=$V251_RELEASE_URL"
echo "v251_gate_run=$V251_GATE_URL"
echo "v251_gate_jobs=${V251_GATE_JOBS_SUCCESS}/${V251_GATE_JOBS_TOTAL}"
echo "v251_tag_sha=$V251_TAG_SHA"
echo "v251_tag_is_ancestor_of_origin_main=true"
echo "v26_capability_track=product_hardening_foundation_only"
echo "v26_runtime_capability_inherited=false"
echo "new_submit_capability=false"
echo "production_order_mutation_allowed=false"
echo "execution_adapter_call_allowed=false"
echo "adapter_send_allowed=false"
echo "live_exchange_request_allowed=false"
echo "retry_scheduler_enabled=false"
echo "automatic_remediation_allowed=false"
echo "dashboard_operation_controls_enabled=false"
echo "dashboard_trading_controls_enabled=false"
echo "product_grade_trading_terminal_claim=false"
echo "negative_selftest=1"
