#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

CONTRACT_PATH="${NTPRO_V21_DASHBOARD_CONTRACT:-docs/rust-cutover/release/v0_21_0_trader_terminal_readonly_dashboard.md}"
UNIFIED_SCHEMA_PATH="${NTPRO_V21_READ_MODEL_SCHEMA:-docs/rust-cutover/release/v0_21_0_unified_read_model_schema.json}"
TRACE_PATH="${NTPRO_V21_DASHBOARD_TRACE:-tests/golden/read_model_dashboard_schema.jsonl}"
PYTHON_BIN="${PYTHON_BIN:-}"

if [ -z "$PYTHON_BIN" ]; then
  if command -v python3 >/dev/null 2>&1; then
    PYTHON_BIN=python3
  elif command -v python >/dev/null 2>&1; then
    PYTHON_BIN=python
  else
    echo "python3 or python is required" >&2
    exit 127
  fi
fi

for path in "$CONTRACT_PATH" "$UNIFIED_SCHEMA_PATH" "$TRACE_PATH"; do
  if [ ! -f "$path" ]; then
    echo "missing required v21 dashboard read-only foundation file: $path" >&2
    exit 1
  fi
done

scripts/ai/ntpro_governance.sh golden-trace "$TRACE_PATH" --mode validate-only

CONTRACT_PATH="$CONTRACT_PATH" \
UNIFIED_SCHEMA_PATH="$UNIFIED_SCHEMA_PATH" \
TRACE_PATH="$TRACE_PATH" \
"$PYTHON_BIN" <<'PY'
import json
import os
from pathlib import Path
from typing import Any

CONTRACT_VERSION = "ntpro.v210.unified_read_model.v1"
SCHEMA_VERSION = "ntpro.v210.unified_read_model.schema.v1"
DASHBOARD_TRANSFORM = "ntpro.v210.trader_terminal_readonly_dashboard.v1"
REQUIRED_COMPONENTS = {"account", "positions", "orders", "fills", "risk", "lifecycle_status"}
REQUIRED_PANELS = [
    "accounts",
    "positions",
    "orders",
    "fills",
    "risk",
    "audit_provenance_diagnostics",
]
DISABLED_CONTROLS = ["submit", "approval", "cancel", "retry", "replace", "amend", "flatten"]
EXPECTED_CASES = {
    "read_model.dashboard.readonly_complete.001": {
        "health_status": "healthy",
        "terminal_status": "foundation_only_readonly",
        "missing_evidence": [],
        "blocked_controls": [],
        "blocking_reasons": [],
        "component_statuses": {
            "account": "healthy",
            "positions": "healthy",
            "orders": "healthy",
            "fills": "healthy",
            "risk": "healthy",
            "lifecycle_status": "healthy",
        },
    },
    "read_model.dashboard.missing_evidence_degraded.001": {
        "health_status": "degraded",
        "terminal_status": "degraded_missing_evidence",
        "missing_evidence": ["fills"],
        "blocked_controls": [],
        "blocking_reasons": ["missing_fill_evidence"],
        "component_statuses": {
            "account": "healthy",
            "positions": "healthy",
            "orders": "healthy",
            "fills": "unavailable",
            "risk": "degraded",
            "lifecycle_status": "degraded",
        },
    },
    "read_model.dashboard.forbidden_controls_blocked.001": {
        "health_status": "fail_closed",
        "terminal_status": "blocked_forbidden_controls",
        "missing_evidence": [],
        "blocked_controls": DISABLED_CONTROLS,
        "blocking_reasons": ["dashboard_forbidden_controls_requested"],
        "component_statuses": {
            "account": "healthy",
            "positions": "healthy",
            "orders": "healthy",
            "fills": "healthy",
            "risk": "healthy",
            "lifecycle_status": "fail_closed",
        },
    },
}
FALSE_BOUNDARY_FLAGS = {
    "new_submit_capability",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "dashboard_order_controls_enabled",
    "dashboard_approval_controls_enabled",
    "dashboard_cancel_controls_enabled",
    "dashboard_retry_controls_enabled",
    "retry_replace_amend_flatten_allowed",
    "product_grade_trading_terminal_claim",
    "dashboard_submit_controls_enabled",
    "dashboard_replace_controls_enabled",
    "dashboard_amend_controls_enabled",
    "dashboard_flatten_controls_enabled",
    "trader_terminal_order_ticket_enabled",
    "trader_terminal_live_trading_claim",
}


def fail(message: str) -> None:
    raise SystemExit(message)


def require(condition: bool, message: str) -> None:
    if not condition:
        fail(message)


def load_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        fail(f"{path}: invalid JSON: {exc}")


