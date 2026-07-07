#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

TRACE_PATH="${NTPRO_V270_BOUNDARY_TRACE:-tests/golden/v270_product_operations_runtime_integration_boundary_contract.jsonl}"
TASK_PATH="${NTPRO_V270_BOUNDARY_TASK:-docs/rust-cutover/tasks/V270-001.md}"
EVIDENCE_PATH="${NTPRO_V270_BOUNDARY_EVIDENCE:-docs/rust-cutover/evidence/V270-001.md}"
CONTRACT_PATH="${NTPRO_V270_BOUNDARY_CONTRACT:-docs/rust-cutover/release/v0_27_0_product_operations_runtime_integration_boundary_contract.md}"
INTAKE_PATH="${NTPRO_V270_BOUNDARY_INTAKE:-docs/rust-cutover/release/v0_27_0_intake_gate.md}"
REPLAY_SCOPE_PATH="${NTPRO_V270_BOUNDARY_REPLAY_SCOPE:-docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"
SELFTEST="${NTPRO_V270_BOUNDARY_SELFTEST:-1}"

fail() {
  echo "v27 product operations runtime integration boundary contract failed: $*" >&2
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

for path in "$TRACE_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$CONTRACT_PATH" "$INTAKE_PATH" "$REPLAY_SCOPE_PATH"; do
  require_file "$path"
done

require_contains "$INTAKE_PATH" "start_gate_status = satisfied"
require_contains "$INTAKE_PATH" "v0.27.0 capability track = product_operations_runtime_integration_foundation_only"
require_contains "$TASK_PATH" "GitHub issue: \`#854\`"
require_contains "$EVIDENCE_PATH" "Task: \`V270-001\` / GitHub issue \`#854\`"
require_contains "$CONTRACT_PATH" "release_scope = product_operations_runtime_integration_foundation_only"
require_contains "$CONTRACT_PATH" "production_execution_runtime_claim = false"
require_contains "$CONTRACT_PATH" "product_grade_live_trading_terminal_claim = false"
require_contains "$CONTRACT_PATH" "source_provenance_required = true"
require_contains "$CONTRACT_PATH" "freshness_semantics_required = true"
require_contains "$CONTRACT_PATH" "redaction_required = true"
require_contains "$CONTRACT_PATH" "lineage_required = true"
require_contains "$CONTRACT_PATH" "failure_semantics = fail_closed"

for marker in \
  "new_submit_capability = false" \
  "production_order_submission_allowed = false" \
  "production_order_mutation_allowed = false" \
  "cancel_order_allowed = false" \
  "replace_order_allowed = false" \
  "amend_order_allowed = false" \
  "flatten_position_allowed = false" \
  "execution_adapter_call_allowed = false" \
  "adapter_send_allowed = false" \
  "live_exchange_request_allowed = false" \
  "retry_scheduler_enabled = false" \
  "automatic_remediation_allowed = false" \
  "dashboard_operation_controls_enabled = false" \
  "dashboard_trading_controls_enabled = false" \
  "trader_terminal_order_ticket_enabled = false" \
  "manual_operation_submit_allowed = false" \
  "product_grade_trading_terminal_claim = false"; do
  require_contains "$CONTRACT_PATH" "$marker"
done

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
    "read_model.product_operations_boundary.foundation_scope.healthy.001",
    "read_model.product_operations_boundary.source_provenance_fail_closed.001",
    "read_model.product_operations_boundary.freshness_stale_fail_closed.001",
    "read_model.product_operations_boundary.redaction_lineage_fail_closed.001",
    "read_model.product_operations_boundary.trading_boundary_fail_closed.001",
    "read_model.product_operations_boundary.terminal_claim_fail_closed.001",
    "read_model.product_operations_boundary.missing_required_false_fail_closed.001",
]

FOUNDATION_FLAGS = [
    "external_identity_permission_integration",
    "persistent_operation_audit_storage",
    "deployment_upgrade_rollback_orchestration",
    "long_run_telemetry_ingestion",
    "admin_workbench_state_bridge",
    "fail_closed_runtime_integration",
    "read_only_admin_surface",
]
PROVENANCE_TRUE_FLAGS = [
    "source_provenance_required",
    "freshness_semantics_required",
    "redaction_required",
    "lineage_required",
    "failure_semantics_fail_closed",
]
CAPABILITY_FALSE_FLAGS = [
    "production_execution_runtime_claim",
    "product_grade_live_trading_terminal_claim",
    "strategy_driven_production_execution",
    "shared_approval_consumption",
]
BOUNDARY_FALSE_FLAGS = [
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
    "retry_scheduler_enabled",
    "automatic_remediation_allowed",
    "dashboard_operation_controls_enabled",
    "dashboard_trading_controls_enabled",
    "trader_terminal_order_ticket_enabled",
    "manual_operation_submit_allowed",
    "product_grade_trading_terminal_claim",
]
CONTRACT_VERSION = "ntpro.v270.product_operations_runtime_boundary.v1"
SCHEMA_VERSION = "ntpro.v270.product_operations_runtime_boundary.schema.v1"
RELEASE_SCOPE = "product_operations_runtime_integration_foundation_only"
RELEASE_CLAIM = "product_operations_runtime_integration_foundation"
HARNESS = "scripts/ai/verify_release.sh v27-product-operations-boundary-contract"


