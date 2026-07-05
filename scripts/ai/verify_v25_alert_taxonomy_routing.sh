#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

TRACE_PATH="${NTPRO_V250_ALERT_TRACE:-tests/golden/v250_alert_taxonomy_routing.jsonl}"
SELFTEST="${NTPRO_V250_ALERT_SELFTEST:-1}"

if [[ ! -f "$TRACE_PATH" ]]; then
  echo "missing V250 alert taxonomy trace: $TRACE_PATH" >&2
  exit 1
fi

python3 - "$TRACE_PATH" "$SELFTEST" <<'PY'
from __future__ import annotations

import copy
import json
import re
import sys
from pathlib import Path
from typing import Any

trace_path = Path(sys.argv[1])
selftest = sys.argv[2] != "0"

EXPECTED_CASES = [
    "read_model.alert_taxonomy_routing.valid_matrix.001",
    "read_model.alert_taxonomy_routing.missing_required_fail_closed.001",
    "read_model.alert_taxonomy_routing.redaction_secret_fail_closed.001",
    "read_model.alert_taxonomy_routing.automatic_action_fail_closed.001",
]
EXPECTED_CATEGORIES = [
    "missing_provenance",
    "order_control_preview_blocked",
    "release_gate_drift",
    "risk_fail_closed",
    "stale_data",
]
ALLOWED_SEVERITIES = {"info", "warning", "critical", "halt"}
ALLOWED_CATEGORIES = set(EXPECTED_CATEGORIES)
ALLOWED_SOURCES = {
    "monitoring_observability",
    "read_model_runtime",
    "risk_projection",
    "order_control_preview",
    "release_gate",
}
ALLOWED_ROUTING_KINDS = {
    "manual_review_queue",
    "runbook_review_queue",
    "release_gatekeeper_review",
}
BOUNDARY_FALSE_FIELDS = [
    "external_paging_service_connected",
    "automatic_remediation_allowed",
    "automatic_submit_allowed",
    "automatic_cancel_allowed",
    "automatic_retry_allowed",
    "automatic_replace_allowed",
    "automatic_amend_allowed",
    "automatic_flatten_allowed",
    "live_exchange_request_allowed",
    "adapter_send_allowed",
]
REQUIRED_SOURCE_FIELDS = ["source_type", "source_ref", "producer", "collected_at"]
FORBIDDEN_KEY_PATTERNS = [
    re.compile(pattern, re.IGNORECASE)
    for pattern in (
        r"(^|_)(secret|credential|signature|signed_payload|raw_order_id|raw_credential)(_|$)",
        r"api[_-]?key",
        r"client[_-]?order[_-]?id",
    )
]
FORBIDDEN_VALUE_PATTERNS = [
    re.compile(pattern, re.IGNORECASE)
    for pattern in (
        r"api[_-]?secret\s*=",
        r"credential\s*=",
        r"signed[_-]?payload\s*=",
        r"signature\s*=",
        r"raw[_-]?order[_-]?id\s*=",
        r"client[_-]?order[_-]?id\s*=",
    )
]
CONTRACT_VERSION = "ntpro.v250.alert_taxonomy_routing.v1"
SCHEMA_VERSION = "ntpro.v250.alert_taxonomy_routing.schema.v1"


def fail(message: str) -> None:
    raise SystemExit(f"v25 alert taxonomy routing failed: {message}")


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


