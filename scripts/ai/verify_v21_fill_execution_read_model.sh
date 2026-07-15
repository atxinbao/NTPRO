#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

CONTRACT_PATH="${NTPRO_V21_FILL_CONTRACT:-docs/rust-cutover/release/v0_21_0_fill_execution_read_model.md}"
UNIFIED_SCHEMA_PATH="${NTPRO_V21_READ_MODEL_SCHEMA:-docs/rust-cutover/release/v0_21_0_unified_read_model_schema.json}"
TRACE_PATH="${NTPRO_V21_FILL_TRACE:-tests/golden/read_model_fill_execution_schema.jsonl}"
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
    echo "missing required v21 fill/execution read model file: $path" >&2
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
FILL_TRANSFORM = "ntpro.v210.fill_execution_read_model.v1"
EXPECTED_CASES = {
    "read_model.fill_execution.reconciled.001": {
        "health_status": "degraded",
        "component_status": "healthy",
        "fill_status": "filled",
        "reconciliation_status": "reconciled",
        "order_linkage_status": "linked",
        "freshness": "fresh",
        "duplicate": False,
        "partial": False,
        "reasons": [],
    },
    "read_model.fill_execution.partial_fill.001": {
        "health_status": "degraded",
        "component_status": "degraded",
        "fill_status": "partial",
        "reconciliation_status": "partial_fill_visible",
        "order_linkage_status": "linked",
        "freshness": "fresh",
        "duplicate": False,
        "partial": True,
        "reasons": [],
    },
    "read_model.fill_execution.duplicate_fill.001": {
        "health_status": "fail_closed",
        "component_status": "fail_closed",
        "fill_status": "duplicate",
        "reconciliation_status": "duplicate_rejected",
        "order_linkage_status": "linked",
        "freshness": "fresh",
        "duplicate": True,
        "partial": False,
        "reasons": ["duplicate_fill"],
    },
    "read_model.fill_execution.missing_order_linkage.001": {
        "health_status": "fail_closed",
        "component_status": "fail_closed",
        "fill_status": "missing_linkage",
        "reconciliation_status": "missing_order_linkage",
        "order_linkage_status": "missing",
        "freshness": "fresh",
        "duplicate": False,
        "partial": False,
        "reasons": ["missing_order_linkage"],
    },
    "read_model.fill_execution.stale_source.001": {
        "health_status": "fail_closed",
        "component_status": "fail_closed",
        "fill_status": "stale_source",
        "reconciliation_status": "stale_execution_source",
        "order_linkage_status": "linked",
        "freshness": "stale",
        "duplicate": False,
        "partial": False,
        "reasons": ["stale_execution_source"],
    },
    "read_model.fill_execution.ambiguous_source.001": {
        "health_status": "fail_closed",
        "component_status": "fail_closed",
        "fill_status": "ambiguous_source",
        "reconciliation_status": "ambiguous_source",
        "order_linkage_status": "ambiguous",
        "freshness": "ambiguous",
        "duplicate": False,
        "partial": False,
        "reasons": ["ambiguous_fill_source"],
    },
}
REQUIRED_FILL_DATA = {
    "fill_id",
    "execution_id",
    "order_id",
    "client_order_id",
    "order_linkage_status",
    "fill_status",
    "reconciliation_status",
    "duplicate_fill_detected",
    "partial_fill_detected",
    "quantity",
    "cumulative_quantity",
    "remaining_quantity",
    "quantity_precision",
    "price",
    "price_precision",
    "precision_status",
    "source_provenance_ref",
    "risk_projection_input",
    "values_are_exchange_truth",
    "redaction_state",
    "no_execution_algorithm",
    "automatic_reconciliation_repair_allowed",
    "dashboard_readonly_visible",
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
    "dashboard_fill_controls_enabled",
    "execution_algorithm_allowed",
    "automatic_fill_repair_allowed",
    "automatic_reconciliation_repair_allowed",
}
FORBIDDEN_KEY_FRAGMENTS = (
    "api",
    "credential",
    "header",
    "raw",
    "secret",
    "signature",
    "signed",
)


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


