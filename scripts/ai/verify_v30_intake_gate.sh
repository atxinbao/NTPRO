#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V30_INTAKE_REPO:-atxinbao/NTPRO}"
V291_RELEASE_VERSION="${NTPRO_V30_INTAKE_V291_RELEASE_VERSION:-v0.29.1}"
V291_RELEASE_TAG="${NTPRO_V30_INTAKE_V291_RELEASE_TAG:-ntpro-rust-only-v0.29.1}"
V291_RELEASE_NAME="${NTPRO_V30_INTAKE_V291_RELEASE_NAME:-NTPRO Rust-only v0.29.1}"
V291_RELEASE_URL="${NTPRO_V30_INTAKE_V291_RELEASE_URL:-https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.29.1}"
V291_GATE_RUN_ID="${NTPRO_V30_INTAKE_V291_GATE_RUN_ID:-29130876713}"
V291_TAG_OBJECT_SHA="${NTPRO_V30_INTAKE_V291_TAG_OBJECT_SHA:-d3d398530835342dab4aafe355d1c842be0fdd47}"
V291_TAG_SHA="${NTPRO_V30_INTAKE_V291_TAG_SHA:-a831d802e4321f50ed6e10481aea35b15a74b01e}"
V291_BODY_NORMALIZED_SHA="${NTPRO_V30_INTAKE_V291_BODY_NORMALIZED_SHA:-611c6cfe89480054d5c3a4718215740701ee43536e3e92fa0ff458f7730b204b}"
V291_BODY_RAW_SHA="${NTPRO_V30_INTAKE_V291_BODY_RAW_SHA:-5d5b7c34ceb7bca1a389e8261d04cc7fd28cea0a9d1e48ffe609f449b22ef2d1}"
V291_RELEASE_NOTES="${NTPRO_V30_INTAKE_V291_NOTES:-docs/rust-cutover/release/v0_29_1_release_notes.md}"
V291_RELEASE_MANIFEST="${NTPRO_V30_INTAKE_V291_MANIFEST:-docs/rust-cutover/release/v0_29_1_release_manifest.json}"
V291_CLOSEOUT="${NTPRO_V30_INTAKE_V291_CLOSEOUT:-docs/rust-cutover/release/v0_29_1_release_closeout_evidence.md}"
V30_INTAKE_PATH="${NTPRO_V30_INTAKE_REPORT:-docs/rust-cutover/release/v0_30_0_intake_gate.md}"
V291_MILESTONE_NUMBER="${NTPRO_V30_INTAKE_V291_MILESTONE_NUMBER:-25}"
V291_MILESTONE_TITLE="${NTPRO_V30_INTAKE_V291_MILESTONE_TITLE:-v0.29.1}"
V300_MILESTONE_TITLE="${NTPRO_V30_INTAKE_V300_MILESTONE_TITLE:-v0.30.0}"
CURRENT_ISSUE="${NTPRO_V30_INTAKE_CURRENT_ISSUE:-969}"

