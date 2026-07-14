#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

REPO="${NTPRO_V32_INTAKE_REPO:-atxinbao/NTPRO}"
V311_RELEASE_VERSION="${NTPRO_V32_INTAKE_V311_RELEASE_VERSION:-v0.31.1}"
V311_RELEASE_TAG="${NTPRO_V32_INTAKE_V311_RELEASE_TAG:-ntpro-rust-only-v0.31.1}"
V311_RELEASE_NAME="${NTPRO_V32_INTAKE_V311_RELEASE_NAME:-NTPRO Rust-only v0.31.1}"
V311_GATE_RUN_ID="${NTPRO_V32_INTAKE_V311_GATE_RUN_ID:-29359951505}"
V311_PUBLISH_RUN_ID="${NTPRO_V32_INTAKE_V311_PUBLISH_RUN_ID:-29365747453}"
V311_TAG_OBJECT_SHA="${NTPRO_V32_INTAKE_V311_TAG_OBJECT_SHA:-526d2403c7f7a64b55988a365d45f784e2c08808}"
V311_TAG_SHA="${NTPRO_V32_INTAKE_V311_TAG_SHA:-41c13405867b143d2db54b34909913157f19dbdd}"
V311_BODY_NORMALIZED_SHA="${NTPRO_V32_INTAKE_V311_BODY_NORMALIZED_SHA:-7004cf49ae21e45fef12df009add4763af75e24cf153da3c6c383119ce449b5d}"
V311_BODY_RAW_SHA="${NTPRO_V32_INTAKE_V311_BODY_RAW_SHA:-0d7fc609be3d50545e4c83f0dc8f98b2b3c4fc48592a1e9f5df25a42e214612b}"
V311_RELEASE_NOTES="${NTPRO_V32_INTAKE_V311_NOTES:-docs/rust-cutover/release/v0_31_1_release_notes.md}"
V311_RELEASE_MANIFEST="${NTPRO_V32_INTAKE_V311_MANIFEST:-docs/rust-cutover/release/v0_31_1_release_manifest.json}"
V311_START_GATE_JSON="${NTPRO_V32_INTAKE_V311_START_GATE_JSON:-docs/rust-cutover/release/v0_31_1_v32_start_gate.json}"
V32_INTAKE_MD="${NTPRO_V32_INTAKE_MD:-docs/rust-cutover/release/v0_32_0_intake_gate.md}"
V32_INTAKE_JSON="${NTPRO_V32_INTAKE_JSON:-docs/rust-cutover/release/v0_32_0_intake_gate.json}"
V311_MILESTONE_NUMBER="${NTPRO_V32_INTAKE_V311_MILESTONE_NUMBER:-29}"
V311_MILESTONE_TITLE="${NTPRO_V32_INTAKE_V311_MILESTONE_TITLE:-v0.31.1}"
V320_MILESTONE_TITLE="${NTPRO_V32_INTAKE_V320_MILESTONE_TITLE:-v0.32.0}"
CURRENT_ISSUE="${NTPRO_V32_INTAKE_CURRENT_ISSUE:-1042}"

