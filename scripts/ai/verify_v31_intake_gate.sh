#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V31_INTAKE_REPO:-atxinbao/NTPRO}"
V301_RELEASE_VERSION="${NTPRO_V31_INTAKE_V301_RELEASE_VERSION:-v0.30.1}"
V301_RELEASE_TAG="${NTPRO_V31_INTAKE_V301_RELEASE_TAG:-ntpro-rust-only-v0.30.1}"
V301_RELEASE_NAME="${NTPRO_V31_INTAKE_V301_RELEASE_NAME:-NTPRO Rust-only v0.30.1}"
V301_GATE_RUN_ID="${NTPRO_V31_INTAKE_V301_GATE_RUN_ID:-29194173422}"
V301_TAG_OBJECT_SHA="${NTPRO_V31_INTAKE_V301_TAG_OBJECT_SHA:-17d2b48ed4df2b21f1a0b20bf739fd46f33659be}"
V301_TAG_SHA="${NTPRO_V31_INTAKE_V301_TAG_SHA:-5b66335a8f625062dbcdd4f7441cfacab57b5ede}"
V301_BODY_NORMALIZED_SHA="${NTPRO_V31_INTAKE_V301_BODY_NORMALIZED_SHA:-1a9a71278ca7716a681b17667f5f7ef9c174f9eebacae0683a3c5a91cc4de4f9}"
V301_BODY_RAW_SHA="${NTPRO_V31_INTAKE_V301_BODY_RAW_SHA:-112045169e1cc733db164a19ceafe94406fb2fe93154a488e053a5b58c96e982}"
V301_RELEASE_NOTES="${NTPRO_V31_INTAKE_V301_NOTES:-docs/rust-cutover/release/v0_30_1_release_notes.md}"
V301_RELEASE_MANIFEST="${NTPRO_V31_INTAKE_V301_MANIFEST:-docs/rust-cutover/release/v0_30_1_release_manifest.json}"
V301_CLOSEOUT="${NTPRO_V31_INTAKE_V301_CLOSEOUT:-docs/rust-cutover/release/v0_30_1_release_closeout_evidence.md}"
V31_INTAKE_MD="${NTPRO_V31_INTAKE_MD:-docs/rust-cutover/release/v0_31_0_intake_gate.md}"
V31_INTAKE_JSON="${NTPRO_V31_INTAKE_JSON:-docs/rust-cutover/release/v0_31_0_intake_gate.json}"
V301_MILESTONE_NUMBER="${NTPRO_V31_INTAKE_V301_MILESTONE_NUMBER:-27}"
V301_MILESTONE_TITLE="${NTPRO_V31_INTAKE_V301_MILESTONE_TITLE:-v0.30.1}"
V310_MILESTONE_TITLE="${NTPRO_V31_INTAKE_V310_MILESTONE_TITLE:-v0.31.0}"
CURRENT_ISSUE="${NTPRO_V31_INTAKE_CURRENT_ISSUE:-1006}"

