#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

TRACE_PATH="${NTPRO_V250_RUNBOOK_TRACE:-tests/golden/v250_runbook_audit_evidence.jsonl}"
SELFTEST="${NTPRO_V250_RUNBOOK_SELFTEST:-1}"

if [[ ! -f "$TRACE_PATH" ]]; then
  echo "missing V250 runbook audit trace: $TRACE_PATH" >&2
  exit 1
fi

python3 - "$TRACE_PATH" "$SELFTEST" <<'PY'
from __future__ import annotations

import copy
import json
import re
import sys
from pathlib import Path
from typing import Any

trace_path = Path(sys.argv[1])
selftest = sys.argv[2] != "0"

EXPECTED_CASES = [
    "read_model.runbook_audit_evidence.valid_manual_matrix.001",
    "read_model.runbook_audit_evidence.stale_runbook_fail_closed.001",
    "read_model.runbook_audit_evidence.missing_version_fail_closed.001",
    "read_model.runbook_audit_evidence.missing_audit_trace_fail_closed.001",
    "read_model.runbook_audit_evidence.unapproved_action_fail_closed.001",
    "read_model.runbook_audit_evidence.redaction_secret_fail_closed.001",
    "read_model.runbook_audit_evidence.automatic_execution_fail_closed.001",
]
EXPECTED_DECISION_TYPES = [
    "manual_observation",
    "manual_acknowledgement",
    "manual_escalation",
    "manual_rollback_recommendation",
]
APPROVED_STATUSES = {"owner_approved", "audit_gate_approved", "blocked_recommendation"}
BOUNDARY_FALSE_FIELDS = [
    "shell_execution_allowed",
    "runbook_automation_allowed",
    "permission_system_extension_allowed",
    "automatic_remediation_allowed",
    "automatic_strategy_stop_allowed",
    "automatic_submit_allowed",
    "automatic_cancel_allowed",
    "automatic_retry_allowed",
    "automatic_replace_allowed",
    "automatic_amend_allowed",
    "automatic_flatten_allowed",
    "live_exchange_request_allowed",
    "adapter_send_allowed",
    "dashboard_trading_control_allowed",
]
REQUIRED_SOURCE_FIELDS = ["source_type", "source_ref", "producer", "collected_at"]
REQUIRED_VERSIONED_SOURCE_FIELDS = ["source_ref", "version", "checksum"]
REQUIRED_INPUT_EVIDENCE_FIELDS = ["evidence_id", "source_ref", "source_provenance"]
REQUIRED_AUDIT_FIELDS = ["audit_id", "actor", "action", "at", "evidence_ref"]
FORBIDDEN_KEY_PATTERNS = [
    re.compile(pattern, re.IGNORECASE)
    for pattern in (
        r"(^|_)(secret|credential|signature|signed_payload|raw_order_id|raw_credential)(_|$)",
        r"api[_-]?key",
        r"client[_-]?order[_-]?id",
    )
]
FORBIDDEN_VALUE_PATTERNS = [
    re.compile(pattern, re.IGNORECASE)
    for pattern in (
        r"api[_-]?secret\s*=",
        r"credential\s*=",
        r"signed[_-]?payload\s*=",
        r"signature\s*=",
        r"raw[_-]?order[_-]?id\s*=",
        r"client[_-]?order[_-]?id\s*=",
    )
]
CONTRACT_VERSION = "ntpro.v250.runbook_audit_evidence.v1"
SCHEMA_VERSION = "ntpro.v250.runbook_audit_evidence.schema.v1"


def fail(message: str) -> None:
    raise SystemExit(f"v25 runbook audit evidence failed: {message}")


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
    events = row.get(section, {}).get("events")
    if not isinstance(events, list) or len(events) != 1 or not isinstance(events[0], dict):
        fail(f"{case_id}: {section}.events must contain exactly one object")
    return events[0]


