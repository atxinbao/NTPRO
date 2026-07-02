#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

VERSION="${NTPRO_V220_VERSION:-v0.22.0}"
RELEASE_TAG="${NTPRO_V220_RELEASE_TAG:-ntpro-rust-only-v0.22.0}"
BASE_VERSION="${NTPRO_V220_BASE_VERSION:-v0.21.1}"
BASE_TAG="${NTPRO_V220_BASE_TAG:-ntpro-rust-only-v0.21.1}"
MANIFEST_PATH="${NTPRO_V220_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_22_0_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V220_RELEASE_NOTES:-docs/rust-cutover/release/v0_22_0_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V220_READINESS_REPORT:-docs/rust-cutover/release/v0_22_0_readiness_report.md}"
BASE_MANIFEST_PATH="${NTPRO_V220_BASE_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_21_1_release_manifest.json}"
SCOPE_PATH="${NTPRO_V220_SCOPE:-docs/rust-cutover/scope/v0_22_0_trader_terminal_workbench_scope.md}"
CURRENT_ISSUE="${NTPRO_V220_CURRENT_ISSUE:-690}"

fail() {
  echo "v22 release gate failed: $*" >&2
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

require_file "$MANIFEST_PATH"
require_file "$RELEASE_NOTES_PATH"
require_file "$READINESS_REPORT_PATH"
require_file "$BASE_MANIFEST_PATH"
require_file "$SCOPE_PATH"

for task_id in V220-000 V220-001 V220-002 V220-003 V220-004 V220-005 V220-006 V220-007; do
  require_file "docs/rust-cutover/evidence/${task_id}.md"
  require_contains "docs/rust-cutover/evidence/${task_id}.md" "$task_id"
done

for release_note in \
  docs/rust-cutover/release/v0_22_0_trader_terminal_workbench_shell.md \
  docs/rust-cutover/release/v0_22_0_account_position_workbench_panels.md \
  docs/rust-cutover/release/v0_22_0_order_fill_workbench_panels.md \
  docs/rust-cutover/release/v0_22_0_risk_alert_audit_provenance_workbench_panels.md \
  docs/rust-cutover/release/v0_22_0_gated_manual_operation_entry_contract.md \
  docs/rust-cutover/release/v0_22_0_runtime_degradation_boundary_tests.md; do
  require_file "$release_note"
  require_contains "$release_note" "V220-"
done

for marker in \
  "Status: RELEASED" \
  "Tag: \`$RELEASE_TAG\`" \
  "Release name: \`NTPRO Rust-only $VERSION\`" \
  "Trader Terminal Workbench" \
  "This release is read-only first" \
  "This release is not a product-grade live trading terminal" \
  "This release does not add submit capability" \
  "V220-000" \
  "V220-007" \
  "scripts/ai/verify_release.sh v22-release-gates" \
  "scripts/ai/verify_release.sh v22-strict-provenance" \
  "scripts/ai/verify_release_strict.sh v22"; do
  require_contains "$RELEASE_NOTES_PATH" "$marker"
done

for marker in \
  "Milestone: \`$RELEASE_TAG\`" \
  "Status: RELEASED" \
  "V220-000 evidence" \
  "V220-007 evidence" \
  "v22 runtime boundary tests = required" \
  "v22 strict provenance = required" \
  "read_only_first = true" \
  "gated_operation_boundary = true" \
  "product_grade_trading_terminal_claim = false"; do
  require_contains "$READINESS_REPORT_PATH" "$marker"
done

scripts/ai/verify_release.sh v22-runtime-boundary-tests

VERSION="$VERSION" \
RELEASE_TAG="$RELEASE_TAG" \
BASE_VERSION="$BASE_VERSION" \
BASE_TAG="$BASE_TAG" \
MANIFEST_PATH="$MANIFEST_PATH" \
RELEASE_NOTES_PATH="$RELEASE_NOTES_PATH" \
READINESS_REPORT_PATH="$READINESS_REPORT_PATH" \
BASE_MANIFEST_PATH="$BASE_MANIFEST_PATH" \
SCOPE_PATH="$SCOPE_PATH" \
python3 <<'PY'
import json
import os
from pathlib import Path

manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))
base_manifest = json.loads(Path(os.environ["BASE_MANIFEST_PATH"]).read_text(encoding="utf-8"))
release_notes = Path(os.environ["RELEASE_NOTES_PATH"]).read_text(encoding="utf-8")
readiness = Path(os.environ["READINESS_REPORT_PATH"]).read_text(encoding="utf-8")
scope = Path(os.environ["SCOPE_PATH"]).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def require_false(mapping: dict, key: str) -> None:
    require(mapping.get(key) is False, f"boundary flag must be false: {key}")


