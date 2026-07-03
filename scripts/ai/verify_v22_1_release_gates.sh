#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

PATCH_VERSION="${NTPRO_V221_PATCH_VERSION:-v0.22.1}"
PATCH_TAG="${NTPRO_V221_PATCH_TAG:-ntpro-rust-only-v0.22.1}"
BASE_VERSION="${NTPRO_V221_BASE_VERSION:-v0.22.0}"
BASE_TAG="${NTPRO_V221_BASE_TAG:-ntpro-rust-only-v0.22.0}"
MANIFEST_PATH="${NTPRO_V221_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_22_1_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V221_RELEASE_NOTES:-docs/rust-cutover/release/v0_22_1_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V221_READINESS_REPORT:-docs/rust-cutover/release/v0_22_1_readiness_report.md}"
BASE_MANIFEST_PATH="${NTPRO_V221_BASE_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_22_0_release_manifest.json}"
GOLDEN_TRACE_MANIFEST_PATH="${NTPRO_V221_GOLDEN_TRACE_MANIFEST:-docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"
WORKBENCH_RENDER_FIXTURE="${NTPRO_V221_WORKBENCH_RENDER_FIXTURE:-tests/golden/v221/workbench_render_snapshot.json}"
CURRENT_ISSUE="${NTPRO_V221_CURRENT_ISSUE:-710}"

fail() {
  echo "v22.1 release gate failed: $*" >&2
  exit 1
}

require_file() {
  local path="$1"
  [[ -f "$path" ]] || fail "missing required file: $path"
}

require_contains() {
  local path="$1"
  local needle="$2"
  if ! grep -F -- "$needle" "$path" >/dev/null; then
    fail "missing marker in $path: $needle"
  fi
}

gh_with_retry() {
  local attempt=1
  local max_attempts=3
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

require_file "$MANIFEST_PATH"
require_file "$RELEASE_NOTES_PATH"
require_file "$READINESS_REPORT_PATH"
require_file "$BASE_MANIFEST_PATH"
require_file "$GOLDEN_TRACE_MANIFEST_PATH"
require_file "$WORKBENCH_RENDER_FIXTURE"

for path in \
  docs/rust-cutover/release/v0_22_1_release_closeout_evidence.md \
  docs/rust-cutover/release/v0_22_1_required_false_runtime_boundary.md \
  docs/rust-cutover/release/v0_22_1_read_model_executable_replay.md \
  docs/rust-cutover/release/v0_22_1_gate_before_publish.md; do
  require_file "$path"
done

for task_id in V221-001 V221-002 V221-003 V221-004 V221-005 V221-006; do
  require_file "docs/rust-cutover/evidence/${task_id}.md"
  require_contains "docs/rust-cutover/evidence/${task_id}.md" "$task_id"
done

for marker in \
  "Status: RELEASED" \
  "Tag: \`$PATCH_TAG\`" \
  "Release name: \`NTPRO Rust-only $PATCH_VERSION\`" \
  "Trader Terminal Workbench hardening patch" \
  "This release is read-only first" \
  "This release is not a product-grade live trading terminal" \
  "This release does not add submit capability" \
  "V221-001" \
  "V221-006" \
  "scripts/ai/verify_release.sh v22.1-release-gates" \
  "scripts/ai/verify_release.sh v22.1-strict-provenance" \
  "scripts/ai/verify_v22_1_release_gates.sh" \
  "scripts/ai/verify_v22_1_strict_provenance.sh" \
  "scripts/ai/publish_ntpro_release_after_gate.sh" \
  "The next capability track is \`v0.23.0\`"; do
  require_contains "$RELEASE_NOTES_PATH" "$marker"
done

for marker in \
  "Milestone: \`$PATCH_TAG\`" \
  "Status: RELEASED" \
  "V221-001 evidence" \
  "V221-006 evidence" \
  "v22.1 release gates = required" \
  "v22.1 strict provenance = required" \
  "release publish after gate = required" \
  "complete_executable_read_model_runtime = false" \
  "product_grade_trading_terminal_claim = false" \
  "#710 V221-006 = stays open until tag, hosted gate, public release, and publication evidence are recorded"; do
  require_contains "$READINESS_REPORT_PATH" "$marker"
done

scripts/ai/verify_release.sh v22-runtime-boundary-tests
scripts/ai/verify_release.sh v21.1-read-model-projection-replay
scripts/ai/verify_v22_workbench_render_smoke.sh
scripts/ai/verify_release.sh release-publish-after-gate

PATCH_VERSION="$PATCH_VERSION" \
PATCH_TAG="$PATCH_TAG" \
BASE_VERSION="$BASE_VERSION" \
BASE_TAG="$BASE_TAG" \
MANIFEST_PATH="$MANIFEST_PATH" \
RELEASE_NOTES_PATH="$RELEASE_NOTES_PATH" \
READINESS_REPORT_PATH="$READINESS_REPORT_PATH" \
BASE_MANIFEST_PATH="$BASE_MANIFEST_PATH" \
GOLDEN_TRACE_MANIFEST_PATH="$GOLDEN_TRACE_MANIFEST_PATH" \
WORKBENCH_RENDER_FIXTURE="$WORKBENCH_RENDER_FIXTURE" \
python3 <<'PY'
import json
import os
from collections import Counter
from pathlib import Path

manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))
base_manifest = json.loads(Path(os.environ["BASE_MANIFEST_PATH"]).read_text(encoding="utf-8"))
golden_trace_manifest = json.loads(Path(os.environ["GOLDEN_TRACE_MANIFEST_PATH"]).read_text(encoding="utf-8"))
release_notes = Path(os.environ["RELEASE_NOTES_PATH"]).read_text(encoding="utf-8")
readiness = Path(os.environ["READINESS_REPORT_PATH"]).read_text(encoding="utf-8")
render_fixture = json.loads(Path(os.environ["WORKBENCH_RENDER_FIXTURE"]).read_text(encoding="utf-8"))


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def require_false(mapping: dict, key: str) -> None:
    require(mapping.get(key) is False, f"boundary flag must be false: {key}")


