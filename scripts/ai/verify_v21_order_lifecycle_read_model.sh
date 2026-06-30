#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

CONTRACT_PATH="${NTPRO_V21_ORDER_CONTRACT:-docs/rust-cutover/release/v0_21_0_order_lifecycle_read_model.md}"
UNIFIED_SCHEMA_PATH="${NTPRO_V21_READ_MODEL_SCHEMA:-docs/rust-cutover/release/v0_21_0_unified_read_model_schema.json}"
TRACE_PATH="${NTPRO_V21_ORDER_TRACE:-tests/golden/read_model_order_lifecycle_schema.jsonl}"
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
    echo "missing required v21 order lifecycle read model file: $path" >&2
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
ORDER_TRANSFORM = "ntpro.v210.order_lifecycle_read_model.v1"
EXPECTED_CASES = {
    "read_model.order_lifecycle.matched.001": {
        "health_status": "healthy",
        "component_status": "healthy",
        "lifecycle_status": "readback_matched",
        "readback_status": "matched",
        "ledger_present": True,
        "duplicate_attempt": False,
        "reasons": [],
    },
    "read_model.order_lifecycle.unknown_response.001": {
        "health_status": "fail_closed",
        "component_status": "fail_closed",
        "lifecycle_status": "unknown_response",
        "readback_status": "unavailable",
        "ledger_present": True,
        "duplicate_attempt": False,
        "reasons": ["unknown_order_response_no_retry"],
    },
    "read_model.order_lifecycle.readback_mismatch.001": {
        "health_status": "fail_closed",
        "component_status": "fail_closed",
        "lifecycle_status": "readback_mismatch",
        "readback_status": "mismatch",
        "ledger_present": True,
        "duplicate_attempt": False,
        "reasons": ["order_readback_mismatch"],
    },
    "read_model.order_lifecycle.duplicate_attempt.001": {
        "health_status": "fail_closed",
        "component_status": "fail_closed",
        "lifecycle_status": "duplicate_attempt",
        "readback_status": "not_attempted",
        "ledger_present": True,
        "duplicate_attempt": True,
        "reasons": ["duplicate_submit_attempt"],
    },
    "read_model.order_lifecycle.missing_ledger.001": {
        "health_status": "fail_closed",
        "component_status": "fail_closed",
        "lifecycle_status": "missing_ledger",
        "readback_status": "not_attempted",
        "ledger_present": False,
        "duplicate_attempt": False,
        "reasons": ["missing_attempt_ledger"],
    },
}
REQUIRED_ORDER_DATA = {
    "order_id",
    "client_order_id",
    "request_digest",
    "attempt_id",
    "approval_id",
    "ledger_present",
    "duplicate_attempt_detected",
    "lifecycle_status",
    "submitted",
    "accepted",
    "rejected",
    "readback_status",
    "cancel_evidence_state",
    "audit_state",
    "refs",
    "redaction_state",
    "no_retry",
    "automatic_remediation_allowed",
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
    "retry_order_allowed",
    "automatic_order_remediation_allowed",
    "automatic_cancel_allowed",
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
    require(case_id in EXPECTED_CASES, f"unexpected order lifecycle case {case_id}")
    expected = EXPECTED_CASES[case_id]
    snapshot = snapshot_from_row(row)
    payload = expected_payload(row)
    orders = snapshot.get("components", {}).get("orders")
    require(isinstance(orders, dict), f"{case_id}: missing orders component")
    data = orders.get("data")
    require(isinstance(data, dict), f"{case_id}: missing orders data")

    require(snapshot.get("contract_version") == CONTRACT_VERSION, f"{case_id}: contract version mismatch")
    require(snapshot.get("schema_version") == SCHEMA_VERSION, f"{case_id}: schema version mismatch")
    require(snapshot.get("health_status") == expected["health_status"], f"{case_id}: health status mismatch")
    require(payload.get("health_status") == expected["health_status"], f"{case_id}: expected payload health mismatch")
    require(orders.get("component_status") == expected["component_status"], f"{case_id}: order component status mismatch")
    require(orders.get("lineage", {}).get("transform") == ORDER_TRANSFORM, f"{case_id}: order transform mismatch")
    require(REQUIRED_ORDER_DATA <= set(data), f"{case_id}: missing order data {sorted(REQUIRED_ORDER_DATA - set(data))}")
    require(data.get("lifecycle_status") == expected["lifecycle_status"], f"{case_id}: lifecycle status mismatch")
    require(data.get("readback_status") == expected["readback_status"], f"{case_id}: readback status mismatch")
    require(data.get("ledger_present") is expected["ledger_present"], f"{case_id}: ledger presence mismatch")
    require(data.get("duplicate_attempt_detected") is expected["duplicate_attempt"], f"{case_id}: duplicate attempt mismatch")
    require(data.get("redaction_state") == "redacted_refs_only", f"{case_id}: redaction state mismatch")
    require(data.get("no_retry") is True, f"{case_id}: no_retry must be true")
    require(data.get("automatic_remediation_allowed") is False, f"{case_id}: automatic remediation must be false")
    require(data.get("dashboard_readonly_visible") is True, f"{case_id}: dashboard read-only visibility required")
    require_false_boundary(snapshot, case_id)
    walk_forbidden_keys(data, f"{case_id}.orders.data")

    refs = data.get("refs", {})
    for key in ("candidate_ref", "attempt_ref", "approval_ref", "audit_ref", "provenance_ref"):
        require(key in refs, f"{case_id}: missing ref {key}")

    if expected["health_status"] == "healthy":
        require(snapshot.get("blocking_reasons") == [], f"{case_id}: healthy case must not block")
        require(data.get("submitted") is True, f"{case_id}: matched case must record submitted")
        require(data.get("accepted") is True, f"{case_id}: matched case must record accepted")
        require(data.get("audit_state") == "audit_closed", f"{case_id}: matched case must close audit")
    else:
        for reason in expected["reasons"]:
            require(reason in snapshot.get("blocking_reasons", []), f"{case_id}: missing blocking reason {reason}")
            require(reason in payload.get("blocking_reasons", []), f"{case_id}: expected payload missing reason {reason}")
            require(reason in orders.get("diagnostics", []), f"{case_id}: missing order diagnostic {reason}")


contract_path = Path(os.environ["CONTRACT_PATH"])
schema_path = Path(os.environ["UNIFIED_SCHEMA_PATH"])
trace_path = Path(os.environ["TRACE_PATH"])

contract = contract_path.read_text(encoding="utf-8")
schema = load_json(schema_path)
rows = load_jsonl(trace_path)

for marker in (
    ORDER_TRANSFORM,
    "unknown_order_response_no_retry",
    "order_readback_mismatch",
    "duplicate_submit_attempt",
    "missing_attempt_ledger",
    "scripts/ai/verify_release.sh v21-order-lifecycle-read-model",
):
    require(marker in contract, f"contract missing marker {marker}")

require(schema.get("schema_version") == SCHEMA_VERSION, "unified schema version mismatch")
require(len(rows) == 5, "order lifecycle golden trace must contain exactly five rows")
require({row.get("case_id") for row in rows} == set(EXPECTED_CASES), "unexpected order lifecycle case set")

for row in rows:
    validate_case(row)

print("v21_order_lifecycle_read_model status=ok trace_cases=5 matched=covered unknown_response=covered readback_mismatch=covered duplicate_attempt=covered missing_ledger=covered no_retry=true automatic_remediation=false")
PY