def snapshot_from_row(row: dict[str, Any]) -> dict[str, Any]:
    events = row.get("input", {}).get("events", [])
    require(len(events) == 1, f"{row.get('case_id')}: expected one input event")
    snapshot = events[0].get("payload", {}).get("snapshot")
    require(isinstance(snapshot, dict), f"{row.get('case_id')}: missing input snapshot")
    return snapshot


def expected_payload(row: dict[str, Any]) -> dict[str, Any]:
    events = row.get("expected", {}).get("events", [])
    require(len(events) == 1, f"{row.get('case_id')}: expected one output event")
    payload = events[0].get("payload")
    require(isinstance(payload, dict), f"{row.get('case_id')}: missing expected payload")
    return payload


def walk_forbidden_keys(value: Any, path: str) -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            lowered = str(key).lower()
            for fragment in FORBIDDEN_KEY_FRAGMENTS:
                require(fragment not in lowered, f"{path}.{key} contains forbidden fragment {fragment}")
            walk_forbidden_keys(child, f"{path}.{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            walk_forbidden_keys(child, f"{path}[{index}]")


def require_false_boundary(snapshot: dict[str, Any], case_id: str) -> None:
    boundary = snapshot.get("capability_boundary")
    require(isinstance(boundary, dict), f"{case_id}: missing capability boundary")
    for key in sorted(FALSE_BOUNDARY_FLAGS):
        require(boundary.get(key) is False, f"{case_id}: {key} must be false")


def validate_case(row: dict[str, Any]) -> None:
    case_id = row.get("case_id")
    require(case_id in EXPECTED_CASES, f"unexpected fill/execution case {case_id}")
    expected = EXPECTED_CASES[case_id]
    snapshot = snapshot_from_row(row)
    payload = expected_payload(row)
    fills = snapshot.get("components", {}).get("fills")
    require(isinstance(fills, dict), f"{case_id}: missing fills component")
    data = fills.get("data")
    require(isinstance(data, dict), f"{case_id}: missing fills data")

    require(snapshot.get("contract_version") == CONTRACT_VERSION, f"{case_id}: contract version mismatch")
    require(snapshot.get("schema_version") == SCHEMA_VERSION, f"{case_id}: schema version mismatch")
    require(snapshot.get("health_status") == expected["health_status"], f"{case_id}: health status mismatch")
    require(payload.get("health_status") == expected["health_status"], f"{case_id}: expected payload health mismatch")
    require(fills.get("component_status") == expected["component_status"], f"{case_id}: fills status mismatch")
    require(fills.get("lineage", {}).get("transform") == FILL_TRANSFORM, f"{case_id}: fill transform mismatch")
    require(fills.get("freshness", {}).get("status") == expected["freshness"], f"{case_id}: fill freshness mismatch")
    require(REQUIRED_FILL_DATA <= set(data), f"{case_id}: missing fill data {sorted(REQUIRED_FILL_DATA - set(data))}")
    require(data.get("fill_status") == expected["fill_status"], f"{case_id}: fill status mismatch")
    require(data.get("reconciliation_status") == expected["reconciliation_status"], f"{case_id}: reconciliation status mismatch")
    require(data.get("order_linkage_status") == expected["order_linkage_status"], f"{case_id}: order linkage status mismatch")
    require(data.get("duplicate_fill_detected") is expected["duplicate"], f"{case_id}: duplicate flag mismatch")
    require(data.get("partial_fill_detected") is expected["partial"], f"{case_id}: partial flag mismatch")
    require(data.get("precision_status") == "valid", f"{case_id}: precision status must be valid for scoped cases")
    require(data.get("values_are_exchange_truth") is False, f"{case_id}: values must not claim exchange truth")
    require(data.get("redaction_state") == "redacted_refs_only", f"{case_id}: redaction state mismatch")
    require(data.get("no_execution_algorithm") is True, f"{case_id}: execution algorithm must be absent")
    require(data.get("automatic_reconciliation_repair_allowed") is False, f"{case_id}: automatic repair must be false")
    require(data.get("dashboard_readonly_visible") is True, f"{case_id}: dashboard read-only visibility required")
    require_false_boundary(snapshot, case_id)
    walk_forbidden_keys(data, f"{case_id}.fills.data")

    risk = data.get("risk_projection_input", {})
    for key in (
        "fill_reconciliation_status",
        "realized_fill_quantity",
        "remaining_order_quantity",
        "risk_state",
        "blocking_reasons",
        "automatic_reconciliation_repair_allowed",
        "execution_algorithm_allowed",
    ):
        require(key in risk, f"{case_id}: missing risk projection input {key}")
    require(risk.get("fill_reconciliation_status") == expected["reconciliation_status"], f"{case_id}: risk reconciliation mismatch")
    require(risk.get("automatic_reconciliation_repair_allowed") is False, f"{case_id}: risk repair must be false")
    require(risk.get("execution_algorithm_allowed") is False, f"{case_id}: risk execution algorithm must be false")

    if expected["partial"]:
        require(data.get("remaining_quantity") != "0", f"{case_id}: partial fill must expose remaining quantity")
        require(risk.get("risk_state") == "risk_visible_partial_fill", f"{case_id}: partial fill risk state mismatch")
    else:
        require(risk.get("risk_state") in {"risk_visible", "risk_blocked"}, f"{case_id}: risk state mismatch")

    if expected["reasons"]:
        for reason in expected["reasons"]:
            require(reason in snapshot.get("blocking_reasons", []), f"{case_id}: missing blocking reason {reason}")
            require(reason in payload.get("blocking_reasons", []), f"{case_id}: expected payload missing reason {reason}")
            require(reason in fills.get("diagnostics", []), f"{case_id}: missing fill diagnostic {reason}")
            require(reason in risk.get("blocking_reasons", []), f"{case_id}: risk input missing reason {reason}")
    else:
        require(snapshot.get("blocking_reasons") == [], f"{case_id}: non-fail case must not block")
        require(risk.get("blocking_reasons") == [], f"{case_id}: risk input must not block")

    if case_id.endswith("missing_order_linkage.001"):
        require(data.get("order_id") == "order-unavailable", f"{case_id}: missing linkage fixture must not invent order id")
    else:
        require(str(data.get("order_id", "")).startswith("order-redacted-"), f"{case_id}: order id must be redacted reference")


contract_path = Path(os.environ["CONTRACT_PATH"])
schema_path = Path(os.environ["UNIFIED_SCHEMA_PATH"])
trace_path = Path(os.environ["TRACE_PATH"])

contract = contract_path.read_text(encoding="utf-8")
schema = load_json(schema_path)
rows = load_jsonl(trace_path)

for marker in (
    FILL_TRANSFORM,
    "duplicate_fill",
    "missing_order_linkage",
    "stale_execution_source",
    "ambiguous_fill_source",
    "scripts/ai/verify_release.sh v21-fill-execution-read-model",
):
    require(marker in contract, f"contract missing marker {marker}")

require(schema.get("schema_version") == SCHEMA_VERSION, "unified schema version mismatch")
require(len(rows) == 6, "fill/execution golden trace must contain exactly six rows")
require({row.get("case_id") for row in rows} == set(EXPECTED_CASES), "unexpected fill/execution case set")

for row in rows:
    validate_case(row)

print("v21_fill_execution_read_model status=ok trace_cases=6 reconciled=covered partial_fill=covered duplicate_fill=covered missing_order_linkage=covered stale_execution_source=covered ambiguous_source=covered execution_algorithm=false automatic_reconciliation_repair=false")
PY
