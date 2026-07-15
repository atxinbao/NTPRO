#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

CONTRACT_PATH="${NTPRO_V211_HEALTH_STATUS_CONTRACT:-docs/rust-cutover/release/v0_21_0_unified_read_model_contract.md}"
SCHEMA_PATH="${NTPRO_V211_HEALTH_STATUS_SCHEMA:-docs/rust-cutover/release/v0_21_0_unified_read_model_schema.json}"
SEMANTICS_TRACE_PATH="${NTPRO_V211_HEALTH_STATUS_TRACE:-tests/golden/v211/read_model_health_status_semantics_schema.jsonl}"
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

TRACE_PATHS=(
  "$SEMANTICS_TRACE_PATH"
  tests/golden/read_model_contract_schema.jsonl
  tests/golden/read_model_account_snapshot_schema.jsonl
  tests/golden/read_model_position_schema.jsonl
  tests/golden/read_model_order_lifecycle_schema.jsonl
  tests/golden/read_model_fill_execution_schema.jsonl
  tests/golden/read_model_risk_state_schema.jsonl
  tests/golden/read_model_dashboard_schema.jsonl
)

for path in "$CONTRACT_PATH" "$SCHEMA_PATH" "${TRACE_PATHS[@]}"; do
  if [ ! -f "$path" ]; then
    echo "missing required V211 health status semantics file: $path" >&2
    exit 1
  fi
done

scripts/ai/ntpro_governance.sh golden-trace "$SEMANTICS_TRACE_PATH" --mode validate-only

CONTRACT_PATH="$CONTRACT_PATH" \
SCHEMA_PATH="$SCHEMA_PATH" \
SEMANTICS_TRACE_PATH="$SEMANTICS_TRACE_PATH" \
"$PYTHON_BIN" <<'PY'
import json
import os
from pathlib import Path
from typing import Any

