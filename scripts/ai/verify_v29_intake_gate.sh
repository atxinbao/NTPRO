#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V29_INTAKE_REPO:-atxinbao/NTPRO}"
V281_RELEASE_VERSION="${NTPRO_V29_INTAKE_V281_RELEASE_VERSION:-v0.28.1}"
V281_RELEASE_TAG="${NTPRO_V29_INTAKE_V281_RELEASE_TAG:-ntpro-rust-only-v0.28.1}"
V281_RELEASE_NAME="${NTPRO_V29_INTAKE_V281_RELEASE_NAME:-NTPRO Rust-only v0.28.1}"
V281_RELEASE_URL="${NTPRO_V29_INTAKE_V281_RELEASE_URL:-https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.28.1}"
V281_MANIFEST_PATH="${NTPRO_V29_INTAKE_V281_MANIFEST:-docs/rust-cutover/release/v0_28_1_release_manifest.json}"
V281_READINESS_PATH="${NTPRO_V29_INTAKE_V281_READINESS:-docs/rust-cutover/release/v0_28_1_readiness_report.md}"
V281_RELEASE_NOTES_PATH="${NTPRO_V29_INTAKE_V281_NOTES:-docs/rust-cutover/release/v0_28_1_release_notes.md}"
V281_CLOSEOUT_PATH="${NTPRO_V29_INTAKE_V281_CLOSEOUT:-docs/rust-cutover/release/v0_28_1_release_closeout_evidence.md}"
V290_INTAKE_PATH="${NTPRO_V29_INTAKE_REPORT:-docs/rust-cutover/release/v0_29_0_intake_gate.md}"
V281_MILESTONE_NUMBER="${NTPRO_V29_INTAKE_V281_MILESTONE_NUMBER:-23}"
V281_MILESTONE_TITLE="${NTPRO_V29_INTAKE_V281_MILESTONE_TITLE:-v0.28.1}"
V290_MILESTONE_TITLE="${NTPRO_V29_INTAKE_V290_MILESTONE_TITLE:-v0.29.0}"
V281_RELEASE_GATE_RUN_ID="${NTPRO_V29_INTAKE_V281_RELEASE_GATE_RUN_ID:-29044397184}"
ALLOW_UNPUBLISHED="${NTPRO_V29_INTAKE_ALLOW_UNPUBLISHED:-0}"

fail() {
  echo "v29 intake gate failed: $*" >&2
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
  "$V281_MANIFEST_PATH" \
  "$V281_READINESS_PATH" \
  "$V281_RELEASE_NOTES_PATH" \
  "$V281_CLOSEOUT_PATH" \
  "$V290_INTAKE_PATH" \
  scripts/ai/verify_v28_1_release_gates.sh \
  scripts/ai/verify_v28_1_strict_provenance.sh; do
  require_file "$path"
done

require_file "docs/rust-cutover/evidence/V290-000.md"
require_file "docs/rust-cutover/tasks/V290-000.md"

for task_id in V281-001 V281-002 V281-003 V281-004 V281-005 V281-006 V281-007 V281-008 V281-009 V281-010; do
  require_file "docs/rust-cutover/evidence/${task_id}.md"
  require_file "docs/rust-cutover/tasks/${task_id}.md"
done

V281_RELEASE_VERSION="$V281_RELEASE_VERSION" \
V281_RELEASE_TAG="$V281_RELEASE_TAG" \
V281_RELEASE_NAME="$V281_RELEASE_NAME" \
V281_RELEASE_URL="$V281_RELEASE_URL" \
V281_MANIFEST_PATH="$V281_MANIFEST_PATH" \
V281_READINESS_PATH="$V281_READINESS_PATH" \
V281_RELEASE_NOTES_PATH="$V281_RELEASE_NOTES_PATH" \
V281_CLOSEOUT_PATH="$V281_CLOSEOUT_PATH" \
V290_INTAKE_PATH="$V290_INTAKE_PATH" \
python3 <<'PY'
import json
import os
from pathlib import Path

manifest = json.loads(Path(os.environ["V281_MANIFEST_PATH"]).read_text(encoding="utf-8"))
readiness = Path(os.environ["V281_READINESS_PATH"]).read_text(encoding="utf-8")
notes = Path(os.environ["V281_RELEASE_NOTES_PATH"]).read_text(encoding="utf-8")
closeout = Path(os.environ["V281_CLOSEOUT_PATH"]).read_text(encoding="utf-8")
intake = Path(os.environ["V290_INTAKE_PATH"]).read_text(encoding="utf-8")
expected = {
    "V281-001": 919,
    "V281-002": 920,
    "V281-003": 921,
    "V281-004": 922,
    "V281-005": 923,
    "V281-006": 924,
    "V281-007": 925,
    "V281-008": 944,
    "V281-009": 946,
    "V281-010": 948,
}

def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)