fail() {
  echo "v31 intake gate failed: $*" >&2
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

for path in \
  "$V301_RELEASE_NOTES" \
  "$V301_RELEASE_MANIFEST" \
  "$V301_CLOSEOUT" \
  "$V31_INTAKE_MD" \
  "$V31_INTAKE_JSON" \
  docs/rust-cutover/release/v0_30_1_readiness_report.md \
  docs/rust-cutover/release/v0_30_1_v31_start_gate.md \
  docs/rust-cutover/release/v0_30_1_v31_start_gate.json \
  docs/rust-cutover/release/README.md \
  docs/rust-cutover/tasks/V310-000.md \
  docs/rust-cutover/evidence/V310-000.md \
  scripts/ai/check_github_release_published.sh \
  scripts/ai/verify_v30_1_v31_start_gate.sh \
  scripts/ai/verify_v31_intake_gate.sh; do
  require_file "$path"
done

for task_id in V301-001 V301-002 V301-003 V301-004 V301-005 V301-006 V301-007; do
  require_file "docs/rust-cutover/evidence/${task_id}.md"
  require_file "docs/rust-cutover/tasks/${task_id}.md"
done

NTPRO_CURRENT_RELEASE_VERSION="$V301_RELEASE_VERSION" \
  NTPRO_CURRENT_RELEASE_TAG="$V301_RELEASE_TAG" \
  NTPRO_CURRENT_RELEASE_NAME="$V301_RELEASE_NAME" \
  scripts/ai/check_github_release_published.sh >/dev/null

scripts/ai/verify_v30_1_v31_start_gate.sh source >/dev/null

V301_RELEASE_MANIFEST="$V301_RELEASE_MANIFEST" \
V301_CLOSEOUT="$V301_CLOSEOUT" \
V31_INTAKE_MD="$V31_INTAKE_MD" \
V31_INTAKE_JSON="$V31_INTAKE_JSON" \
V301_RELEASE_TAG="$V301_RELEASE_TAG" \
V301_RELEASE_NAME="$V301_RELEASE_NAME" \
V301_GATE_RUN_ID="$V301_GATE_RUN_ID" \
V301_TAG_OBJECT_SHA="$V301_TAG_OBJECT_SHA" \
V301_TAG_SHA="$V301_TAG_SHA" \
V301_BODY_NORMALIZED_SHA="$V301_BODY_NORMALIZED_SHA" \
V301_BODY_RAW_SHA="$V301_BODY_RAW_SHA" \
python3 <<'PY'
import copy
import json
import os
from pathlib import Path

false_flags = [
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


def require_contains(text: str, marker: str, label: str) -> None:
    require(marker in text, f"{label} missing marker: {marker}")


def classify(case: dict, boundary: dict) -> str:
    scenario_boundary = copy.deepcopy(boundary)
    scenario_boundary.update(case.get("boundary_flags_override") or {})
    if case.get("v30_1_release_present") is not True:
        return "fail_closed_missing_v30_1_release_evidence"
    if case.get("v30_1_milestone_closed") is not True:
        return "fail_closed_missing_v30_1_milestone_closeout"
    if any(value is not False for value in scenario_boundary.values()):
        return "fail_closed_inherited_execution"
    return "dependency_proof_satisfied_scoped_intake_only"


manifest = json.loads(Path(os.environ["V301_RELEASE_MANIFEST"]).read_text(encoding="utf-8"))
closeout = Path(os.environ["V301_CLOSEOUT"]).read_text(encoding="utf-8")
intake_md = Path(os.environ["V31_INTAKE_MD"]).read_text(encoding="utf-8")
intake_json = json.loads(Path(os.environ["V31_INTAKE_JSON"]).read_text(encoding="utf-8"))
notes = Path("docs/rust-cutover/release/v0_30_1_release_notes.md").read_text(encoding="utf-8")
readiness = Path("docs/rust-cutover/release/v0_30_1_readiness_report.md").read_text(encoding="utf-8")
task = Path("docs/rust-cutover/tasks/V310-000.md").read_text(encoding="utf-8")
evidence = Path("docs/rust-cutover/evidence/V310-000.md").read_text(encoding="utf-8")
release_index = Path("docs/rust-cutover/release/README.md").read_text(encoding="utf-8")

require(manifest.get("release_status") == "released", "v30.1 manifest must be released")
published = manifest.get("published_release") or {}
require(published.get("tag") == os.environ["V301_RELEASE_TAG"], "published tag mismatch")
require(published.get("tag_sha") == os.environ["V301_TAG_SHA"], "published tag SHA mismatch")
require(published.get("tag_object_sha") == os.environ["V301_TAG_OBJECT_SHA"], "published tag object mismatch")
require(published.get("release_gate_run_id") == int(os.environ["V301_GATE_RUN_ID"]), "published gate run mismatch")
require(published.get("release_gate_jobs_success") == 94, "hosted gate success count mismatch")
require(published.get("release_gate_jobs_total") == 94, "hosted gate total count mismatch")
require(published.get("published_after_hosted_gate") is True, "published-after-gate missing")
require(published.get("release_body_normalized_sha256") == os.environ["V301_BODY_NORMALIZED_SHA"], "normalized hash mismatch")
require(published.get("release_body_raw_sha256") == os.environ["V301_BODY_RAW_SHA"], "raw hash mismatch")
post_pub = manifest.get("post_publication_closeout") or {}
require(post_pub.get("status") == "source_controlled_closeout_recorded", "post-publication closeout status mismatch")
require(post_pub.get("milestone_state") == "closed", "v30.1 milestone must be closed")
require(post_pub.get("v31_intake_consumes_closeout") is True, "v31 intake consumption missing")

for marker in [
    "Status: RELEASED",
    "published release status = published_after_gate",
    "hosted release gate run = 29194173422",
    "hosted release gate jobs = 94/94 success",
    "published after hosted gate = true",
    "v0.30.1 milestone = closed",
    "v0.30.1 milestone open issues = 0",
]:
    require_contains(notes, marker, "release notes")
    require_contains(readiness, marker, "readiness")

require_contains(notes, "release body hash semantics = normalized_sha256", "release notes")
require_contains(readiness, f"release body normalized sha256 = {os.environ['V301_BODY_NORMALIZED_SHA']}", "readiness")

for marker in [
    "Status: CLOSEOUT EVIDENCE RECORDED",
    f"annotated tag object = {os.environ['V301_TAG_OBJECT_SHA']}",
    f"annotated tag peeled commit = {os.environ['V301_TAG_SHA']}",
    "hosted release gate jobs = 94/94 success",
    f"release body normalized sha256 = {os.environ['V301_BODY_NORMALIZED_SHA']}",
    "v0.30.1 milestone state = closed",
    "v0.31.0 intake gate = dependency proof may be recorded, scoped approval still required",
]:
    require_contains(closeout, marker, "closeout")

require(intake_json.get("schema_version") == "ntpro.v310_intake_gate.v1", "intake schema mismatch")
require(intake_json.get("task_id") == "V310-000", "intake task mismatch")
require(intake_json.get("github_issue") == 1006, "intake issue mismatch")
require(intake_json.get("intake_status") == "dependency_proof_satisfied_scoped_intake_only", "intake status mismatch")
predecessor = intake_json.get("predecessor_release") or {}
require(predecessor.get("tag") == os.environ["V301_RELEASE_TAG"], "predecessor release tag mismatch")
require(predecessor.get("release_gate_run_id") == int(os.environ["V301_GATE_RUN_ID"]), "predecessor release gate mismatch")
require(predecessor.get("release_body_normalized_sha256") == os.environ["V301_BODY_NORMALIZED_SHA"], "predecessor hash mismatch")
require(intake_json.get("required_v301_issues") == list(range(999, 1006)), "V301 issue set mismatch")
require(intake_json.get("required_v310_issues") == list(range(1006, 1016)), "V310 issue set mismatch")
boundary = intake_json.get("non_inheritance_boundary") or {}
require(boundary, "missing non-inheritance boundary")
for key, value in boundary.items():
    require(value is False, f"boundary must remain false: {key}")
for case in intake_json.get("readiness_cases") or []:
    expected = case.get("expected_status")
    got = classify(case, boundary)
    require(got == expected, f"case {case.get('case_id')} expected {expected} got {got}")
enablement = intake_json.get("execution_enablement_status") or {}
for key, value in enablement.items():
    if key.endswith("_allowed"):
        require(value is False, f"execution enablement must stay false: {key}")
require(enablement.get("explicit_scoped_approval_required") is True, "scoped approval requirement missing")
require(intake_json.get("runtime_behavior_changed") is False, "runtime behavior must not change")
require(intake_json.get("trading_behavior_changed") is False, "trading behavior must not change")

for marker in [
    "intake_status = dependency_proof_satisfied_scoped_intake_only",
    "V301 issues closed = 7/7",
    "V301 milestone = closed",
    "v0.30.1 release evidence = published",
    "v0.30.1 hosted release gate jobs = 94/94 success",
    "v0.31.0 capability track = controlled_backend_production_enablement_candidate",
    "v0.31.0 default production submit = false",
    "backend_go_live_claim = false",
]:
    require_contains(intake_md, marker, "intake markdown")

for label, text in {"task": task, "evidence": evidence, "README": release_index}.items():
    require_contains(text, "V310-000", label)
    require_contains(text, "v0.31.0", label)

for key in false_flags:
    require_contains(intake_md, f"{key} = false", "intake markdown")
    require_contains(evidence, f"{key} = false", "evidence")
PY

release_json="$(gh_with_retry release view "$V301_RELEASE_TAG" --repo "$REPO" --json tagName,name,isDraft,isPrerelease,publishedAt,url)"
run_json="$(gh_with_retry run view "$V301_GATE_RUN_ID" --repo "$REPO" --json status,conclusion,headSha,url,updatedAt,jobs)"
milestone_json="$(gh_with_retry api "repos/$REPO/milestones/$V301_MILESTONE_NUMBER")"
v301_issues_json="$(gh_with_retry issue list --repo "$REPO" --state all --milestone "$V301_MILESTONE_TITLE" --limit 100 --json number,state,title)"
v310_issues_json="$(gh_with_retry issue list --repo "$REPO" --state all --milestone "$V310_MILESTONE_TITLE" --limit 100 --json number,state,title)"
current_issue_json="$(gh_with_retry issue view "$CURRENT_ISSUE" --repo "$REPO" --json number,state,title)"

RELEASE_JSON="$release_json" \
RUN_JSON="$run_json" \
MILESTONE_JSON="$milestone_json" \
V301_ISSUES_JSON="$v301_issues_json" \
V310_ISSUES_JSON="$v310_issues_json" \
CURRENT_ISSUE_JSON="$current_issue_json" \
V301_RELEASE_TAG="$V301_RELEASE_TAG" \
V301_RELEASE_NAME="$V301_RELEASE_NAME" \
V301_GATE_RUN_ID="$V301_GATE_RUN_ID" \
V301_TAG_SHA="$V301_TAG_SHA" \
python3 <<'PY'
import json
import os


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


release = json.loads(os.environ["RELEASE_JSON"])
require(release.get("tagName") == os.environ["V301_RELEASE_TAG"], "live release tag mismatch")
require(release.get("name") == os.environ["V301_RELEASE_NAME"], "live release name mismatch")
require(release.get("isDraft") is False, "live release must not be draft")
require(release.get("isPrerelease") is False, "live release must not be prerelease")

run = json.loads(os.environ["RUN_JSON"])
require(run.get("status") == "completed", "hosted gate must be completed")
require(run.get("conclusion") == "success", "hosted gate must succeed")
require(run.get("headSha") == os.environ["V301_TAG_SHA"], "hosted gate head SHA mismatch")
jobs = run.get("jobs") or []
require(len(jobs) == 94, f"hosted gate job count mismatch: {len(jobs)}")
require(all(job.get("conclusion") == "success" for job in jobs), "hosted gate contains non-success job")

milestone = json.loads(os.environ["MILESTONE_JSON"])
require(milestone.get("state") == "closed", "v0.30.1 milestone must be closed")
require(milestone.get("open_issues") == 0, "v0.30.1 milestone open issues must be 0")
require(milestone.get("closed_issues") == 7, "v0.30.1 milestone closed issue count mismatch")

v301_issues = json.loads(os.environ["V301_ISSUES_JSON"])
v301_map = {item["number"]: item for item in v301_issues}
require(set(v301_map) == set(range(999, 1006)), f"V301 issue set mismatch: {sorted(v301_map)}")
require(all(item["state"] == "CLOSED" for item in v301_map.values()), "not all V301 issues are closed")

v310_issues = json.loads(os.environ["V310_ISSUES_JSON"])
v310_numbers = {item["number"] for item in v310_issues}
require(v310_numbers == set(range(1006, 1016)), f"V310 issue set mismatch: {sorted(v310_numbers)}")

current = json.loads(os.environ["CURRENT_ISSUE_JSON"])
require(current.get("number") == 1006, "current issue number mismatch")

print(
    "v31_intake_gate_live "
    "v301_issues=7/7_closed "
    "v301_milestone=closed "
    "v301_release_present=true "
    "release_gate_jobs=94/94 "
    f"current_issue_state={current.get('state')} "
    "v310_issues=10 "
    "intake_status=dependency_proof_satisfied_scoped_intake_only"
)
PY

echo "v31_intake_gate=pass release_tag=$V301_RELEASE_TAG tag_sha=$V301_TAG_SHA v301_issues=7/7_closed v310_issues=10 no_inherited_execution=true"