def require_true(mapping: dict, key: str) -> None:
    require(mapping.get(key) is True, f"capability flag must be true: {key}")


require(manifest.get("schema_version") == "ntpro.v220_release_manifest.v1", "manifest schema mismatch")
require(manifest.get("task_id") == "V220-007", "manifest task_id mismatch")
require(manifest.get("product_version") == os.environ["VERSION"], "manifest product version mismatch")
require(manifest.get("release_status") == "published", "manifest release status mismatch")
require(manifest.get("release_scope") == "trader_terminal_workbench", "manifest release scope mismatch")

base = manifest.get("base_release") or {}
require(base.get("tag") == os.environ["BASE_TAG"], "base tag mismatch")
require(base.get("version") == os.environ["BASE_VERSION"], "base version mismatch")
require(base.get("github_release_url") == f"https://github.com/atxinbao/NTPRO/releases/tag/{os.environ['BASE_TAG']}", "base release URL mismatch")
require(base_manifest.get("release_status") == "published", "base manifest status mismatch")
require((base_manifest.get("planned_release") or {}).get("tag") == os.environ["BASE_TAG"], "base manifest tag mismatch")

planned = manifest.get("planned_release") or {}
require(planned.get("tag") == os.environ["RELEASE_TAG"], "planned release tag mismatch")
require(planned.get("name") == f"NTPRO Rust-only {os.environ['VERSION']}", "planned release name mismatch")
require(planned.get("github_release_url") == f"https://github.com/atxinbao/NTPRO/releases/tag/{os.environ['RELEASE_TAG']}", "planned release URL mismatch")
require(planned.get("target_commitish") == "main", "planned target_commitish mismatch")
require(planned.get("draft") is False, "planned release must not be draft")
require(planned.get("prerelease") is False, "planned release must not be prerelease")

expected = {
    "V220-000": 683,
    "V220-001": 684,
    "V220-002": 685,
    "V220-003": 686,
    "V220-004": 687,
    "V220-005": 688,
    "V220-006": 689,
    "V220-007": 690,
}
evidence = manifest.get("v220_evidence") or []
require(len(evidence) == len(expected), "V220 evidence count mismatch")
for item in evidence:
    task_id = item.get("task_id")
    require(expected.get(task_id) == item.get("issue"), f"V220 evidence issue mismatch: {task_id}")
    path = Path(item.get("path", ""))
    require(path.is_file(), f"V220 evidence file missing: {path}")
    require(task_id in path.read_text(encoding="utf-8"), f"V220 evidence task marker missing: {path}")

workbench = manifest.get("workbench_evidence") or []
require(len(workbench) == 7, "workbench evidence count mismatch")
for item in workbench:
    path = Path(item.get("path", ""))
    require(path.is_file(), f"workbench evidence file missing: {path}")

commands = {
    gate.get("command")
    for gate in manifest.get("release_gates", [])
    if gate.get("required") is True
}
for command in (
    "scripts/ai/verify_release.sh v22-runtime-boundary-tests",
    "scripts/ai/verify_release.sh v22-release-gates",
    "scripts/ai/verify_release.sh v22-strict-provenance",
    "scripts/ai/verify_v22_release_gates.sh",
    "scripts/ai/verify_v22_strict_provenance.sh",
    "scripts/ai/verify_release_strict.sh v22",
    "scripts/ai/verify_release.sh release-surface-current-guard",
    "scripts/ai/verify_release.sh release-publication-guard",
):
    require(command in commands, f"required release gate missing: {command}")

capability = manifest.get("capability") or {}
for key in (
    "trader_terminal_workbench",
    "read_only_first",
    "account_panel",
    "position_panel",
    "order_panel",
    "fill_panel",
    "risk_alerts_panel",
    "audit_panel",
    "provenance_drill_down",
    "manual_operation_entry_design",
    "gated_operation_boundary",
    "owner_approval_gate_required",
    "risk_gate_required",
    "audit_gate_required",
    "strict_provenance",
):
    require_true(capability, key)