REQUIRED_COMPONENTS = {
    "account",
    "positions",
    "orders",
    "fills",
    "risk",
    "lifecycle_status",
}
REQUIRED_COMPONENT_EVIDENCE = {
    "source_provenance",
    "lineage",
    "freshness",
    "redaction",
}
COMPONENT_TRACE_PATHS = (
    Path("tests/golden/read_model_account_snapshot_schema.jsonl"),
    Path("tests/golden/read_model_position_schema.jsonl"),
    Path("tests/golden/read_model_order_lifecycle_schema.jsonl"),
    Path("tests/golden/read_model_fill_execution_schema.jsonl"),
    Path("tests/golden/read_model_risk_state_schema.jsonl"),
)
DASHBOARD_TRACE_PATH = Path("tests/golden/read_model_dashboard_schema.jsonl")
UNIFIED_TRACE_PATH = Path("tests/golden/read_model_contract_schema.jsonl")
EXPECTED_SEMANTICS_CASES = {
    "read_model.health_status.component_snapshot.account_local_healthy.001",
    "read_model.health_status.unified_snapshot.full_healthy.001",
    "read_model.health_status.dashboard_view.missing_component_degraded.001",
    "read_model.health_status.unified_snapshot.fail_closed_missing_evidence.001",
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


def component_is_complete_healthy(component: Any) -> bool:
    if not isinstance(component, dict):
        return False
    if component.get("component_status") != "healthy":
        return False
    if component.get("freshness", {}).get("status") != "fresh":
        return False
    return REQUIRED_COMPONENT_EVIDENCE <= set(component)


def unified_healthy_ready(snapshot: dict[str, Any]) -> bool:
    components = snapshot.get("components")
    if not isinstance(components, dict):
        return False
    if set(components) != REQUIRED_COMPONENTS:
        return False
    return all(component_is_complete_healthy(components.get(name)) for name in REQUIRED_COMPONENTS)


def has_missing_or_unavailable_component(snapshot: dict[str, Any]) -> bool:
    components = snapshot.get("components")
    if not isinstance(components, dict):
        return True
    if REQUIRED_COMPONENTS - set(components):
        return True
    for component in components.values():
        if not isinstance(component, dict):
            return True
        if component.get("component_status") in {"degraded", "fail_closed", "unavailable"}:
            return True
        if component.get("freshness", {}).get("status") in {"missing", "stale", "ambiguous"}:
            return True
    return False


def validate_semantics_fixture(row: dict[str, Any]) -> None:
    case_id = row.get("case_id")
    require(case_id in EXPECTED_SEMANTICS_CASES, f"unexpected semantics case {case_id}")
    snapshot = snapshot_from_row(row)
    payload = expected_payload(row)
    kind = snapshot.get("snapshot_kind")

    require(payload.get("snapshot_kind") == kind, f"{case_id}: expected payload kind mismatch")
    require(payload.get("health_status") == snapshot.get("health_status"), f"{case_id}: expected payload health mismatch")

    if case_id.endswith("account_local_healthy.001"):
        require(kind == "component_snapshot", f"{case_id}: kind mismatch")
        require(snapshot.get("health_status") == "degraded", f"{case_id}: local component snapshot must not be top-level healthy")
        require(snapshot.get("components", {}).get("account", {}).get("component_status") == "healthy", f"{case_id}: account component must remain healthy")
        require(set(snapshot.get("missing_required_components", [])) == REQUIRED_COMPONENTS - {"account"}, f"{case_id}: missing components mismatch")
        require(payload.get("component_health_status") == "healthy", f"{case_id}: expected component health mismatch")
        return

    if case_id.endswith("full_healthy.001"):
        require(kind == "unified_snapshot", f"{case_id}: kind mismatch")
        require(snapshot.get("health_status") == "healthy", f"{case_id}: full unified snapshot must be healthy")
        require(unified_healthy_ready(snapshot), f"{case_id}: full unified healthy requires every required component to be complete and healthy")
        return

    if case_id.endswith("missing_component_degraded.001"):
        require(kind == "dashboard_view", f"{case_id}: kind mismatch")
        require(snapshot.get("health_status") == "degraded", f"{case_id}: dashboard view with missing evidence must be degraded")
        require(has_missing_or_unavailable_component(snapshot), f"{case_id}: dashboard degraded fixture must expose missing or unavailable evidence")
        require(payload.get("dashboard_status") == "degraded", f"{case_id}: dashboard expected status mismatch")
        return

    if case_id.endswith("fail_closed_missing_evidence.001"):
        require(kind == "unified_snapshot", f"{case_id}: kind mismatch")
        require(snapshot.get("health_status") == "fail_closed", f"{case_id}: missing unified evidence must fail closed")
        require(snapshot.get("blocking_reasons"), f"{case_id}: fail-closed fixture requires blocking reasons")
        require(not unified_healthy_ready(snapshot), f"{case_id}: fail-closed fixture must not satisfy unified healthy readiness")
        require(payload.get("fail_closed") is True, f"{case_id}: expected fail_closed marker mismatch")
        return

    fail(f"unhandled semantics case {case_id}")


contract = Path(os.environ["CONTRACT_PATH"]).read_text(encoding="utf-8")
schema = load_json(Path(os.environ["SCHEMA_PATH"]))
semantics_rows = load_jsonl(Path(os.environ["SEMANTICS_TRACE_PATH"]))

for marker in (
    "component_snapshot",
    "unified_snapshot",
    "dashboard_view",
    "top-level `health_status=healthy`",
    "scripts/ai/verify_release.sh v21.1-health-status-semantics",
):
    require(marker in contract, f"contract missing V211 semantics marker {marker}")

snapshot_kind = schema.get("properties", {}).get("snapshot_kind", {})
require(set(snapshot_kind.get("enum", [])) == {"component_snapshot", "unified_snapshot", "dashboard_view"}, "schema snapshot_kind enum mismatch")

require({row.get("case_id") for row in semantics_rows} == EXPECTED_SEMANTICS_CASES, "semantics fixture case set mismatch")
for row in semantics_rows:
    validate_semantics_fixture(row)

for row in load_jsonl(UNIFIED_TRACE_PATH):
    snapshot = snapshot_from_row(row)
    payload = expected_payload(row)
    case_id = row.get("case_id")
    require(snapshot.get("snapshot_kind") == "unified_snapshot", f"{case_id}: contract trace must be unified_snapshot")
    require(payload.get("health_status") == snapshot.get("health_status"), f"{case_id}: expected payload health mismatch")
    if snapshot.get("health_status") == "healthy":
        require(unified_healthy_ready(snapshot), f"{case_id}: unified healthy requires complete healthy components")

for path in COMPONENT_TRACE_PATHS:
    for row in load_jsonl(path):
        snapshot = snapshot_from_row(row)
        payload = expected_payload(row)
        case_id = row.get("case_id")
        require(snapshot.get("snapshot_kind") == "component_snapshot", f"{case_id}: component trace must be component_snapshot")
        require(payload.get("health_status") == snapshot.get("health_status"), f"{case_id}: expected payload health mismatch")
        require(snapshot.get("health_status") != "healthy", f"{case_id}: component_snapshot must not advertise unified top-level healthy")

for row in load_jsonl(DASHBOARD_TRACE_PATH):
    snapshot = snapshot_from_row(row)
    payload = expected_payload(row)
    case_id = row.get("case_id")
    require(snapshot.get("snapshot_kind") == "dashboard_view", f"{case_id}: dashboard trace must be dashboard_view")
    require(payload.get("health_status") == snapshot.get("health_status"), f"{case_id}: expected payload health mismatch")
    if has_missing_or_unavailable_component(snapshot):
        require(snapshot.get("health_status") != "healthy", f"{case_id}: dashboard missing/unavailable evidence must not be healthy")

print("v211_health_status_semantics status=ok component_snapshot_top_healthy=false unified_snapshot_healthy_requires_all_components=true dashboard_missing_evidence_degraded=true fail_closed_missing_evidence=true")
PY
