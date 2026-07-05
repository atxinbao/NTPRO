#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V25_INTAKE_REPO:-atxinbao/NTPRO}"
V241_RELEASE_VERSION="${NTPRO_V25_INTAKE_V241_RELEASE_VERSION:-v0.24.1}"
V241_RELEASE_TAG="${NTPRO_V25_INTAKE_V241_RELEASE_TAG:-ntpro-rust-only-v0.24.1}"
V241_RELEASE_NAME="${NTPRO_V25_INTAKE_V241_RELEASE_NAME:-NTPRO Rust-only v0.24.1}"
V241_RELEASE_URL="${NTPRO_V25_INTAKE_V241_RELEASE_URL:-https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.24.1}"
V241_TAG_SHA="${NTPRO_V25_INTAKE_V241_TAG_SHA:-fa5bb537a3002655efb3e5abc7e47fdf957bb298}"
V241_GATE_RUN_ID="${NTPRO_V25_INTAKE_V241_GATE_RUN_ID:-28747902599}"
V241_GATE_URL="${NTPRO_V25_INTAKE_V241_GATE_URL:-https://github.com/atxinbao/NTPRO/actions/runs/28747902599}"
V241_GATE_COMPLETED_AT="${NTPRO_V25_INTAKE_V241_GATE_COMPLETED_AT:-2026-07-05T18:09:15Z}"
V241_GATE_JOBS_TOTAL="${NTPRO_V25_INTAKE_V241_GATE_JOBS_TOTAL:-72}"
V241_GATE_JOBS_SUCCESS="${NTPRO_V25_INTAKE_V241_GATE_JOBS_SUCCESS:-72}"
V241_PUBLISHED_AT="${NTPRO_V25_INTAKE_V241_PUBLISHED_AT:-2026-07-05T18:11:48Z}"
V241_MILESTONE_NUMBER="${NTPRO_V25_INTAKE_V241_MILESTONE_NUMBER:-15}"

TASK_PATH="${NTPRO_V25_INTAKE_TASK_PATH:-docs/rust-cutover/tasks/V250-000.md}"
EVIDENCE_PATH="${NTPRO_V25_INTAKE_EVIDENCE_PATH:-docs/rust-cutover/evidence/V250-000.md}"
INTAKE_PATH="${NTPRO_V25_INTAKE_DOC_PATH:-docs/rust-cutover/release/v0_25_0_intake_gate.md}"
V241_MANIFEST_PATH="${NTPRO_V25_INTAKE_V241_MANIFEST:-docs/rust-cutover/release/v0_24_1_release_manifest.json}"
V241_READINESS_PATH="${NTPRO_V25_INTAKE_V241_READINESS:-docs/rust-cutover/release/v0_24_1_readiness_report.md}"
V241_SCHEMA_REPORT_PATH="${NTPRO_V25_INTAKE_V241_SCHEMA_REPORT:-docs/rust-cutover/release/v0_24_1_schema_replay_classification.md}"