require(manifest.get("schema_version") == "ntpro.v281_patch_release_manifest.v1", "manifest schema mismatch")
require(manifest.get("task_id") == "V281-010", "manifest task mismatch")
require(manifest.get("product_version") == os.environ["V281_RELEASE_VERSION"], "manifest product version mismatch")
planned = manifest.get("planned_release") or {}
require(planned.get("tag") == os.environ["V281_RELEASE_TAG"], "planned release tag mismatch")
require(planned.get("name") == os.environ["V281_RELEASE_NAME"], "planned release name mismatch")
require(planned.get("github_release_url") == os.environ["V281_RELEASE_URL"], "planned release URL mismatch")
evidence = manifest.get("v281_evidence") or []
require(len(evidence) == 10, "V281 evidence count mismatch")
for item in evidence:
    task_id = item.get("task_id")
    require(expected.get(task_id) == item.get("issue"), f"V281 issue mismatch: {task_id}")
    require(Path(item.get("path", "")).is_file(), f"missing V281 evidence: {item}")
scope = manifest.get("release_scope") or {}
require(scope.get("final_release_scope_issue_count") == 10, "V281 final issue count mismatch")
require(scope.get("final_release_scope_evidence_count") == 10, "V281 final evidence count mismatch")
require(scope.get("exact_milestone_issue_numbers") == list(expected.values()), "V281 exact issue numbers mismatch")
requirements = manifest.get("post_publication_requirements") or {}
require(requirements.get("github_release_published_required") is True, "GitHub release requirement missing")
require(requirements.get("hosted_release_gate_success_required") is True, "hosted gate requirement missing")
require(requirements.get("source_controlled_closeout_evidence_required") is True, "source closeout requirement missing")
require(requirements.get("generated_publication_evidence_sole_proof_allowed") is False, "generated-only proof must be false")
require(requirements.get("v0_29_start_gate_fails_without_v281_release_evidence") is True, "v29 hard-block requirement missing")
next_tracks = manifest.get("next_tracks") or {}
require(next_tracks.get("capability") == "v0.29.0", "next capability mismatch")
require(next_tracks.get("start_gate") == "blocked_until_v281_release_evidence_published", "next start gate mismatch")
for marker in (
    "v0.29.0 start gate = blocked until v0.28.1 release evidence is published",
    "source-controlled closeout evidence = docs/rust-cutover/release/v0_28_1_release_closeout_evidence.md",
):
    require(marker in readiness, f"missing readiness marker: {marker}")
require("v29 intake gate = hard-blocked until v0.28.1 publication evidence exists" in notes, "notes hard-block marker missing")
for marker in (
    "Status: CLOSEOUT EVIDENCE RECORDED",
    "publication status = published_after_gate",
    "published after hosted gate = true",
    "hosted release gate jobs = 86/86 success",
    "release body matches tracked release notes = true",
    "generated publication evidence sole proof allowed = false",
    "v0.29.0 intake requires this source-controlled closeout evidence = true",
):
    require(marker in closeout, f"missing v28.1 closeout marker: {marker}")
