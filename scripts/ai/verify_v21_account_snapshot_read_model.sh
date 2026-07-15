#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

CONTRACT_PATH="${NTPRO_V21_ACCOUNT_SNAPSHOT_CONTRACT:-docs/rust-cutover/release/v0_21_0_account_snapshot_read_model.md}"
UNIFIED_SCHEMA_PATH="${NTPRO_V21_READ_MODEL_SCHEMA:-docs/rust-cutover/release/v0_21_0_unified_read_model_schema.json}"
TRACE_PATH="${NTPRO_V21_ACCOUNT_SNAPSHOT_TRACE:-tests/golden/read_model_account_snapshot_schema.jsonl}"
DASHBOARD_PATH="${NTPRO_V21_DASHBOARD_PATH:-crates/cli/src/dashboard.rs}"
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

for path in "$CONTRACT_PATH" "$UNIFIED_SCHEMA_PATH" "$TRACE_PATH" "$DASHBOARD_PATH"; do
  if [ ! -f "$path" ]; then
    echo "missing required v21 account snapshot read model file: $path" >&2
    exit 1
  fi
done

scripts/ai/ntpro_governance.sh golden-trace "$TRACE_PATH" --mode validate-only

CONTRACT_PATH="$CONTRACT_PATH" \
UNIFIED_SCHEMA_PATH="$UNIFIED_SCHEMA_PATH" \
TRACE_PATH="$TRACE_PATH" \
DASHBOARD_PATH="$DASHBOARD_PATH" \
"$PYTHON_BIN" <<'PY'
import json
import os
from pathlib import Path
from typing import Any