fail() {
  echo "v30 intake gate failed: $*" >&2
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
  "$V291_RELEASE_NOTES" \
  "$V291_RELEASE_MANIFEST" \
  "$V291_CLOSEOUT" \
  "$V30_INTAKE_PATH" \
  docs/rust-cutover/release/v0_29_1_v30_start_gate.md \
  docs/rust-cutover/release/v0_29_1_v30_start_gate_requirements.json \
  docs/rust-cutover/release/README.md \
  docs/rust-cutover/tasks/V300-000.md \
  docs/rust-cutover/evidence/V300-000.md \
  .github/workflows/rust-cutover-smoke.yml \
  scripts/ai/check_github_release_published.sh \
  scripts/ai/check_release_surface_current.sh \
  scripts/ai/verify_v29_1_release_gates.sh \
  scripts/ai/verify_v29_1_strict_provenance.sh \
  scripts/ai/verify_v29_1_v30_start_gate.sh \
  scripts/ai/verify_v29_1_release_publish_after_gate_current_binding.sh \
  scripts/ai/verify_v30_intake_gate.sh; do
  require_file "$path"
done

for task_id in V291-001 V291-002 V291-003 V291-004 V291-005 V291-006; do
  require_file "docs/rust-cutover/evidence/${task_id}.md"
  require_file "docs/rust-cutover/tasks/${task_id}.md"
done

NTPRO_CURRENT_RELEASE_VERSION="$V291_RELEASE_VERSION" \
  NTPRO_CURRENT_RELEASE_TAG="$V291_RELEASE_TAG" \
  NTPRO_CURRENT_RELEASE_NAME="$V291_RELEASE_NAME" \
  scripts/ai/check_github_release_published.sh

scripts/ai/verify_v29_1_release_publish_after_gate_current_binding.sh >/dev/null
scripts/ai/verify_v29_1_v30_start_gate.sh >/dev/null

V291_RELEASE_MANIFEST="$V291_RELEASE_MANIFEST" \
V291_CLOSEOUT="$V291_CLOSEOUT" \
V30_INTAKE_PATH="$V30_INTAKE_PATH" \
V291_RELEASE_TAG="$V291_RELEASE_TAG" \
V291_RELEASE_NAME="$V291_RELEASE_NAME" \
V291_RELEASE_URL="$V291_RELEASE_URL" \
V291_GATE_RUN_ID="$V291_GATE_RUN_ID" \
V291_TAG_OBJECT_SHA="$V291_TAG_OBJECT_SHA" \
V291_TAG_SHA="$V291_TAG_SHA" \
V291_BODY_NORMALIZED_SHA="$V291_BODY_NORMALIZED_SHA" \
V291_BODY_RAW_SHA="$V291_BODY_RAW_SHA" \
python3 <<'PY'
import copy
import json
import os
from pathlib import Path

manifest = json.loads(Path(os.environ["V291_RELEASE_MANIFEST"]).read_text(encoding="utf-8"))
closeout = Path(os.environ["V291_CLOSEOUT"]).read_text(encoding="utf-8")
intake = Path(os.environ["V30_INTAKE_PATH"]).read_text(encoding="utf-8")
task = Path("docs/rust-cutover/tasks/V300-000.md").read_text(encoding="utf-8")
evidence = Path("docs/rust-cutover/evidence/V300-000.md").read_text(encoding="utf-8")
release_index = Path("docs/rust-cutover/release/README.md").read_text(encoding="utf-8")
workflow = Path(".github/workflows/rust-cutover-smoke.yml").read_text(encoding="utf-8")

false_flags = [
    "new_submit_capability",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "cancel_order_allowed",
    "replace_order_allowed",
    "amend_order_allowed",
    "flatten_position_allowed",
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
    "backend_go_live_claim",
    "product_grade_trading_terminal_claim",
]

def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)

def validate_manifest(candidate: dict) -> None:
    require(candidate.get("schema_version") == "ntpro.v291_patch_release_manifest.v1", "V291 manifest schema mismatch")
    require(candidate.get("task_id") == "V291-006", "V291 manifest task mismatch")
    require(candidate.get("product_version") == "v0.29.1", "V291 manifest product version mismatch")
    require(candidate.get("release_status") == "released", "V291 manifest must be released for v30 intake")
    published = candidate.get("published_release") or {}
    require(published.get("tag") == os.environ["V291_RELEASE_TAG"], "V291 published release tag mismatch")
    require(published.get("tag_sha") == os.environ["V291_TAG_SHA"], "V291 published tag SHA mismatch")
    post_pub = candidate.get("post_publication_closeout") or {}
    require(post_pub.get("status") == "source_controlled_closeout_recorded", "V291 post-publication closeout status mismatch")
    require(post_pub.get("release_gate_run_id") == int(os.environ["V291_GATE_RUN_ID"]), "V291 post-publication gate run mismatch")
    require(post_pub.get("published_after_hosted_gate") is True, "V291 published-after-gate marker missing")
    contract = candidate.get("authoritative_predecessor_closeout_contract") or {}
    require(contract.get("contract_id") == "v0_29_1_authoritative_closeout_contract", "V291 authoritative contract missing")
    require(contract.get("release_status") == "released", "V291 authoritative contract status mismatch")
    require(contract.get("v30_intake_consumes_contract") is True, "V291 authoritative contract not consumed by v30")
    require(contract.get("release_body_normalized_sha256") == os.environ["V291_BODY_NORMALIZED_SHA"], "V291 authoritative contract normalized hash mismatch")
    require(contract.get("release_body_raw_sha256") == os.environ["V291_BODY_RAW_SHA"], "V291 authoritative contract raw hash mismatch")
    scope = candidate.get("release_scope") or {}
    require(scope.get("exact_milestone_issue_numbers") == [963, 964, 965, 966, 967, 968], "V291 exact issue numbers mismatch")
    require(scope.get("final_release_scope_issue_count") == 6, "V291 issue count mismatch")
    require(scope.get("final_release_scope_evidence_count") == 6, "V291 evidence count mismatch")
    requirements = candidate.get("post_publication_requirements") or {}
    require(requirements.get("github_release_published_required") is True, "V291 GitHub Release requirement missing")
    require(requirements.get("hosted_release_gate_success_required") is True, "V291 hosted gate requirement missing")
    require(requirements.get("publication_after_hosted_gate_required") is True, "V291 publish-after-gate requirement missing")
    require(requirements.get("source_controlled_closeout_evidence_required") is True, "V291 closeout requirement missing")
    require(requirements.get("generated_publication_evidence_sole_proof_allowed") is False, "generated-only proof must be false")
    require(requirements.get("v0_30_start_gate_fails_without_v291_release_evidence") is True, "v30 missing-release blocker missing")
    next_tracks = candidate.get("next_tracks") or {}
    require(next_tracks.get("capability") == "v0.30.0", "next capability mismatch")
    require(next_tracks.get("implementation_started") is False, "v30 implementation must not have started in V291")
    require(next_tracks.get("inherits_backend_go_live_claim") is False, "backend go-live inheritance must be false")
    for key in false_flags:
        require((candidate.get("boundary_flags") or {}).get(key) is False, f"boundary must remain false: {key}")

