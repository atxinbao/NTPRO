#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

TRACE_PATH="${NTPRO_V260_PRODUCT_HARDENING_TRACE:-tests/golden/v260_product_hardening_boundary_contract.jsonl}"
TASK_PATH="${NTPRO_V260_PRODUCT_HARDENING_TASK:-docs/rust-cutover/tasks/V260-001.md}"
EVIDENCE_PATH="${NTPRO_V260_PRODUCT_HARDENING_EVIDENCE:-docs/rust-cutover/evidence/V260-001.md}"
CONTRACT_PATH="${NTPRO_V260_PRODUCT_HARDENING_CONTRACT:-docs/rust-cutover/release/v0_26_0_product_hardening_boundary_contract.md}"
INTAKE_PATH="${NTPRO_V260_PRODUCT_HARDENING_INTAKE:-docs/rust-cutover/release/v0_26_0_intake_gate.md}"
REPLAY_SCOPE_PATH="${NTPRO_V260_PRODUCT_HARDENING_REPLAY_SCOPE:-docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"
SELFTEST="${NTPRO_V260_PRODUCT_HARDENING_SELFTEST:-1}"

fail() {
  echo "v26 product hardening boundary contract failed: $*" >&2
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
require_contains "$INTAKE_PATH" "v0.26.0 capability track = product_hardening_foundation_only"
require_contains "$TASK_PATH" "GitHub issue: \`#813\`"
require_contains "$EVIDENCE_PATH" "Task: \`V260-001\` / GitHub issue \`#813\`"
require_contains "$CONTRACT_PATH" "release_scope = product_hardening_foundation_only"
require_contains "$CONTRACT_PATH" "product_grade_trading_system_claim = false"
require_contains "$CONTRACT_PATH" "product_grade_live_trading_terminal_claim = false"
require_contains "$CONTRACT_PATH" "v0.26.0 scope covers real trading execution = false"

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
    "read_model.product_hardening_boundary.foundation_scope.healthy.001",
    "read_model.product_hardening_boundary.submit_enabled_fail_closed.001",
    "read_model.product_hardening_boundary.mutation_enabled_fail_closed.001",
    "read_model.product_hardening_boundary.adapter_send_fail_closed.001",
    "read_model.product_hardening_boundary.automatic_remediation_fail_closed.001",
    "read_model.product_hardening_boundary.dashboard_trading_controls_fail_closed.001",
    "read_model.product_hardening_boundary.terminal_claim_fail_closed.001",
    "read_model.product_hardening_boundary.missing_required_false_fail_closed.001",
]

