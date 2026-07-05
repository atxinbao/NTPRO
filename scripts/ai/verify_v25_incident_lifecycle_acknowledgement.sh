#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

TRACE_PATH="${NTPRO_V250_INCIDENT_TRACE:-tests/golden/v250_incident_lifecycle_acknowledgement.jsonl}"
SELFTEST="${NTPRO_V250_INCIDENT_SELFTEST:-1}"

if [[ ! -f "$TRACE_PATH" ]]; then
  echo "missing V250 incident lifecycle trace: $TRACE_PATH" >&2
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
    "read_model.incident_lifecycle_acknowledgement.valid_lifecycle.001",
    "read_model.incident_lifecycle_acknowledgement.invalid_transition_fail_closed.001",
    "read_model.incident_lifecycle_acknowledgement.missing_owner_source_alert_fail_closed.001",
    "read_model.incident_lifecycle_acknowledgement.missing_ack_resolved_fail_closed.001",
    "read_model.incident_lifecycle_acknowledgement.stale_incident_fail_closed.001",
    "read_model.incident_lifecycle_acknowledgement.redaction_secret_fail_closed.001",
    "read_model.incident_lifecycle_acknowledgement.automatic_action_fail_closed.001",
]
EXPECTED_STATES = [
    "opened",
    "triaged",
    "acknowledged",
    "mitigated",
    "resolved",
    "postmortem",
]
ALLOWED_TRANSITIONS = {
    ("none", "opened"),
    ("opened", "triaged"),
    ("triaged", "acknowledged"),
    ("acknowledged", "mitigated"),
    ("mitigated", "resolved"),
    ("resolved", "postmortem"),
}
ACK_REQUIRED_STATES = {"acknowledged", "mitigated", "resolved", "postmortem"}
BOUNDARY_FALSE_FIELDS = [
    "external_ticket_system_connected",
    "external_ticket_mutation_allowed",
    "automatic_paging_allowed",
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
REQUIRED_SOURCE_ALERT_FIELDS = ["alert_id", "alert_case_id", "dedupe_key", "severity", "category"]
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
CONTRACT_VERSION = "ntpro.v250.incident_lifecycle_acknowledgement.v1"
SCHEMA_VERSION = "ntpro.v250.incident_lifecycle_acknowledgement.schema.v1"


def fail(message: str) -> None:
    raise SystemExit(f"v25 incident lifecycle acknowledgement failed: {message}")


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


def source_alert_complete(value: Any) -> bool:
    return isinstance(value, dict) and all(non_empty(value.get(field)) for field in REQUIRED_SOURCE_ALERT_FIELDS)


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


def scan_redaction(value: Any, reasons: list[str], path: str = "incident") -> bool:
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


def acknowledgement_complete(ack: Any, owner: Any) -> bool:
    return (
        isinstance(ack, dict)
        and ack.get("status") == "acknowledged"
        and non_empty(ack.get("acknowledged_by"))
        and ack.get("acknowledged_by") == owner
        and non_empty(ack.get("acknowledged_at"))
        and non_empty(ack.get("evidence_ref"))
        and non_empty(ack.get("source_alert_id"))
    )


def operation_intent_clean(value: Any, reasons: list[str], incident_index: int) -> tuple[bool, bool]:
    if not isinstance(value, dict):
        push_reason(reasons, f"missing_operation_intent:incident:{incident_index}")
        return False, False

    action_boundary = False
    automatic_allowed = False
    if value.get("side_effect") != "none":
        action_boundary = True
        automatic_allowed = True
        push_reason(reasons, f"operation_side_effect:{value.get('side_effect')}:incident:{incident_index}")

    for field in BOUNDARY_FALSE_FIELDS:
        if value.get(field) is not False:
            action_boundary = True
            if value.get(field) is True:
                automatic_allowed = True
            push_reason(reasons, f"operation_boundary_true:{field}:incident:{incident_index}")

    return not action_boundary, automatic_allowed


def classify(snapshot: dict[str, Any], case_id: str) -> dict[str, Any]:
    if snapshot.get("contract_version") != CONTRACT_VERSION:
        fail(f"{case_id}: contract_version must be {CONTRACT_VERSION}")
    if snapshot.get("schema_version") != SCHEMA_VERSION:
        fail(f"{case_id}: schema_version must be {SCHEMA_VERSION}")

    reasons: list[str] = list(snapshot.get("blocking_reasons") or [])
    if any(not non_empty(reason) for reason in reasons):
        fail(f"{case_id}: blocking_reasons must contain non-empty strings")

    incidents = snapshot.get("incidents")
    if not isinstance(incidents, list) or not incidents:
        fail(f"{case_id}: incidents must be a non-empty array")

    boundary = snapshot.get("incident_boundary")
    if not isinstance(boundary, dict):
        fail(f"{case_id}: incident_boundary must be an object")

    missing_required = False
    invalid_transition = False
    missing_acknowledgement = False
    stale_incident = False
    redaction_breach = False
    action_boundary = False
    automatic_allowed = False
    states_seen: set[str] = set()
    transitions_checked = 0
    acknowledged_count = 0
    audit_events_checked = 0
    source_alerts_linked = 0

    if boundary.get("incident_mode") != "evidence_only":
        action_boundary = True
        push_reason(reasons, "incident_mode_not_evidence_only")
    for field in BOUNDARY_FALSE_FIELDS:
        if boundary.get(field) is not False:
            action_boundary = True
            if boundary.get(field) is True:
                automatic_allowed = True
            push_reason(reasons, f"boundary_true:{field}")

    for index, incident in enumerate(incidents):
        if not isinstance(incident, dict):
            missing_required = True
            push_reason(reasons, f"incident_not_object:{index}")
            continue

        incident_id = incident.get("incident_id")
        current_state = incident.get("current_state")
        owner = incident.get("owner")
        assignee = incident.get("assignee")

        for field_name, value in (
            ("incident_id", incident_id),
            ("current_state", current_state),
            ("owner", owner),
            ("assignee", assignee),
            ("opened_at", incident.get("opened_at")),
            ("updated_at", incident.get("updated_at")),
        ):
            if not non_empty(value):
                missing_required = True
                push_reason(reasons, f"missing_{field_name}:incident:{index}")

        if non_empty(current_state):
            states_seen.add(str(current_state))
            if current_state not in EXPECTED_STATES:
                invalid_transition = True
                push_reason(reasons, f"invalid_current_state:{current_state}:incident:{index}")

        if not source_alert_complete(incident.get("source_alert")):
            missing_required = True
            push_reason(reasons, f"missing_source_alert:incident:{index}")
        else:
            source_alerts_linked += 1

        if not source_provenance_complete(incident.get("source_provenance")):
            missing_required = True
            push_reason(reasons, f"missing_source_provenance:incident:{index}")

        if not lineage_complete(incident.get("lineage")):
            missing_required = True
            push_reason(reasons, f"missing_lineage:incident:{index}")

        audit_trace = incident.get("audit_trace")
        if not audit_trace_complete(audit_trace):
            missing_required = True
            push_reason(reasons, f"missing_audit_trace:incident:{index}")
        else:
            audit_events_checked += len(audit_trace)

        freshness = incident.get("freshness")
        if not isinstance(freshness, dict):
            missing_required = True
            push_reason(reasons, f"missing_freshness:incident:{index}")
        elif freshness.get("status") != "fresh":
            stale_incident = True
            push_reason(reasons, f"stale_incident_freshness:incident:{index}")

        if incident.get("redaction_state") != "redacted":
            redaction_breach = True
            push_reason(reasons, f"redaction_state_not_redacted:incident:{index}")
        if scan_redaction(incident, reasons, f"incident:{index}"):
            redaction_breach = True

        operation_clean, incident_automatic_allowed = operation_intent_clean(
            incident.get("operation_intent"),
            reasons,
            index,
        )
        if not operation_clean:
            action_boundary = True
        automatic_allowed = automatic_allowed or incident_automatic_allowed

        transitions = incident.get("transitions")
        if not isinstance(transitions, list) or not transitions:
            missing_required = True
            push_reason(reasons, f"missing_transitions:incident:{index}")
        else:
            last_state: str | None = None
            for transition_index, transition in enumerate(transitions):
                transitions_checked += 1
                if not isinstance(transition, dict):
                    missing_required = True
                    push_reason(reasons, f"transition_not_object:incident:{index}:transition:{transition_index}")
                    continue
                from_state = str(transition.get("from_state") or "none")
                to_state = transition.get("to_state")
                actor = transition.get("actor")
                transitioned_at = transition.get("transitioned_at")
                audit_ref = transition.get("audit_ref")
                source_alert_id = transition.get("source_alert_id")
                for field_name, value in (
                    ("to_state", to_state),
                    ("actor", actor),
                    ("transitioned_at", transitioned_at),
                    ("audit_ref", audit_ref),
                    ("source_alert_id", source_alert_id),
                ):
                    if not non_empty(value):
                        missing_required = True
                        push_reason(
                            reasons,
                            f"missing_transition_{field_name}:incident:{index}:transition:{transition_index}",
                        )
                if non_empty(to_state):
                    states_seen.add(str(to_state))
                    if to_state not in EXPECTED_STATES:
                        invalid_transition = True
                        push_reason(
                            reasons,
                            f"invalid_transition_state:{to_state}:incident:{index}:transition:{transition_index}",
                        )
                    if (from_state, str(to_state)) not in ALLOWED_TRANSITIONS:
                        invalid_transition = True
                        push_reason(
                            reasons,
                            f"invalid_transition:{from_state}->{to_state}:incident:{index}:transition:{transition_index}",
                        )
                    last_state = str(to_state)
            if non_empty(current_state) and last_state != current_state:
                invalid_transition = True
                push_reason(reasons, f"current_state_not_last_transition:{current_state}!={last_state}:incident:{index}")

        ack = incident.get("operator_acknowledgement")
        if current_state in ACK_REQUIRED_STATES:
            if acknowledgement_complete(ack, owner):
                acknowledged_count += 1
            else:
                missing_acknowledgement = True
                push_reason(reasons, f"missing_owner_acknowledgement:incident:{index}")
        elif acknowledgement_complete(ack, owner):
            acknowledged_count += 1

    if redaction_breach:
        status = "fail_closed_redaction_breach"
    elif action_boundary:
        status = "fail_closed_action_boundary"
    elif missing_acknowledgement:
        status = "fail_closed_missing_acknowledgement"
    elif invalid_transition:
        status = "fail_closed_invalid_transition"
    elif stale_incident:
        status = "fail_closed_stale_incident"
    elif missing_required:
        status = "fail_closed_missing_required"
    else:
        status = "acknowledged_readonly"

    return {
        "case_id": case_id,
        "incident_lifecycle_status": status,
        "fail_closed": status.startswith("fail_closed"),
        "incident_count": len(incidents),
        "transitions_checked": transitions_checked,
        "states_seen": [state for state in EXPECTED_STATES if state in states_seen],
        "acknowledged_count": acknowledged_count,
        "audit_events_checked": audit_events_checked,
        "source_alerts_linked": source_alerts_linked,
        "owner_assignment_complete": not any(
            reason.startswith("missing_owner:") or reason.startswith("missing_assignee:")
            for reason in reasons
        ),
        "source_alert_linkage_complete": not any(reason.startswith("missing_source_alert") for reason in reasons),
        "provenance_lineage_audit_complete": not any(
            reason.startswith("missing_source_provenance")
            or reason.startswith("missing_lineage")
            or reason.startswith("missing_audit_trace")
            for reason in reasons
        ),
        "redaction_clean": not redaction_breach,
        "freshness_fresh": not stale_incident and not any(reason.startswith("missing_freshness") for reason in reasons),
        "operation_boundary_readonly": not action_boundary,
        "automatic_actions_allowed": automatic_allowed,
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
    if case_id.endswith("valid_lifecycle.001"):
        if computed["states_seen"] != EXPECTED_STATES:
            fail(f"{case_id}: valid lifecycle must cover all expected states")
        valid_snapshot = copy.deepcopy(snapshot)

if selftest:
    if valid_snapshot is None:
        fail("negative selftest requires valid lifecycle fixture")
    valid_snapshot["incidents"][0]["transitions"][1]["to_state"] = "resolved"
    failed = classify(valid_snapshot, "negative.selftest.invalid_transition")
    if failed["incident_lifecycle_status"] != "fail_closed_invalid_transition":
        fail("negative selftest changed opened->triaged but did not fail closed as invalid transition")
    if "invalid_transition:opened->resolved:incident:0:transition:1" not in failed["blocking_reasons"]:
        fail("negative selftest did not report invalid opened->resolved transition")

print(
    "v25_incident_lifecycle_acknowledgement "
    f"status=ok trace={trace_path.as_posix()} "
    f"cases={len(rows)} states={len(EXPECTED_STATES)} "
    f"negative_selftest={1 if selftest else 0}"
)
PY
