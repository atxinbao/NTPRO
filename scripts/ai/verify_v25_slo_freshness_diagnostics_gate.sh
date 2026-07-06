#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

TRACE_PATH="${NTPRO_V250_SLO_DIAGNOSTICS_TRACE:-tests/golden/v250_slo_freshness_diagnostics_gate.jsonl}"
SELFTEST="${NTPRO_V250_SLO_DIAGNOSTICS_SELFTEST:-1}"

if [[ ! -f "$TRACE_PATH" ]]; then
  echo "missing V250 SLO freshness diagnostics trace: $TRACE_PATH" >&2
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
    "read_model.slo_freshness_diagnostics.ready.001",
    "read_model.slo_freshness_diagnostics.missing_component_fail_closed.001",
    "read_model.slo_freshness_diagnostics.stale_source_degraded.001",
    "read_model.slo_freshness_diagnostics.partial_projection_degraded.001",
    "read_model.slo_freshness_diagnostics.unknown_adapter_truth_fail_closed.001",
    "read_model.slo_freshness_diagnostics.release_provenance_drift_fail_closed.001",
    "read_model.slo_freshness_diagnostics.forbidden_action_fail_closed.001",
]
REQUIRED_COMPONENTS = [
    "monitoring",
    "alert",
    "incident",
    "runbook",
    "dr_preview",
]
FORBIDDEN_ACTION_FIELDS = [
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
    "automatic_actions_allowed",
    "remediation_action_allowed",
    "trading_action_allowed",
]


def fail(message: str) -> None:
    raise SystemExit(f"v25 SLO freshness diagnostics gate failed: {message}")


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


def component(name: str) -> dict[str, Any]:
    return {
        "component_status": "healthy",
        "source_provenance": {
            "source_type": "artifact",
            "source_ref": f"artifact://v0_25/{name}.json",
            "producer": "v250_slo_freshness_diagnostics_gate",
        },
        "freshness": {
            "status": "fresh",
            "observed_age_ms": 100,
            "max_age_ms": 60000,
        },
        "redaction": {"status": "redacted"},
        "data": {
            "slo_evidence_ref": f"slo:v250:{name}:freshness",
            "diagnostic_severity": "info",
            "source_truth_status": "artifact_truth_only",
            "adapter_truth_status": "not_integrated",
            "release_provenance_status": "matched",
            "partial_projection": False,
            "operation_boundary_readonly": True,
            "forbidden_control_detected": False,
            "dashboard_trading_control_allowed": False,
            "live_exchange_request_allowed": False,
            "adapter_send_allowed": False,
            "automatic_remediation_allowed": False,
            "automatic_actions_allowed": False,
            "remediation_action_allowed": False,
            "trading_action_allowed": False,
        },
    }


def snapshot_for_scenario(scenario: str) -> dict[str, Any]:
    components = {name: component(name) for name in REQUIRED_COMPONENTS}
    if scenario == "ready":
        pass
    elif scenario == "missing_component_fail_closed":
        components.pop("dr_preview")
    elif scenario == "stale_source_degraded":
        components["alert"]["freshness"] = {
            "status": "stale",
            "observed_age_ms": 120000,
            "max_age_ms": 60000,
            "staleness_reason": "source_lag_exceeded",
        }
        components["alert"]["data"]["diagnostic_severity"] = "warning"
    elif scenario == "partial_projection_degraded":
        components["monitoring"]["data"]["partial_projection"] = True
        components["monitoring"]["data"]["diagnostic_severity"] = "warning"
    elif scenario == "unknown_adapter_truth_fail_closed":
        components["monitoring"]["data"]["adapter_truth_status"] = "unknown"
    elif scenario == "release_provenance_drift_fail_closed":
        components["runbook"]["data"]["release_provenance_status"] = "drift"
    elif scenario == "forbidden_action_fail_closed":
        components["dr_preview"]["data"]["remediation_action_allowed"] = True
        components["dr_preview"]["data"]["trading_action_allowed"] = True
    else:
        fail(f"unknown scenario: {scenario}")
    return {"components": components, "blocking_reasons": []}


