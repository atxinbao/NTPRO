#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

VERSION="${NTPRO_V21_VERSION:-v0.21.0}"
RELEASE_TAG="${NTPRO_V21_RELEASE_TAG:-ntpro-rust-only-v0.21.0}"
BASE_TAG="${NTPRO_V21_BASE_TAG:-ntpro-rust-only-v0.20.1}"
MANIFEST_PATH="${NTPRO_V21_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_21_0_release_manifest.json}"
RELEASE_NOTES_PATH="${NTPRO_V21_RELEASE_NOTES:-docs/rust-cutover/release/v0_21_0_release_notes.md}"
READINESS_REPORT_PATH="${NTPRO_V21_READINESS_REPORT:-docs/rust-cutover/release/v0_21_0_readiness_report.md}"
SCHEMA_PATH="${NTPRO_V21_SCHEMA:-docs/rust-cutover/release/v0_21_0_unified_read_model_schema.json}"
GOLDEN_SCOPE_PATH="${NTPRO_V21_GOLDEN_SCOPE:-docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"
GATE_ROOT="${NTPRO_V21_RELEASE_GATE_ROOT:-target/ntpro-v210/v21-release-gates}"

fail() {
  echo "v21 release gate failed: $*" >&2
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
require_file "$SCHEMA_PATH"
require_file "$GOLDEN_SCOPE_PATH"

for task_id in V210-000 V210-001 V210-002 V210-003 V210-004 V210-005 V210-006 V210-007 V210-008; do
  require_file "docs/rust-cutover/evidence/${task_id}.md"
  require_contains "docs/rust-cutover/evidence/${task_id}.md" "$task_id"
done

for script in \
  scripts/ai/verify_v21_read_model_contract.sh \
  scripts/ai/verify_v21_account_snapshot_read_model.sh \
  scripts/ai/verify_v21_position_read_model.sh \
  scripts/ai/verify_v21_order_lifecycle_read_model.sh \
  scripts/ai/verify_v21_fill_execution_read_model.sh \
  scripts/ai/verify_v21_risk_state_projection.sh \
  scripts/ai/verify_v21_trader_terminal_readonly_dashboard.sh; do
  require_file "$script"
done

scripts/ai/verify_v21_read_model_contract.sh
scripts/ai/verify_v21_account_snapshot_read_model.sh
scripts/ai/verify_v21_position_read_model.sh
scripts/ai/verify_v21_order_lifecycle_read_model.sh
scripts/ai/verify_v21_fill_execution_read_model.sh
scripts/ai/verify_v21_risk_state_projection.sh
scripts/ai/verify_v21_trader_terminal_readonly_dashboard.sh
python3 scripts/ai/validate_golden_trace_release_scope.py --manifest "$GOLDEN_SCOPE_PATH" --trace-glob 'tests/golden/*.jsonl'

for marker in \
  "Status: RELEASED" \
  "Tag: \`$RELEASE_TAG\`" \
  "Release name: \`NTPRO Rust-only $VERSION\`" \
  "Unified Read Model Foundation" \
  "scripts/ai/verify_release.sh v21-release-gates" \
  "scripts/ai/verify_release.sh v21-strict-provenance" \
  "product-grade live trading terminal readiness" \
  "Dashboard order, approval, cancel, retry, submit, replace, amend, flatten, or order-ticket controls"; do
  require_contains "$RELEASE_NOTES_PATH" "$marker"
done

for marker in \
  "Milestone: \`$RELEASE_TAG\`" \
  "Status: RELEASED" \
  "V210-000 evidence" \
  "V210-008 evidence" \
  "release scope manifest cases = 83" \
  "read model schema-only cases = 32" \
  "product_grade_trading_terminal_claim = false"; do
  require_contains "$READINESS_REPORT_PATH" "$marker"
done

VERSION="$VERSION" \
RELEASE_TAG="$RELEASE_TAG" \
BASE_TAG="$BASE_TAG" \
MANIFEST_PATH="$MANIFEST_PATH" \
RELEASE_NOTES_PATH="$RELEASE_NOTES_PATH" \
READINESS_REPORT_PATH="$READINESS_REPORT_PATH" \
SCHEMA_PATH="$SCHEMA_PATH" \
GOLDEN_SCOPE_PATH="$GOLDEN_SCOPE_PATH" \
GATE_ROOT="$GATE_ROOT" \
python3 <<'PY'
import json
import os
from pathlib import Path

manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))
schema = json.loads(Path(os.environ["SCHEMA_PATH"]).read_text(encoding="utf-8"))
scope = json.loads(Path(os.environ["GOLDEN_SCOPE_PATH"]).read_text(encoding="utf-8"))
release_notes = Path(os.environ["RELEASE_NOTES_PATH"]).read_text(encoding="utf-8")
readiness = Path(os.environ["READINESS_REPORT_PATH"]).read_text(encoding="utf-8")