def require_true(mapping: dict, key: str) -> None:
    require(mapping.get(key) is True, f"capability flag must be true: {key}")


require(manifest.get("schema_version") == "ntpro.v221_patch_release_manifest.v1", "manifest schema mismatch")
require(manifest.get("task_id") == "V221-006", "manifest task_id mismatch")
require(manifest.get("product_version") == os.environ["PATCH_VERSION"], "manifest product version mismatch")
require(manifest.get("release_status") == "published", "manifest release status mismatch")
require(manifest.get("patch_scope") == "trader_terminal_workbench_hardening_patch", "manifest patch scope mismatch")

base = manifest.get("base_release") or {}
require(base.get("tag") == os.environ["BASE_TAG"], "base tag mismatch")
require(base.get("version") == os.environ["BASE_VERSION"], "base version mismatch")
require(base.get("github_release_url") == f"https://github.com/atxinbao/NTPRO/releases/tag/{os.environ['BASE_TAG']}", "base release URL mismatch")
require(base_manifest.get("release_status") == "published", "base manifest status mismatch")
require((base_manifest.get("planned_release") or {}).get("tag") == os.environ["BASE_TAG"], "base manifest planned tag mismatch")

planned = manifest.get("planned_release") or {}
require(planned.get("tag") == os.environ["PATCH_TAG"], "planned patch tag mismatch")
require(planned.get("name") == f"NTPRO Rust-only {os.environ['PATCH_VERSION']}", "planned release name mismatch")
require(planned.get("github_release_url") == f"https://github.com/atxinbao/NTPRO/releases/tag/{os.environ['PATCH_TAG']}", "planned release URL mismatch")
require(planned.get("target_commitish") == "main", "planned target_commitish mismatch")
require(planned.get("draft") is False, "planned release must not be draft")
require(planned.get("prerelease") is False, "planned release must not be prerelease")

expected = {
    "V221-001": 705,
    "V221-002": 706,
    "V221-003": 707,
    "V221-004": 708,
    "V221-005": 709,
    "V221-006": 710,
}
evidence = manifest.get("v221_evidence") or []
require(len(evidence) == len(expected), "V221 evidence count mismatch")
for item in evidence:
    task_id = item.get("task_id")
    require(expected.get(task_id) == item.get("issue"), f"V221 evidence issue mismatch: {task_id}")
    path = Path(item.get("path", ""))
    require(path.is_file(), f"V221 evidence file missing: {path}")
    text = path.read_text(encoding="utf-8")
    require(task_id in text, f"V221 evidence task marker missing: {path}")

required_inputs = manifest.get("release_inputs") or {}
for key in (
    "release_notes_path",
    "readiness_report_path",
    "release_manifest_path",
    "base_release_manifest_path",
    "release_closeout_evidence_path",
    "required_false_runtime_boundary_path",
    "read_model_executable_replay_path",
    "gate_before_publish_path",
    "workbench_render_fixture_path",
    "golden_trace_manifest_path",
):
    path = Path(required_inputs.get(key, ""))
    require(path.is_file(), f"release input missing: {key} -> {path}")

commands = {
    gate.get("command")
    for gate in manifest.get("release_gates", [])
    if gate.get("required") is True
}
for command in (
    "scripts/ai/verify_release.sh v22-runtime-boundary-tests",
    "scripts/ai/verify_release.sh v21.1-read-model-projection-replay",
    "scripts/ai/verify_v22_workbench_render_smoke.sh",
    "scripts/ai/verify_release.sh release-publish-after-gate",
    "scripts/ai/verify_release.sh v22.1-release-gates",
    "scripts/ai/verify_release.sh v22.1-strict-provenance",
    "scripts/ai/verify_v22_1_release_gates.sh",
    "scripts/ai/verify_v22_1_strict_provenance.sh",
):
    require(command in commands, f"required release gate missing: {command}")