fail() {
  echo "v32 intake gate failed: $*" >&2
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
  "$V311_RELEASE_NOTES" \
  "$V311_RELEASE_MANIFEST" \
  "$V311_START_GATE_JSON" \
  "$V32_INTAKE_MD" \
  "$V32_INTAKE_JSON" \
  docs/rust-cutover/release/v0_31_1_readiness_report.md \
  docs/rust-cutover/release/v0_31_1_v32_start_gate.md \
  docs/rust-cutover/release/README.md \
  docs/rust-cutover/tasks/V320-000.md \
  docs/rust-cutover/evidence/V320-000.md \
  scripts/ai/verify_v31_1_v32_start_gate.sh \
  scripts/ai/verify_v32_intake_gate.sh; do
  require_file "$path"
done

for task_id in V311-001 V311-002 V311-003 V311-004 V311-005 V311-006; do
  require_file "docs/rust-cutover/evidence/${task_id}.md"
  require_file "docs/rust-cutover/tasks/${task_id}.md"
done

scripts/ai/verify_v31_1_v32_start_gate.sh live >/dev/null

V311_RELEASE_MANIFEST="$V311_RELEASE_MANIFEST" \
V311_RELEASE_NOTES="$V311_RELEASE_NOTES" \
V311_START_GATE_JSON="$V311_START_GATE_JSON" \
V32_INTAKE_MD="$V32_INTAKE_MD" \
V32_INTAKE_JSON="$V32_INTAKE_JSON" \
V311_RELEASE_TAG="$V311_RELEASE_TAG" \
V311_RELEASE_NAME="$V311_RELEASE_NAME" \
V311_GATE_RUN_ID="$V311_GATE_RUN_ID" \
V311_PUBLISH_RUN_ID="$V311_PUBLISH_RUN_ID" \
V311_TAG_OBJECT_SHA="$V311_TAG_OBJECT_SHA" \
V311_TAG_SHA="$V311_TAG_SHA" \
V311_BODY_NORMALIZED_SHA="$V311_BODY_NORMALIZED_SHA" \
V311_BODY_RAW_SHA="$V311_BODY_RAW_SHA" \
python3 <<'PY'
import copy
import hashlib
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
    "frontend_completion_claim",
    "backend_go_live_claim",
    "product_grade_trading_terminal_claim",
    "product_grade_live_trading_terminal_claim",
    "default_production_execution_allowed",
]


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def require_contains(text: str, marker: str, label: str) -> None:
    require(marker in text, f"{label} missing marker: {marker}")


def normalized_sha256(text: str) -> str:
    normalized = "\n".join(line.rstrip() for line in text.strip().splitlines())
    return hashlib.sha256(normalized.encode()).hexdigest()


def raw_sha256(text: str) -> str:
    return hashlib.sha256(text.encode()).hexdigest()


def classify(case: dict, boundary: dict) -> str:
    scenario_boundary = copy.deepcopy(boundary)
    scenario_boundary.update(case.get("boundary_flags_override") or {})
    if case.get("v31_1_release_present") is not True:
        return "fail_closed_missing_v31_1_release_evidence"
    if case.get("v31_1_milestone_closed") is not True:
        return "fail_closed_missing_v31_1_milestone_closeout"
    if any(value is not False for value in scenario_boundary.values()):
        return "fail_closed_inherited_execution"
    return "dependency_proof_satisfied_backend_closeout_scoped_intake_only"


manifest = json.loads(Path(os.environ["V311_RELEASE_MANIFEST"]).read_text(encoding="utf-8"))
notes = Path(os.environ["V311_RELEASE_NOTES"]).read_text(encoding="utf-8")
start_gate = json.loads(Path(os.environ["V311_START_GATE_JSON"]).read_text(encoding="utf-8"))
intake_md = Path(os.environ["V32_INTAKE_MD"]).read_text(encoding="utf-8")
intake_json = json.loads(Path(os.environ["V32_INTAKE_JSON"]).read_text(encoding="utf-8"))
task = Path("docs/rust-cutover/tasks/V320-000.md").read_text(encoding="utf-8")
evidence = Path("docs/rust-cutover/evidence/V320-000.md").read_text(encoding="utf-8")
release_index = Path("docs/rust-cutover/release/README.md").read_text(encoding="utf-8")

require(manifest.get("release_status") == "release_gate_ready", "v31.1 source manifest should remain release-gate-ready input")
require(start_gate.get("schema_version") == "ntpro.v311.v32_backend_closeout_start_gate.v1", "v31.1 start gate schema mismatch")
require(start_gate.get("required_patch_release", {}).get("tag") == os.environ["V311_RELEASE_TAG"], "v31.1 start gate tag mismatch")
require(normalized_sha256(notes) == os.environ["V311_BODY_NORMALIZED_SHA"], "tracked v31.1 release notes normalized hash mismatch")
require(raw_sha256(notes) == os.environ["V311_BODY_RAW_SHA"], "tracked v31.1 release notes raw hash mismatch")

