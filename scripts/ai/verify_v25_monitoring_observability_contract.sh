#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

TRACE_PATH="${NTPRO_V250_MONITORING_TRACE:-tests/golden/v250_monitoring_observability_contract.jsonl}"
SELFTEST="${NTPRO_V250_MONITORING_SELFTEST:-1}"

if [[ ! -f "$TRACE_PATH" ]]; then
  echo "missing V250 monitoring observability trace: $TRACE_PATH" >&2
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

EXPECTED_CASES = [
    "read_model.monitoring_observability.healthy_runtime_truth.001",
    "read_model.monitoring_observability.missing_source_provenance_degraded.001",
    "read_model.monitoring_observability.stale_partial_degraded.001",
    "read_model.monitoring_observability.redaction_breach_fail_closed.001",
    "read_model.monitoring_observability.side_effect_boundary_fail_closed.001",
]
COMPONENTS = ["account", "orders", "fills", "risk", "order_control_preview"]
SIDE_EFFECT_FIELDS = [
    "submit_order",
    "cancel_order",
    "replace_order",
    "amend_order",
    "flatten_position",
    "retry_scheduler",
    "automatic_remediation",
    "live_exchange_request",
    "adapter_send",
]
REQUIRED_SOURCE_FIELDS = ["source_type", "source_ref", "producer", "collected_at"]
CONTRACT_VERSION = "ntpro.v250.monitoring_observability.v1"
SCHEMA_VERSION = "ntpro.v250.monitoring_observability.schema.v1"
MONITORING_TRUTH_SCOPE = "runtime_monitoring_evidence_only"


def fail(message: str) -> None:
    raise SystemExit(f"v25 monitoring observability contract failed: {message}")


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
    try:
        events = row[section]["events"]
    except KeyError as exc:
        fail(f"{case_id}: missing {section}.events: {exc}")
    if not isinstance(events, list) or len(events) != 1 or not isinstance(events[0], dict):
        fail(f"{case_id}: {section}.events must contain exactly one object")
    return events[0]