def non_empty(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


def push_reason(reasons: list[str], reason: str) -> None:
    if reason not in reasons:
        reasons.append(reason)


def source_provenance_complete(value: Any) -> bool:
    return isinstance(value, dict) and all(non_empty(value.get(field)) for field in REQUIRED_SOURCE_FIELDS)


def versioned_source_complete(value: Any) -> bool:
    return isinstance(value, dict) and all(non_empty(value.get(field)) for field in REQUIRED_VERSIONED_SOURCE_FIELDS)


def input_evidence_complete(value: Any) -> bool:
    if not isinstance(value, list) or not value:
        return False
    for entry in value:
        if not isinstance(entry, dict):
            return False
        if not all(entry.get(field) is not None for field in REQUIRED_INPUT_EVIDENCE_FIELDS):
            return False
        if not non_empty(entry.get("evidence_id")) or not non_empty(entry.get("source_ref")):
            return False
        if not source_provenance_complete(entry.get("source_provenance")):
            return False
    return True


def lineage_complete(value: Any) -> bool:
    return (
        isinstance(value, list)
        and bool(value)
        and all(source_provenance_complete(entry) for entry in value)
    )


def audit_trace_complete(value: Any) -> bool:
    return (
        isinstance(value, list)
        and bool(value)
        and all(isinstance(entry, dict) and all(non_empty(entry.get(field)) for field in REQUIRED_AUDIT_FIELDS) for entry in value)
    )


def scan_redaction(value: Any, reasons: list[str], path: str = "runbook") -> bool:
    breached = False
    if isinstance(value, dict):
        for key, nested in value.items():
            if any(pattern.search(str(key)) for pattern in FORBIDDEN_KEY_PATTERNS):
                push_reason(reasons, f"forbidden_sensitive_key:{path}.{key}")
                breached = True
            if scan_redaction(nested, reasons, f"{path}.{key}"):
                breached = True
    elif isinstance(value, list):
        for index, nested in enumerate(value):
            if scan_redaction(nested, reasons, f"{path}[{index}]"):
                breached = True
    elif isinstance(value, str):
        if any(pattern.search(value) for pattern in FORBIDDEN_VALUE_PATTERNS):
            push_reason(reasons, f"forbidden_sensitive_value:{path}")
            breached = True
    return breached


def classify(snapshot: dict[str, Any], case_id: str) -> dict[str, Any]:
    if snapshot.get("contract_version") != CONTRACT_VERSION:
        fail(f"{case_id}: contract_version must be {CONTRACT_VERSION}")
    if snapshot.get("schema_version") != SCHEMA_VERSION:
        fail(f"{case_id}: schema_version must be {SCHEMA_VERSION}")

    reasons: list[str] = list(snapshot.get("blocking_reasons") or [])
    if any(not non_empty(reason) for reason in reasons):
        fail(f"{case_id}: blocking_reasons must contain non-empty strings")

    runbook_boundary = snapshot.get("runbook_boundary")
    if not isinstance(runbook_boundary, dict):
        fail(f"{case_id}: runbook_boundary must be an object")

    steps = snapshot.get("steps")
    if not isinstance(steps, list) or not steps:
        fail(f"{case_id}: steps must be a non-empty array")

    missing_required = False
    stale_runbook = False
    redaction_breach = False
    unapproved_action = False
    execution_boundary = False
    automatic_allowed = False
    decision_types: set[str] = set()
    audit_events_checked = 0
    input_evidence_count = 0

    if runbook_boundary.get("runbook_mode") != "evidence_only":
        execution_boundary = True
        push_reason(reasons, "runbook_mode_not_evidence_only")
    for field in BOUNDARY_FALSE_FIELDS:
        if runbook_boundary.get(field) is not False:
            execution_boundary = True
            if runbook_boundary.get(field) is True:
                automatic_allowed = True
            push_reason(reasons, f"boundary_true:{field}")

    for index, step in enumerate(steps):
        if not isinstance(step, dict):
            missing_required = True
            push_reason(reasons, f"step_not_object:{index}")
            continue

        for field_name in ("runbook_id", "runbook_version", "step_id", "step_name", "owner"):
            if not non_empty(step.get(field_name)):
                missing_required = True
                push_reason(reasons, f"missing_{field_name}:step:{index}")

        if not versioned_source_complete(step.get("versioned_source")):
            missing_required = True
            push_reason(reasons, f"missing_versioned_source:step:{index}")

        input_evidence = step.get("input_evidence")
        if not input_evidence_complete(input_evidence):
            missing_required = True
            push_reason(reasons, f"missing_input_evidence:step:{index}")
        else:
            input_evidence_count += len(input_evidence)

        if not source_provenance_complete(step.get("source_provenance")):
            missing_required = True
            push_reason(reasons, f"missing_source_provenance:step:{index}")

        if not lineage_complete(step.get("lineage")):
            missing_required = True
            push_reason(reasons, f"missing_lineage:step:{index}")

        audit_trace = step.get("audit_trace")
        if not audit_trace_complete(audit_trace):
            missing_required = True
            push_reason(reasons, f"missing_audit_trace:step:{index}")
        else:
            audit_events_checked += len(audit_trace)

        freshness = step.get("freshness")
        if not isinstance(freshness, dict):
            missing_required = True
            push_reason(reasons, f"missing_freshness:step:{index}")
        elif freshness.get("status") != "fresh":
            stale_runbook = True
            push_reason(reasons, f"stale_runbook_freshness:step:{index}")

        if step.get("redaction_state") != "redacted":
            redaction_breach = True
            push_reason(reasons, f"redaction_state_not_redacted:step:{index}")
        if scan_redaction(step, reasons, f"step:{index}"):
            redaction_breach = True

        decision = step.get("decision_output")
        if not isinstance(decision, dict):
            missing_required = True
            push_reason(reasons, f"missing_decision_output:step:{index}")
        else:
            decision_type = decision.get("decision_type")
            approval_status = decision.get("approval_status")
            visible_result = decision.get("visible_result")
            side_effect = decision.get("side_effect")
            if decision_type in EXPECTED_DECISION_TYPES:
                decision_types.add(str(decision_type))
            else:
                missing_required = True
                push_reason(reasons, f"invalid_decision_type:{decision_type}:step:{index}")
            if not non_empty(visible_result):
                missing_required = True
                push_reason(reasons, f"missing_visible_result:step:{index}")
            if approval_status == "unapproved":
                unapproved_action = True
                if visible_result != "blocked_recommendation":
                    push_reason(reasons, f"unapproved_action_not_blocked:step:{index}")
                else:
                    push_reason(reasons, f"unapproved_action_blocked:step:{index}")
            elif approval_status not in APPROVED_STATUSES:
                missing_required = True
                push_reason(reasons, f"invalid_approval_status:{approval_status}:step:{index}")
            if side_effect != "none":
                execution_boundary = True
                automatic_allowed = True
                push_reason(reasons, f"decision_side_effect:{side_effect}:step:{index}")

    if redaction_breach:
        status = "fail_closed_redaction_breach"
    elif execution_boundary:
        status = "fail_closed_execution_boundary"
    elif unapproved_action:
        status = "fail_closed_unapproved_action"
    elif stale_runbook:
        status = "fail_closed_stale_runbook"
    elif missing_required:
        status = "fail_closed_missing_required"
    else:
        status = "audit_evidence_readonly"

    return {
        "case_id": case_id,
        "runbook_audit_status": status,
        "fail_closed": status.startswith("fail_closed"),
        "step_count": len(steps),
        "decision_types": [kind for kind in EXPECTED_DECISION_TYPES if kind in decision_types],
        "audit_events_checked": audit_events_checked,
        "input_evidence_count": input_evidence_count,
        "owner_gate_complete": not any(
            reason.startswith("missing_owner:") or reason.startswith("invalid_approval_status:")
            for reason in reasons
        ),
        "versioned_source_complete": not any(reason.startswith("missing_runbook_version:") or reason.startswith("missing_versioned_source:") for reason in reasons),
        "audit_trace_complete": not any(reason.startswith("missing_audit_trace:") for reason in reasons),
        "redaction_clean": not redaction_breach,
        "freshness_fresh": not stale_runbook and not any(reason.startswith("missing_freshness:") for reason in reasons),
        "execution_boundary_readonly": not execution_boundary,
        "automatic_execution_allowed": automatic_allowed,
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

valid_snapshot: dict[str, Any] | None = None
for row in rows:
    case_id = str(row.get("case_id"))
    if row.get("schema_version") != "golden-trace-v1":
        fail(f"{case_id}: schema_version must be golden-trace-v1")
    if row.get("category") != "read_model":
        fail(f"{case_id}: category must be read_model")
    if not non_empty(row.get("description")):
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
    if computed != expected_event.get("payload"):
        fail(
            f"{case_id}: computed payload mismatch\n"
            f"expected={json.dumps(expected_event.get('payload'), sort_keys=True)}\n"
            f"actual={json.dumps(computed, sort_keys=True)}"
        )
    if case_id.endswith("valid_manual_matrix.001"):
        if computed["decision_types"] != EXPECTED_DECISION_TYPES:
            fail(f"{case_id}: valid matrix must cover all expected decision types")
        valid_snapshot = copy.deepcopy(snapshot)

if selftest:
    if valid_snapshot is None:
        fail("negative selftest requires valid runbook fixture")
    valid_snapshot["steps"][0].pop("runbook_version", None)
    failed = classify(valid_snapshot, "negative.selftest.missing_runbook_version")
    if failed["runbook_audit_status"] != "fail_closed_missing_required":
        fail("negative selftest removed runbook_version but did not fail closed")
    if "missing_runbook_version:step:0" not in failed["blocking_reasons"]:
        fail("negative selftest did not report missing_runbook_version:step:0")

print(
    "v25_runbook_audit_evidence "
    f"status=ok trace={trace_path.as_posix()} "
    f"cases={len(rows)} decision_types={len(EXPECTED_DECISION_TYPES)} "
    f"negative_selftest={1 if selftest else 0}"
)
PY