def fail(message: str) -> None:
    raise SystemExit(f"v27 product operations runtime integration boundary contract failed: {message}")


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


def push_reason(reasons: list[str], reason: str) -> None:
    if reason not in reasons:
        reasons.append(reason)


def classify(snapshot: dict[str, Any], case_id: str) -> dict[str, Any]:
    if snapshot.get("contract_version") != CONTRACT_VERSION:
        fail(f"{case_id}: contract_version must be {CONTRACT_VERSION}")
    if snapshot.get("schema_version") != SCHEMA_VERSION:
        fail(f"{case_id}: schema_version must be {SCHEMA_VERSION}")

    reasons: list[str] = list(snapshot.get("blocking_reasons") or [])
    if any(not isinstance(reason, str) or not reason.strip() for reason in reasons):
        fail(f"{case_id}: blocking_reasons must be non-empty strings")

    capability_flags = snapshot.get("capability_flags")
    provenance_flags = snapshot.get("provenance_flags")
    boundary_flags = snapshot.get("boundary_flags")
    if not isinstance(capability_flags, dict):
        fail(f"{case_id}: capability_flags must be an object")
    if not isinstance(provenance_flags, dict):
        fail(f"{case_id}: provenance_flags must be an object")
    if not isinstance(boundary_flags, dict):
        fail(f"{case_id}: boundary_flags must be an object")

    foundation_scope_complete = True
    capability_flags_complete = True
    provenance_flags_complete = True
    boundary_flags_required_false = True
    opened: list[str] = []
    missing: list[str] = []

    if snapshot.get("dependency_start_gate") != "satisfied":
        foundation_scope_complete = False
        push_reason(reasons, "v270_intake_gate_not_satisfied")
    if snapshot.get("release_scope") != RELEASE_SCOPE:
        foundation_scope_complete = False
        push_reason(reasons, "product_operations_scope_mismatch")
    if snapshot.get("release_claim") != RELEASE_CLAIM:
        foundation_scope_complete = False
        push_reason(reasons, f"release_claim_mismatch:{snapshot.get('release_claim')}")

    for key in FOUNDATION_FLAGS:
        if capability_flags.get(key) is not True:
            foundation_scope_complete = False
            capability_flags_complete = False
            push_reason(reasons, f"missing_foundation_capability:{key}")

    for key in PROVENANCE_TRUE_FLAGS:
        if provenance_flags.get(key) is not True:
            provenance_flags_complete = False
            missing.append(f"provenance_flags.{key}")
            push_reason(reasons, f"missing_required_provenance:{key}")

    if provenance_flags.get("freshness_status") != "fresh":
        provenance_flags_complete = False
        opened.append("provenance_flags.freshness_status")
        push_reason(reasons, f"stale_freshness:{provenance_flags.get('freshness_status')}")
    if provenance_flags.get("redaction_status") != "redacted":
        provenance_flags_complete = False
        opened.append("provenance_flags.redaction_status")
        push_reason(reasons, f"redaction_violation:{provenance_flags.get('redaction_status')}")
    if provenance_flags.get("lineage_status") != "linked":
        provenance_flags_complete = False
        opened.append("provenance_flags.lineage_status")
        push_reason(reasons, f"lineage_violation:{provenance_flags.get('lineage_status')}")

    for key in CAPABILITY_FALSE_FLAGS:
        if key not in capability_flags:
            capability_flags_complete = False
            missing.append(f"capability_flags.{key}")
            push_reason(reasons, f"missing_required_false_capability:{key}")
        elif capability_flags.get(key) is not False:
            capability_flags_complete = False
            opened.append(f"capability_flags.{key}")
            push_reason(reasons, f"forbidden_capability_flag:{key}")

    for key in BOUNDARY_FALSE_FLAGS:
        if key not in boundary_flags:
            boundary_flags_required_false = False
            missing.append(f"boundary_flags.{key}")
            push_reason(reasons, f"missing_required_false_boundary:{key}")
        elif boundary_flags.get(key) is not False:
            boundary_flags_required_false = False
            opened.append(f"boundary_flags.{key}")
            push_reason(reasons, f"forbidden_boundary_flag:{key}")

    fail_closed = bool(
        opened
        or missing
        or not foundation_scope_complete
        or not capability_flags_complete
        or not provenance_flags_complete
    )
    status = "fail_closed_boundary_violation" if fail_closed else "foundation_ready"

    return {
        "case_id": case_id,
        "release_scope": snapshot.get("release_scope"),
        "effective_product_operations_status": status,
        "foundation_scope_complete": foundation_scope_complete,
        "capability_flags_complete": capability_flags_complete,
        "provenance_flags_complete": provenance_flags_complete,
        "boundary_flags_required_false": boundary_flags_required_false,
        "forbidden_capability_opened": bool(opened),
        "product_operations_foundation_only": snapshot.get("release_scope") == RELEASE_SCOPE,
        "production_execution_runtime_claim_allowed": False,
        "product_grade_terminal_claim_allowed": False,
        "fail_closed": fail_closed,
        "forbidden_fields_opened": opened,
        "missing_required_fields": missing,
        "blocking_reasons": reasons,
    }


