#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

TRACE_PATH="${NTPRO_V270_ORCHESTRATION_TRACE:-tests/golden/v270_deployment_orchestration_foundation.jsonl}"
TASK_PATH="${NTPRO_V270_ORCHESTRATION_TASK:-docs/rust-cutover/tasks/V270-004.md}"
EVIDENCE_PATH="${NTPRO_V270_ORCHESTRATION_EVIDENCE:-docs/rust-cutover/evidence/V270-004.md}"
CONTRACT_PATH="${NTPRO_V270_ORCHESTRATION_CONTRACT:-docs/rust-cutover/release/v0_27_0_deployment_orchestration_foundation.md}"
BOUNDARY_PATH="${NTPRO_V270_ORCHESTRATION_BOUNDARY:-docs/rust-cutover/release/v0_27_0_product_operations_runtime_integration_boundary_contract.md}"
AUDIT_STORAGE_PATH="${NTPRO_V270_ORCHESTRATION_AUDIT_STORAGE:-docs/rust-cutover/release/v0_27_0_persistent_operation_audit_storage_foundation.md}"
V26_DEPLOYMENT_PATH="${NTPRO_V270_ORCHESTRATION_V26_DEPLOYMENT:-docs/rust-cutover/release/v0_26_0_deployment_provenance_model.md}"
V26_RUNBOOK_PATH="${NTPRO_V270_ORCHESTRATION_V26_RUNBOOK:-docs/rust-cutover/release/v0_26_0_upgrade_rollback_runbook_evidence.md}"
REPLAY_SCOPE_PATH="${NTPRO_V270_ORCHESTRATION_REPLAY_SCOPE:-docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"
SELFTEST="${NTPRO_V270_ORCHESTRATION_SELFTEST:-1}"