def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)

def require_false(mapping: dict, key: str) -> None:
    require(mapping.get(key) is False, f"boundary flag must be false: {key}")

require(manifest.get("schema_version") == "ntpro.v210_release_manifest.v1", "manifest schema mismatch")
require(manifest.get("task_id") == "V210-008", "manifest task_id mismatch")
require(manifest.get("product_version") == os.environ["VERSION"], "manifest version mismatch")
require(
    manifest.get("release_status") in {"published_in_source_tree", "published_closeout_complete"},
    "manifest status mismatch",
)
require(schema.get("schema_version") == "ntpro.v210.unified_read_model.schema.v1", "schema version mismatch")

base = manifest.get("base_release") or {}
require(base.get("tag") == os.environ["BASE_TAG"], "base tag mismatch")
require(base.get("github_release_url") == f"https://github.com/atxinbao/NTPRO/releases/tag/{os.environ['BASE_TAG']}", "base release URL mismatch")
require(base.get("release_gate_run") == 28452719493, "base release gate run mismatch")

planned = manifest.get("planned_release") or {}
require(planned.get("tag") == os.environ["RELEASE_TAG"], "planned tag mismatch")
require(planned.get("name") == f"NTPRO Rust-only {os.environ['VERSION']}", "planned name mismatch")
require(planned.get("github_release_url") == f"https://github.com/atxinbao/NTPRO/releases/tag/{os.environ['RELEASE_TAG']}", "planned URL mismatch")
require(planned.get("target_commitish") == "main", "target_commitish mismatch")
require(planned.get("draft") is False, "draft flag mismatch")
require(planned.get("prerelease") is False, "prerelease flag mismatch")

expected_evidence = {
    "V210-000": 651,
    "V210-001": 652,
    "V210-002": 653,
    "V210-003": 654,
    "V210-004": 655,
    "V210-005": 656,
    "V210-006": 657,
    "V210-007": 658,
    "V210-008": 659,
}
evidence = manifest.get("v210_evidence") or []
require(len(evidence) == len(expected_evidence), "V210 evidence count mismatch")
for item in evidence:
    task_id = item.get("task_id")
    require(expected_evidence.get(task_id) == item.get("issue"), f"V210 evidence issue mismatch: {task_id}")
    path = Path(item.get("path", ""))
    require(path.is_file(), f"evidence file missing: {path}")
    require(task_id in path.read_text(encoding="utf-8"), f"evidence task marker missing: {path}")

trace_paths = set(manifest.get("read_model_traces") or [])
for path in (
    "tests/golden/read_model_contract_schema.jsonl",
    "tests/golden/read_model_account_snapshot_schema.jsonl",
    "tests/golden/read_model_position_schema.jsonl",
    "tests/golden/read_model_order_lifecycle_schema.jsonl",
    "tests/golden/read_model_fill_execution_schema.jsonl",
    "tests/golden/read_model_risk_state_schema.jsonl",
    "tests/golden/read_model_dashboard_schema.jsonl",
):
    require(path in trace_paths, f"read model trace missing from manifest: {path}")
    require(Path(path).is_file(), f"read model trace file missing: {path}")

commands = {
    gate.get("command")
    for gate in manifest.get("release_gates", [])
    if gate.get("required") is True
}
for command in (
    "scripts/ai/verify_release.sh v21-read-model-contract",
    "scripts/ai/verify_release.sh v21-account-snapshot-read-model",
    "scripts/ai/verify_release.sh v21-position-read-model",
    "scripts/ai/verify_release.sh v21-order-lifecycle-read-model",
    "scripts/ai/verify_release.sh v21-fill-execution-read-model",
    "scripts/ai/verify_release.sh v21-risk-state-projection",
    "scripts/ai/verify_release.sh v21-trader-terminal-readonly-dashboard",
    "scripts/ai/verify_release.sh v21-release-gates",
    "scripts/ai/verify_release.sh v21-strict-provenance",
    "scripts/ai/verify_release_strict.sh v21",
):
    require(command in commands, f"required release gate missing: {command}")

capability = manifest.get("capability") or {}
require(capability.get("capability_expansion") == "unified_read_model_foundation", "capability expansion mismatch")
require(capability.get("trader_terminal_scope") == "read_only_foundation", "terminal scope mismatch")