validate_manifest(manifest)

for marker in [
    "Status: RELEASED",
    f"release tag = {os.environ['V291_RELEASE_TAG']}",
    f"release name = {os.environ['V291_RELEASE_NAME']}",
    f"release URL = {os.environ['V291_RELEASE_URL']}",
    f"tag object SHA = {os.environ['V291_TAG_OBJECT_SHA']}",
    f"peeled tag commit = {os.environ['V291_TAG_SHA']}",
    f"hosted release gate run = https://github.com/atxinbao/NTPRO/actions/runs/{os.environ['V291_GATE_RUN_ID']}",
    "hosted release gate conclusion = success",
    "hosted release gate jobs = 90/90 success",
    "published after hosted gate = true",
    f"release body normalized sha256 = {os.environ['V291_BODY_NORMALIZED_SHA']}",
    f"release body raw sha256 = {os.environ['V291_BODY_RAW_SHA']}",
    "v0.29.1 milestone = closed",
    "v0.29.1 milestone open issues = 0",
    "v0.30.0 start gate = ready",
    "authoritative predecessor closeout contract = v0_29_1_authoritative_closeout_contract",
]:
    require(marker in closeout, f"closeout missing marker: {marker}")

for marker in [
    "start_gate_status = satisfied",
    "V291 issues closed = 6/6",
    "V291 milestone = closed",
    "V291 exact milestone issue set = #963-#968",
    "v0.29.1 release evidence = published",
    "v0.29.1 authoritative predecessor closeout contract = v0_29_1_authoritative_closeout_contract",
    "v0.29.1 hosted release gate jobs = 90/90 success",
    f"v0.29.1 tag object SHA = {os.environ['V291_TAG_OBJECT_SHA']}",
    f"v0.29.1 tag SHA = {os.environ['V291_TAG_SHA']}",
    f"v0.29.1 GitHub Release body normalized sha256 = {os.environ['V291_BODY_NORMALIZED_SHA']}",
    "v29 publish-after-gate current binding points at v0.29.0 = true",
    "v29 publish-after-gate current binding points at v0.28.0 = false",
    "v0.30.0 milestone issue set = #969-#980",
    "V300 issue count = 12",
    "v0.30.0 capability track = backend_production_go_live_candidate_foundation_only",
    "v0.30.0 default production submit = false",
    "v0.30.0 default adapter send = false",
    "v0.30.0 default live exchange request = false",
    "v0.30.0 default automatic remediation = false",
    "product_grade_trading_terminal_claim = false",
]:
    require(marker in intake, f"intake missing marker: {marker}")

for marker, text, label in [
    ("GitHub issue: #969", task, "task"),
    ("Status: READY FOR PR", task, "task"),
    ("V300-000 records the live dependency proof", evidence, "evidence"),
    ("Status: PASS", evidence, "evidence"),
    ("v30_intake_gate=pass", evidence, "evidence"),
    ("v0_30_0_intake_gate.md", release_index, "release index"),
    ("V300-000", release_index, "release index"),
    ("verify_v30_*.sh", workflow, "rust cutover smoke workflow"),
    ("heavy Rust checks skipped for v30 release verification script change set", workflow, "rust cutover smoke workflow"),
]:
    require(marker in text, f"{label} missing marker: {marker}")

for key in false_flags:
    marker = f"{key} = false"
    require(marker in closeout, f"closeout boundary missing: {marker}")
    require(marker in intake, f"intake boundary missing: {marker}")

bad_manifest = copy.deepcopy(manifest)
bad_manifest["boundary_flags"]["adapter_send_allowed"] = True
try:
    validate_manifest(bad_manifest)