fail() {
  echo "v25 intake gate failed: $*" >&2
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

run_json_path="$(mktemp "${TMPDIR:-/tmp}/ntpro-v25-intake-run.XXXXXX.json")"
trap 'rm -f "$run_json_path"' EXIT

for path in \
  "$TASK_PATH" \
  "$EVIDENCE_PATH" \
  "$INTAKE_PATH" \
  "$V241_MANIFEST_PATH" \
  "$V241_READINESS_PATH" \
  "$V241_SCHEMA_REPORT_PATH" \
  docs/rust-cutover/evidence/V241-001.md \
  docs/rust-cutover/evidence/V241-002.md \
  docs/rust-cutover/evidence/V241-003.md \
  docs/rust-cutover/evidence/V241-004.md \
  docs/rust-cutover/evidence/V241-005.md \
  docs/rust-cutover/evidence/V241-006.md \
  docs/rust-cutover/evidence/V241-007.md; do
  require_file "$path"
done

for marker in \
  "Task: \`V250-000\` / GitHub issue \`#777\`" \
  "start_gate_status = satisfied" \
  "V241 issues closed = 7/7" \
  "v0.24.1 release evidence = published" \
  "v0.24.1 hosted release gate = success" \
  "v0.25.0 capability track = monitoring_incident_disaster_recovery_foundation_only" \
  "v0.25.0 runtime capability inherited from v0.24.1 = false" \
  "validator_executable_replay = 39" \
  "schema_only_scoped = 0" \
  "runtime_adapter_integration = false" \
  "new_submit_capability = false" \
  "production_order_mutation_allowed = false" \
  "execution_adapter_call_allowed = false" \
  "live_exchange_request_allowed = false" \
  "retry_scheduler_enabled = false" \
  "dashboard_operation_controls_enabled = false" \
  "product_grade_trading_terminal_claim = false" \
  "$V241_RELEASE_TAG" \
  "$V241_GATE_URL" \
  "$V241_RELEASE_URL" \
  "$V241_TAG_SHA"; do
  require_contains "$INTAKE_PATH" "$marker"
done

for marker in \
  "Status: LOCAL VALIDATION PASSED" \
  "Task: \`V250-000\` / GitHub issue \`#777\`" \
  "scripts/ai/verify_release.sh v25-intake-gate"; do
  require_contains "$TASK_PATH" "$marker"
  require_contains "$EVIDENCE_PATH" "$marker"
done

for marker in \
  "No V250 implementation starts until all V241 issues are closed and v0.24.1 release evidence is published" \
  "V241 issue set = 7/7 closed before publication" \
  "v0.25.0"; do
  require_contains "$V241_READINESS_PATH" "$marker"
done

V241_RELEASE_VERSION="$V241_RELEASE_VERSION" \
V241_RELEASE_TAG="$V241_RELEASE_TAG" \
V241_RELEASE_NAME="$V241_RELEASE_NAME" \
V241_RELEASE_URL="$V241_RELEASE_URL" \
V241_MANIFEST_PATH="$V241_MANIFEST_PATH" \
python3 <<'PY'
import copy
import json
import os
from pathlib import Path

manifest = json.loads(Path(os.environ["V241_MANIFEST_PATH"]).read_text(encoding="utf-8"))


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def require_false(mapping: dict, key: str) -> None:
    require(mapping.get(key) is False, f"boundary flag must be false: {key}")


def validate(candidate: dict) -> None:
    require(candidate.get("schema_version") == "ntpro.v241_patch_release_manifest.v1", "manifest schema mismatch")
    require(candidate.get("product_version") == os.environ["V241_RELEASE_VERSION"], "manifest product version mismatch")
    require(candidate.get("release_status") == "released", "manifest release status mismatch")

    release = candidate.get("planned_release") or {}
    require(release.get("tag") == os.environ["V241_RELEASE_TAG"], "planned release tag mismatch")
    require(release.get("name") == os.environ["V241_RELEASE_NAME"], "planned release name mismatch")
    require(release.get("github_release_url") == os.environ["V241_RELEASE_URL"], "planned release URL mismatch")
    require(release.get("target_commitish") == "main", "planned release target mismatch")
    require(release.get("draft") is False, "planned release must not be draft")
    require(release.get("prerelease") is False, "planned release must not be prerelease")

    expected_evidence = {
        "V241-001": 770,
        "V241-002": 771,
        "V241-003": 772,
        "V241-004": 773,
        "V241-005": 774,
        "V241-006": 775,
        "V241-007": 776,
    }
    evidence = candidate.get("v241_evidence") or []
    require(len(evidence) == len(expected_evidence), "V241 evidence count mismatch")
    for item in evidence:
        task_id = item.get("task_id")
        require(expected_evidence.get(task_id) == item.get("issue"), f"V241 evidence issue mismatch: {task_id}")
        path = Path(item.get("path", ""))
        require(path.is_file(), f"V241 evidence file missing: {path}")

    next_tracks = candidate.get("next_tracks") or {}
    require(next_tracks.get("capability") == "v0.25.0", "next capability mismatch")
    require(next_tracks.get("capability_entry") == "monitoring_incident_disaster_recovery_track", "next capability entry mismatch")
    require(next_tracks.get("start_gate") == "blocked_until_v241_release_evidence_published", "next start gate mismatch")
    require_false(next_tracks, "implementation_started")
    require_false(next_tracks, "inherits_production_submit")
    require_false(next_tracks, "inherits_production_mutation")
    require_false(next_tracks, "inherits_retry_scheduler")
    require_false(next_tracks, "inherits_dashboard_operation_controls")

    post_publication = candidate.get("post_publication_requirements") or {}
    require(post_publication.get("all_v241_issues_closed_required") is True, "V241 issue closeout requirement missing")
    require(post_publication.get("github_release_published_required") is True, "release publication requirement missing")
    require(post_publication.get("hosted_release_gate_success_required") is True, "hosted gate requirement missing")
    require(post_publication.get("strict_release_body_match_required") is True, "strict release body requirement missing")
    require(post_publication.get("v0_25_start_gate_fails_without_v241_release_evidence") is True, "V250 fail-closed requirement missing")

    boundary = candidate.get("boundary_flags") or {}
    for key in (
        "new_submit_capability",
        "production_order_submission_allowed",
        "production_order_mutation_allowed",
        "execution_adapter_call_allowed",
        "live_exchange_request_allowed",
        "retry_scheduler_enabled",
        "dashboard_operation_controls_enabled",
        "trader_terminal_order_ticket_enabled",
        "manual_operation_submit_allowed",
        "product_grade_trading_terminal_claim",
    ):
        require_false(boundary, key)


validate(manifest)

negative = copy.deepcopy(manifest)
negative.setdefault("next_tracks", {})["implementation_started"] = True
try:
    validate(negative)
except SystemExit:
    pass
else:
    raise SystemExit("negative selftest failed: implementation_started=true was accepted")
PY

scripts/ai/verify_release.sh v24.1-schema-replay-classification
NTPRO_V241_STRICT_REQUIRE_PUBLICATION=1 scripts/ai/verify_release.sh v24.1-strict-provenance

if ! command -v gh >/dev/null 2>&1; then
  fail "gh is required for live intake proof"
fi
gh auth status >/dev/null 2>&1 || fail "gh authentication is required for live intake proof"

for issue in 770 771 772 773 774 775 776; do
  issue_json="$(gh_with_retry issue view "$issue" --repo "$REPO" --json state,title,url)"
  state="$(json_field "$issue_json" state)"
  [[ "$state" == "CLOSED" ]] || fail "V241 dependency issue #$issue is not closed: $state"
done

milestone_json="$(gh_with_retry api "repos/$REPO/milestones/$V241_MILESTONE_NUMBER")"
MILESTONE_JSON="$milestone_json" python3 <<'PY'
import json
import os

milestone = json.loads(os.environ["MILESTONE_JSON"])
assert milestone["title"] == "v0.24.1", milestone
assert milestone["state"] == "closed", milestone
assert milestone["open_issues"] == 0, milestone
assert milestone["closed_issues"] == 7, milestone
PY

release_json="$(gh_with_retry release view "$V241_RELEASE_TAG" --repo "$REPO" --json tagName,name,isDraft,isPrerelease,url,publishedAt,targetCommitish)"
RELEASE_JSON="$release_json" \
V241_RELEASE_TAG="$V241_RELEASE_TAG" \
V241_RELEASE_NAME="$V241_RELEASE_NAME" \
V241_RELEASE_URL="$V241_RELEASE_URL" \
V241_PUBLISHED_AT="$V241_PUBLISHED_AT" \
python3 <<'PY'
import json
import os

release = json.loads(os.environ["RELEASE_JSON"])
assert release["tagName"] == os.environ["V241_RELEASE_TAG"], release
assert release["name"] == os.environ["V241_RELEASE_NAME"], release
assert release["url"] == os.environ["V241_RELEASE_URL"], release
assert release["publishedAt"] == os.environ["V241_PUBLISHED_AT"], release
assert release["isDraft"] is False, release
assert release["isPrerelease"] is False, release
assert release["targetCommitish"] == "main", release
PY

remote_tag_sha="$(git ls-remote --tags origin "refs/tags/$V241_RELEASE_TAG" | awk '{print $1}')"
[[ "$remote_tag_sha" == "$V241_TAG_SHA" ]] || fail "remote tag SHA mismatch: $remote_tag_sha"

origin_main_sha="$(git ls-remote origin refs/heads/main | awk '{print $1}')"
[[ "$origin_main_sha" == "$V241_TAG_SHA" ]] || fail "v0.24.1 tag must match origin/main: tag=$V241_TAG_SHA origin_main=$origin_main_sha"

gh_with_retry run view "$V241_GATE_RUN_ID" --repo "$REPO" --json status,conclusion,workflowName,headSha,url,updatedAt,jobs >"$run_json_path"
RUN_JSON_PATH="$run_json_path" \
V241_GATE_URL="$V241_GATE_URL" \
V241_GATE_COMPLETED_AT="$V241_GATE_COMPLETED_AT" \
V241_GATE_JOBS_TOTAL="$V241_GATE_JOBS_TOTAL" \
V241_GATE_JOBS_SUCCESS="$V241_GATE_JOBS_SUCCESS" \
V241_TAG_SHA="$V241_TAG_SHA" \
python3 <<'PY'
import json
import os
from pathlib import Path

run = json.loads(Path(os.environ["RUN_JSON_PATH"]).read_text(encoding="utf-8"))
assert run["status"] == "completed", run
assert run["conclusion"] == "success", run
assert run["workflowName"] == "Rust Cutover Release Gate", run
assert run["url"] == os.environ["V241_GATE_URL"], run
assert run["updatedAt"] == os.environ["V241_GATE_COMPLETED_AT"], run
assert run["headSha"] == os.environ["V241_TAG_SHA"], run
jobs = run.get("jobs") or []
assert len(jobs) == int(os.environ["V241_GATE_JOBS_TOTAL"]), len(jobs)
assert sum(1 for job in jobs if job.get("conclusion") == "success") == int(os.environ["V241_GATE_JOBS_SUCCESS"]), jobs
PY

timestamp_ge "$V241_PUBLISHED_AT" "$V241_GATE_COMPLETED_AT" || fail "v0.24.1 release was not published after hosted gate success"

echo "v25_intake_gate=pass"
echo "start_gate_status=satisfied"
echo "v241_issues_closed=7/7"
echo "v241_release_tag=$V241_RELEASE_TAG"
echo "v241_release_url=$V241_RELEASE_URL"
echo "v241_gate_run=$V241_GATE_URL"
echo "v241_gate_jobs=${V241_GATE_JOBS_SUCCESS}/${V241_GATE_JOBS_TOTAL}"
echo "v241_tag_sha=$V241_TAG_SHA"
echo "v241_tag_matches_origin_main=true"
echo "v25_capability_track=monitoring_incident_disaster_recovery_foundation_only"
echo "v25_runtime_capability_inherited=false"
echo "new_submit_capability=false"
echo "production_order_mutation_allowed=false"
echo "execution_adapter_call_allowed=false"
echo "live_exchange_request_allowed=false"
echo "retry_scheduler_enabled=false"
echo "dashboard_operation_controls_enabled=false"
echo "product_grade_trading_terminal_claim=false"
echo "negative_selftest=1"