def non_empty(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


def source_provenance_complete(value: Any) -> bool:
    if not isinstance(value, dict):
        return False
    return all(non_empty(value.get(field)) for field in REQUIRED_SOURCE_FIELDS)


def push_reason(reasons: list[str], reason: str) -> None:
    if reason not in reasons:
        reasons.append(reason)


def classify(snapshot: dict[str, Any], case_id: str) -> dict[str, Any]:
    if snapshot.get("contract_version") != CONTRACT_VERSION:
        fail(f"{case_id}: contract_version must be {CONTRACT_VERSION}")
    if snapshot.get("schema_version") != SCHEMA_VERSION:
        fail(f"{case_id}: schema_version must be {SCHEMA_VERSION}")

    reasons: list[str] = list(snapshot.get("blocking_reasons") or [])
    if any(not non_empty(reason) for reason in reasons):
        fail(f"{case_id}: blocking_reasons must be non-empty strings")

    missing_provenance = False
    freshness_bad = False
    redaction_problem = False
    redaction_breach = False
    component_nonhealthy = False
    boundary_violation = False

    if not source_provenance_complete(snapshot.get("source_provenance")):
        missing_provenance = True
        push_reason(reasons, "missing_source_provenance:runtime")

    freshness = snapshot.get("freshness")
    if not isinstance(freshness, dict) or freshness.get("status") != "fresh":
        freshness_bad = True
        push_reason(reasons, "stale_freshness:runtime")
    elif not isinstance(freshness.get("max_age_ms"), int) or not isinstance(
        freshness.get("observed_lag_ms"), int
    ):
        freshness_bad = True
        push_reason(reasons, "missing_freshness_window:runtime")
    elif freshness["observed_lag_ms"] > freshness["max_age_ms"]:
        freshness_bad = True
        push_reason(reasons, "freshness_lag_exceeds_max_age:runtime")

    redaction_state = snapshot.get("redaction_state")
    if redaction_state != "redacted":
        redaction_problem = True
        if redaction_state in {"breach", "raw", "unredacted"}:
            redaction_breach = True
            push_reason(reasons, "redaction_breach:runtime")
        else:
            push_reason(reasons, "missing_redaction:runtime")

    if snapshot.get("monitoring_truth_scope") != MONITORING_TRUTH_SCOPE:
        boundary_violation = True
        push_reason(reasons, "monitoring_truth_scope_mismatch:runtime")
    if snapshot.get("exchange_truth_claim") is not False:
        boundary_violation = True
        push_reason(reasons, "exchange_truth_claim:runtime")
    if snapshot.get("adapter_runtime_truth_claim") is not False:
        boundary_violation = True
        push_reason(reasons, "adapter_runtime_truth_claim:runtime")

    side_effects = snapshot.get("operation_side_effects")
    if not isinstance(side_effects, dict):
        boundary_violation = True
        push_reason(reasons, "missing_operation_side_effects:runtime")
        side_effects = {}
    for field in SIDE_EFFECT_FIELDS:
        if side_effects.get(field) is not False:
            boundary_violation = True
            push_reason(reasons, f"operation_side_effect:{field}")

    components = snapshot.get("components")
    if not isinstance(components, dict):
        fail(f"{case_id}: components must be an object")

    for component_name in COMPONENTS:
        component = components.get(component_name)
        if not isinstance(component, dict):
            component_nonhealthy = True
            missing_provenance = True
            push_reason(reasons, f"missing_component:{component_name}")
            continue

        component_health = component.get("health")
        if component_health != "healthy":
            component_nonhealthy = True
            push_reason(reasons, f"component_partial:{component_name}")
            if component_health == "fail_closed":
                boundary_violation = True

        if not non_empty(component.get("source_provenance")):
            missing_provenance = True
            push_reason(reasons, f"missing_component_source_provenance:{component_name}")

        if component.get("freshness_status") != "fresh":
            freshness_bad = True
            push_reason(reasons, f"stale_component_freshness:{component_name}")

        component_redaction = component.get("redaction_state")
        if component_redaction != "redacted":
            redaction_problem = True
            if component_redaction in {"breach", "raw", "unredacted"}:
                redaction_breach = True
                push_reason(reasons, f"component_redaction_breach:{component_name}")
            else:
                push_reason(reasons, f"missing_component_redaction:{component_name}")

        if component.get("operation_side_effect_allowed") is not False:
            boundary_violation = True
            push_reason(reasons, f"component_operation_side_effect_allowed:{component_name}")

    if redaction_breach:
        status = "fail_closed_redaction_breach"
        fail_closed = True
    elif boundary_violation:
        status = "fail_closed_boundary_violation"
        fail_closed = True
    elif missing_provenance:
        status = "degraded_missing_provenance"
        fail_closed = False
    elif redaction_problem:
        status = "degraded_missing_redaction"
        fail_closed = False
    elif freshness_bad or component_nonhealthy:
        status = "degraded_stale_or_partial"
        fail_closed = False
    else:
        status = "healthy"
        fail_closed = False

    exchange_truth_boundary_preserved = snapshot.get("exchange_truth_claim") is False
    adapter_runtime_truth_boundary_preserved = snapshot.get("adapter_runtime_truth_claim") is False
    operation_side_effects_absent = not any(
        side_effects.get(field) is not False for field in SIDE_EFFECT_FIELDS
    ) and all(
        isinstance(components.get(component_name), dict)
        and components[component_name].get("operation_side_effect_allowed") is False
        for component_name in COMPONENTS
    )
    monitoring_truth_only = (
        snapshot.get("monitoring_truth_scope") == MONITORING_TRUTH_SCOPE
        and exchange_truth_boundary_preserved
        and adapter_runtime_truth_boundary_preserved
        and operation_side_effects_absent
    )

    return {
        "case_id": case_id,
        "runtime_health_status": snapshot.get("runtime_health_status"),
        "effective_monitoring_status": status,
        "display_healthy_allowed": status == "healthy",
        "component_count": len(COMPONENTS),
        "components_checked": COMPONENTS,
        "source_provenance_complete": not missing_provenance,
        "freshness_complete": not freshness_bad,
        "redaction_complete": not redaction_problem,
        "stale_or_partial_degraded": freshness_bad or component_nonhealthy,
        "fail_closed": fail_closed,
        "operation_side_effects_absent": operation_side_effects_absent,
        "monitoring_truth_only": monitoring_truth_only,
        "exchange_truth_boundary_preserved": exchange_truth_boundary_preserved,
        "adapter_runtime_truth_boundary_preserved": adapter_runtime_truth_boundary_preserved,
        "blocking_reasons": reasons,
    }


rows = load_rows(trace_path)
if [row.get("case_id") for row in rows] != EXPECTED_CASES:
    fail(
        "case order mismatch: expected "
        + ", ".join(EXPECTED_CASES)
        + " got "
        + ", ".join(str(row.get("case_id")) for row in rows)
    )

healthy_snapshot: dict[str, Any] | None = None
for row in rows:
    case_id = row.get("case_id")
    if row.get("schema_version") != "golden-trace-v1":
        fail(f"{case_id}: schema_version must be golden-trace-v1")
    if row.get("category") != "read_model":
        fail(f"{case_id}: category must be read_model")
    if not non_empty(row.get("description")):
        fail(f"{case_id}: description is required")

    input_event = single_event(row, "input", str(case_id))
    expected_event = single_event(row, "expected", str(case_id))
    expected_event_type = input_event.get("event_type", "").replace(".input", ".validated")
    if expected_event.get("event_type") != expected_event_type:
        fail(f"{case_id}: expected event_type must be {expected_event_type}")
    for key in ("ts_event", "ts_init", "instrument_id", "venue", "correlation_id"):
        if expected_event.get(key) != input_event.get(key):
            fail(f"{case_id}: expected.{key} must match input.{key}")

    payload = input_event.get("payload")
    if not isinstance(payload, dict) or not isinstance(payload.get("snapshot"), dict):
        fail(f"{case_id}: input payload snapshot is required")
    snapshot = payload["snapshot"]
    computed = classify(snapshot, str(case_id))
    expected_payload = expected_event.get("payload")
    if computed != expected_payload:
        fail(
            f"{case_id}: computed payload mismatch\n"
            f"expected={json.dumps(expected_payload, sort_keys=True)}\n"
            f"actual={json.dumps(computed, sort_keys=True)}"
        )
    if computed["display_healthy_allowed"] and computed["effective_monitoring_status"] != "healthy":
        fail(f"{case_id}: non-healthy status cannot allow healthy display")
    if str(case_id).endswith("healthy_runtime_truth.001"):
        healthy_snapshot = copy.deepcopy(snapshot)

if selftest:
    if healthy_snapshot is None:
        fail("negative selftest requires healthy fixture")
    healthy_snapshot.pop("source_provenance", None)
    degraded = classify(healthy_snapshot, "negative.selftest.missing_runtime_provenance")
    if degraded["display_healthy_allowed"] or degraded["effective_monitoring_status"] == "healthy":
        fail("negative selftest removed provenance but still allowed healthy display")
    if "missing_source_provenance:runtime" not in degraded["blocking_reasons"]:
        fail("negative selftest did not surface missing_source_provenance:runtime")

print(
    "v25_monitoring_observability_contract "
    f"status=ok trace={trace_path.as_posix()} "
    f"cases={len(rows)} components_checked={len(rows) * len(COMPONENTS)} "
    f"negative_selftest={1 if selftest else 0}"
)
PY
