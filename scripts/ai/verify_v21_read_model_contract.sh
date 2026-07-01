#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

CONTRACT_PATH="${NTPRO_V21_READ_MODEL_CONTRACT:-docs/rust-cutover/release/v0_21_0_unified_read_model_contract.md}"
SCHEMA_PATH="${NTPRO_V21_READ_MODEL_SCHEMA:-docs/rust-cutover/release/v0_21_0_unified_read_model_schema.json}"
TRACE_PATH="${NTPRO_V21_READ_MODEL_TRACE:-tests/golden/read_model_contract_schema.jsonl}"
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

for path in "$CONTRACT_PATH" "$SCHEMA_PATH" "$TRACE_PATH"; do
  if [ ! -f "$path" ]; then
    echo "missing required v21 read model contract file: $path" >&2
    exit 1
  fi
done

"$PYTHON_BIN" scripts/ai/golden_trace_runner.py "$TRACE_PATH" --mode validate-only

CONTRACT_PATH="$CONTRACT_PATH" SCHEMA_PATH="$SCHEMA_PATH" TRACE_PATH="$TRACE_PATH" "$PYTHON_BIN" <<'PY'
import json
import os
from pathlib import Path
from typing import Any

CONTRACT_VERSION = "ntpro.v210.unified_read_model.v1"
SCHEMA_VERSION = "ntpro.v210.unified_read_model.schema.v1"
REQUIRED_TOP_LEVEL = {
    "contract_version",
    "schema_version",
    "snapshot_id",
    "snapshot_identity",
    "as_of_unix_ns",
    "health_status",
    "freshness",
    "source_provenance",
    "lineage",
    "components",
    "blocking_reasons",
    "redaction",
    "capability_boundary",
}
REQUIRED_COMPONENTS = {
    "account",
    "positions",
    "orders",
    "fills",
    "risk",
    "lifecycle_status",
}
REQUIRED_COMPONENT_FIELDS = {
    "component_status",
    "source_provenance",
    "lineage",
    "freshness",
    "redaction",
    "data",
    "diagnostics",
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
}
FORBIDDEN_KEY_FRAGMENTS = (
    "api_key",
    "api_secret",
    "secret",
    "signed_url",
    "signed_query",
    "raw_response",
)
ALLOWED_FALSE_BOUNDARY_KEYS = {
    "raw_secret_persisted",
    "raw_exchange_response_persisted",
}


def fail(message: str) -> None:
    raise SystemExit(message)


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


def require(condition: bool, message: str) -> None:
    if not condition:
        fail(message)


def require_false_flags(boundary: dict[str, Any], path: str) -> None:
    for key in sorted(FALSE_BOUNDARY_FLAGS):
        require(boundary.get(key) is False, f"{path}.{key} must be false")