except SystemExit:
    pass
else:
    raise SystemExit("negative self-test unexpectedly allowed adapter_send_allowed")
PY

command -v gh >/dev/null 2>&1 || fail "gh is required for live v30 intake proof"
gh_with_retry auth status >/dev/null 2>&1 || fail "gh authentication is required for live v30 intake proof"

tag_object_sha="$(git_ls_remote_with_retry --tags origin "refs/tags/$V291_RELEASE_TAG" | awk '{print $1}')"
remote_tag_commit="$(git_ls_remote_with_retry --tags origin "refs/tags/$V291_RELEASE_TAG^{}" | awk '{print $1}')"
[[ "$tag_object_sha" == "$V291_TAG_OBJECT_SHA" ]] || fail "tag object mismatch: $tag_object_sha"
[[ "$remote_tag_commit" == "$V291_TAG_SHA" ]] || fail "tag commit mismatch: $remote_tag_commit"

if git rev-parse -q --verify origin/main^{commit} >/dev/null; then
  origin_main_sha="$(git rev-parse origin/main)"
else
  git fetch --no-tags --depth=1 origin +refs/heads/main:refs/remotes/origin/main >/dev/null 2>&1 || true
  origin_main_sha="$(git rev-parse origin/main)"
fi
git merge-base --is-ancestor "$remote_tag_commit" "$origin_main_sha" || fail "v0.29.1 tag is not ancestor of origin/main"

release_json="$(gh_with_retry release view "$V291_RELEASE_TAG" --repo "$REPO" --json tagName,name,isDraft,isPrerelease,publishedAt,url,targetCommitish,body)" || fail "missing GitHub Release for $V291_RELEASE_TAG"
milestone_json="$(gh_with_retry api "/repos/$REPO/milestones/$V291_MILESTONE_NUMBER")" || fail "could not read v0.29.1 milestone"
v291_issues_json="$(gh_with_retry issue list --repo "$REPO" --milestone "$V291_MILESTONE_TITLE" --state all --limit 100 --json number,state,title)" || fail "could not read V291 milestone issues"
v300_issues_json="$(gh_with_retry issue list --repo "$REPO" --milestone "$V300_MILESTONE_TITLE" --state all --limit 100 --json number,state,title)" || fail "could not read V300 milestone issues"
run_json="$(gh_with_retry run view "$V291_GATE_RUN_ID" --repo "$REPO" --json status,conclusion,headSha,url,createdAt,updatedAt,workflowName,jobs)" || fail "could not read v0.29.1 hosted gate run"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT
printf '%s' "$release_json" >"$tmp_dir/release.json"
printf '%s' "$milestone_json" >"$tmp_dir/milestone.json"
printf '%s' "$v291_issues_json" >"$tmp_dir/v291_issues.json"
printf '%s' "$v300_issues_json" >"$tmp_dir/v300_issues.json"
printf '%s' "$run_json" >"$tmp_dir/run.json"

RELEASE_JSON_PATH="$tmp_dir/release.json" \
MILESTONE_JSON_PATH="$tmp_dir/milestone.json" \
V291_ISSUES_JSON_PATH="$tmp_dir/v291_issues.json" \
V300_ISSUES_JSON_PATH="$tmp_dir/v300_issues.json" \
RUN_JSON_PATH="$tmp_dir/run.json" \
REMOTE_TAG_COMMIT="$remote_tag_commit" \
V291_RELEASE_TAG="$V291_RELEASE_TAG" \
V291_RELEASE_NAME="$V291_RELEASE_NAME" \
V291_RELEASE_URL="$V291_RELEASE_URL" \
V291_RELEASE_NOTES="$V291_RELEASE_NOTES" \
V291_GATE_RUN_ID="$V291_GATE_RUN_ID" \
V291_BODY_NORMALIZED_SHA="$V291_BODY_NORMALIZED_SHA" \
V291_BODY_RAW_SHA="$V291_BODY_RAW_SHA" \
CURRENT_ISSUE="$CURRENT_ISSUE" \
python3 <<'PY'
import hashlib
import json
import os
from datetime import datetime, timezone
from pathlib import Path

release = json.loads(Path(os.environ["RELEASE_JSON_PATH"]).read_text(encoding="utf-8"))
milestone = json.loads(Path(os.environ["MILESTONE_JSON_PATH"]).read_text(encoding="utf-8"))
v291_issues = json.loads(Path(os.environ["V291_ISSUES_JSON_PATH"]).read_text(encoding="utf-8"))
v300_issues = json.loads(Path(os.environ["V300_ISSUES_JSON_PATH"]).read_text(encoding="utf-8"))
run = json.loads(Path(os.environ["RUN_JSON_PATH"]).read_text(encoding="utf-8"))

