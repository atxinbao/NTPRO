#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V24_INTAKE_REPO:-atxinbao/NTPRO}"
V231_RELEASE_VERSION="${NTPRO_V24_INTAKE_V231_RELEASE_VERSION:-v0.23.1}"
V231_RELEASE_TAG="${NTPRO_V24_INTAKE_V231_RELEASE_TAG:-ntpro-rust-only-v0.23.1}"
V231_RELEASE_NAME="${NTPRO_V24_INTAKE_V231_RELEASE_NAME:-NTPRO Rust-only v0.23.1}"
V231_RELEASE_URL="${NTPRO_V24_INTAKE_V231_RELEASE_URL:-https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.23.1}"
V231_TAG_SHA="${NTPRO_V24_INTAKE_V231_TAG_SHA:-11133f216503d4d5b13485acb53787413799c8d0}"
V231_GATE_RUN_ID="${NTPRO_V24_INTAKE_V231_GATE_RUN_ID:-28713340051}"
V231_GATE_URL="${NTPRO_V24_INTAKE_V231_GATE_URL:-https://github.com/atxinbao/NTPRO/actions/runs/28713340051}"
V231_GATE_COMPLETED_AT="${NTPRO_V24_INTAKE_V231_GATE_COMPLETED_AT:-2026-07-04T18:31:43Z}"
V231_GATE_JOBS_TOTAL="${NTPRO_V24_INTAKE_V231_GATE_JOBS_TOTAL:-68}"
V231_GATE_JOBS_SUCCESS="${NTPRO_V24_INTAKE_V231_GATE_JOBS_SUCCESS:-68}"
V231_PUBLISHED_AT="${NTPRO_V24_INTAKE_V231_PUBLISHED_AT:-2026-07-04T18:35:51Z}"
V231_MILESTONE_NUMBER="${NTPRO_V24_INTAKE_V231_MILESTONE_NUMBER:-13}"

TASK_PATH="${NTPRO_V24_INTAKE_TASK_PATH:-docs/rust-cutover/tasks/V240-000.md}"
EVIDENCE_PATH="${NTPRO_V24_INTAKE_EVIDENCE_PATH:-docs/rust-cutover/evidence/V240-000.md}"
INTAKE_PATH="${NTPRO_V24_INTAKE_DOC_PATH:-docs/rust-cutover/release/v0_24_0_intake_gate.md}"
V231_MANIFEST_PATH="${NTPRO_V24_INTAKE_V231_MANIFEST:-docs/rust-cutover/release/v0_23_1_release_manifest.json}"