require(intake_json.get("schema_version") == "ntpro.v320_intake_gate.v1", "intake schema mismatch")
require(intake_json.get("task_id") == "V320-000", "intake task mismatch")
require(intake_json.get("github_issue") == 1042, "intake issue mismatch")
require(intake_json.get("milestone") == "v0.32.0", "intake milestone mismatch")
require(intake_json.get("capability_track") == "backend_production_closeout", "capability track mismatch")
require(intake_json.get("intake_status") == "dependency_proof_satisfied_backend_closeout_scoped_intake_only", "intake status mismatch")

predecessor = intake_json.get("predecessor_release") or {}
require(predecessor.get("version") == "v0.31.1", "predecessor version mismatch")
require(predecessor.get("tag") == os.environ["V311_RELEASE_TAG"], "predecessor release tag mismatch")
require(predecessor.get("name") == os.environ["V311_RELEASE_NAME"], "predecessor release name mismatch")
require(predecessor.get("release_gate_run_id") == int(os.environ["V311_GATE_RUN_ID"]), "predecessor release gate mismatch")
require(predecessor.get("release_gate_jobs_success") == 98, "hosted gate success count mismatch")
require(predecessor.get("release_gate_jobs_total") == 98, "hosted gate total count mismatch")
require(predecessor.get("publish_run_id") == int(os.environ["V311_PUBLISH_RUN_ID"]), "publish run mismatch")
require(predecessor.get("published_after_hosted_gate") is True, "published-after-gate missing")
require(predecessor.get("release_body_normalized_sha256") == os.environ["V311_BODY_NORMALIZED_SHA"], "predecessor normalized hash mismatch")
require(predecessor.get("release_body_raw_sha256") == os.environ["V311_BODY_RAW_SHA"], "predecessor raw hash mismatch")
require(predecessor.get("tag_object_sha") == os.environ["V311_TAG_OBJECT_SHA"], "tag object mismatch")
require(predecessor.get("tag_sha") == os.environ["V311_TAG_SHA"], "tag SHA mismatch")

require(intake_json.get("required_v311_issues") == list(range(1036, 1042)), "V311 issue set mismatch")
require(intake_json.get("required_v311_issue_count") == 6, "V311 issue count mismatch")
require(intake_json.get("required_v320_issues") == list(range(1042, 1052)), "V320 issue set mismatch")
require(intake_json.get("required_v320_issue_count") == 10, "V320 issue count mismatch")

scope = intake_json.get("backend_closeout_scope") or {}
require(scope.get("v0_32_backend_closeout_version") is True, "backend closeout version marker missing")
require(scope.get("scoped_backend_closeout_intake_open") is True, "scoped intake marker missing")
require(scope.get("explicit_scoped_approval_issue") == 1043, "scoped approval issue mismatch")
require(scope.get("explicit_scoped_approval_present") is False, "scoped approval must not be present at intake")
require(scope.get("production_execution_capability_enabled") is False, "production execution must remain disabled")
require(scope.get("frontend_completion") is False, "frontend completion must be false")
require(scope.get("product_grade_live_trading_terminal") is False, "product-grade terminal must be false")

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
    if key.startswith("default_") and key.endswith("_allowed"):
        require(value is False, f"execution enablement must stay false: {key}")
for key in [
    "explicit_scoped_approval_required",
    "owner_operator_approval_required",
    "risk_gate_required",
    "audit_gate_required",
    "go_no_go_required",
    "rollback_dr_required",
    "telemetry_slo_gate_required",
    "config_venue_provenance_required",
    "backend_read_model_admin_bridge_required",
    "fail_closed_negative_tests_required",
    "release_gate_required",
    "strict_provenance_required",
]:
    require(enablement.get(key) is True, f"missing required gate: {key}")

require(intake_json.get("runtime_behavior_changed") is False, "runtime behavior must not change")
require(intake_json.get("trading_behavior_changed") is False, "trading behavior must not change")

for marker in [
    "intake_status = dependency_proof_satisfied_backend_closeout_scoped_intake_only",
    "V311 issues closed = 6/6",
    "V311 milestone = closed",
    "v0.31.1 release evidence = published",
    "v0.31.1 hosted release gate jobs = 98/98 success",
    "v0.32.0 capability track = backend_production_closeout",
    "v0.32.0 backend closeout version = true",
    "v0.32.0 production execution enabled = false",
    "frontend_completion_claim = false",
    "backend_go_live_claim = false",
]:
    require_contains(intake_md, marker, "intake markdown")