for marker in (
    "start_gate_status = satisfied",
    "V281 issues closed = 10/10",
    "V281 milestone closed issues = 10",
    "V281 exact milestone issue set = #919-#925, #944, #946, #948",
    "v0.28.1 milestone = closed",
    "v0.28.1 milestone open issues = 0",
    "v0.28.1 release evidence = published",
    "v0.28.1 release closeout evidence = docs/rust-cutover/release/v0_28_1_release_closeout_evidence.md",
    "v0.28.1 hosted release gate = success",
    "v0.28.1 hosted release gate jobs = 86/86 success",
    "v0.28.1 release tag = ntpro-rust-only-v0.28.1",
    "v0.28.1 release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.28.1",
    "v0.28.1 hosted release gate URL = https://github.com/atxinbao/NTPRO/actions/runs/29044397184",
    "v0.28.1 tag SHA = 8b42671d5095ad5f32bc7947002900019eeb8269",
    "v0.28.1 tag is ancestor of origin/main = true",
    "v0.28.1 GitHub Release published at = 2026-07-09T20:57:07Z",
    "v0.28.1 hosted release gate completed at = 2026-07-09T20:53:43Z",
    "v0.28.1 GitHub Release published after hosted gate = true",
    "v0.28.1 GitHub Release body normalized sha256 = 7817ff5c9d448f608cb7352cbe34d337ddad5c5538b1a2ec7298e5a6e846c3bf",
    "v0.29.0 milestone issue set = #926-#936",
    "V290 issue count = 11",
    "v0.29.0 capability track = backend_production_readiness_foundation_only",
    "v0.29.0 runtime capability inherited from v0.28.1 = false",
    "v0.29.0 trading controls inherited from v0.28.0 = false",
    "v0.29.0 trading controls inherited from v0.28.1 = false",
):
    require(marker in intake, f"missing v29 intake marker: {marker}")
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
    require((manifest.get("boundary_flags") or {}).get(key) is False, f"v28.1 boundary must remain false: {key}")
    require(f"{key} = false" in closeout, f"missing v28.1 closeout boundary marker: {key}")
    require(f"{key} = false" in intake, f"missing v29 intake boundary marker: {key}")
require("backend_go_live_claim = false" in closeout, "missing v28.1 backend go-live boundary")
require("backend_go_live_claim = false" in intake, "missing v29 backend go-live boundary")
PY

if [[ "$ALLOW_UNPUBLISHED" == "1" ]]; then
  echo "v29_intake_gate=blocked_until_v28_1_publication source_gate=ok release_tag=$V281_RELEASE_TAG"
  exit 0
fi

command -v gh >/dev/null 2>&1 || fail "gh is required for live v29 intake proof"
gh_with_retry auth status >/dev/null 2>&1 || fail "gh authentication is required for live v29 intake proof"

remote_tag_commit="$(git_ls_remote_with_retry --tags origin "refs/tags/$V281_RELEASE_TAG^{}" | awk '{print $1}')"
if [[ -z "$remote_tag_commit" ]]; then
  remote_tag_commit="$(git_ls_remote_with_retry --tags origin "refs/tags/$V281_RELEASE_TAG" | awk '{print $1}')"
fi
[[ -n "$remote_tag_commit" ]] || fail "missing remote release tag: $V281_RELEASE_TAG"

if git rev-parse -q --verify origin/main^{commit} >/dev/null; then
  origin_main_sha="$(git rev-parse origin/main)"
else
  git fetch --no-tags --depth=1 origin +refs/heads/main:refs/remotes/origin/main >/dev/null 2>&1 || true
  origin_main_sha="$(git rev-parse origin/main)"
fi
git merge-base --is-ancestor "$remote_tag_commit" "$origin_main_sha" || fail "v0.28.1 tag is not ancestor of origin/main: tag=$remote_tag_commit origin_main=$origin_main_sha"

release_json="$(gh_with_retry api "/repos/$REPO/releases/tags/$V281_RELEASE_TAG")" || fail "missing GitHub Release for $V281_RELEASE_TAG"
milestone_json="$(gh_with_retry api "/repos/$REPO/milestones/$V281_MILESTONE_NUMBER")" || fail "could not read v0.28.1 milestone"
milestone_issues_json="$(gh_with_retry issue list --repo "$REPO" --milestone "$V281_MILESTONE_TITLE" --state all --limit 100 --json number,state,title)" || fail "could not read v0.28.1 milestone issues"
v290_milestone_issues_json="$(gh_with_retry issue list --repo "$REPO" --milestone "$V290_MILESTONE_TITLE" --state all --limit 100 --json number,state,title)" || fail "could not read v0.29.0 milestone issues"
run_json="$(gh_with_retry run view "$V281_RELEASE_GATE_RUN_ID" --repo "$REPO" --json status,conclusion,headSha,url,updatedAt,workflowName,jobs)" || fail "could not read v0.28.1 hosted release gate run"

RELEASE_JSON="$release_json" \
MILESTONE_JSON="$milestone_json" \
MILESTONE_ISSUES_JSON="$milestone_issues_json" \
V290_MILESTONE_ISSUES_JSON="$v290_milestone_issues_json" \
RUN_JSON="$run_json" \
REMOTE_TAG_COMMIT="$remote_tag_commit" \
V281_RELEASE_TAG="$V281_RELEASE_TAG" \
V281_RELEASE_NAME="$V281_RELEASE_NAME" \
V281_RELEASE_URL="$V281_RELEASE_URL" \
V281_RELEASE_GATE_RUN_ID="$V281_RELEASE_GATE_RUN_ID" \
V281_RELEASE_NOTES_PATH="$V281_RELEASE_NOTES_PATH" \
python3 <<'PY'
import hashlib
import json
import os
from datetime import datetime, timezone
from pathlib import Path

