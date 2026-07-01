#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

CONTRACT_PATH="${NTPRO_V21_RISK_CONTRACT:-docs/rust-cutover/release/v0_21_0_risk_state_projection.md}"
UNIFIED_SCHEMA_PATH="${NTPRO_V21_READ_MODEL_SCHEMA:-docs/rust-cutover/release/v0_21_0_unified_read_model_schema.json}"
TRACE_PATH="${NTPRO_V21_RISK_TRACE:-tests/golden/read_model_risk_state_schema.jsonl}"
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
    echo "missing required v21 risk state projection file: $path" >&2
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
RISK_TRANSFORM = "ntpro.v210.risk_state_projection.v1"
EXPECTED_CASES = {
    "read_model.risk_state.healthy.001": {
        "health_status": "degraded",
        "component_status": "healthy",
        "risk_state": "healthy",
        "freshness": "fresh",
        "priority": 0,
        "audit_closed_allowed": True,
        "critical_evidence_complete": True,
        "manual_review_required": False,
        "halted": False,
        "reasons": [],
    },
    "read_model.risk_state.risk_visible.001": {
        "health_status": "degraded",
        "component_status": "degraded",
        "risk_state": "risk_visible",
        "freshness": "fresh",
        "priority": 1,
        "audit_closed_allowed": False,
        "critical_evidence_complete": True,
        "manual_review_required": False,
        "halted": False,
        "reasons": [],
    },
    "read_model.risk_state.manual_review.001": {
        "health_status": "degraded",
        "component_status": "degraded",
        "risk_state": "manual_review",
        "freshness": "fresh",
        "priority": 2,
        "audit_closed_allowed": False,
        "critical_evidence_complete": True,
        "manual_review_required": True,
        "halted": False,
        "reasons": ["manual_review_required"],
    },
    "read_model.risk_state.halted.001": {
        "health_status": "fail_closed",
        "component_status": "fail_closed",
        "risk_state": "halted",
        "freshness": "fresh",
        "priority": 5,
        "audit_closed_allowed": False,
        "critical_evidence_complete": False,
        "manual_review_required": True,
        "halted": True,
        "reasons": ["halted_by_risk_state"],
    },
    "read_model.risk_state.stale.001": {
        "health_status": "fail_closed",
        "component_status": "fail_closed",
        "risk_state": "stale",
        "freshness": "stale",
        "priority": 3,
        "audit_closed_allowed": False,
        "critical_evidence_complete": False,
        "manual_review_required": True,
        "halted": False,
        "reasons": ["stale_component_freshness"],
    },
    "read_model.risk_state.mismatch.001": {
        "health_status": "fail_closed",
        "component_status": "fail_closed",
        "risk_state": "mismatch",
        "freshness": "fresh",
        "priority": 4,
        "audit_closed_allowed": False,
        "critical_evidence_complete": False,
        "manual_review_required": True,
        "halted": False,
        "reasons": ["component_lineage_mismatch"],
    },
}
REQUIRED_RISK_DATA = {
    "risk_state",
    "risk_state_priority",
    "component_inputs",
    "freshness_rollup",
    "lineage_rollup",
    "provenance_rollup",
    "mismatch_risks",
    "blocking_reasons",
    "manual_review_required",
    "halted",
    "audit_closed_allowed",
    "critical_evidence_complete",
    "dashboard_readonly_visible",
    "automatic_trading_action_allowed",
    "automatic_remediation_allowed",
    "production_mutation_allowed",
}
REQUIRED_COMPONENT_INPUTS = {"account", "positions", "orders", "fills"}
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
    "dashboard_risk_controls_enabled",
    "automatic_risk_action_allowed",
    "automatic_risk_repair_allowed",
    "execution_algorithm_allowed",
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
    require(case_id in EXPECTED_CASES, f"unexpected risk state case {case_id}")
    expected = EXPECTED_CASES[case_id]
    snapshot = snapshot_from_row(row)
    payload = expected_payload(row)
    risk = snapshot.get("components", {}).get("risk")
    lifecycle = snapshot.get("components", {}).get("lifecycle_status", {})
    require(isinstance(risk, dict), f"{case_id}: missing risk component")
    data = risk.get("data")
    require(isinstance(data, dict), f"{case_id}: missing risk data")

    require(snapshot.get("contract_version") == CONTRACT_VERSION, f"{case_id}: contract version mismatch")
    require(snapshot.get("schema_version") == SCHEMA_VERSION, f"{case_id}: schema version mismatch")
    require(snapshot.get("health_status") == expected["health_status"], f"{case_id}: health status mismatch")
    require(payload.get("health_status") == expected["health_status"], f"{case_id}: expected payload health mismatch")
    require(risk.get("component_status") == expected["component_status"], f"{case_id}: risk status mismatch")
    require(risk.get("lineage", {}).get("transform") == RISK_TRANSFORM, f"{case_id}: risk transform mismatch")
    require(risk.get("freshness", {}).get("status") == expected["freshness"], f"{case_id}: risk freshness mismatch")
    require(REQUIRED_RISK_DATA <= set(data), f"{case_id}: missing risk data {sorted(REQUIRED_RISK_DATA - set(data))}")
    require(data.get("risk_state") == expected["risk_state"], f"{case_id}: risk state mismatch")
    require(data.get("risk_state_priority") == expected["priority"], f"{case_id}: risk state priority mismatch")
    require(data.get("audit_closed_allowed") is expected["audit_closed_allowed"], f"{case_id}: audit closed flag mismatch")
    require(data.get("critical_evidence_complete") is expected["critical_evidence_complete"], f"{case_id}: critical evidence mismatch")
    require(data.get("manual_review_required") is expected["manual_review_required"], f"{case_id}: manual review flag mismatch")
    require(data.get("halted") is expected["halted"], f"{case_id}: halted flag mismatch")
    require(data.get("dashboard_readonly_visible") is True, f"{case_id}: dashboard visibility required")
    require(data.get("automatic_trading_action_allowed") is False, f"{case_id}: automatic trading action must be false")
    require(data.get("automatic_remediation_allowed") is False, f"{case_id}: automatic remediation must be false")
    require(data.get("production_mutation_allowed") is False, f"{case_id}: production mutation must be false")
    require_false_boundary(snapshot, case_id)
    walk_forbidden_keys(data, f"{case_id}.risk.data")

    component_inputs = data.get("component_inputs", {})
    require(REQUIRED_COMPONENT_INPUTS <= set(component_inputs), f"{case_id}: missing component input rollups")
    for component in REQUIRED_COMPONENT_INPUTS:
        rollup = component_inputs.get(component)
        require(isinstance(rollup, dict), f"{case_id}: component input {component} must be object")
        for key in ("component_status", "freshness_status", "lineage_status", "provenance_status"):
            require(key in rollup, f"{case_id}: component input {component} missing {key}")

    lifecycle_data = lifecycle.get("data", {}) if isinstance(lifecycle, dict) else {}
    if expected["audit_closed_allowed"]:
        require(lifecycle_data.get("lifecycle_status") == "audit_closed", f"{case_id}: complete risk component evidence must audit close")
        require(data.get("blocking_reasons") == [], f"{case_id}: complete risk component evidence must not block")
        require(snapshot.get("blocking_reasons") == [], f"{case_id}: complete risk component evidence must not block")
    else:
        require(lifecycle_data.get("lifecycle_status") != "audit_closed", f"{case_id}: non-healthy case must not audit close")
        require(data.get("risk_state") != "healthy", f"{case_id}: non-healthy case must not show healthy")

    for reason in expected["reasons"]:
        require(reason in snapshot.get("blocking_reasons", []), f"{case_id}: missing snapshot reason {reason}")
        require(reason in payload.get("blocking_reasons", []), f"{case_id}: expected payload missing reason {reason}")
        require(reason in risk.get("diagnostics", []), f"{case_id}: missing risk diagnostic {reason}")
        require(reason in data.get("blocking_reasons", []), f"{case_id}: missing data reason {reason}")


contract_path = Path(os.environ["CONTRACT_PATH"])
schema_path = Path(os.environ["UNIFIED_SCHEMA_PATH"])
trace_path = Path(os.environ["TRACE_PATH"])

contract = contract_path.read_text(encoding="utf-8")
schema = load_json(schema_path)
rows = load_jsonl(trace_path)

for marker in (
    RISK_TRANSFORM,
    "halted > mismatch > stale > manual_review > risk_visible > healthy",
    "stale_component_freshness",
    "component_lineage_mismatch",
    "critical_evidence_missing",
    "scripts/ai/verify_release.sh v21-risk-state-projection",
):
    require(marker in contract, f"contract missing marker {marker}")

require(schema.get("schema_version") == SCHEMA_VERSION, "unified schema version mismatch")
require(len(rows) == 6, "risk state golden trace must contain exactly six rows")
require({row.get("case_id") for row in rows} == set(EXPECTED_CASES), "unexpected risk state case set")

for row in rows:
    validate_case(row)

print("v21_risk_state_projection status=ok trace_cases=6 healthy=covered risk_visible=covered manual_review=covered halted=covered stale=covered mismatch=covered audit_closed_requires_complete_evidence=true automatic_actions=false")
PY