rows = load_rows(trace_path)
if [row.get("case_id") for row in rows] != EXPECTED_CASES:
    fail(
        "case order mismatch: expected "
        + ", ".join(EXPECTED_CASES)
        + " got "
        + ", ".join(str(row.get("case_id")) for row in rows)
    )

healthy_snapshot: dict[str, Any] | None = None
for row in rows:
    case_id = str(row.get("case_id"))
    input_event = single_event(row, "input", case_id)
    expected_event = single_event(row, "expected", case_id)
    snapshot = input_event.get("payload", {}).get("snapshot")
    if not isinstance(snapshot, dict):
        fail(f"{case_id}: input snapshot must be an object")
    actual = classify(snapshot, case_id)
    expected = expected_event.get("payload")
    if actual != expected:
        fail(
            f"{case_id}: expected payload mismatch\n"
            + "actual="
            + json.dumps(actual, sort_keys=True)
            + "\nexpected="
            + json.dumps(expected, sort_keys=True)
        )
    if case_id.endswith("healthy.001"):
        healthy_snapshot = snapshot

if healthy_snapshot is None:
    fail("missing healthy baseline")

scope = json.loads(replay_scope_path.read_text(encoding="utf-8"))
cases = scope.get("cases")
if not isinstance(cases, list):
    fail("release replay scope cases must be a list")
scope_by_case = {case.get("case_id"): case for case in cases if isinstance(case, dict)}
for case_id in EXPECTED_CASES:
    entry = scope_by_case.get(case_id)
    if not entry:
        fail(f"missing release replay scope entry: {case_id}")
    if entry.get("status") != "validator_executable_replay":
        fail(f"{case_id}: release replay scope status mismatch")
    if entry.get("harness") != HARNESS:
        fail(f"{case_id}: release replay scope harness mismatch")
    if entry.get("evidence_id") != "V270-001":
        fail(f"{case_id}: release replay scope evidence mismatch")
    if entry.get("adapter_send_allowed") is not False or entry.get("live_exchange_request_allowed") is not False:
        fail(f"{case_id}: release replay scope must keep adapter/live exchange disabled")
    if entry.get("dashboard_trading_controls_enabled") is not False:
        fail(f"{case_id}: release replay scope must keep Dashboard trading controls disabled")
    if entry.get("product_grade_live_trading_terminal") is not False:
        fail(f"{case_id}: release replay scope must keep product-grade terminal false")

if selftest:
    opened = copy.deepcopy(healthy_snapshot)
    opened["boundary_flags"]["adapter_send_allowed"] = True
    opened_result = classify(opened, "negative.adapter_send")
    if not opened_result["fail_closed"]:
        fail("negative self-test unexpectedly allowed adapter_send_allowed")

    missing = copy.deepcopy(healthy_snapshot)
    del missing["provenance_flags"]["source_provenance_required"]
    missing_result = classify(missing, "negative.missing_provenance")
    if not missing_result["fail_closed"]:
        fail("negative self-test unexpectedly allowed missing source provenance")

print(
    "v27_product_operations_boundary_contract=pass "
    f"cases={len(rows)} required_false_flags={len(BOUNDARY_FALSE_FLAGS)} "
    f"provenance_flags={len(PROVENANCE_TRUE_FLAGS)} negative_selftest={int(selftest)}"
)
PY