CONTRACT_VERSION = "ntpro.v210.unified_read_model.v1"
SCHEMA_VERSION = "ntpro.v210.unified_read_model.schema.v1"
ACCOUNT_TRANSFORM = "ntpro.v210.account_snapshot_read_model.v1"
EXPECTED_CASES = {
    "read_model.account_snapshot.fresh.001": {
        "health_status": "degraded",
        "account_status": "healthy",
        "reasons": [],
        "freshness": "fresh",
        "redaction_status": "redacted",
    },
    "read_model.account_snapshot.stale.001": {
        "health_status": "fail_closed",
        "account_status": "fail_closed",
        "reasons": ["stale_account_freshness"],
        "freshness": "stale",
        "redaction_status": "redacted",
    },
    "read_model.account_snapshot.missing_provenance.001": {
        "health_status": "fail_closed",
        "account_status": "fail_closed",
        "reasons": ["missing_account_source_provenance"],
        "freshness": "fresh",
        "redaction_status": "redacted",
    },
    "read_model.account_snapshot.redaction_breach.001": {
        "health_status": "fail_closed",
        "account_status": "fail_closed",
        "reasons": ["unredacted_sensitive_field", "raw_account_payload_persisted"],
        "freshness": "fresh",
        "redaction_status": "fail_closed",
    },
}
ACCOUNT_DATA_FIELDS = {
    "account_status",
    "balance_entry_count",
    "available_balance",
    "margin_available",
    "equity",
    "risk_state",
    "response_shape",
    "response_shape_validated",
}
FORBIDDEN_ACCOUNT_DATA_FRAGMENTS = (
    "api",
    "credential",
    "header",
    "permission",
    "raw",
    "secret",
    "signature",
    "signed",
    "uid",
)
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
    "funds_transfer_allowed",
    "account_configuration_mutation_allowed",
    "order_permission_control_allowed",
}
FORBIDDEN_DASHBOARD_CONTROL_MARKERS = (
    "funds_transfer_button",
    "account_transfer_button",
    "account_configuration_mutation_button",
    "account_submit_button",
    "account_cancel_button",
    "account_retry_button",
    "order_permission_control_button",
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


def require_false_boundary(snapshot: dict[str, Any], case_id: str) -> None:
    boundary = snapshot.get("capability_boundary")
    require(isinstance(boundary, dict), f"{case_id}: missing capability boundary")
    for key in sorted(FALSE_BOUNDARY_FLAGS):
        require(boundary.get(key) is False, f"{case_id}: {key} must be false")


def require_account_data_redacted(data: dict[str, Any], case_id: str) -> None:
    require(ACCOUNT_DATA_FIELDS <= set(data), f"{case_id}: account data missing fields {sorted(ACCOUNT_DATA_FIELDS - set(data))}")
    for key in data:
        lowered = key.lower()
        for fragment in FORBIDDEN_ACCOUNT_DATA_FRAGMENTS:
            require(fragment not in lowered, f"{case_id}: account data key {key} exposes forbidden fragment {fragment}")
    require(data.get("response_shape") == "binance_account_snapshot_v1", f"{case_id}: response shape mismatch")
    require(data.get("risk_state") == "risk_visible", f"{case_id}: account risk state must remain visible")


def validate_case(row: dict[str, Any]) -> None:
    case_id = row.get("case_id")
    require(case_id in EXPECTED_CASES, f"unexpected account snapshot case {case_id}")
    expected = EXPECTED_CASES[case_id]
    snapshot = snapshot_from_row(row)
    payload = expected_payload(row)
    account = snapshot.get("components", {}).get("account")
    require(isinstance(account, dict), f"{case_id}: missing account component")
    data = account.get("data")
    require(isinstance(data, dict), f"{case_id}: missing account data")

    require(snapshot.get("contract_version") == CONTRACT_VERSION, f"{case_id}: contract version mismatch")
    require(snapshot.get("schema_version") == SCHEMA_VERSION, f"{case_id}: schema version mismatch")
    require(snapshot.get("health_status") == expected["health_status"], f"{case_id}: health status mismatch")
    require(payload.get("health_status") == expected["health_status"], f"{case_id}: expected payload health mismatch")
    require(account.get("component_status") == expected["account_status"], f"{case_id}: account component status mismatch")
    require(account.get("lineage", {}).get("transform") == ACCOUNT_TRANSFORM, f"{case_id}: account transform mismatch")
    require(account.get("freshness", {}).get("status") == expected["freshness"], f"{case_id}: account freshness mismatch")
    require(account.get("redaction", {}).get("status") == expected["redaction_status"], f"{case_id}: account redaction status mismatch")
    require_account_data_redacted(data, case_id)
    require_false_boundary(snapshot, case_id)

    for reason in expected["reasons"]:
        require(reason in snapshot.get("blocking_reasons", []), f"{case_id}: missing blocking reason {reason}")
        require(reason in payload.get("blocking_reasons", []), f"{case_id}: expected payload missing reason {reason}")
        require(reason in account.get("diagnostics", []) or reason in snapshot.get("blocking_reasons", []), f"{case_id}: missing account diagnostic {reason}")

    if not expected["reasons"]:
        require(snapshot.get("blocking_reasons") == [], f"{case_id}: non-blocking component case must not block")
        require(account.get("source_provenance", {}).get("redaction_state") == "redacted", f"{case_id}: healthy source must be redacted")
        require(data.get("response_shape_validated") is True, f"{case_id}: fresh case must validate response shape")
    else:
        require(snapshot.get("blocking_reasons"), f"{case_id}: fail-closed case requires blocking reasons")
        require(data.get("response_shape_validated") is False or case_id.endswith("stale.001"), f"{case_id}: blocked non-stale case must not claim shape validation")

    if case_id.endswith("missing_provenance.001"):
        provenance = account.get("source_provenance", {})
        require(provenance.get("source_type") == "unavailable", f"{case_id}: account source provenance must be unavailable in missing provenance smoke")
        require(provenance.get("redaction_state") == "unavailable", f"{case_id}: account redaction state must be unavailable in missing provenance smoke")
        require(provenance.get("exchange_truth") is False, f"{case_id}: unavailable provenance must not claim exchange truth")
        require(provenance.get("adapter_runtime_integrated") is False, f"{case_id}: unavailable provenance must not claim adapter runtime")
    else:
        require(isinstance(account.get("source_provenance"), dict), f"{case_id}: account source provenance required")

    if case_id.endswith("redaction_breach.001"):
        require(account.get("redaction", {}).get("credential_material_persisted") is False, f"{case_id}: credential material must still be schema-fail-closed")
        require(account.get("redaction", {}).get("raw_account_payload_persisted") is False, f"{case_id}: raw account payload must still be schema-fail-closed")
        require("credential_material_detected" in account.get("diagnostics", []), f"{case_id}: redaction breach fixture must record credential material detection")
        require("raw_account_payload_detected" in account.get("diagnostics", []), f"{case_id}: redaction breach fixture must record raw payload detection")
    else:
        require(account.get("redaction", {}).get("credential_material_persisted") is False, f"{case_id}: credential material must not persist")
        require(account.get("redaction", {}).get("raw_account_payload_persisted") is False, f"{case_id}: raw account payload must not persist")


contract_path = Path(os.environ["CONTRACT_PATH"])
schema_path = Path(os.environ["UNIFIED_SCHEMA_PATH"])
trace_path = Path(os.environ["TRACE_PATH"])
dashboard_path = Path(os.environ["DASHBOARD_PATH"])

contract = contract_path.read_text(encoding="utf-8")
schema = load_json(schema_path)
rows = load_jsonl(trace_path)
dashboard = dashboard_path.read_text(encoding="utf-8")

for marker in (
    ACCOUNT_TRANSFORM,
    "missing_account_source_provenance",
    "stale_account_freshness",
    "unredacted_sensitive_field",
    "scripts/ai/verify_release.sh v21-account-snapshot-read-model",
):
    require(marker in contract, f"contract missing marker {marker}")

require(schema.get("schema_version") == SCHEMA_VERSION, "unified schema version mismatch")
require(len(rows) == 4, "account snapshot golden trace must contain exactly four rows")
require({row.get("case_id") for row in rows} == set(EXPECTED_CASES), "unexpected account snapshot case set")

for row in rows:
    validate_case(row)

for marker in ("account_snapshot_status", "account_snapshot_endpoint_class", "account_snapshot_path"):
    require(marker in dashboard, f"dashboard missing read-only account marker {marker}")
for marker in FORBIDDEN_DASHBOARD_CONTROL_MARKERS:
    require(marker not in dashboard, f"dashboard contains forbidden account operation control marker {marker}")

print("v21_account_snapshot_read_model status=ok trace_cases=4 fresh=covered stale=covered missing_provenance=covered redaction_breach=covered dashboard_account_state_visible=true dashboard_operation_controls=false")
PY