def push_reason(reasons: list[str], reason: str) -> None:
    if reason not in reasons:
        reasons.append(reason)


def classify(snapshot: dict[str, Any], case_id: str) -> dict[str, Any]:
    components = snapshot.get("components")
    if not isinstance(components, dict):
        fail(f"{case_id}: components must be an object")

    reasons: list[str] = []
    missing_component = False
    threshold_missing = False
    threshold_exceeded = False
    staleness_reasons: list[str] = []
    slo_complete = True
    partial_projection = False
    unknown_truth = False
    release_drift = False
    operation_controls_absent = True

    for component_name in REQUIRED_COMPONENTS:
        item = components.get(component_name)
        if not isinstance(item, dict):
            missing_component = True
            push_reason(reasons, f"{component_name}:component_missing")
            continue

        freshness = item.get("freshness")
        if not isinstance(freshness, dict):
            threshold_missing = True
            push_reason(reasons, f"{component_name}:freshness_threshold_missing")
        else:
            observed = freshness.get("observed_age_ms")
            max_age = freshness.get("max_age_ms")
            if not isinstance(observed, int) or not isinstance(max_age, int):
                threshold_missing = True
                push_reason(reasons, f"{component_name}:freshness_threshold_missing")
            elif observed > max_age:
                threshold_exceeded = True
                push_reason(reasons, f"{component_name}:freshness_threshold_exceeded:{observed}>{max_age}")
            if freshness.get("status") != "fresh":
                reason = freshness.get("staleness_reason")
                if not isinstance(reason, str) or not reason:
                    push_reason(reasons, f"{component_name}:staleness_reason_missing")
                    threshold_missing = True
                else:
                    value = f"{component_name}:staleness_reason:{reason}"
                    staleness_reasons.append(value)
                    push_reason(reasons, value)

        data = item.get("data")
        if not isinstance(data, dict):
            push_reason(reasons, f"{component_name}:data_missing")
            slo_complete = False
            continue

        if not isinstance(data.get("slo_evidence_ref"), str) or not data["slo_evidence_ref"]:
            slo_complete = False
            push_reason(reasons, f"{component_name}:slo_evidence_missing")
        if data.get("partial_projection") is True:
            partial_projection = True
            push_reason(reasons, f"{component_name}:partial_projection")
        if data.get("source_truth_status") != "artifact_truth_only" or data.get("adapter_truth_status") != "not_integrated":
            unknown_truth = True
            push_reason(reasons, f"{component_name}:unknown_adapter_truth")
        if data.get("release_provenance_status") != "matched":
            release_drift = True
            push_reason(reasons, f"{component_name}:release_provenance_drift")
        if data.get("operation_boundary_readonly") is not True:
            operation_controls_absent = False
            push_reason(reasons, f"{component_name}:operation_boundary_not_readonly")
        for field in FORBIDDEN_ACTION_FIELDS:
            if data.get(field) is True:
                operation_controls_absent = False
                push_reason(reasons, f"{component_name}:boundary_true:{field}")

    fail_closed = missing_component or (not slo_complete) or unknown_truth or release_drift or not operation_controls_absent or threshold_missing
    if missing_component:
        diagnostics_gate_status = "fail_closed_missing_component"
    elif unknown_truth:
        diagnostics_gate_status = "fail_closed_unknown_source_truth"
    elif release_drift:
        diagnostics_gate_status = "fail_closed_release_provenance_drift"
    elif not operation_controls_absent:
        diagnostics_gate_status = "fail_closed_forbidden_action"
    elif partial_projection:
        diagnostics_gate_status = "degraded_partial_projection"
    elif threshold_exceeded:
        diagnostics_gate_status = "degraded_stale_source"
    else:
        diagnostics_gate_status = "ready_slo_freshness_gate"

    if not slo_complete:
        slo_status = "fail_closed_missing_slo_evidence"
    else:
        slo_status = "slo_evidence_ready"

    if threshold_missing:
        freshness_threshold_status = "fail_closed_threshold_missing"
    elif threshold_exceeded:
        freshness_threshold_status = "degraded_threshold_exceeded"
    else:
        freshness_threshold_status = "freshness_thresholds_ready"

    if fail_closed:
        diagnostic_severity = "critical"
    elif threshold_exceeded or partial_projection:
        diagnostic_severity = "warning"
    else:
        diagnostic_severity = "info"

    return {
        "case_id": case_id,
        "diagnostics_gate_status": diagnostics_gate_status,
        "display_healthy_allowed": diagnostics_gate_status == "ready_slo_freshness_gate",
        "component_count": len([item for item in components.values() if isinstance(item, dict)]),
        "slo_status": slo_status,
        "freshness_threshold_status": freshness_threshold_status,
        "staleness_reasons": ",".join(staleness_reasons) if staleness_reasons else "none",
        "diagnostic_severity": diagnostic_severity,
        "source_truth_status": "fail_closed_unknown_source_truth" if unknown_truth else "artifact_truth_only",
        "release_provenance_status": "fail_closed_release_provenance_drift" if release_drift else "matched",
        "no_remediation_status": "no_remediation_no_trading_actions" if operation_controls_absent else "fail_closed_forbidden_action",
        "operation_controls_absent": operation_controls_absent,
        "fail_closed": fail_closed,
        "blocking_reasons": reasons,
    }


