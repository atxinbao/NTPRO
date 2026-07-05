#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

TRACE_PATH="${NTPRO_V250_DASHBOARD_SURFACE_TRACE:-tests/golden/v250_dashboard_monitoring_surface.jsonl}"
SELFTEST="${NTPRO_V250_DASHBOARD_SURFACE_SELFTEST:-1}"

if [[ ! -f "$TRACE_PATH" ]]; then
  echo "missing V250 Dashboard monitoring surface trace: $TRACE_PATH" >&2
  exit 1
fi

python3 - "$TRACE_PATH" "$SELFTEST" <<'PY'
from __future__ import annotations

import copy
import json
import sys
from pathlib import Path
from typing import Any

trace_path = Path(sys.argv[1])
selftest = sys.argv[2] != "0"
dashboard_rs = Path("crates/cli/src/dashboard.rs")

EXPECTED_CASES = [
    "read_model.dashboard_monitoring_surface.ready_readonly.001",
    "read_model.dashboard_monitoring_surface.missing_provenance_fail_closed.001",
    "read_model.dashboard_monitoring_surface.stale_artifact_degraded.001",
    "read_model.dashboard_monitoring_surface.missing_redaction_fail_closed.001",
    "read_model.dashboard_monitoring_surface.forbidden_control_fail_closed.001",
]
REQUIRED_COMPONENTS = [
    "monitoring",
    "alert",
    "incident",
    "runbook",
    "dr_preview",
]
FORBIDDEN_TRUE_FIELDS = [
    "submit_order_allowed",
    "cancel_order_allowed",
    "retry_order_allowed",
    "replace_order_allowed",
    "amend_order_allowed",
    "flatten_position_allowed",
    "order_ticket_enabled",
    "dashboard_trading_control_allowed",
    "live_exchange_request_allowed",
    "adapter_send_allowed",
    "automatic_remediation_allowed",
]
REQUIRED_SOURCE_FIELDS = ["source_type", "source_ref", "producer", "collected_at"]


def fail(message: str) -> None:
    raise SystemExit(f"v25 Dashboard monitoring surface failed: {message}")


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


def complete_source(value: Any) -> bool:
    return isinstance(value, dict) and all(non_empty(value.get(field)) for field in REQUIRED_SOURCE_FIELDS)


def push_reason(reasons: list[str], reason: str) -> None:
    if reason not in reasons:
        reasons.append(reason)


def classify(snapshot: dict[str, Any], case_id: str) -> dict[str, Any]:
    components = snapshot.get("components")
    if not isinstance(components, dict):
        fail(f"{case_id}: components must be an object")

    reasons: list[str] = list(snapshot.get("blocking_reasons") or [])
    if any(not non_empty(reason) for reason in reasons):
        fail(f"{case_id}: blocking_reasons must contain non-empty strings")

    missing_component = False
    source_complete = True
    freshness_complete = True
    redaction_complete = True
    forbidden_control = False
    operation_controls_absent = True

    for component_name in REQUIRED_COMPONENTS:
        component = components.get(component_name)
        if not isinstance(component, dict):
            missing_component = True
            source_complete = False
            push_reason(reasons, f"missing_component:{component_name}")
            continue

        if not complete_source(component.get("source_provenance")):
            source_complete = False
            push_reason(reasons, f"missing_source_provenance:{component_name}")

        freshness = component.get("freshness")
        freshness_status = freshness.get("status") if isinstance(freshness, dict) else None
        if freshness_status != "fresh":
            freshness_complete = False
            push_reason(reasons, f"freshness_not_fresh:{component_name}:{freshness_status or 'missing'}")

        if component.get("redaction_state") != "redacted":
            redaction_complete = False
            push_reason(reasons, f"redaction_not_ready:{component_name}")

        if component.get("operation_boundary_readonly") is not True:
            forbidden_control = True
            operation_controls_absent = False
            push_reason(reasons, f"operation_boundary_not_readonly:{component_name}")

        for field in FORBIDDEN_TRUE_FIELDS:
            if component.get(field) is True:
                forbidden_control = True
                operation_controls_absent = False
                push_reason(reasons, f"forbidden_control:{component_name}:{field}")

    fail_closed = forbidden_control or not source_complete or not redaction_complete
    degraded = missing_component or not freshness_complete
    if fail_closed:
        surface_status = "fail_closed_surface_artifact"
    elif degraded:
        surface_status = "degraded_surface_artifact"
    else:
        surface_status = "ready_readonly_surface"

    return {
        "case_id": case_id,
        "surface_status": surface_status,
        "display_healthy_allowed": surface_status == "ready_readonly_surface",
        "component_count": len([component for component in components.values() if isinstance(component, dict)]),
        "source_provenance_complete": source_complete,
        "freshness_complete": freshness_complete,
        "redaction_complete": redaction_complete,
        "operation_controls_absent": operation_controls_absent,
        "forbidden_control_fail_closed": forbidden_control,
        "fail_closed": fail_closed,
        "blocking_reasons": reasons,
    }