for key in (
    "new_submit_capability",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
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

cases = scope.get("cases") or []
require(len(cases) == 83, "golden trace release scope case count mismatch")
read_model_cases = [case for case in cases if case.get("category") == "read_model"]
require(len(read_model_cases) == 32, "read model release case count mismatch")
read_model_executable_cases = [
    case for case in read_model_cases
    if case.get("status") == "executable_replay"
]
read_model_schema_only_cases = [
    case for case in read_model_cases
    if case.get("status") == "schema_only_scoped"
]
expected_executable = {
    "read_model.account_snapshot.fresh.001",
    "read_model.account_snapshot.stale.001",
    "read_model.order_lifecycle.matched.001",
    "read_model.order_lifecycle.missing_ledger.001",
    "read_model.risk_state.healthy.001",
    "read_model.risk_state.mismatch.001",
    "read_model.dashboard.readonly_complete.001",
    "read_model.dashboard.missing_evidence_degraded.001",
}
actual_executable = {case.get("case_id") for case in read_model_executable_cases}
require(actual_executable == expected_executable, "read model executable replay case set mismatch")
require(len(read_model_schema_only_cases) == 24, "read model schema-only case count mismatch")
owners = {case.get("scope_owner") for case in read_model_cases}
for owner in ("V210-001", "V210-002", "V210-003", "V210-004", "V210-005", "V210-006", "V210-007"):
    require(owner in owners, f"read model scope owner missing: {owner}")

for text, label in ((release_notes, "release notes"), (readiness, "readiness report")):
    for forbidden in (
        "new production submit capability = true",
        "production order mutation = true",
        "implicit retry allowed",
        "automatic cancel allowed",
        "automatic remediation allowed",
        "product-grade live trading terminal readiness = true",
    ):
        require(forbidden not in text, f"{label} contains forbidden expansion wording: {forbidden}")
PY

if [[ "${NTPRO_V21_SKIP_GITHUB_DEPENDENCY:-0}" != "1" ]]; then
  if ! command -v gh >/dev/null 2>&1; then
    fail "gh is required for v21 dependency proof"
  fi
  if ! gh auth status >/dev/null 2>&1; then
    fail "gh auth is required for v21 dependency proof"
  fi

  gh release view "$BASE_TAG" --repo atxinbao/NTPRO --json tagName,isDraft,isPrerelease,url >/tmp/ntpro-v21-base-release.json
  BASE_TAG="$BASE_TAG" python3 <<'PY'
import json
import os
from pathlib import Path
payload = json.loads(Path("/tmp/ntpro-v21-base-release.json").read_text(encoding="utf-8"))
if payload.get("tagName") != os.environ["BASE_TAG"]:
    raise SystemExit("base release tag mismatch")
if payload.get("isDraft") or payload.get("isPrerelease"):
    raise SystemExit("base release must be final")
PY

  for issue in 644 645 646 647 648 649 650; do
    state="$(gh issue view "$issue" --repo atxinbao/NTPRO --json state --jq .state)"
    [[ "$state" == "CLOSED" ]] || fail "V201 dependency issue #$issue is not closed"
  done
  for issue in 651 652 653 654 655 656 657 658; do
    state="$(gh issue view "$issue" --repo atxinbao/NTPRO --json state --jq .state)"
    [[ "$state" == "CLOSED" ]] || fail "V210 prerequisite issue #$issue is not closed"
  done
  if [[ "${NTPRO_V21_REQUIRE_ALL_ISSUES_CLOSED:-0}" == "1" || "${GITHUB_REF_TYPE:-}" == "tag" ]]; then
    state="$(gh issue view 659 --repo atxinbao/NTPRO --json state --jq .state)"
    [[ "$state" == "CLOSED" ]] || fail "V210 final issue #659 is not closed"
  fi
fi

rm -rf "$GATE_ROOT"
mkdir -p "$GATE_ROOT"
python3 - <<'PY' > "$GATE_ROOT/v21-release-gate-summary.json"
import json
payload = {
    "status": "ok",
    "target": "v21-release-gates",
    "product_version": "v0.21.0",
    "release_tag": "ntpro-rust-only-v0.21.0",
    "read_model_executable_replay_cases": 8,
    "read_model_schema_only_cases": 24,
    "release_scope_cases": 83,
    "unified_read_model_foundation": True,
    "read_only_foundation": True,
    "new_submit_capability": False,
    "product_grade_trading_terminal_claim": False,
}
print(json.dumps(payload, indent=2, sort_keys=True))
PY

echo "v21_release_gates status=ok version=$VERSION release_tag=$RELEASE_TAG base_tag=$BASE_TAG read_model_cases=32 release_scope_cases=83 unified_read_model_foundation=true read_only_foundation=true new_submit_capability=false product_grade_trading_terminal_claim=false"