def assert_dashboard_source_markers() -> None:
    text = dashboard_rs.read_text(encoding="utf-8")
    required = [
        "v25_diagnostics_gate_status",
        "v25_freshness_threshold_status",
        "v25_source_truth_status",
        "v25_release_provenance_status",
        "v25_no_remediation_status",
        "v25_diagnostics_gate_status(",
        "dashboard_v25_slo_freshness_threshold_stale_source_degrades",
        "dashboard_v25_unknown_adapter_truth_fails_closed",
        "dashboard_v25_release_provenance_drift_fails_closed",
        "dashboard_v25_diagnostics_gate_forbidden_actions_fail_closed",
        "workbench-panel-v25-monitoring-surface",
    ]
    for marker in required:
        if marker not in text:
            fail(f"dashboard source missing marker: {marker}")
    renderer = dashboard_js_function_body(text, "renderTraderTerminalWorkbench")
    forbidden = [
        "<button",
        "<form",
        "fetch(",
        "data-workbench-action",
        "/actions/submit",
        "/actions/cancel",
        "/actions/retry",
        "/actions/replace",
        "/actions/amend",
        "/actions/flatten",
    ]
    for marker in forbidden:
        if marker in renderer:
            fail(f"dashboard source exposes forbidden action marker: {marker}")


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
    scenario = input_event.get("payload", {}).get("scenario")
    if not isinstance(scenario, str) or not scenario:
        fail(f"{case_id}: input payload.scenario must be a non-empty string")
    actual = classify(snapshot_for_scenario(scenario), case_id)
    expected = expected_event.get("payload")
    if actual != expected:
        fail(
            f"{case_id}: classification mismatch\n"
            f"expected={json.dumps(expected, sort_keys=True)}\n"
            f"actual={json.dumps(actual, sort_keys=True)}"
        )

if selftest:
    negative = snapshot_for_scenario("ready")
    negative["components"]["incident"]["data"]["trading_action_allowed"] = True
    actual = classify(negative, "selftest")
    if actual["diagnostics_gate_status"] != "fail_closed_forbidden_action":
        fail("negative selftest did not fail closed for forbidden trading action")

assert_dashboard_source_markers()

print(
    "v25_slo_freshness_diagnostics_gate status=ok "
    f"trace={trace_path} cases={len(rows)} components={len(REQUIRED_COMPONENTS)} "
    f"negative_selftest={1 if selftest else 0}"
)
PY