capability = manifest.get("capability") or {}
for key in (
    "trader_terminal_workbench",
    "read_only_first",
    "runtime_bridge",
    "required_false_runtime_boundary",
    "expanded_executable_replay",
    "workbench_render_smoke",
    "gate_before_publish",
    "strict_provenance",
):
    require_true(capability, key)
require(capability.get("capability_expansion") == "none_patch_hardening_only", "capability expansion mismatch")
require(capability.get("complete_executable_read_model_runtime") is False, "complete executable runtime claim must be false")
require(capability.get("product_grade_live_trading_terminal") is False, "product-grade terminal claim must be false")

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
    "implicit_retry_allowed",
    "automatic_cancel_allowed",
    "automatic_remediation_allowed",
    "automatic_order_remediation_allowed",
    "retry_replace_amend_flatten_allowed",
    "order_permission_control_allowed",
    "retry_order_allowed",
    "funds_transfer_allowed",
    "account_configuration_mutation_allowed",
    "auto_flatten_position_allowed",
    "automatic_position_repair_allowed",
    "execution_algorithm_allowed",
    "automatic_fill_repair_allowed",
    "automatic_reconciliation_repair_allowed",
    "automatic_risk_action_allowed",
    "automatic_risk_repair_allowed",
    "automatic_alert_action_allowed",
    "automatic_audit_action_allowed",
    "automatic_provenance_repair_allowed",
    "automatic_operation_action_allowed",
    "strategy_driven_production_execution_allowed",
    "multi_account_execution_allowed",
    "multi_strategy_execution_allowed",
    "multi_venue_execution_allowed",
    "dashboard_order_controls_enabled",
    "dashboard_approval_controls_enabled",
    "dashboard_cancel_controls_enabled",
    "dashboard_retry_controls_enabled",
    "dashboard_fill_controls_enabled",
    "dashboard_risk_controls_enabled",
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
    require_false(manifest.get("boundary_flags") or {}, key)

cases = golden_trace_manifest.get("cases") or []
require(golden_trace_manifest.get("schema_version") == "golden-trace-release-scope-v1", "golden trace manifest schema mismatch")
status_counts = Counter(case.get("status") for case in cases)
read_model_status_counts = Counter(
    case.get("status") for case in cases if case.get("category") == "read_model"
)
replay = manifest.get("read_model_replay") or {}
require(replay.get("manifest_cases") == len(cases), "manifest case count mismatch")
require(replay.get("manifest_executable_replay_cases") == status_counts["executable_replay"], "manifest executable count mismatch")
require(replay.get("manifest_schema_only_scoped_cases") == status_counts["schema_only_scoped"], "manifest schema-only count mismatch")
require(replay.get("read_model_executable_replay_rows") == read_model_status_counts["executable_replay"], "read-model executable count mismatch")
require(replay.get("read_model_schema_only_scoped_rows") == read_model_status_counts["schema_only_scoped"], "read-model schema-only count mismatch")
remaining = sorted(
    case.get("case_id")
    for case in cases
    if case.get("category") == "read_model" and case.get("status") == "schema_only_scoped"
)
require(sorted(replay.get("remaining_schema_only_read_model_rows") or []) == remaining, "remaining read-model schema-only rows mismatch")

require(isinstance(render_fixture, dict), "workbench render fixture must be a JSON object")

publication = manifest.get("publication_governance") or {}
require(publication.get("gate_before_publish") is True, "gate-before-publish flag mismatch")
require(publication.get("release_gate_workflow_name") == "Rust Cutover Release Gate", "release gate workflow mismatch")
require(publication.get("publish_script") == "scripts/ai/publish_ntpro_release_after_gate.sh", "publish script mismatch")
require(publication.get("public_release_requires_successful_hosted_gate_for_same_tag_commit") is True, "same-tag gate requirement mismatch")

dependency = manifest.get("v230_dependency") or {}
require(dependency.get("milestone") == "v0.23.0", "v230 dependency milestone mismatch")
require(dependency.get("blocked_issues") == [711, 712, 713, 714, 715, 716, 717, 718], "v230 blocked issue set mismatch")
require(dependency.get("blocked_by_issues") == [705, 706, 707, 708, 709, 710], "v230 blocked-by issue set mismatch")
require(dependency.get("dependency_status") == "blocked_until_v0_22_1_publication", "v230 dependency status mismatch")