def assert_dashboard_source_markers() -> None:
    text = dashboard_rs.read_text(encoding="utf-8")
    required = [
        "V25_MONITORING_OBSERVABILITY_COMPONENT",
        "V25_INCIDENT_LIFECYCLE_COMPONENT",
        "V25_RUNBOOK_AUDIT_COMPONENT",
        "V25_DR_PREVIEW_COMPONENT",
        "workbench-panel-v25-monitoring-surface",
        "v25_dashboard_surface_status",
        "v25_surface_blocking_reasons",
        "validate_v25_dashboard_surface_component",
    ]
    for marker in required:
        if marker not in text:
            fail(f"dashboard source missing marker: {marker}")
    renderer = dashboard_js_function_body(text, "renderTraderTerminalWorkbench")
    renderer += "\n" + dashboard_js_function_body(text, "renderReadModelRuntime")
    forbidden = [
        "data-workbench-action=\"submit",
        "data-workbench-action=\"cancel",
        "data-workbench-action=\"replace",
        "data-workbench-action=\"amend",
        "data-workbench-action=\"flatten",
        "/actions/submit",
        "/actions/cancel",
        "/actions/replace",
        "/actions/amend",
        "/actions/flatten",
    ]
    for marker in forbidden:
        if marker in renderer:
            fail(f"dashboard source exposes forbidden control marker: {marker}")


def dashboard_js_function_body(text: str, function_name: str) -> str:
    needle = f"function {function_name}"
    start = text.find(needle)
    if start < 0:
        fail(f"dashboard source missing JS function: {function_name}")
    after_start = start + len(needle)
    end = text.find("\nfunction ", after_start)
    if end < 0:
        end = len(text)
    return text[start:end]


rows = load_rows(trace_path)
case_ids = [str(row.get("case_id")) for row in rows]
if case_ids != EXPECTED_CASES:
    fail(f"case ordering mismatch: {case_ids}")

for row in rows:
    case_id = str(row["case_id"])
    input_event = single_event(row, "input", case_id)
    expected_event = single_event(row, "expected", case_id)
    snapshot = input_event.get("payload", {}).get("snapshot")
    if not isinstance(snapshot, dict):
        fail(f"{case_id}: input payload.snapshot must be an object")
    actual = classify(snapshot, case_id)
    expected = expected_event.get("payload")
    if actual != expected:
        fail(
            f"{case_id}: classification mismatch\n"
            f"expected={json.dumps(expected, sort_keys=True)}\n"
            f"actual={json.dumps(actual, sort_keys=True)}"
        )

if selftest:
    negative = copy.deepcopy(rows[0])
    negative_snapshot = single_event(negative, "input", str(negative["case_id"]))["payload"]["snapshot"]
    negative_snapshot["components"]["incident"]["dashboard_trading_control_allowed"] = True
    if classify(negative_snapshot, "selftest")["surface_status"] != "fail_closed_surface_artifact":
        fail("negative selftest did not fail closed for Dashboard trading control")

assert_dashboard_source_markers()

print(
    "v25_dashboard_monitoring_surface status=ok "
    f"trace={trace_path} cases={len(rows)} components={len(REQUIRED_COMPONENTS)} "
    f"negative_selftest={1 if selftest else 0}"
)
PY
