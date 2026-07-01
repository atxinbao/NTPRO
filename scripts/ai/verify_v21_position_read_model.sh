#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

CONTRACT_PATH="${NTPRO_V21_POSITION_CONTRACT:-docs/rust-cutover/release/v0_21_0_position_read_model.md}"
UNIFIED_SCHEMA_PATH="${NTPRO_V21_READ_MODEL_SCHEMA:-docs/rust-cutover/release/v0_21_0_unified_read_model_schema.json}"
TRACE_PATH="${NTPRO_V21_POSITION_TRACE:-tests/golden/read_model_position_schema.jsonl}"
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
    echo "missing required v21 position read model file: $path" >&2
    exit 1
  fi
done

"$PYTHON_BIN" scripts/ai/golden_trace_runner.py "$TRACE_PATH" --mode validate-only

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
POSITION_TRANSFORM = "ntpro.v210.position_read_model.v1"
EXPECTED_CASES = {
    "read_model.position.long.001": {
        "health_status": "degraded",
        "component_status": "healthy",
        "side": "long",
        "reasons": [],
        "freshness": "fresh",
        "precision_status": "valid",
    },
    "read_model.position.short.001": {
        "health_status": "degraded",
        "component_status": "healthy",
        "side": "short",
        "reasons": [],
        "freshness": "fresh",
        "precision_status": "valid",
    },
    "read_model.position.flat.001": {
        "health_status": "degraded",
        "component_status": "healthy",
        "side": "flat",
        "reasons": [],
        "freshness": "fresh",
        "precision_status": "valid",
    },
    "read_model.position.precision_mismatch.001": {
        "health_status": "fail_closed",
        "component_status": "fail_closed",
        "side": "long",
        "reasons": ["position_quantity_precision_mismatch"],
        "freshness": "fresh",
        "precision_status": "mismatch",
    },
    "read_model.position.stale_source.001": {
        "health_status": "fail_closed",
        "component_status": "fail_closed",
        "side": "long",
        "reasons": ["stale_position_source"],
        "freshness": "stale",
        "precision_status": "valid",
    },
    "read_model.position.account_mismatch.001": {
        "health_status": "fail_closed",
        "component_status": "fail_closed",
        "side": "short",
        "reasons": ["account_position_lineage_mismatch"],
        "freshness": "fresh",
        "precision_status": "valid",
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
    "auto_flatten_position_allowed",
    "automatic_position_repair_allowed",
}
REQUIRED_POSITION_DATA = {
    "position_count",
    "net_position_side",
    "quantity",
    "quantity_precision",
    "instrument_quantity_precision",
    "precision_status",
    "cost_basis",
    "mark_price",
    "notional",
    "risk_projection_input",
    "instrument_identity",
    "position_id",
    "account_id_ref",
    "values_are_exchange_truth",
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
    require(case_id in EXPECTED_CASES, f"unexpected position case {case_id}")
    expected = EXPECTED_CASES[case_id]
    snapshot = snapshot_from_row(row)
    payload = expected_payload(row)
    positions = snapshot.get("components", {}).get("positions")
    require(isinstance(positions, dict), f"{case_id}: missing positions component")
    data = positions.get("data")
    require(isinstance(data, dict), f"{case_id}: missing positions data")

    require(snapshot.get("contract_version") == CONTRACT_VERSION, f"{case_id}: contract version mismatch")
    require(snapshot.get("schema_version") == SCHEMA_VERSION, f"{case_id}: schema version mismatch")
    require(snapshot.get("health_status") == expected["health_status"], f"{case_id}: health status mismatch")
    require(payload.get("health_status") == expected["health_status"], f"{case_id}: expected payload health mismatch")
    require(positions.get("component_status") == expected["component_status"], f"{case_id}: position status mismatch")
    require(positions.get("lineage", {}).get("transform") == POSITION_TRANSFORM, f"{case_id}: position transform mismatch")
    require(positions.get("freshness", {}).get("status") == expected["freshness"], f"{case_id}: position freshness mismatch")
    require(REQUIRED_POSITION_DATA <= set(data), f"{case_id}: missing position data {sorted(REQUIRED_POSITION_DATA - set(data))}")
    require(data.get("net_position_side") == expected["side"], f"{case_id}: side mismatch")
    require(data.get("precision_status") == expected["precision_status"], f"{case_id}: precision status mismatch")
    require(data.get("values_are_exchange_truth") is False, f"{case_id}: values must not claim exchange truth")
    require_false_boundary(snapshot, case_id)
    walk_forbidden_keys(data, f"{case_id}.positions.data")

    instrument = data.get("instrument_identity", {})
    require(instrument.get("instrument_id") == "BTCUSDT.BINANCE", f"{case_id}: instrument identity mismatch")
    risk = data.get("risk_projection_input", {})
    for key in ("current_position_notional", "projected_position_notional", "max_position_notional", "risk_state"):
        require(key in risk, f"{case_id}: missing risk projection input {key}")
    require(risk.get("automatic_position_repair_allowed") is False, f"{case_id}: automatic repair must be false")
    require(risk.get("auto_flatten_position_allowed") is False, f"{case_id}: auto flatten must be false")

    if not expected["reasons"]:
        require(snapshot.get("blocking_reasons") == [], f"{case_id}: non-blocking component case must not block")
        require(data.get("position_count") in (0, 1), f"{case_id}: position count must be scoped")
        if expected["side"] == "flat":
            require(data.get("quantity") == "0", f"{case_id}: flat quantity must be zero")
    else:
        for reason in expected["reasons"]:
            require(reason in snapshot.get("blocking_reasons", []), f"{case_id}: missing blocking reason {reason}")
            require(reason in payload.get("blocking_reasons", []), f"{case_id}: expected payload missing reason {reason}")
            require(reason in positions.get("diagnostics", []), f"{case_id}: missing position diagnostic {reason}")

    account_id = snapshot.get("snapshot_identity", {}).get("account_id")
    if case_id.endswith("account_mismatch.001"):
        require(data.get("account_id_ref") != account_id, f"{case_id}: mismatch fixture must differ from account identity")
    else:
        require(data.get("account_id_ref") == account_id, f"{case_id}: account and position lineage must match")


contract_path = Path(os.environ["CONTRACT_PATH"])
schema_path = Path(os.environ["UNIFIED_SCHEMA_PATH"])
trace_path = Path(os.environ["TRACE_PATH"])

contract = contract_path.read_text(encoding="utf-8")
schema = load_json(schema_path)
rows = load_jsonl(trace_path)

for marker in (
    POSITION_TRANSFORM,
    "position_quantity_precision_mismatch",
    "stale_position_source",
    "account_position_lineage_mismatch",
    "scripts/ai/verify_release.sh v21-position-read-model",
):
    require(marker in contract, f"contract missing marker {marker}")

require(schema.get("schema_version") == SCHEMA_VERSION, "unified schema version mismatch")
require(len(rows) == 6, "position golden trace must contain exactly six rows")
require({row.get("case_id") for row in rows} == set(EXPECTED_CASES), "unexpected position case set")

for row in rows:
    validate_case(row)

print("v21_position_read_model status=ok trace_cases=6 long=covered short=covered flat=covered precision_mismatch=covered stale_source=covered account_position_mismatch=covered auto_flatten=false automatic_repair=false")
PY