def parse_ts(value: str) -> datetime:
    parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)

if release.get("tagName") != os.environ["V291_RELEASE_TAG"]:
    raise SystemExit("release tag mismatch")
if release.get("name") != os.environ["V291_RELEASE_NAME"]:
    raise SystemExit("release name mismatch")
if release.get("isDraft") is not False or release.get("isPrerelease") is not False:
    raise SystemExit("release must be public, non-draft, and non-prerelease")
if release.get("url") != os.environ["V291_RELEASE_URL"]:
    raise SystemExit("release URL mismatch")
if release.get("targetCommitish") != "main":
    raise SystemExit("release target commitish mismatch")
body = release.get("body") or ""
notes = Path(os.environ["V291_RELEASE_NOTES"]).read_text(encoding="utf-8")
normalized_body = "\n".join(line.rstrip() for line in body.splitlines()).strip()
normalized_notes = "\n".join(line.rstrip() for line in notes.splitlines()).strip()
if hashlib.sha256(body.encode()).hexdigest() != os.environ["V291_BODY_RAW_SHA"]:
    raise SystemExit("release body raw sha mismatch")
if hashlib.sha256(notes.encode()).hexdigest() != os.environ["V291_BODY_RAW_SHA"]:
    raise SystemExit("tracked notes raw sha mismatch")
if hashlib.sha256(normalized_body.encode()).hexdigest() != os.environ["V291_BODY_NORMALIZED_SHA"]:
    raise SystemExit("release body normalized sha mismatch")
if hashlib.sha256(normalized_notes.encode()).hexdigest() != os.environ["V291_BODY_NORMALIZED_SHA"]:
    raise SystemExit("tracked notes normalized sha mismatch")

if milestone.get("title") != "v0.29.1" or milestone.get("state") != "closed":
    raise SystemExit("v0.29.1 milestone must be closed")
if milestone.get("open_issues") != 0 or milestone.get("closed_issues") != 6:
    raise SystemExit("v0.29.1 milestone closeout counts mismatch")

expected_v291 = {963, 964, 965, 966, 967, 968}
states = {int(item["number"]): item["state"] for item in v291_issues}
if set(states) != expected_v291:
    raise SystemExit(f"V291 issue set mismatch: {sorted(states)}")
if any(state != "CLOSED" for state in states.values()):
    raise SystemExit(f"V291 issue not closed: {states}")

expected_v300 = set(range(969, 981))
v300_states = {int(item["number"]): item["state"] for item in v300_issues}
if set(v300_states) != expected_v300:
    raise SystemExit(f"V300 issue set mismatch: {sorted(v300_states)}")
current_issue = int(os.environ["CURRENT_ISSUE"])
if current_issue not in v300_states:
    raise SystemExit("current V300 issue missing from milestone")

if int(os.environ["V291_GATE_RUN_ID"]) != 29130876713:
    raise SystemExit("release gate run id mismatch")
if run.get("status") != "completed" or run.get("conclusion") != "success":
    raise SystemExit("hosted release gate must be completed/success")
if run.get("workflowName") != "Rust Cutover Release Gate":
    raise SystemExit("hosted release gate workflow mismatch")
if run.get("headSha") != os.environ["REMOTE_TAG_COMMIT"]:
    raise SystemExit("hosted release gate headSha must match tag commit")
jobs = run.get("jobs") or []
success = sum(1 for item in jobs if item.get("conclusion") == "success")
if len(jobs) != 90 or success != 90:
    raise SystemExit(f"hosted release gate jobs mismatch: {success}/{len(jobs)}")
published_at = parse_ts(release.get("publishedAt", ""))
gate_completed = parse_ts(run.get("updatedAt", ""))
if published_at < gate_completed:
    raise SystemExit("release published before hosted gate completed")

print(
    "v30_intake_gate=pass "
    f"release_tag={os.environ['V291_RELEASE_TAG']} "
    f"tag_sha={os.environ['REMOTE_TAG_COMMIT']} "
    "v291_issues=6/6_closed "
    "v291_milestone=closed "
    f"v300_milestone_issues={len(v300_states)} "
    f"current_issue_state={v300_states[current_issue]} "
    f"release_gate_jobs={success}/{len(jobs)} "
    f"release_body_normalized_sha256={os.environ['V291_BODY_NORMALIZED_SHA']} "
    "negative_selftest=1"
)
PY