for label, text in {"task": task, "evidence": evidence, "README": release_index}.items():
    require_contains(text, "V320-000", label)
    require_contains(text, "v0.32.0", label)

for key in false_flags:
    require_contains(intake_md, f"{key} = false", "intake markdown")
    require_contains(evidence, f"{key} = false", "evidence")
PY

release_json="$(gh_with_retry release view "$V311_RELEASE_TAG" --repo "$REPO" --json tagName,name,isDraft,isPrerelease,publishedAt,url,body)"
gate_run_json="$(gh_with_retry run view "$V311_GATE_RUN_ID" --repo "$REPO" --json status,conclusion,headSha,headBranch,url,updatedAt,jobs)"
publish_run_json="$(gh_with_retry run view "$V311_PUBLISH_RUN_ID" --repo "$REPO" --json status,conclusion,headSha,headBranch,url,updatedAt,jobs)"
milestone_json="$(gh_with_retry api "repos/$REPO/milestones/$V311_MILESTONE_NUMBER")"
v311_issues_json="$(gh_with_retry issue list --repo "$REPO" --state all --milestone "$V311_MILESTONE_TITLE" --limit 100 --json number,state,title)"
v320_issues_json="$(gh_with_retry issue list --repo "$REPO" --state all --milestone "$V320_MILESTONE_TITLE" --limit 100 --json number,state,title)"
current_issue_json="$(gh_with_retry issue view "$CURRENT_ISSUE" --repo "$REPO" --json number,state,title)"

LIVE_JSON_DIR="$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v32-intake-live-json.XXXXXX")"
trap 'rm -rf "$LIVE_JSON_DIR"' EXIT
printf '%s' "$release_json" >"$LIVE_JSON_DIR/release.json"
printf '%s' "$gate_run_json" >"$LIVE_JSON_DIR/gate_run.json"
printf '%s' "$publish_run_json" >"$LIVE_JSON_DIR/publish_run.json"
printf '%s' "$milestone_json" >"$LIVE_JSON_DIR/milestone.json"
printf '%s' "$v311_issues_json" >"$LIVE_JSON_DIR/v311_issues.json"
printf '%s' "$v320_issues_json" >"$LIVE_JSON_DIR/v320_issues.json"
printf '%s' "$current_issue_json" >"$LIVE_JSON_DIR/current_issue.json"

RELEASE_JSON_PATH="$LIVE_JSON_DIR/release.json" \
GATE_RUN_JSON_PATH="$LIVE_JSON_DIR/gate_run.json" \
PUBLISH_RUN_JSON_PATH="$LIVE_JSON_DIR/publish_run.json" \
MILESTONE_JSON_PATH="$LIVE_JSON_DIR/milestone.json" \
V311_ISSUES_JSON_PATH="$LIVE_JSON_DIR/v311_issues.json" \
V320_ISSUES_JSON_PATH="$LIVE_JSON_DIR/v320_issues.json" \
CURRENT_ISSUE_JSON_PATH="$LIVE_JSON_DIR/current_issue.json" \
V311_RELEASE_TAG="$V311_RELEASE_TAG" \
V311_RELEASE_NAME="$V311_RELEASE_NAME" \
V311_TAG_SHA="$V311_TAG_SHA" \
V311_BODY_NORMALIZED_SHA="$V311_BODY_NORMALIZED_SHA" \
V311_BODY_RAW_SHA="$V311_BODY_RAW_SHA" \
python3 <<'PY'
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def parse_dt(value: str) -> datetime:
    return datetime.fromisoformat(value.replace("Z", "+00:00")).astimezone(timezone.utc)


def normalized_sha256(text: str) -> str:
    normalized = "\n".join(line.rstrip() for line in text.strip().splitlines())
    return hashlib.sha256(normalized.encode()).hexdigest()


def raw_sha256(text: str) -> str:
    return hashlib.sha256(text.encode()).hexdigest()