fail() {
  echo "v27 deployment orchestration foundation failed: $*" >&2
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

for path in "$TRACE_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$CONTRACT_PATH" "$BOUNDARY_PATH" "$AUDIT_STORAGE_PATH" "$V26_DEPLOYMENT_PATH" "$V26_RUNBOOK_PATH" "$REPLAY_SCOPE_PATH"; do
  require_file "$path"
done

require_contains "$TASK_PATH" "GitHub issue: \`#857\`"
require_contains "$EVIDENCE_PATH" "Task: \`V270-004\` / GitHub issue \`#857\`"
require_contains "$BOUNDARY_PATH" "release_scope = product_operations_runtime_integration_foundation_only"
require_contains "$AUDIT_STORAGE_PATH" "persistent_audit_storage_scope = operation_audit_storage_foundation_only"
require_contains "$V26_DEPLOYMENT_PATH" "deployment_provenance_scope = deployment_provenance_evidence_only"
require_contains "$V26_RUNBOOK_PATH" "runbook_artifact_scope = upgrade_rollback_runbook_preview_only"
require_contains "$CONTRACT_PATH" "orchestration_scope = deployment_upgrade_rollback_orchestration_foundation_only"
require_contains "$CONTRACT_PATH" "orchestration_mode = preview_first_gated"
require_contains "$CONTRACT_PATH" "dependency_contracts = V270-001,V270-002,V270-003,V260-004,V260-005"
require_contains "$CONTRACT_PATH" "owner_approval_required = true"
require_contains "$CONTRACT_PATH" "release_gate_evidence_required = true"
require_contains "$CONTRACT_PATH" "environment_provenance_required = true"
require_contains "$CONTRACT_PATH" "rollback_plan_lineage_required = true"
require_contains "$CONTRACT_PATH" "deploy_execution_allowed = false"
require_contains "$CONTRACT_PATH" "rollback_execution_allowed = false"
require_contains "$CONTRACT_PATH" "automatic_remediation_allowed = false"
require_contains "$CONTRACT_PATH" "adapter_send_allowed = false"
require_contains "$CONTRACT_PATH" "live_exchange_request_allowed = false"
require_contains "$CONTRACT_PATH" "dashboard_controls_enabled = false"

python3 - "$TRACE_PATH" "$REPLAY_SCOPE_PATH" "$SELFTEST" <<'PY'
from __future__ import annotations

import copy
import json
import sys
from pathlib import Path
from typing import Any

trace_path = Path(sys.argv[1])
replay_scope_path = Path(sys.argv[2])
selftest = sys.argv[3] != "0"

EXPECTED_CASES = [
    "read_model.deployment_orchestration.ready_preview.001",
    "read_model.deployment_orchestration.stale_environment_fail_closed.001",
    "read_model.deployment_orchestration.missing_approval_blocked.001",
    "read_model.deployment_orchestration.tag_source_mismatch_fail_closed.001",
    "read_model.deployment_orchestration.failed_preflight_fail_closed.001",
    "read_model.deployment_orchestration.unsafe_automation_fail_closed.001",
    "read_model.deployment_orchestration.forbidden_operation_boundary_fail_closed.001",
]
CONTRACT_VERSION = "ntpro.v270.deployment_orchestration_foundation.v1"
SCHEMA_VERSION = "ntpro.v270.deployment_orchestration_foundation.schema.v1"
ORCHESTRATION_SCOPE = "deployment_upgrade_rollback_orchestration_foundation_only"
HARNESS = "scripts/ai/verify_release.sh v27-deployment-orchestration-foundation"
DEPENDENCIES = ["V270-001", "V270-002", "V270-003", "V260-004", "V260-005"]
STATE_TYPES = ["deploy", "upgrade", "rollback", "post_check"]
BOUNDARY_FALSE_FLAGS = [
    "deploy_execution_allowed",
    "rollback_execution_allowed",
    "automatic_deploy_allowed",
    "automatic_rollback_allowed",
    "automatic_remediation_allowed",
    "adapter_send_allowed",
    "live_exchange_request_allowed",
    "retry_scheduler_enabled",
    "dashboard_operation_controls_enabled",
    "dashboard_trading_controls_enabled",
    "trading_operation_allowed",
    "manual_operation_submit_allowed",
    "product_grade_trading_terminal_claim",
]


def fail(message: str) -> None:
    raise SystemExit(f"v27 deployment orchestration foundation failed: {message}")


def non_empty(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


def push_reason(reasons: list[str], reason: str) -> None:
    if reason not in reasons:
        reasons.append(reason)


def load_rows(path: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    with path.open(encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, 1):
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            try:
                row = json.loads(line)
            except json.JSONDecodeError as exc:
                fail(f"{path}:{line_number}: invalid JSON: {exc}")
            if not isinstance(row, dict):
                fail(f"{path}:{line_number}: row must be an object")
            rows.append(row)
    return rows


def single_event(row: dict[str, Any], section: str, case_id: str) -> dict[str, Any]:
    try:
        events = row[section]["events"]
    except KeyError as exc:
        fail(f"{case_id}: missing {section}.events: {exc}")
    if not isinstance(events, list) or len(events) != 1 or not isinstance(events[0], dict):
        fail(f"{case_id}: {section}.events must contain exactly one object")
    return events[0]


def classify(artifact: dict[str, Any], case_id: str) -> dict[str, Any]:
    if artifact.get("contract_version") != CONTRACT_VERSION:
        fail(f"{case_id}: contract_version must be {CONTRACT_VERSION}")
    if artifact.get("schema_version") != SCHEMA_VERSION:
        fail(f"{case_id}: schema_version must be {SCHEMA_VERSION}")

    reasons: list[str] = list(artifact.get("blocking_reasons") or [])
    stale_environment = False
    missing_approval = False
    tag_source_mismatch = False
    failed_preflight = False
    unsafe_automation = False
    forbidden_boundary = False

    if artifact.get("orchestration_scope") != ORCHESTRATION_SCOPE:
        tag_source_mismatch = True
        push_reason(reasons, "orchestration_scope_mismatch")
    if artifact.get("orchestration_mode") != "preview_first_gated":
        unsafe_automation = True
        push_reason(reasons, f"orchestration_mode_not_preview_first:{artifact.get('orchestration_mode')}")
    if artifact.get("dependency_contracts") != DEPENDENCIES:
        tag_source_mismatch = True
        push_reason(reasons, "dependency_contracts_mismatch")

    boundary_flags = artifact.get("boundary_flags")
    if not isinstance(boundary_flags, dict):
        forbidden_boundary = True
        push_reason(reasons, "missing_boundary_flags")
        boundary_flags = {}
    for key in BOUNDARY_FALSE_FLAGS:
        if key not in boundary_flags:
            forbidden_boundary = True
            push_reason(reasons, f"missing_required_false_boundary:{key}")
        elif boundary_flags.get(key) is not False:
            forbidden_boundary = True
            push_reason(reasons, f"forbidden_boundary_flag:{key}")

    environment = artifact.get("environment_provenance")
    if not isinstance(environment, dict):
        stale_environment = True
        push_reason(reasons, "missing_environment_provenance")
        environment = {}
    for key in ["environment_id", "environment_provenance_ref", "release_tag", "source_ref", "artifact_digest", "runtime_boundary"]:
        if not non_empty(environment.get(key)):
            tag_source_mismatch = True
            push_reason(reasons, f"missing_environment_field:{key}")
    if environment.get("freshness_status") != "fresh":
        stale_environment = True
        push_reason(reasons, f"environment_freshness_not_fresh:{environment.get('freshness_status')}")
    if environment.get("release_tag") != artifact.get("release_tag"):
        tag_source_mismatch = True
        push_reason(reasons, f"release_tag_mismatch:{environment.get('release_tag')}!={artifact.get('release_tag')}")
    if environment.get("source_ref") != artifact.get("source_ref"):
        tag_source_mismatch = True
        push_reason(reasons, "source_ref_mismatch")

    owner_approval = artifact.get("owner_approval")
    if not isinstance(owner_approval, dict):
        missing_approval = True
        push_reason(reasons, "missing_owner_approval")
        owner_approval = {}
    if owner_approval.get("required") is not True:
        missing_approval = True
        push_reason(reasons, "owner_approval_not_required")
    if owner_approval.get("status") != "approved":
        missing_approval = True
        push_reason(reasons, f"owner_approval_not_approved:{owner_approval.get('status')}")
    if not non_empty(owner_approval.get("approval_ref")):
        missing_approval = True
        push_reason(reasons, "missing_owner_approval_ref")

    release_gate = artifact.get("release_gate_evidence")
    if not isinstance(release_gate, dict):
        tag_source_mismatch = True
        push_reason(reasons, "missing_release_gate_evidence")
        release_gate = {}
    if release_gate.get("status") != "passed":
        failed_preflight = True
        push_reason(reasons, f"release_gate_not_passed:{release_gate.get('status')}")
    if release_gate.get("release_tag") != artifact.get("release_tag"):
        tag_source_mismatch = True
        push_reason(reasons, f"release_gate_tag_mismatch:{release_gate.get('release_tag')}!={artifact.get('release_tag')}")

    rollback_plan = artifact.get("rollback_plan")
    if not isinstance(rollback_plan, dict):
        tag_source_mismatch = True
        push_reason(reasons, "missing_rollback_plan")
        rollback_plan = {}
    for key in ["plan_ref", "lineage_ref", "recovery_point_ref"]:
        if not non_empty(rollback_plan.get(key)):
            tag_source_mismatch = True
            push_reason(reasons, f"missing_rollback_plan_field:{key}")

    states = artifact.get("orchestration_states")
    if not isinstance(states, list) or not states:
        fail(f"{case_id}: orchestration_states must be a non-empty list")
    state_types_checked: list[str] = []
    for state in states:
        if not isinstance(state, dict):
            fail(f"{case_id}: orchestration state must be an object")
        state_type = str(state.get("state_type") or "unknown")
        state_types_checked.append(state_type)
        if state_type not in STATE_TYPES:
            tag_source_mismatch = True
            push_reason(reasons, f"unknown_state_type:{state_type}")
        if state.get("preview_only") is not True:
            unsafe_automation = True
            push_reason(reasons, f"state_not_preview_only:{state_type}")
        if state.get("execution_triggered") is not False:
            unsafe_automation = True
            push_reason(reasons, f"state_execution_triggered:{state_type}")
        if state.get("automatic_action_requested") is not False:
            unsafe_automation = True
            push_reason(reasons, f"automatic_action_requested:{state_type}")
        if state.get("preflight_status") != "passed":
            failed_preflight = True
            push_reason(reasons, f"preflight_not_passed:{state_type}:{state.get('preflight_status')}")
        if not non_empty(state.get("evidence_ref")):
            tag_source_mismatch = True
            push_reason(reasons, f"missing_state_evidence_ref:{state_type}")

    if stale_environment:
        effective_status = "fail_closed_stale_environment_provenance"
    elif missing_approval:
        effective_status = "blocked_preview_missing_approval"
    elif tag_source_mismatch:
        effective_status = "fail_closed_tag_or_source_mismatch"
    elif failed_preflight:
        effective_status = "fail_closed_failed_preflight"
    elif unsafe_automation:
        effective_status = "fail_closed_unsafe_automation_request"
    elif forbidden_boundary:
        effective_status = "fail_closed_forbidden_operation_boundary"
    elif reasons:
        effective_status = "fail_closed_tag_or_source_mismatch"
    else:
        effective_status = "orchestration_foundation_ready"

    return {
        "orchestration_scope": artifact.get("orchestration_scope"),
        "effective_orchestration_status": effective_status,
        "state_count": len(states),
        "state_types_checked": state_types_checked,
        "preview_first_gated": artifact.get("orchestration_mode") == "preview_first_gated",
        "environment_provenance_fresh": not stale_environment,
        "owner_approval_complete": not missing_approval,
        "release_gate_complete": not failed_preflight,
        "rollback_lineage_complete": not tag_source_mismatch,
        "execution_disabled": not unsafe_automation and not forbidden_boundary,
        "operation_boundary_closed": not forbidden_boundary,
        "fail_closed": effective_status != "orchestration_foundation_ready",
        "blocking_reasons": reasons,
    }


rows = load_rows(trace_path)
case_ids = [str(row.get("case_id")) for row in rows]
if case_ids != EXPECTED_CASES:
    fail(f"case order mismatch: {case_ids}")

healthy_artifact: dict[str, Any] | None = None
for row in rows:
    case_id = str(row.get("case_id"))
    if row.get("schema_version") != "golden-trace-v1":
        fail(f"{case_id}: schema_version must be golden-trace-v1")
    if row.get("category") != "read_model":
        fail(f"{case_id}: category must be read_model")
    input_event = single_event(row, "input", case_id)
    expected_event = single_event(row, "expected", case_id)
    if input_event.get("event_type") != "read_model.deployment_orchestration.input":
        fail(f"{case_id}: unexpected input event_type")
    if expected_event.get("event_type") != "read_model.deployment_orchestration.validated":
        fail(f"{case_id}: unexpected expected event_type")
    artifact = input_event.get("payload", {}).get("artifact")
    if not isinstance(artifact, dict):
        fail(f"{case_id}: input artifact must be an object")
    actual = classify(artifact, case_id)
    actual["case_id"] = case_id
    expected_payload = expected_event.get("payload")
    if actual != expected_payload:
        fail(
            f"{case_id}: expected payload mismatch\n"
            f"actual={json.dumps(actual, sort_keys=True)}\n"
            f"expected={json.dumps(expected_payload, sort_keys=True)}"
        )
    if case_id == EXPECTED_CASES[0]:
        healthy_artifact = artifact

if healthy_artifact is None:
    fail("missing healthy artifact")

if selftest:
    mutated = copy.deepcopy(healthy_artifact)
    mutated["orchestration_states"][0]["execution_triggered"] = True
    status = classify(mutated, "negative_selftest")["effective_orchestration_status"]
    if status != "fail_closed_unsafe_automation_request":
        fail(f"negative selftest did not fail closed: {status}")

scope = json.loads(replay_scope_path.read_text(encoding="utf-8"))
entries = scope.get("cases")
if not isinstance(entries, list):
    fail("release replay scope cases must be a list")
by_case = {entry.get("case_id"): entry for entry in entries if isinstance(entry, dict)}
for case_id in EXPECTED_CASES:
    entry = by_case.get(case_id)
    if not isinstance(entry, dict):
        fail(f"missing release replay scope entry: {case_id}")
    if entry.get("trace") != str(trace_path):
        fail(f"{case_id}: replay scope trace mismatch")
    if entry.get("status") != "validator_executable_replay":
        fail(f"{case_id}: replay scope status must be validator_executable_replay")
    if entry.get("evidence_id") != "V270-004":
        fail(f"{case_id}: replay scope evidence_id must be V270-004")
    if entry.get("harness") != HARNESS:
        fail(f"{case_id}: replay scope harness mismatch")
    for key in ("runtime_adapter_integration", "automatic_remediation_allowed", "adapter_send_allowed", "live_exchange_request_allowed", "dashboard_trading_controls_enabled"):
        if entry.get(key) is not False:
            fail(f"{case_id}: {key} must be false")

print(
    "v27_deployment_orchestration_foundation=pass "
    f"cases={len(rows)} states={len(healthy_artifact['orchestration_states'])} "
    f"boundary_flags={len(BOUNDARY_FALSE_FLAGS)} negative_selftest={1 if selftest else 0}"
)
PY