def load_jsonl(path: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
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


def input_payload(row: dict[str, Any]) -> dict[str, Any]:
    events = row.get("input", {}).get("events", [])
    require(len(events) == 1, f"{row.get('case_id')}: expected one input event")
    payload = events[0].get("payload")
    require(isinstance(payload, dict), f"{row.get('case_id')}: missing input payload")
    return payload


def expected_payload(row: dict[str, Any]) -> dict[str, Any]:
    events = row.get("expected", {}).get("events", [])
    require(len(events) == 1, f"{row.get('case_id')}: expected one output event")
    payload = events[0].get("payload")
    require(isinstance(payload, dict), f"{row.get('case_id')}: missing expected payload")
    return payload


def require_false_boundary(snapshot: dict[str, Any], case_id: str) -> None:
    boundary = snapshot.get("capability_boundary")
    require(isinstance(boundary, dict), f"{case_id}: missing capability boundary")
    for key in sorted(FALSE_BOUNDARY_FLAGS):
        require(boundary.get(key) is False, f"{case_id}: {key} must be false")


def validate_component(component: dict[str, Any], name: str, case_id: str, expected_status: str) -> None:
    require(component.get("component_status") == expected_status, f"{case_id}: {name} status mismatch")
    for key in ("source_provenance", "lineage", "freshness", "redaction", "data", "diagnostics"):
        require(key in component, f"{case_id}: {name} missing {key}")
    require(component.get("source_provenance", {}).get("adapter_runtime_integrated") is False, f"{case_id}: {name} must remain fixture/artifact only")
    require(component.get("source_provenance", {}).get("exchange_truth") is False, f"{case_id}: {name} must not claim exchange truth")
    require(component.get("data", {}).get("dashboard_visible") is True, f"{case_id}: {name} must remain visible in readonly dashboard")


def validate_case(row: dict[str, Any]) -> None:
    case_id = row.get("case_id")
    require(case_id in EXPECTED_CASES, f"unexpected dashboard case {case_id}")
    expected = EXPECTED_CASES[case_id]
    payload = input_payload(row)
    snapshot = payload.get("snapshot")
    request = payload.get("dashboard_request")
    output = expected_payload(row)

    require(isinstance(snapshot, dict), f"{case_id}: missing input snapshot")
    require(isinstance(request, dict), f"{case_id}: missing dashboard request")
    require(snapshot.get("contract_version") == CONTRACT_VERSION, f"{case_id}: contract version mismatch")
    require(snapshot.get("schema_version") == SCHEMA_VERSION, f"{case_id}: schema version mismatch")
    require(snapshot.get("health_status") == expected["health_status"], f"{case_id}: input health mismatch")
    require(snapshot.get("lineage", {}).get("transform") == DASHBOARD_TRANSFORM, f"{case_id}: transform mismatch")
    require(request.get("view_mode") == "trader_terminal_foundation", f"{case_id}: request view mode mismatch")
    require(request.get("foundation_only") is True, f"{case_id}: request must be foundation only")

    components = snapshot.get("components")
    require(isinstance(components, dict), f"{case_id}: missing components")
    require(REQUIRED_COMPONENTS <= set(components), f"{case_id}: missing required components")
    for component_name, expected_status in expected["component_statuses"].items():
        validate_component(components[component_name], component_name, case_id, expected_status)

    require_false_boundary(snapshot, case_id)

    require(output.get("case_id") == case_id, f"{case_id}: expected payload case id mismatch")
    require(output.get("health_status") == expected["health_status"], f"{case_id}: expected payload health mismatch")
    require(output.get("terminal_status") == expected["terminal_status"], f"{case_id}: terminal status mismatch")
    require(output.get("foundation_only") is True, f"{case_id}: output must be foundation only")
    require(output.get("read_only") is True, f"{case_id}: output must be read only")
    require(output.get("no_submit_controls") is True, f"{case_id}: no submit controls flag required")
    require(output.get("visible_panels") == REQUIRED_PANELS, f"{case_id}: panel list mismatch")
    require(output.get("missing_evidence") == expected["missing_evidence"], f"{case_id}: missing evidence mismatch")
    require(output.get("blocked_controls") == expected["blocked_controls"], f"{case_id}: blocked controls mismatch")
    require(output.get("disabled_controls") == DISABLED_CONTROLS, f"{case_id}: disabled controls mismatch")
    require(output.get("display_claim") == "read_only_foundation", f"{case_id}: display claim must be read-only foundation")
    require(output.get("product_grade_trading_terminal_claim") is False, f"{case_id}: product-grade claim must be false")
    require(output.get("behavior_impact") == "display_only", f"{case_id}: behavior impact must be display only")
    require(output.get("blocking_reasons") == expected["blocking_reasons"], f"{case_id}: blocking reasons mismatch")
    for control in DISABLED_CONTROLS:
        require(output.get("control_flags", {}).get(control) is False, f"{case_id}: {control} control must be false")

    if expected["blocked_controls"]:
        require(request.get("requested_controls") == DISABLED_CONTROLS, f"{case_id}: forbidden control request set mismatch")
        require("dashboard_forbidden_controls_requested" in snapshot.get("blocking_reasons", []), f"{case_id}: missing forbidden-control blocker")
    else:
        require(request.get("requested_controls") == [], f"{case_id}: readonly cases must request no controls")


contract_path = Path(os.environ["CONTRACT_PATH"])
schema_path = Path(os.environ["UNIFIED_SCHEMA_PATH"])
trace_path = Path(os.environ["TRACE_PATH"])

contract = contract_path.read_text(encoding="utf-8")
schema = load_json(schema_path)
rows = load_jsonl(trace_path)

for marker in (
    DASHBOARD_TRANSFORM,
    "foundation_only",
    "read_only",
    "no_submit_controls",
    "submit, approval, cancel, retry, replace, amend, flatten",
    "scripts/ai/verify_release.sh v21-trader-terminal-readonly-dashboard",
):
    require(marker in contract, f"contract missing marker {marker}")

require(schema.get("schema_version") == SCHEMA_VERSION, "unified schema version mismatch")
require(len(rows) == 3, "dashboard golden trace must contain exactly three rows")
require({row.get("case_id") for row in rows} == set(EXPECTED_CASES), "unexpected dashboard case set")

for row in rows:
    validate_case(row)

print("v21_trader_terminal_readonly_dashboard status=ok trace_cases=3 readonly_display=covered missing_evidence_degradation=covered forbidden_controls_blocked=covered foundation_only=true read_only=true no_submit_controls=true product_grade_claim=false")
PY