require(capability.get("capability_expansion") == "read_only_first_trader_terminal_workbench", "capability expansion mismatch")

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
    "retry_replace_amend_flatten_allowed",
    "strategy_driven_production_execution_allowed",
    "multi_account_execution_allowed",
    "multi_venue_execution_allowed",
    "dashboard_order_controls_enabled",
    "dashboard_approval_controls_enabled",
    "dashboard_cancel_controls_enabled",
    "dashboard_retry_controls_enabled",
    "dashboard_submit_controls_enabled",
    "dashboard_replace_controls_enabled",
    "dashboard_amend_controls_enabled",
    "dashboard_flatten_controls_enabled",
    "trader_terminal_order_ticket_enabled",
    "product_grade_trading_terminal_claim",
):
    require_false(manifest.get("boundary_flags") or {}, key)

for text, label in ((release_notes, "release notes"), (readiness, "readiness report"), (scope, "scope")):
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
        "dashboard_order_controls_enabled = true",
    ):
        require(forbidden not in text, f"{label} contains forbidden expansion wording: {forbidden}")
PY

if [[ "${NTPRO_V220_SKIP_GITHUB_DEPENDENCY:-0}" != "1" ]]; then
  if ! command -v gh >/dev/null 2>&1; then
    fail "gh is required for GitHub dependency proof"
  fi
  if ! gh auth status >/dev/null 2>&1; then
    fail "gh auth is required for GitHub dependency proof"
  fi

  gh release view "$BASE_TAG" --repo atxinbao/NTPRO --json tagName,isDraft,isPrerelease,url >/tmp/ntpro-v220-base-release.json
  BASE_TAG="$BASE_TAG" python3 <<'PY'
import json
import os
from pathlib import Path

payload = json.loads(Path("/tmp/ntpro-v220-base-release.json").read_text(encoding="utf-8"))
if payload.get("tagName") != os.environ["BASE_TAG"]:
    raise SystemExit("base release tag mismatch")
if payload.get("isDraft") or payload.get("isPrerelease"):
    raise SystemExit("base release must be final")
PY

  for issue in 683 684 685 686 687 688 689; do
    state="$(gh issue view "$issue" --repo atxinbao/NTPRO --json state --jq .state)"
    [[ "$state" == "CLOSED" ]] || fail "V220 dependency issue #$issue is not closed"
  done

  current_state="$(gh issue view "$CURRENT_ISSUE" --repo atxinbao/NTPRO --json state --jq .state)"
  if [[ "${NTPRO_V220_REQUIRE_CURRENT_ISSUE_CLOSED:-0}" == "1" || "${NTPRO_RELEASE_GATE:-0}" == "1" ]]; then
    [[ "$current_state" == "CLOSED" ]] || fail "current V220 issue #$CURRENT_ISSUE is not closed"
  fi

  gh api "repos/atxinbao/NTPRO/milestones?state=all" --paginate --jq '
    def require(cond; msg): if cond then empty else error(msg) end;
    [.[] | select(.title=="v0.21.1" or .title=="v0.22.0")] as $m |
    require(($m | length) == 2; "missing v0.21.1/v0.22.0 milestones") |
    ($m[] | select(.title=="v0.21.1")) as $v211 |
    ($m[] | select(.title=="v0.22.0")) as $v220 |
    require(($v211.open_issues == 0); "v0.21.1 milestone still has open issues") |
    require(($v220.description | contains("#683-#690")); "v0.22.0 milestone missing #683-#690") |
    require(($v220.description | contains("Hard-blocked by v0.21.1")); "v0.22.0 milestone missing hard-blocked wording") |
    require(($v220.description | contains("read-only first")); "v0.22.0 milestone missing read-only-first wording") |
    require(($v220.description | contains("No ungated submit/cancel/retry/replace/amend/flatten")); "v0.22.0 milestone missing operation boundary wording") |
    "github milestone v220 proof ok"
  ' >/tmp/ntpro-v220-milestone-proof.txt
  cat /tmp/ntpro-v220-milestone-proof.txt
fi

echo "v22_release_gates status=ok version=$VERSION release_tag=$RELEASE_TAG base_tag=$BASE_TAG v220_evidence=complete workbench_evidence=complete read_only_first=true gated_operation_boundary=true current_issue_state=${current_state:-unknown}"