release = json.loads(Path(os.environ["RELEASE_JSON_PATH"]).read_text(encoding="utf-8"))
gate_run = json.loads(Path(os.environ["GATE_RUN_JSON_PATH"]).read_text(encoding="utf-8"))
publish_run = json.loads(Path(os.environ["PUBLISH_RUN_JSON_PATH"]).read_text(encoding="utf-8"))
milestone = json.loads(Path(os.environ["MILESTONE_JSON_PATH"]).read_text(encoding="utf-8"))
v311_issues = json.loads(Path(os.environ["V311_ISSUES_JSON_PATH"]).read_text(encoding="utf-8"))
v320_issues = json.loads(Path(os.environ["V320_ISSUES_JSON_PATH"]).read_text(encoding="utf-8"))
current_issue = json.loads(Path(os.environ["CURRENT_ISSUE_JSON_PATH"]).read_text(encoding="utf-8"))

require(release.get("tagName") == os.environ["V311_RELEASE_TAG"], "release tag mismatch")
require(release.get("name") == os.environ["V311_RELEASE_NAME"], "release name mismatch")
require(release.get("isDraft") is False, "release must not be draft")
require(release.get("isPrerelease") is False, "release must not be prerelease")
require(normalized_sha256(release.get("body") or "") == os.environ["V311_BODY_NORMALIZED_SHA"], "live normalized body hash mismatch")
require(raw_sha256(release.get("body") or "") == os.environ["V311_BODY_RAW_SHA"], "live raw body hash mismatch")

require(gate_run.get("status") == "completed", "gate run must be completed")
require(gate_run.get("conclusion") == "success", "gate run must be success")
require(gate_run.get("headSha") == os.environ["V311_TAG_SHA"], "gate run head SHA mismatch")
require(gate_run.get("headBranch") == os.environ["V311_RELEASE_TAG"], "gate run head branch mismatch")
gate_jobs = gate_run.get("jobs") or []
require(len(gate_jobs) == 98, f"unexpected gate job count: {len(gate_jobs)}")
require(all(job.get("conclusion") == "success" for job in gate_jobs), "all gate jobs must be success")

require(publish_run.get("status") == "completed", "publish run must be completed")
require(publish_run.get("conclusion") == "success", "publish run must be success")
require(publish_run.get("headSha") == os.environ["V311_TAG_SHA"], "publish run head SHA mismatch")
publish_jobs = publish_run.get("jobs") or []
require(len(publish_jobs) == 1, f"unexpected publish job count: {len(publish_jobs)}")
require(all(job.get("conclusion") == "success" for job in publish_jobs), "publish job must be success")
require(parse_dt(release["publishedAt"]) > parse_dt(gate_run["updatedAt"]), "release must be published after hosted gate")

require(milestone.get("state") == "closed", "v0.31.1 milestone must be closed")
require(milestone.get("open_issues") == 0, "v0.31.1 milestone must have zero open issues")
require(milestone.get("closed_issues") == 6, "v0.31.1 milestone closed issue count mismatch")

v311_map = {item["number"]: item for item in v311_issues}
expected_v311 = list(range(1036, 1042))
missing_v311 = [number for number in expected_v311 if number not in v311_map]
require(not missing_v311, f"missing V311 issues: {missing_v311}")
require(all(v311_map[number]["state"] == "CLOSED" for number in expected_v311), "all V311 issues must be closed")

v320_map = {item["number"]: item for item in v320_issues}
expected_v320 = list(range(1042, 1052))
missing_v320 = [number for number in expected_v320 if number not in v320_map]
require(not missing_v320, f"missing V320 issues: {missing_v320}")
require(current_issue.get("number") == 1042, "current issue mismatch")

print(
    "v32_intake_gate_live "
    "v311_issues=6/6_closed "
    "v311_milestone=closed "
    "v311_release_present=true "
    "release_gate_jobs=98/98 "
    f"current_issue_state={current_issue.get('state')} "
    "v320_issues=10 "
    "intake_status=dependency_proof_satisfied_backend_closeout_scoped_intake_only"
)
PY

echo "v32_intake_gate=pass release_tag=$V311_RELEASE_TAG tag_sha=$V311_TAG_SHA v311_issues=6/6_closed v320_issues=10 no_inherited_execution=true"