for text, label in ((release_notes, "release notes"), (readiness, "readiness report")):
    for forbidden in (
        "new production submit capability = true",
        "production order mutation = true",
        "ungated submit = true",
        "ungated cancel = true",
        "ungated retry = true",
        "ungated replace = true",
        "ungated amend = true",
        "ungated flatten = true",
        "product-grade live trading terminal readiness = true",
        "complete executable read-model runtime = true",
        "dashboard_order_controls_enabled = true",
    ):
        require(forbidden not in text, f"{label} contains forbidden expansion wording: {forbidden}")
PY

if [[ "${NTPRO_V221_SKIP_GITHUB_DEPENDENCY:-0}" != "1" ]]; then
  if ! command -v gh >/dev/null 2>&1; then
    fail "gh is required for GitHub dependency proof"
  fi
  if ! gh_with_retry auth status >/dev/null 2>&1; then
    fail "gh auth is required for GitHub dependency proof"
  fi

  gh_with_retry release view "$BASE_TAG" --repo atxinbao/NTPRO --json tagName,isDraft,isPrerelease,url >/tmp/ntpro-v221-base-release.json
  BASE_TAG="$BASE_TAG" python3 <<'PY'
import json
import os
from pathlib import Path

payload = json.loads(Path("/tmp/ntpro-v221-base-release.json").read_text(encoding="utf-8"))
if payload.get("tagName") != os.environ["BASE_TAG"]:
    raise SystemExit("base release tag mismatch")
if payload.get("isDraft") or payload.get("isPrerelease"):
    raise SystemExit("base release must be final")
PY

  for issue in 705 706 707 708 709; do
    state="$(gh_with_retry issue view "$issue" --repo atxinbao/NTPRO --json state --jq .state)"
    [[ "$state" == "CLOSED" ]] || fail "V221 dependency issue #$issue is not closed"
  done

  current_state="$(gh_with_retry issue view "$CURRENT_ISSUE" --repo atxinbao/NTPRO --json state --jq .state)"
  if [[ "${NTPRO_V221_REQUIRE_CURRENT_ISSUE_CLOSED:-0}" == "1" ]]; then
    [[ "$current_state" == "CLOSED" ]] || fail "current V221 issue #$CURRENT_ISSUE is not closed"
  fi

  gh_with_retry api "repos/atxinbao/NTPRO/milestones?state=all" --paginate --jq '
    def require(cond; msg): if cond then empty else error(msg) end;
    [.[] | select(.title=="v0.22.1" or .title=="v0.23.0")] as $m |
    require(($m | length) == 2; "missing v0.22.1/v0.23.0 milestones") |
    ($m[] | select(.title=="v0.22.1")) as $v221 |
    ($m[] | select(.title=="v0.23.0")) as $v230 |
    require(($v221.description | contains("Hard-blocks v0.23.0")); "v0.22.1 milestone missing v0.23 hard block") |
    require(($v221.description | contains("release evidence is published")); "v0.22.1 milestone missing publication evidence rule") |
    require(($v230.description | contains("Hard-blocked by v0.22.1")); "v0.23.0 milestone missing hard-blocked wording") |
    require(($v230.description | contains("all V221 issues are closed")); "v0.23.0 milestone missing V221 closure rule") |
    require(($v230.description | contains("v0.22.1 release evidence is published")); "v0.23.0 milestone missing release evidence rule") |
    "github milestone v221/v230 dependency proof ok"
  ' >/tmp/ntpro-v221-milestone-proof.txt
  cat /tmp/ntpro-v221-milestone-proof.txt

  for issue in 711 712 713 714 715 716 717 718; do
    payload="$(gh_with_retry issue view "$issue" --repo atxinbao/NTPRO --json body,state)"
    BODY_AND_STATE="$payload" ISSUE="$issue" python3 <<'PY'
import json
import os

payload = json.loads(os.environ["BODY_AND_STATE"])
body = payload.get("body") or ""
issue = os.environ["ISSUE"]
if payload.get("state") != "OPEN":
    raise SystemExit(f"V230 issue #{issue} must remain open before v0.22.1 publication")
if "Hard-blocked" not in body and "hard-blocked" not in body:
    raise SystemExit(f"V230 issue #{issue} missing hard-blocked wording")
for marker in ("#705", "#706", "#707", "#708", "#709", "#710"):
    if marker not in body:
        raise SystemExit(f"V230 issue #{issue} missing V221 dependency marker: {marker}")
if issue == "711" and "v0.22.1 release evidence" not in body:
    raise SystemExit("V230 intake issue #711 must name v0.22.1 release evidence before unblocking")
PY
  done
fi

echo "v22_1_release_gates status=ok version=$PATCH_VERSION release_tag=$PATCH_TAG base_tag=$BASE_TAG v221_evidence=complete workbench_render_smoke=required gate_before_publish=required current_issue_state=${current_state:-unknown}"