def walk_forbidden_keys(value: Any, path: str) -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            lowered = str(key).lower()
            if lowered not in ALLOWED_FALSE_BOUNDARY_KEYS:
                for fragment in FORBIDDEN_KEY_FRAGMENTS:
                    require(fragment not in lowered, f"{path}.{key} contains forbidden raw/sensitive key fragment {fragment}")
            walk_forbidden_keys(child, f"{path}.{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            walk_forbidden_keys(child, f"{path}[{index}]")


def validate_schema(schema: dict[str, Any]) -> None:
    require(schema.get("schema_version") == SCHEMA_VERSION, "schema version mismatch")
    require(schema.get("properties", {}).get("contract_version", {}).get("const") == CONTRACT_VERSION, "contract_version const missing")
    required = set(schema.get("required", []))
    require(REQUIRED_TOP_LEVEL <= required, f"schema missing top-level required fields: {sorted(REQUIRED_TOP_LEVEL - required)}")
    unified_component_requirements = []
    for clause in schema.get("allOf", []):
        condition = clause.get("if", {}).get("properties", {}).get("snapshot_kind", {})
        if condition.get("const") == "unified_snapshot":
            required_components = (
                clause.get("then", {})
                .get("properties", {})
                .get("components", {})
                .get("required", [])
            )
            unified_component_requirements.extend(required_components)
    require(REQUIRED_COMPONENTS <= set(unified_component_requirements), "schema missing unified_snapshot required components")
    boundary = schema.get("properties", {}).get("capability_boundary", {})
    boundary_required = schema.get("$defs", {}).get("capability_boundary", {}).get("required", boundary.get("required", []))
    require(FALSE_BOUNDARY_FLAGS <= set(boundary_required), "schema missing false boundary flags")


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


def validate_healthy_snapshot(snapshot: dict[str, Any], payload: dict[str, Any]) -> None:
    require(set(snapshot) >= REQUIRED_TOP_LEVEL, "healthy snapshot missing required top-level fields")
    require(snapshot.get("contract_version") == CONTRACT_VERSION, "healthy contract version mismatch")
    require(snapshot.get("schema_version") == SCHEMA_VERSION, "healthy schema version mismatch")
    require(snapshot.get("health_status") == "healthy", "healthy snapshot must be healthy")
    require(payload.get("health_status") == "healthy", "healthy expected payload must be healthy")
    require(snapshot.get("blocking_reasons") == [], "healthy snapshot must not have blocking reasons")
    components = snapshot.get("components")
    require(isinstance(components, dict), "healthy components must be object")
    require(REQUIRED_COMPONENTS == set(components), "healthy snapshot must contain the minimal component set")
    for name, component in components.items():
        require(isinstance(component, dict), f"{name}: component must be object")
        missing = REQUIRED_COMPONENT_FIELDS - set(component)
        require(not missing, f"{name}: healthy component missing fields {sorted(missing)}")
        require(component.get("component_status") == "healthy", f"{name}: healthy component status required")
        require(component.get("freshness", {}).get("status") == "fresh", f"{name}: freshness must be fresh")
    require_false_flags(snapshot.get("capability_boundary", {}), "healthy.capability_boundary")
    walk_forbidden_keys(snapshot, "healthy")


def validate_fail_closed_snapshot(snapshot: dict[str, Any], payload: dict[str, Any]) -> None:
    require(snapshot.get("health_status") == "fail_closed", "fail-closed snapshot must not be healthy")
    require(payload.get("health_status") == "fail_closed", "fail-closed expected payload must be fail_closed")
    reasons = snapshot.get("blocking_reasons")
    require(isinstance(reasons, list) and reasons, "fail-closed snapshot must have blocking reasons")
    for expected_reason in (
        "missing_component_lineage:account",
        "missing_component_source_provenance:orders",
        "missing_component_freshness:risk",
        "stale_component_freshness:lifecycle_status",
    ):
        require(expected_reason in reasons, f"missing fail-closed reason {expected_reason}")
        require(expected_reason in payload.get("blocking_reasons", []), f"expected payload missing reason {expected_reason}")

    components = snapshot.get("components", {})
    require(components.get("account", {}).get("lineage", {}).get("transform", "").startswith("missing:"), "account must expose unavailable lineage in fail-closed smoke")
    require(components.get("orders", {}).get("source_provenance", {}).get("source_type") == "unavailable", "orders must expose unavailable source provenance in fail-closed smoke")
    require(components.get("risk", {}).get("freshness", {}).get("status") == "missing", "risk must expose missing freshness in fail-closed smoke")
    require(components.get("lifecycle_status", {}).get("freshness", {}).get("status") == "stale", "lifecycle_status must be stale")
    for name in ("account", "orders", "risk", "lifecycle_status"):
        require(components.get(name, {}).get("component_status") == "fail_closed", f"{name}: fail_closed status required")
    require_false_flags(snapshot.get("capability_boundary", {}), "fail_closed.capability_boundary")
    walk_forbidden_keys(snapshot, "fail_closed")


contract = Path(os.environ["CONTRACT_PATH"]).read_text(encoding="utf-8")
schema = load_json(Path(os.environ["SCHEMA_PATH"]))
rows = load_jsonl(Path(os.environ["TRACE_PATH"]))

for marker in (
    CONTRACT_VERSION,
    "missing_component_lineage",
    "missing_component_source_provenance",
    "missing_component_freshness",
    "stale_component_freshness",
    "scripts/ai/verify_release.sh v21-read-model-contract",
):
    require(marker in contract, f"contract missing marker {marker}")

validate_schema(schema)
require(len(rows) == 2, "read model golden trace smoke must contain exactly two rows")
case_ids = {row.get("case_id") for row in rows}
require(case_ids == {
    "read_model.contract.healthy_minimal.001",
    "read_model.contract.fail_closed_missing_lineage_source_freshness.001",
}, "unexpected read model contract case IDs")

for row in rows:
    snapshot = snapshot_from_row(row)
    payload = expected_payload(row)
    if row["case_id"].endswith("healthy_minimal.001"):
        validate_healthy_snapshot(snapshot, payload)
    else:
        validate_fail_closed_snapshot(snapshot, payload)

print("v21_read_model_contract status=ok schema=valid trace_cases=2 fail_closed_rules=covered new_submit_capability=false dashboard_order_controls=false")
PY
