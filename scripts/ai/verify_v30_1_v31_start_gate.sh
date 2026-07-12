#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

MODE="${1:-source}"
REPO="${NTPRO_V301_V31_START_GATE_REPO:-atxinbao/NTPRO}"
RELEASE_TAG="${NTPRO_V301_V31_START_GATE_RELEASE_TAG:-ntpro-rust-only-v0.30.1}"
RELEASE_NAME="${NTPRO_V301_V31_START_GATE_RELEASE_NAME:-NTPRO Rust-only v0.30.1}"
MILESTONE_TITLE="${NTPRO_V301_V31_START_GATE_MILESTONE_TITLE:-v0.30.1}"
NEXT_MILESTONE_TITLE="${NTPRO_V301_V31_START_GATE_NEXT_MILESTONE_TITLE:-v0.31.0}"

START_GATE_MD="${NTPRO_V301_V31_START_GATE_MD:-docs/rust-cutover/release/v0_30_1_v31_start_gate.md}"
START_GATE_JSON="${NTPRO_V301_V31_START_GATE_JSON:-docs/rust-cutover/release/v0_30_1_v31_start_gate.json}"
HANDOFF_MD="${NTPRO_V301_V31_HANDOFF_MD:-docs/rust-cutover/release/v0_30_0_v31_production_enablement_handoff.md}"
HANDOFF_JSON="${NTPRO_V301_V31_HANDOFF_JSON:-docs/rust-cutover/release/v0_30_0_v31_production_enablement_handoff.json}"
TASK_PATH="${NTPRO_V301_V31_START_GATE_TASK:-docs/rust-cutover/tasks/V301-006.md}"
EVIDENCE_PATH="${NTPRO_V301_V31_START_GATE_EVIDENCE:-docs/rust-cutover/evidence/V301-006.md}"
README_PATH="${NTPRO_V301_V31_START_GATE_README:-docs/rust-cutover/release/README.md}"