fail() {
  echo "v24 intake gate failed: $*" >&2
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

for path in \
  "$TASK_PATH" \
  "$EVIDENCE_PATH" \
  "$INTAKE_PATH" \
  "$V231_MANIFEST_PATH" \
  docs/rust-cutover/evidence/V231-001.md \
  docs/rust-cutover/evidence/V231-002.md \
  docs/rust-cutover/evidence/V231-003.md \
  docs/rust-cutover/evidence/V231-004.md \
  docs/rust-cutover/evidence/V231-005.md \
  docs/rust-cutover/evidence/V231-006.md; do
  require_file "$path"
done

for marker in \
  "Task: \`V240-000\` / GitHub issue \`#743\`" \
  "start_gate_status = satisfied" \
  "V231 issues closed = 6/6" \
  "v0.23.1 release evidence = published" \
  "v0.23.1 hosted release gate = success" \
  "v0.24.0 capability track = gated implementation only" \
  "v0.24.0 runtime capability inherited from v0.23.1 = false" \
  "new_submit_capability = false" \
  "production_order_mutation_allowed = false" \
  "dashboard_operation_controls_enabled = false" \
  "$V231_RELEASE_TAG" \
  "$V231_GATE_URL" \
  "$V231_RELEASE_URL" \
  "$V231_TAG_SHA"; do
  require_contains "$INTAKE_PATH" "$marker"
done

for marker in \
  "Status: LOCAL VALIDATION PASSED" \
  "Task: \`V240-000\` / GitHub issue \`#743\`" \
  "scripts/ai/verify_release.sh v24-intake-gate"; do
  require_contains "$TASK_PATH" "$marker"
  require_contains "$EVIDENCE_PATH" "$marker"
done

V231_RELEASE_VERSION="$V231_RELEASE_VERSION" \
V231_RELEASE_TAG="$V231_RELEASE_TAG" \
V231_RELEASE_NAME="$V231_RELEASE_NAME" \
V231_RELEASE_URL="$V231_RELEASE_URL" \
V231_MANIFEST_PATH="$V231_MANIFEST_PATH" \
python3 <<'PY'
import copy
import json
import os
from pathlib import Path

manifest = json.loads(Path(os.environ["V231_MANIFEST_PATH"]).read_text(encoding="utf-8"))


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def require_false(mapping: dict, key: str) -> None:
    require(mapping.get(key) is False, f"boundary flag must be false: {key}")


def validate(candidate: dict) -> None:
    require(candidate.get("schema_version") == "ntpro.v231_patch_release_manifest.v1", "manifest schema mismatch")
    require(candidate.get("product_version") == os.environ["V231_RELEASE_VERSION"], "manifest product version mismatch")
    require(candidate.get("release_status") == "published", "manifest release status mismatch")

    release = candidate.get("planned_release") or {}
    require(release.get("tag") == os.environ["V231_RELEASE_TAG"], "planned release tag mismatch")
    require(release.get("name") == os.environ["V231_RELEASE_NAME"], "planned release name mismatch")
    require(release.get("github_release_url") == os.environ["V231_RELEASE_URL"], "planned release URL mismatch")
    require(release.get("target_commitish") == "main", "planned release target mismatch")
    require(release.get("draft") is False, "planned release must not be draft")
    require(release.get("prerelease") is False, "planned release must not be prerelease")

    expected_evidence = {
        "V231-001": 737,
        "V231-002": 738,
        "V231-003": 739,
        "V231-004": 740,
        "V231-005": 741,
        "V231-006": 742,
    }
    evidence = candidate.get("v231_evidence") or []
    require(len(evidence) == len(expected_evidence), "V231 evidence count mismatch")
    for item in evidence:
        task_id = item.get("task_id")
        require(expected_evidence.get(task_id) == item.get("issue"), f"V231 evidence issue mismatch: {task_id}")
        path = Path(item.get("path", ""))
        require(path.is_file(), f"V231 evidence file missing: {path}")

    capability = candidate.get("capability") or {}
    require(capability.get("v0_24_start_gate_defined") is True, "v0.24 start gate must be defined")
    require_false(capability, "v0_24_implementation_started")
    require_false(capability, "product_grade_live_trading_terminal")
    require_false(capability, "new_submit_capability")
    require_false(capability, "production_order_mutation_expansion")
    require_false(capability, "dashboard_operation_controls")
    require_false(capability, "complete_executable_read_model_runtime")

    boundary = candidate.get("boundary_flags") or {}
    for key in (
        "new_submit_capability",
        "production_order_submission_allowed",
        "production_order_mutation_allowed",
        "ungated_submit_allowed",
        "ungated_cancel_allowed",
        "ungated_retry_allowed",
        "ungated_replace_allowed",
        "ungated_amend_allowed",
        "ungated_flatten_allowed",
        "automatic_cancel_allowed",
        "automatic_remediation_allowed",
        "automatic_operation_action_allowed",
        "strategy_driven_production_execution_allowed",
        "cross_account_implicit_operation_allowed",
        "cross_strategy_implicit_operation_allowed",
        "cross_venue_implicit_operation_allowed",
        "cross_node_implicit_operation_allowed",
        "shared_approval_consumption_allowed",
        "dashboard_operation_controls_enabled",
        "dashboard_order_controls_enabled",
        "dashboard_approval_controls_enabled",
        "dashboard_cancel_controls_enabled",
        "dashboard_retry_controls_enabled",
        "dashboard_submit_controls_enabled",
        "dashboard_replace_controls_enabled",
        "dashboard_amend_controls_enabled",
        "dashboard_flatten_controls_enabled",
        "trader_terminal_order_ticket_enabled",
        "trader_terminal_live_trading_claim",
        "manual_operation_entry_enabled",
        "manual_operation_submit_allowed",
        "manual_operation_cancel_allowed",
        "manual_operation_retry_allowed",
        "manual_operation_replace_allowed",
        "manual_operation_amend_allowed",
        "manual_operation_flatten_allowed",
        "product_grade_trading_terminal_claim",
    ):
        require_false(boundary, key)


validate(manifest)

negative = copy.deepcopy(manifest)
negative.setdefault("capability", {})["v0_24_implementation_started"] = True
try:
    validate(negative)
except SystemExit:
    pass
else:
    raise SystemExit("negative selftest failed: v0_24_implementation_started=true was accepted")
PY

if ! command -v gh >/dev/null 2>&1; then
  fail "gh is required for live intake proof"
fi
gh auth status >/dev/null 2>&1 || fail "gh authentication is required for live intake proof"

for issue in 737 738 739 740 741 742; do
  issue_json="$(gh_with_retry issue view "$issue" --repo "$REPO" --json state,title,url)"
  state="$(json_field "$issue_json" state)"
  [[ "$state" == "CLOSED" ]] || fail "V231 dependency issue #$issue is not closed: $state"
done

milestone_json="$(gh_with_retry api "repos/$REPO/milestones/$V231_MILESTONE_NUMBER")"
MILESTONE_JSON="$milestone_json" python3 <<'PY'
import json
import os

milestone = json.loads(os.environ["MILESTONE_JSON"])
assert milestone["title"] == "v0.23.1", milestone
assert milestone["state"] == "closed", milestone
assert milestone["open_issues"] == 0, milestone
assert milestone["closed_issues"] == 6, milestone
PY

release_json="$(gh_with_retry release view "$V231_RELEASE_TAG" --repo "$REPO" --json tagName,name,isDraft,isPrerelease,url,publishedAt,targetCommitish)"
RELEASE_JSON="$release_json" \
V231_RELEASE_TAG="$V231_RELEASE_TAG" \
V231_RELEASE_NAME="$V231_RELEASE_NAME" \
V231_RELEASE_URL="$V231_RELEASE_URL" \
V231_PUBLISHED_AT="$V231_PUBLISHED_AT" \
python3 <<'PY'
import json
import os

release = json.loads(os.environ["RELEASE_JSON"])
assert release["tagName"] == os.environ["V231_RELEASE_TAG"], release
assert release["name"] == os.environ["V231_RELEASE_NAME"], release
assert release["url"] == os.environ["V231_RELEASE_URL"], release
assert release["publishedAt"] == os.environ["V231_PUBLISHED_AT"], release
assert release["isDraft"] is False, release
assert release["isPrerelease"] is False, release
assert release["targetCommitish"] == "main", release
PY

remote_tag_sha="$(git ls-remote --tags origin "refs/tags/$V231_RELEASE_TAG" | awk '{print $1}')"
[[ "$remote_tag_sha" == "$V231_TAG_SHA" ]] || fail "remote tag SHA mismatch: $remote_tag_sha"

origin_main_sha="$(git ls-remote origin refs/heads/main | awk '{print $1}')"
[[ "$origin_main_sha" == "$V231_TAG_SHA" ]] || fail "origin/main SHA mismatch: $origin_main_sha"

run_json="$(gh_with_retry run view "$V231_GATE_RUN_ID" --repo "$REPO" --json status,conclusion,workflowName,headSha,url,updatedAt,jobs)"
RUN_JSON="$run_json" \
V231_GATE_URL="$V231_GATE_URL" \
V231_GATE_COMPLETED_AT="$V231_GATE_COMPLETED_AT" \
V231_GATE_JOBS_TOTAL="$V231_GATE_JOBS_TOTAL" \
V231_GATE_JOBS_SUCCESS="$V231_GATE_JOBS_SUCCESS" \
V231_TAG_SHA="$V231_TAG_SHA" \
python3 <<'PY'
import json
import os

run = json.loads(os.environ["RUN_JSON"])
assert run["status"] == "completed", run
assert run["conclusion"] == "success", run
assert run["workflowName"] == "Rust Cutover Release Gate", run
assert run["url"] == os.environ["V231_GATE_URL"], run
assert run["updatedAt"] == os.environ["V231_GATE_COMPLETED_AT"], run
assert run["headSha"] == os.environ["V231_TAG_SHA"], run
jobs = run.get("jobs") or []
assert len(jobs) == int(os.environ["V231_GATE_JOBS_TOTAL"]), len(jobs)
assert sum(1 for job in jobs if job.get("conclusion") == "success") == int(os.environ["V231_GATE_JOBS_SUCCESS"]), jobs
PY

timestamp_ge "$V231_PUBLISHED_AT" "$V231_GATE_COMPLETED_AT" || fail "v0.23.1 release was not published after hosted gate success"

echo "v24_intake_gate=pass"
echo "start_gate_status=satisfied"
echo "v231_issues_closed=6/6"
echo "v231_release_tag=$V231_RELEASE_TAG"
echo "v231_release_url=$V231_RELEASE_URL"
echo "v231_gate_run=$V231_GATE_URL"
echo "v231_gate_jobs=${V231_GATE_JOBS_SUCCESS}/${V231_GATE_JOBS_TOTAL}"
echo "v231_tag_sha=$V231_TAG_SHA"
echo "v24_runtime_capability_inherited=false"
echo "new_submit_capability=false"
echo "production_order_mutation_allowed=false"
echo "dashboard_operation_controls_enabled=false"
echo "negative_selftest=1"