FOUNDATION_FLAGS = [
    "permissions_boundary_contract",
    "operation_audit",
    "deployment_provenance",
    "upgrade_rollback_runbook",
    "slo_runbook_productization",
    "long_run_stability_evidence",
    "read_only_admin_dashboard",
]
CAPABILITY_FALSE_FLAGS = [
    "external_identity_provider_integration",
    "production_execution_capability",
    "product_grade_trading_system_claim",
    "live_trading_terminal_claim",
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
CONTRACT_VERSION = "ntpro.v260.product_hardening_boundary.v1"
SCHEMA_VERSION = "ntpro.v260.product_hardening_boundary.schema.v1"
RELEASE_SCOPE = "product_hardening_foundation_only"
RELEASE_CLAIM = "product_hardening_foundation"
HARNESS = "scripts/ai/verify_release.sh v26-product-hardening-boundary-contract"


def fail(message: str) -> None:
    raise SystemExit(f"v26 product hardening boundary contract failed: {message}")


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
    if not isinstance(capability_flags, dict):
        fail(f"{case_id}: capability_flags must be an object")
    boundary_flags = snapshot.get("boundary_flags")
    if not isinstance(boundary_flags, dict):
        fail(f"{case_id}: boundary_flags must be an object")

    foundation_scope_complete = True
    capability_flags_complete = True
    boundary_flags_required_false = True
    opened: list[str] = []
    missing: list[str] = []

    if snapshot.get("dependency_start_gate") != "satisfied":
        foundation_scope_complete = False
        push_reason(reasons, "v260_intake_gate_not_satisfied")
    if snapshot.get("release_scope") != RELEASE_SCOPE:
        foundation_scope_complete = False
        push_reason(reasons, "product_hardening_scope_mismatch")
    if snapshot.get("release_claim") != RELEASE_CLAIM:
        foundation_scope_complete = False
        push_reason(reasons, f"release_claim_mismatch:{snapshot.get('release_claim')}")

    for key in FOUNDATION_FLAGS:
        if capability_flags.get(key) is not True:
            foundation_scope_complete = False
            capability_flags_complete = False
            push_reason(reasons, f"missing_foundation_capability:{key}")

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

    fail_closed = bool(opened or missing or not foundation_scope_complete or not capability_flags_complete)
    status = "fail_closed_boundary_violation" if fail_closed else "foundation_ready"
    product_grade_claim_allowed = False
    production_trading_execution_allowed = False

    return {
        "case_id": case_id,
        "release_scope": snapshot.get("release_scope"),
        "effective_product_hardening_status": status,
        "foundation_scope_complete": foundation_scope_complete,
        "capability_flags_complete": capability_flags_complete,
        "boundary_flags_required_false": boundary_flags_required_false,
        "forbidden_capability_opened": bool(opened),
        "product_hardening_foundation_only": snapshot.get("release_scope") == RELEASE_SCOPE,
        "product_grade_trading_claim_allowed": product_grade_claim_allowed,
        "production_trading_execution_allowed": production_trading_execution_allowed,
        "fail_closed": fail_closed,
        "forbidden_fields_opened": opened,
        "missing_required_false_fields": missing,
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
    if row.get("schema_version") != "golden-trace-v1":
        fail(f"{case_id}: schema_version must be golden-trace-v1")
    if row.get("category") != "read_model":
        fail(f"{case_id}: category must be read_model")
    if not isinstance(row.get("description"), str) or not row["description"].strip():
        fail(f"{case_id}: description is required")

    input_event = single_event(row, "input", case_id)
    expected_event = single_event(row, "expected", case_id)
    expected_event_type = input_event.get("event_type", "").replace(".input", ".validated")
    if expected_event.get("event_type") != expected_event_type:
        fail(f"{case_id}: expected event_type must be {expected_event_type}")
    for key in ("ts_event", "ts_init", "instrument_id", "venue", "correlation_id"):
        if expected_event.get(key) != input_event.get(key):
            fail(f"{case_id}: expected.{key} must match input.{key}")

    payload = input_event.get("payload")
    if not isinstance(payload, dict) or not isinstance(payload.get("snapshot"), dict):
        fail(f"{case_id}: input payload snapshot is required")
    snapshot = payload["snapshot"]
    computed = classify(snapshot, case_id)
    expected_payload = expected_event.get("payload")
    if computed != expected_payload:
        fail(
            f"{case_id}: computed payload mismatch\n"
            f"expected={json.dumps(expected_payload, sort_keys=True)}\n"
            f"actual={json.dumps(computed, sort_keys=True)}"
        )
    if computed["effective_product_hardening_status"] == "foundation_ready" and computed["fail_closed"]:
        fail(f"{case_id}: foundation_ready cannot be fail_closed")
    if case_id.endswith("foundation_scope.healthy.001"):
        healthy_snapshot = copy.deepcopy(snapshot)

if selftest:
    if healthy_snapshot is None:
        fail("negative selftest requires healthy fixture")
    healthy_snapshot["boundary_flags"]["adapter_send_allowed"] = True
    closed = classify(healthy_snapshot, "negative.selftest.adapter_send_opened")
    if not closed["fail_closed"]:
        fail("negative selftest opened adapter_send_allowed but did not fail closed")
    if "forbidden_boundary_flag:adapter_send_allowed" not in closed["blocking_reasons"]:
        fail("negative selftest did not surface adapter_send_allowed boundary reason")

scope = json.loads(replay_scope_path.read_text(encoding="utf-8"))
cases = {case.get("case_id"): case for case in scope.get("cases", [])}
for case_id in EXPECTED_CASES:
    entry = cases.get(case_id)
    if not isinstance(entry, dict):
        fail(f"missing release replay scope entry: {case_id}")
    expected_pairs = {
        "trace": trace_path.as_posix(),
        "category": "read_model",
        "status": "validator_executable_replay",
        "evidence_id": "V260-001",
        "harness": HARNESS,
        "validator_entrypoint": "scripts/ai/verify_v26_product_hardening_boundary_contract.sh::classify",
        "replay_type": "validator_executable_product_hardening_boundary_contract",
        "classification_owner": "V260-001",
        "source_scope_owner": "V260-001",
        "release_scope": RELEASE_SCOPE,
    }
    for key, expected in expected_pairs.items():
        if entry.get(key) != expected:
            fail(f"{case_id}: release scope {key} mismatch: {entry.get(key)!r}")
    for key in (
        "runtime_adapter_integration",
        "complete_executable_order_control_runtime",
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
        "product_grade_live_trading_terminal",
    ):
        if entry.get(key) is not False:
            fail(f"{case_id}: release scope {key} must be false")

print(
    "v26_product_hardening_boundary_contract "
    f"status=ok trace={trace_path.as_posix()} "
    f"cases={len(rows)} required_false_flags={len(BOUNDARY_FALSE_FLAGS)} "
    f"negative_selftest={1 if selftest else 0}"
)
PY