release = json.loads(os.environ["RELEASE_JSON"])
milestone = json.loads(os.environ["MILESTONE_JSON"])
milestone_issues = json.loads(os.environ["MILESTONE_ISSUES_JSON"])
v290_issues = json.loads(os.environ["V290_MILESTONE_ISSUES_JSON"])
run = json.loads(os.environ["RUN_JSON"])

def parse_ts(value: str) -> datetime:
    parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)

if release.get("tag_name") != os.environ["V281_RELEASE_TAG"]:
    raise SystemExit("release tag mismatch")
if release.get("name") != os.environ["V281_RELEASE_NAME"]:
    raise SystemExit("release name mismatch")
if release.get("draft") is not False or release.get("prerelease") is not False:
    raise SystemExit("release must be public, non-draft, and non-prerelease")
if release.get("html_url") != os.environ["V281_RELEASE_URL"]:
    raise SystemExit("release URL mismatch")
if release.get("target_commitish") != "main":
    raise SystemExit("release target commitish mismatch")
body = release.get("body") or ""
notes = Path(os.environ["V281_RELEASE_NOTES_PATH"]).read_text(encoding="utf-8")
normalized_body = "\n".join(line.rstrip() for line in body.splitlines()).strip()
normalized_notes = "\n".join(line.rstrip() for line in notes.splitlines()).strip()
if hashlib.sha256(normalized_body.encode()).hexdigest() != hashlib.sha256(normalized_notes.encode()).hexdigest():
    raise SystemExit("release body normalized hash mismatch")
if hashlib.sha256(normalized_body.encode()).hexdigest() != "7817ff5c9d448f608cb7352cbe34d337ddad5c5538b1a2ec7298e5a6e846c3bf":
    raise SystemExit("release body normalized sha mismatch")

if milestone.get("title") != "v0.28.1" or milestone.get("state") != "closed":
    raise SystemExit("v0.28.1 milestone must be closed")
if milestone.get("open_issues") != 0 or milestone.get("closed_issues") != 10:
    raise SystemExit("v0.28.1 milestone closeout counts mismatch")
expected_v281 = set(range(919, 926)) | {944, 946, 948}
states = {int(item["number"]): item["state"] for item in milestone_issues}
if set(states) != expected_v281:
    raise SystemExit(f"V281 issue set mismatch: {sorted(states)}")
if any(state != "CLOSED" for state in states.values()):
    raise SystemExit(f"V281 issue not closed: {states}")
expected_v290 = set(range(926, 937))
v290_states = {int(item["number"]): item["state"] for item in v290_issues}
if set(v290_states) != expected_v290:
    raise SystemExit(f"V290 issue set mismatch: {sorted(v290_states)}")

if int(os.environ["V281_RELEASE_GATE_RUN_ID"]) != 29044397184:
    raise SystemExit("release gate run id mismatch")
if run.get("status") != "completed" or run.get("conclusion") != "success":
    raise SystemExit("hosted release gate must be completed/success")
if run.get("workflowName") != "Rust Cutover Release Gate":
    raise SystemExit("hosted release gate workflow mismatch")
if run.get("headSha") != os.environ["REMOTE_TAG_COMMIT"]:
    raise SystemExit("hosted release gate headSha must match tag commit")
jobs = run.get("jobs") or []
success = sum(1 for item in jobs if item.get("conclusion") == "success")
if len(jobs) != 86 or success != 86:
    raise SystemExit(f"hosted release gate jobs mismatch: {success}/{len(jobs)}")
published_at = parse_ts(release.get("published_at", ""))
gate_completed = parse_ts(run.get("updatedAt", ""))
if published_at < gate_completed:
    raise SystemExit("release published before hosted gate completed")

print(
    "v29_intake_gate=pass "
    f"release_tag={os.environ['V281_RELEASE_TAG']} "
    f"tag_sha={os.environ['REMOTE_TAG_COMMIT']} "
    "v281_issues=10/10_closed "
    "v281_milestone_issues=10/10_closed "
    f"v290_milestone_issues={len(v290_states)} "
    f"release_gate_jobs={success}/{len(jobs)} "
    "negative_selftest=1"
)
PY