def non_empty(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


def source_provenance_complete(value: Any) -> bool:
    return isinstance(value, dict) and all(non_empty(value.get(field)) for field in REQUIRED_SOURCE_FIELDS)


def push_reason(reasons: list[str], reason: str) -> None:
    if reason not in reasons:
        reasons.append(reason)


def scan_redaction(value: Any, reasons: list[str], path: str = "alert") -> bool:
    breached = False
    if isinstance(value, dict):
        for key, nested in value.items():
            if any(pattern.search(str(key)) for pattern in FORBIDDEN_KEY_PATTERNS):
                push_reason(reasons, f"forbidden_sensitive_key:{path}.{key}")
                breached = True
            if scan_redaction(nested, reasons, f"{path}.{key}"):
                breached = True
    elif isinstance(value, list):
        for index, nested in enumerate(value):
            if scan_redaction(nested, reasons, f"{path}[{index}]"):
                breached = True
    elif isinstance(value, str):
        if any(pattern.search(value) for pattern in FORBIDDEN_VALUE_PATTERNS):
            push_reason(reasons, f"forbidden_sensitive_value:{path}")
            breached = True
    return breached


def classify(snapshot: dict[str, Any], case_id: str) -> dict[str, Any]:
    if snapshot.get("contract_version") != CONTRACT_VERSION:
        fail(f"{case_id}: contract_version must be {CONTRACT_VERSION}")
    if snapshot.get("schema_version") != SCHEMA_VERSION:
        fail(f"{case_id}: schema_version must be {SCHEMA_VERSION}")

    reasons: list[str] = list(snapshot.get("blocking_reasons") or [])
    if any(not non_empty(reason) for reason in reasons):
        fail(f"{case_id}: blocking_reasons must contain non-empty strings")

    alerts = snapshot.get("alerts")
    if not isinstance(alerts, list) or not alerts:
        fail(f"{case_id}: alerts must be a non-empty array")

    boundary = snapshot.get("routing_boundary")
    if not isinstance(boundary, dict):
        fail(f"{case_id}: routing_boundary must be an object")

    missing_required = False
    redaction_breach = False
    action_boundary = False
    categories: set[str] = set()
    severities: set[str] = set()
    routing_targets: set[str] = set()
    ack_required_count = 0

    if boundary.get("routing_mode") != "evidence_only":
        action_boundary = True
        push_reason(reasons, "routing_mode_not_evidence_only")
    for field in BOUNDARY_FALSE_FIELDS:
        if boundary.get(field) is not False:
            action_boundary = True
            push_reason(reasons, f"boundary_true:{field}")

    for index, alert in enumerate(alerts):
        if not isinstance(alert, dict):
            missing_required = True
            push_reason(reasons, f"alert_not_object:{index}")
            continue

        alert_id = alert.get("alert_id")
        severity = alert.get("severity")
        category = alert.get("category")
        source = alert.get("source")
        dedupe_key = alert.get("dedupe_key")
        ack_required = alert.get("ack_required")
        scope = alert.get("scope")
        freshness = alert.get("freshness")
        routing_target = alert.get("routing_target")

        for field_name, value in (
            ("alert_id", alert_id),
            ("severity", severity),
            ("category", category),
            ("source", source),
            ("dedupe_key", dedupe_key),
        ):
            if not non_empty(value):
                missing_required = True
                push_reason(reasons, f"missing_{field_name}:alert:{index}")

        if non_empty(severity):
            severities.add(str(severity))
            if severity not in ALLOWED_SEVERITIES:
                missing_required = True
                push_reason(reasons, f"invalid_severity:{severity}:alert:{index}")
        if non_empty(category):
            categories.add(str(category))
            if category not in ALLOWED_CATEGORIES:
                missing_required = True
                push_reason(reasons, f"invalid_category:{category}:alert:{index}")
        if non_empty(source) and source not in ALLOWED_SOURCES:
            missing_required = True
            push_reason(reasons, f"invalid_source:{source}:alert:{index}")
        if ack_required is True:
            ack_required_count += 1
        elif ack_required is not False:
            missing_required = True
            push_reason(reasons, f"missing_ack_required:alert:{index}")

        if not isinstance(scope, dict):
            missing_required = True
            push_reason(reasons, f"missing_scope:alert:{index}")
        else:
            for field in ("account_key", "strategy_key", "venue_node_key", "isolation_scope_key"):
                if not non_empty(scope.get(field)):
                    missing_required = True
                    push_reason(reasons, f"missing_scope_{field}:alert:{index}")

        if not isinstance(freshness, dict) or freshness.get("status") != "fresh":
            missing_required = True
            push_reason(reasons, f"missing_or_stale_alert_freshness:alert:{index}")

        if not source_provenance_complete(alert.get("source_provenance")):
            missing_required = True
            push_reason(reasons, f"missing_source_provenance:alert:{index}")

        if alert.get("redaction_state") != "redacted":
            redaction_breach = True
            push_reason(reasons, f"redaction_state_not_redacted:alert:{index}")
        if scan_redaction(alert, reasons, f"alert:{index}"):
            redaction_breach = True

        if not isinstance(routing_target, dict):
            missing_required = True
            push_reason(reasons, f"missing_routing_target:alert:{index}")
        else:
            kind = routing_target.get("kind")
            target_ref = routing_target.get("target_ref")
            side_effect = routing_target.get("side_effect")
            if kind not in ALLOWED_ROUTING_KINDS:
                action_boundary = True
                push_reason(reasons, f"invalid_routing_kind:{kind}:alert:{index}")
            else:
                routing_targets.add(str(kind))
            if not non_empty(target_ref):
                missing_required = True
                push_reason(reasons, f"missing_routing_target_ref:alert:{index}")
            if side_effect != "none":
                action_boundary = True
                push_reason(reasons, f"routing_side_effect:{side_effect}:alert:{index}")

    if redaction_breach:
        status = "fail_closed_redaction_breach"
    elif action_boundary:
        status = "fail_closed_action_boundary"
    elif missing_required:
        status = "fail_closed_missing_required"
    else:
        status = "routable_readonly"

    return {
        "case_id": case_id,
        "alert_taxonomy_status": status,
        "fail_closed": status.startswith("fail_closed"),
        "alert_count": len(alerts),
        "categories": sorted(categories),
        "severities": sorted(severities),
        "routing_targets": sorted(routing_targets),
        "ack_required_count": ack_required_count,
        "required_fields_complete": not missing_required,
        "source_provenance_complete": not any(reason.startswith("missing_source_provenance") for reason in reasons),
        "redaction_clean": not redaction_breach,
        "routing_readonly": not action_boundary,
        "automatic_actions_allowed": False if not action_boundary else any(boundary.get(field) is True for field in BOUNDARY_FALSE_FIELDS),
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

valid_matrix_snapshot: dict[str, Any] | None = None
for row in rows:
    case_id = str(row.get("case_id"))
    if row.get("schema_version") != "golden-trace-v1":
        fail(f"{case_id}: schema_version must be golden-trace-v1")
    if row.get("category") != "read_model":
        fail(f"{case_id}: category must be read_model")
    if not non_empty(row.get("description")):
        fail(f"{case_id}: description is required")

    input_event = single_event(row, "input", case_id)
    expected_event = single_event(row, "expected", case_id)
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
    computed = classify(snapshot, case_id)
    if computed != expected_event.get("payload"):
        fail(
            f"{case_id}: computed payload mismatch\n"
            f"expected={json.dumps(expected_event.get('payload'), sort_keys=True)}\n"
            f"actual={json.dumps(computed, sort_keys=True)}"
        )
    if case_id.endswith("valid_matrix.001"):
        if computed["categories"] != EXPECTED_CATEGORIES:
            fail(f"{case_id}: valid matrix must cover all expected categories")
        valid_matrix_snapshot = copy.deepcopy(snapshot)

if selftest:
    if valid_matrix_snapshot is None:
        fail("negative selftest requires valid matrix fixture")
    valid_matrix_snapshot["alerts"][0].pop("severity", None)
    failed = classify(valid_matrix_snapshot, "negative.selftest.missing_severity")
    if failed["alert_taxonomy_status"] != "fail_closed_missing_required":
        fail("negative selftest removed severity but did not fail closed")
    if "missing_severity:alert:0" not in failed["blocking_reasons"]:
        fail("negative selftest did not report missing_severity:alert:0")

print(
    "v25_alert_taxonomy_routing "
    f"status=ok trace={trace_path.as_posix()} "
    f"cases={len(rows)} valid_categories={len(EXPECTED_CATEGORIES)} "
    f"negative_selftest={1 if selftest else 0}"
)
PY