fail() {
  echo "v30.1 v31 start gate failed: $*" >&2
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
  "$START_GATE_MD" \
  "$START_GATE_JSON" \
  "$HANDOFF_MD" \
  "$HANDOFF_JSON" \
  "$TASK_PATH" \
  "$EVIDENCE_PATH" \
  "$README_PATH"; do
  require_file "$path"
done

run_source_validation() {
  START_GATE_MD="$START_GATE_MD" \
  START_GATE_JSON="$START_GATE_JSON" \
  HANDOFF_MD="$HANDOFF_MD" \
  HANDOFF_JSON="$HANDOFF_JSON" \
  TASK_PATH="$TASK_PATH" \
  EVIDENCE_PATH="$EVIDENCE_PATH" \
  README_PATH="$README_PATH" \
  RELEASE_TAG="$RELEASE_TAG" \
  RELEASE_NAME="$RELEASE_NAME" \
  python3 <<'PY'
import copy
import json
import os
from pathlib import Path


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def require_contains(text: str, marker: str, label: str) -> None:
    require(marker in text, f"{label} missing marker: {marker}")


def classify_start_gate(scenario: dict) -> str:
    boundary = scenario.get("boundary") or {}
    if scenario.get("v30_1_release_evidence_present") is not True:
        return "fail_closed_missing_v30_1_release_evidence"
    if scenario.get("v301_issues_closed") is not True:
        return "fail_closed_missing_v301_closeout"
    if any(value is not False for value in boundary.values()):
        return "fail_closed_inherited_execution"
    if scenario.get("explicit_scoped_approval") is not True:
        return "fail_closed_missing_scoped_approval"
    return "v31_start_gate_ready_for_scoped_intake"


start_gate = json.loads(Path(os.environ["START_GATE_JSON"]).read_text(encoding="utf-8"))
handoff = json.loads(Path(os.environ["HANDOFF_JSON"]).read_text(encoding="utf-8"))
start_gate_md = Path(os.environ["START_GATE_MD"]).read_text(encoding="utf-8")
handoff_md = Path(os.environ["HANDOFF_MD"]).read_text(encoding="utf-8")
task = Path(os.environ["TASK_PATH"]).read_text(encoding="utf-8")
evidence = Path(os.environ["EVIDENCE_PATH"]).read_text(encoding="utf-8")
readme = Path(os.environ["README_PATH"]).read_text(encoding="utf-8")

require(start_gate.get("schema_version") == "ntpro.v301.v31_start_gate.v1", "start gate schema mismatch")
require(start_gate.get("contract_version") == "ntpro.v301.v31_start_gate.v1", "start gate contract mismatch")
require(start_gate.get("task_id") == "V301-006", "start gate task mismatch")
require(start_gate.get("github_issue") == 1004, "start gate issue mismatch")
require(start_gate.get("milestone") == "v0.30.1", "start gate milestone mismatch")
require(start_gate.get("source_patch_track") == "v0.30.1", "source patch mismatch")
require(start_gate.get("next_capability_track") == "v0.31.0", "next capability mismatch")
require(start_gate.get("start_gate_status") == "blocked_until_v301_release_evidence_published", "start status mismatch")
release = start_gate.get("required_patch_release") or {}
require(release.get("tag") == os.environ["RELEASE_TAG"], "required release tag mismatch")
require(release.get("name") == os.environ["RELEASE_NAME"], "required release name mismatch")
require(release.get("source_controlled_closeout_required") is True, "source-controlled closeout requirement missing")
require(release.get("github_release_required") is True, "GitHub release requirement missing")
require(release.get("release_gate_success_required") is True, "release gate success requirement missing")
require(release.get("publication_after_gate_required") is True, "publication-after-gate requirement missing")

expected_issues = [999, 1000, 1001, 1002, 1003, 1004, 1005]
require(start_gate.get("required_v301_issues") == expected_issues, "V301 issue set mismatch")
require(start_gate.get("required_v301_issue_count") == len(expected_issues), "V301 issue count mismatch")
required_inputs = start_gate.get("required_future_inputs") or []
for marker in [
    "v30_1_release_closeout",
    "explicit_scoped_enablement_issue",
    "owner_operator_approval",
    "risk_gate",
    "audit_gate",
    "release_gate",
    "rollback_readiness",
    "telemetry_slo_gate",
    "no_default_trading_boundary",
]:
    require(marker in required_inputs, f"missing future input: {marker}")
require(len(required_inputs) == 9, "future input count mismatch")

boundary = start_gate.get("non_inheritance_boundary") or {}
require(boundary, "missing non-inheritance boundary")
for key, value in boundary.items():
    require(value is False, f"boundary must remain false: {key}")
require(start_gate.get("runtime_behavior_changed") is False, "runtime behavior must not change")
require(start_gate.get("trading_behavior_changed") is False, "trading behavior must not change")
require(start_gate.get("current_expected_status") == "blocked_until_v301_release_evidence_published", "current expected status mismatch")

for case in start_gate.get("readiness_cases") or []:
    scenario = {
        "v30_1_release_evidence_present": True,
        "v301_issues_closed": True,
        "explicit_scoped_approval": True,
        "boundary": copy.deepcopy(boundary),
    }
    for key in ("v30_1_release_evidence_present", "v301_issues_closed", "explicit_scoped_approval"):
        if key in case:
            scenario[key] = case[key]
    if case.get("boundary_flags_override"):
        scenario["boundary"].update(case["boundary_flags_override"])
    status = classify_start_gate(scenario)
    require(status == case.get("expected_status"), f"case {case.get('case_id')} expected {case.get('expected_status')} got {status}")

for marker in [
    "contract_version = ntpro.v301.v31_start_gate.v1",
    "required_patch_release = ntpro-rust-only-v0.30.1",
    "V301 required issue set = #999-#1005",
    "v0.30.1 GitHub Release evidence required = true",
    "v0.30.1 source-controlled closeout evidence required = true",
    "current_v31_start_status = blocked_until_v301_release_evidence_published",
    "v0.31.0 capability work may proceed = false",
]:
    require_contains(start_gate_md, marker, "start gate markdown")

require(handoff.get("schema_version") == "ntpro.v300.v31_production_enablement_handoff.v1", "handoff schema mismatch")
require(handoff.get("v31_start_gate_status") == "blocked_until_v301_release_evidence_published", "handoff start gate status mismatch")
require(handoff.get("v31_start_gate_contract") == os.environ["START_GATE_JSON"], "handoff start gate contract mismatch")
handoff_inputs = handoff.get("required_future_inputs") or []
require("v30_1_release_closeout" in handoff_inputs, "handoff missing v30.1 release closeout")
require(len(handoff_inputs) == 10, "handoff future input count mismatch")
gates = {gate.get("gate_id"): gate for gate in handoff.get("future_enablement_gates") or []}
for gate_id in ["v30_release_evidence", "v30_1_release_evidence", "explicit_scoped_issue", "risk_gate", "audit_gate", "release_gate"]:
    gate = gates.get(gate_id) or {}
    require(gate.get("required") is True, f"handoff gate not required: {gate_id}")
    require(gate.get("current_satisfied") is False, f"handoff gate unexpectedly satisfied: {gate_id}")
    require(gate.get("bypass_allowed") is False, f"handoff gate bypass allowed: {gate_id}")
for marker in [
    "v30_1_release_evidence_required = true",
    f"v31_start_gate_contract = {os.environ['START_GATE_JSON']}",
    "v30_1_release_closeout = required",
    "missing_v30_1_release_evidence => fail_closed_missing_v30_1_release_evidence",
]:
    require_contains(handoff_md, marker, "handoff markdown")

for label, text in {"task": task, "evidence": evidence, "README": readme}.items():
    require_contains(text, "V301-006", label)
    require_contains(text, "v31 start gate", label)

require_contains(evidence, "v0.30.1 release evidence required = true", "evidence")
require_contains(evidence, "v31 non-inheritance preserved = true", "evidence")
PY
}

run_live_validation() {
  command -v gh >/dev/null 2>&1 || fail "gh is required for live validation"
  gh auth status >/dev/null 2>&1 || fail "gh authentication is required for live validation"

  issues_json="$(gh_with_retry issue list --repo "$REPO" --state all --milestone "$MILESTONE_TITLE" --limit 100 --json number,state,title)"
  release_json=""
  release_present=0
  if release_json="$(gh_with_retry release view "$RELEASE_TAG" --repo "$REPO" --json tagName,name,isDraft,isPrerelease,url,publishedAt 2>/dev/null)"; then
    release_present=1
  fi
  next_issues_json="$(gh_with_retry issue list --repo "$REPO" --state all --milestone "$NEXT_MILESTONE_TITLE" --limit 100 --json number,state,title)"

  ISSUES_JSON="$issues_json" \
  NEXT_ISSUES_JSON="$next_issues_json" \
  RELEASE_JSON="$release_json" \
  RELEASE_PRESENT="$release_present" \
  RELEASE_TAG="$RELEASE_TAG" \
  RELEASE_NAME="$RELEASE_NAME" \
  python3 <<'PY'
import json
import os


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


issues = json.loads(os.environ["ISSUES_JSON"])
issue_map = {item["number"]: item for item in issues}
expected = list(range(999, 1006))
missing = [number for number in expected if number not in issue_map]
require(not missing, f"missing V301 issues from v0.30.1 milestone: {missing}")
unexpected = sorted(number for number in issue_map if 999 <= number <= 1005 and number not in expected)
require(not unexpected, f"unexpected V301 issue scope: {unexpected}")
all_closed = all(issue_map[number]["state"] == "CLOSED" for number in expected)

release_present = os.environ["RELEASE_PRESENT"] == "1"
if release_present:
    release = json.loads(os.environ["RELEASE_JSON"])
    require(release.get("tagName") == os.environ["RELEASE_TAG"], "release tag mismatch")
    require(release.get("name") == os.environ["RELEASE_NAME"], "release name mismatch")
    require(release.get("isDraft") is False, "release must not be draft")
    require(release.get("isPrerelease") is False, "release must not be prerelease")

next_issues = json.loads(os.environ["NEXT_ISSUES_JSON"])
expected_v31 = list(range(1006, 1016))
next_numbers = {item["number"] for item in next_issues}
missing_v31 = [number for number in expected_v31 if number not in next_numbers]
require(not missing_v31, f"missing v0.31.0 issues: {missing_v31}")

if not release_present:
    live_status = "fail_closed_missing_v30_1_release_evidence"
elif not all_closed:
    live_status = "fail_closed_missing_v301_closeout"
else:
    live_status = "fail_closed_missing_scoped_approval"

require(live_status != "v31_start_gate_ready_for_scoped_intake", "v31 start gate opened without scoped approval")
print(
    "v30_1_v31_start_gate_live "
    f"v301_issues={len(expected)} all_closed={str(all_closed).lower()} "
    f"v30_1_release_present={str(release_present).lower()} "
    f"live_status={live_status} v31_issues={len(expected_v31)}"
)
PY
}

case "$MODE" in
  source)
    run_source_validation
    ;;
  live)
    run_live_validation
    ;;
  all)
    run_source_validation
    run_live_validation
    ;;
  *)
    fail "unknown mode: $MODE"
    ;;
esac

echo "v30_1_v31_start_gate status=ok mode=$MODE current_status=blocked_until_v301_release_evidence_published v301_required_issues=7 v30_1_release_required=true non_inheritance=true"
