#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

TRACE_PATH="${NTPRO_V250_DASHBOARD_SURFACE_TRACE:-tests/golden/v250_dashboard_monitoring_surface.jsonl}"
SELFTEST="${NTPRO_V250_DASHBOARD_SURFACE_SELFTEST:-1}"

if [[ ! -f "$TRACE_PATH" ]]; then
  echo "missing V250 Dashboard monitoring surface trace: $TRACE_PATH" >&2
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
dashboard_rs = Path("crates/cli/src/dashboard.rs")

EXPECTED_CASES = [
    "read_model.dashboard_monitoring_surface.ready_readonly.001",
    "read_model.dashboard_monitoring_surface.missing_provenance_fail_closed.001",
    "read_model.dashboard_monitoring_surface.stale_artifact_degraded.001",
    "read_model.dashboard_monitoring_surface.missing_redaction_fail_closed.001",
    "read_model.dashboard_monitoring_surface.forbidden_control_fail_closed.001",
]
REQUIRED_COMPONENTS = [
    "monitoring",
    "alert",
    "incident",
    "runbook",
    "dr_preview",
]
FORBIDDEN_TRUE_FIELDS = [
    "submit_order_allowed",
    "cancel_order_allowed",
    "retry_order_allowed",
    "replace_order_allowed",
    "amend_order_allowed",
    "flatten_position_allowed",
    "order_ticket_enabled",
    "dashboard_trading_control_allowed",
    "live_exchange_request_allowed",
    "adapter_send_allowed",
    "automatic_remediation_allowed",
]
REQUIRED_SOURCE_FIELDS = ["source_type", "source_ref", "producer", "collected_at"]
COMPONENT_JSONL_SOURCE_PATHS = {
    "monitoring": "tests/golden/v250_monitoring_observability_contract.jsonl",
    "alert": "tests/golden/v250_alert_taxonomy_routing.jsonl",
    "incident": "tests/golden/v250_incident_lifecycle_acknowledgement.jsonl",
    "runbook": "tests/golden/v250_runbook_audit_evidence.jsonl",
    "dr_preview": "tests/golden/v250_dr_preview_drill_evidence.jsonl",
}
COMPONENT_RELEASE_CONTRACT_REFS = {
    "monitoring": "docs/rust-cutover/release/v0_25_0_monitoring_observability_contract.md#status-rules",
    "alert": "docs/rust-cutover/release/v0_25_0_alert_taxonomy_routing.md#routing-boundary",
    "incident": "docs/rust-cutover/release/v0_25_0_incident_lifecycle_acknowledgement.md#incident-boundary",
    "runbook": "docs/rust-cutover/release/v0_25_0_runbook_audit_evidence.md#execution-boundary",
    "dr_preview": "docs/rust-cutover/release/v0_25_0_dr_preview_drill_evidence.md#preview-boundary",
}
COMPONENT_ALLOWED_REF_PATHS = {
    component: {jsonl_path, COMPONENT_RELEASE_CONTRACT_REFS[component].split("#", 1)[0]}
    for component, jsonl_path in COMPONENT_JSONL_SOURCE_PATHS.items()
}
JSONL_ANCHOR_ALIASES = {
    "tests/golden/v250_monitoring_observability_contract.jsonl": {
        "healthy": "read_model.monitoring_observability.healthy_runtime_truth.001",
        "missing-provenance": "read_model.monitoring_observability.missing_source_provenance_degraded.001",
        "stale-partial": "read_model.monitoring_observability.stale_partial_degraded.001",
        "redaction-breach": "read_model.monitoring_observability.redaction_breach_fail_closed.001",
        "boundary-violation": "read_model.monitoring_observability.side_effect_boundary_fail_closed.001",
    },
    "tests/golden/v250_alert_taxonomy_routing.jsonl": {
        "valid": "read_model.alert_taxonomy_routing.valid_matrix.001",
        "stale": "read_model.alert_taxonomy_routing.valid_matrix.001",
        "redaction-secret": "read_model.alert_taxonomy_routing.redaction_secret_fail_closed.001",
        "action-boundary": "read_model.alert_taxonomy_routing.automatic_action_fail_closed.001",
    },
    "tests/golden/v250_incident_lifecycle_acknowledgement.jsonl": {
        "acknowledged": "read_model.incident_lifecycle_acknowledgement.valid_lifecycle.001",
        "valid-lifecycle": "read_model.incident_lifecycle_acknowledgement.valid_lifecycle.001",
    },
    "tests/golden/v250_runbook_audit_evidence.jsonl": {
        "manual_ack": "read_model.runbook_audit_evidence.valid_manual_matrix.001",
        "valid-manual-matrix": "read_model.runbook_audit_evidence.valid_manual_matrix.001",
        "snapshot-1": "read_model.runbook_audit_evidence.valid_manual_matrix.001",
        "snapshot-2": "read_model.runbook_audit_evidence.valid_manual_matrix.001",
        "snapshot-3": "read_model.runbook_audit_evidence.valid_manual_matrix.001",
        "snapshot-4": "read_model.runbook_audit_evidence.valid_manual_matrix.001",
        "readback-1": "read_model.runbook_audit_evidence.valid_manual_matrix.001",
        "readback-2": "read_model.runbook_audit_evidence.valid_manual_matrix.001",
        "readback-3": "read_model.runbook_audit_evidence.valid_manual_matrix.001",
        "readback-4": "read_model.runbook_audit_evidence.valid_manual_matrix.001",
    },
    "tests/golden/v250_dr_preview_drill_evidence.jsonl": {
        "preview": "read_model.dr_preview_drill_evidence.valid_preview_matrix.001",
        "valid-preview-matrix": "read_model.dr_preview_drill_evidence.valid_preview_matrix.001",
    },
}


class SourceRefError(Exception):
    def __init__(self, reason: str):
        super().__init__(reason)
        self.reason = reason


def fail(message: str) -> None:
    raise SystemExit(f"v25 Dashboard monitoring surface failed: {message}")


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


def complete_source(value: Any) -> bool:
    return isinstance(value, dict) and all(non_empty(value.get(field)) for field in REQUIRED_SOURCE_FIELDS)


def push_reason(reasons: list[str], reason: str) -> None:
    if reason not in reasons:
        reasons.append(reason)


def split_ref(source_ref: str) -> tuple[str, str]:
    index = source_ref.find("#")
    if index < 0:
        return source_ref, ""
    return source_ref[:index], source_ref[index + 1 :]


def jsonl_case_ids(path: str) -> set[str]:
    case_ids: set[str] = set()
    with Path(path).open(encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, 1):
            stripped = line.strip()
            if not stripped:
                continue
            try:
                row = json.loads(stripped)
            except json.JSONDecodeError as exc:
                raise SourceRefError(f"invalid_jsonl:{path}:{line_number}:{exc.msg}") from exc
            case_id = row.get("case_id")
            if not non_empty(case_id):
                raise SourceRefError(f"missing_case_id:{path}:{line_number}")
            case_ids.add(case_id)
    return case_ids


def markdown_slug(heading: str) -> str:
    slug = heading.strip().lower().replace("`", "")
    slug = re.sub(r"[^a-z0-9]+", "-", slug)
    return slug.strip("-")


def markdown_anchors(path: str) -> set[str]:
    text = Path(path).read_text(encoding="utf-8")
    anchors = set(re.findall(r"<a\s+id=[\"']([^\"']+)[\"']\s*></a>", text))
    for line in text.splitlines():
        heading = re.match(r"^#{1,6}\s+(.+)$", line)
        if heading:
            anchors.add(markdown_slug(heading.group(1)))
    return anchors


def resolve_source_ref(source_ref: str, component_name: str | None) -> None:
    if not non_empty(source_ref):
        raise SourceRefError("empty_source_ref")
    if source_ref.startswith(("http://", "https://")):
        raise SourceRefError("remote_source_ref_not_allowed")

    file_path, anchor = split_ref(source_ref.strip())
    if Path(file_path).is_absolute():
        raise SourceRefError(f"absolute_path_not_allowed:{file_path}")
    if not Path(file_path).is_file():
        raise SourceRefError(f"missing_path:{file_path}")

    if component_name is not None:
        allowed_paths = COMPONENT_ALLOWED_REF_PATHS.get(component_name)
        if allowed_paths is None or file_path not in allowed_paths:
            raise SourceRefError(f"unexpected_path:{file_path}")
        if not anchor:
            raise SourceRefError(f"missing_anchor:{file_path}")

    if file_path.endswith(".jsonl"):
        if not anchor:
            raise SourceRefError(f"missing_jsonl_anchor:{file_path}")
        target = JSONL_ANCHOR_ALIASES.get(file_path, {}).get(anchor, anchor)
        if target not in jsonl_case_ids(file_path):
            raise SourceRefError(f"unresolved_jsonl_anchor:{file_path}#{anchor}")
        return

    if file_path.endswith(".md"):
        if anchor and anchor not in markdown_anchors(file_path):
            raise SourceRefError(f"unresolved_markdown_anchor:{file_path}#{anchor}")
        if component_name is not None and not anchor:
            raise SourceRefError(f"missing_markdown_anchor:{file_path}")
        return

    raise SourceRefError(f"unsupported_ref_type:{file_path}")


def validate_component_source_ref(component_name: str, source: dict[str, Any]) -> None:
    source_ref = source.get("source_ref")
    if not isinstance(source_ref, str):
        raise SourceRefError("source_ref_not_string")
    resolve_source_ref(source_ref, component_name)


def classify(snapshot: dict[str, Any], case_id: str) -> dict[str, Any]:
    components = snapshot.get("components")
    if not isinstance(components, dict):
        fail(f"{case_id}: components must be an object")

    reasons: list[str] = list(snapshot.get("blocking_reasons") or [])
    if any(not non_empty(reason) for reason in reasons):
        fail(f"{case_id}: blocking_reasons must contain non-empty strings")

    missing_component = False
    source_complete = True
    freshness_complete = True
    redaction_complete = True
    forbidden_control = False
    operation_controls_absent = True

    for component_name in REQUIRED_COMPONENTS:
        component = components.get(component_name)
        if not isinstance(component, dict):
            missing_component = True
            source_complete = False
            push_reason(reasons, f"missing_component:{component_name}")
            continue

        source_provenance = component.get("source_provenance")
        if not complete_source(source_provenance):
            source_complete = False
            push_reason(reasons, f"missing_source_provenance:{component_name}")
        else:
            try:
                validate_component_source_ref(component_name, source_provenance)
            except SourceRefError as exc:
                source_complete = False
                push_reason(reasons, f"unresolved_source_ref:{component_name}:{exc.reason}")

        freshness = component.get("freshness")
        freshness_status = freshness.get("status") if isinstance(freshness, dict) else None
        if freshness_status != "fresh":
            freshness_complete = False
            push_reason(reasons, f"freshness_not_fresh:{component_name}:{freshness_status or 'missing'}")

        if component.get("redaction_state") != "redacted":
            redaction_complete = False
            push_reason(reasons, f"redaction_not_ready:{component_name}")

        if component.get("operation_boundary_readonly") is not True:
            forbidden_control = True
            operation_controls_absent = False
            push_reason(reasons, f"operation_boundary_not_readonly:{component_name}")

        for field in FORBIDDEN_TRUE_FIELDS:
            if component.get(field) is True:
                forbidden_control = True
                operation_controls_absent = False
                push_reason(reasons, f"forbidden_control:{component_name}:{field}")

    fail_closed = forbidden_control or not source_complete or not redaction_complete
    degraded = missing_component or not freshness_complete
    if fail_closed:
        surface_status = "fail_closed_surface_artifact"
    elif degraded:
        surface_status = "degraded_surface_artifact"
    else:
        surface_status = "ready_readonly_surface"

    return {
        "case_id": case_id,
        "surface_status": surface_status,
        "display_healthy_allowed": surface_status == "ready_readonly_surface",
        "component_count": len([component for component in components.values() if isinstance(component, dict)]),
        "source_provenance_complete": source_complete,
        "freshness_complete": freshness_complete,
        "redaction_complete": redaction_complete,
        "operation_controls_absent": operation_controls_absent,
        "forbidden_control_fail_closed": forbidden_control,
        "fail_closed": fail_closed,
        "blocking_reasons": reasons,
    }


def assert_dashboard_source_markers() -> None:
    text = dashboard_rs.read_text(encoding="utf-8")
    required = [
        "V25_MONITORING_OBSERVABILITY_COMPONENT",
        "V25_INCIDENT_LIFECYCLE_COMPONENT",
        "V25_RUNBOOK_AUDIT_COMPONENT",
        "V25_DR_PREVIEW_COMPONENT",
        "workbench-panel-v25-monitoring-surface",
        "v25_dashboard_surface_status",
        "v25_surface_blocking_reasons",
        "validate_v25_dashboard_surface_component",
    ]
    for marker in required:
        if marker not in text:
            fail(f"dashboard source missing marker: {marker}")
    renderer = dashboard_js_function_body(text, "renderTraderTerminalWorkbench")
    renderer += "\n" + dashboard_js_function_body(text, "renderReadModelRuntime")
    forbidden = [
        "data-workbench-action=\"submit",
        "data-workbench-action=\"cancel",
        "data-workbench-action=\"replace",
        "data-workbench-action=\"amend",
        "data-workbench-action=\"flatten",
        "/actions/submit",
        "/actions/cancel",
        "/actions/replace",
        "/actions/amend",
        "/actions/flatten",
    ]
    for marker in forbidden:
        if marker in renderer:
            fail(f"dashboard source exposes forbidden control marker: {marker}")


def count_resolved_dashboard_source_refs(snapshot: dict[str, Any]) -> int:
    components = snapshot.get("components")
    if not isinstance(components, dict):
        return 0
    count = 0
    for component_name in REQUIRED_COMPONENTS:
        component = components.get(component_name)
        if not isinstance(component, dict):
            continue
        source_provenance = component.get("source_provenance")
        if not complete_source(source_provenance):
            continue
        validate_component_source_ref(component_name, source_provenance)
        count += 1
    return count


def assert_release_contract_ref_resolution() -> int:
    for component_name, source_ref in COMPONENT_RELEASE_CONTRACT_REFS.items():
        resolve_source_ref(source_ref, component_name)
    return len(COMPONENT_RELEASE_CONTRACT_REFS)


def ready_snapshot(rows: list[dict[str, Any]]) -> dict[str, Any]:
    snapshot = single_event(rows[0], "input", str(rows[0]["case_id"]))["payload"]["snapshot"]
    if not isinstance(snapshot, dict):
        fail("ready fixture snapshot missing")
    return copy.deepcopy(snapshot)


def set_component_source_ref(snapshot: dict[str, Any], component_name: str, source_ref: str) -> None:
    snapshot["components"][component_name]["source_provenance"]["source_ref"] = source_ref


def expect_fail_closed_selftest(
    rows: list[dict[str, Any]],
    name: str,
    mutate,
    expected_reason_prefix: str,
) -> None:
    snapshot = ready_snapshot(rows)
    mutate(snapshot)
    actual = classify(snapshot, f"selftest.{name}")
    if actual["surface_status"] != "fail_closed_surface_artifact":
        fail(f"{name}: malformed source_ref did not fail closed: {actual['surface_status']}")
    if not any(reason.startswith(expected_reason_prefix) for reason in actual["blocking_reasons"]):
        fail(f"{name}: missing expected blocking reason prefix {expected_reason_prefix}: {actual['blocking_reasons']}")


def dashboard_js_function_body(text: str, function_name: str) -> str:
    needle = f"function {function_name}"
    start = text.find(needle)
    if start < 0:
        fail(f"dashboard source missing JS function: {function_name}")
    after_start = start + len(needle)
    end = text.find("\nfunction ", after_start)
    if end < 0:
        end = len(text)
    return text[start:end]


rows = load_rows(trace_path)
case_ids = [str(row.get("case_id")) for row in rows]
if case_ids != EXPECTED_CASES:
    fail(f"case ordering mismatch: {case_ids}")

resolved_source_refs = 0
for row in rows:
    case_id = str(row["case_id"])
    input_event = single_event(row, "input", case_id)
    expected_event = single_event(row, "expected", case_id)
    snapshot = input_event.get("payload", {}).get("snapshot")
    if not isinstance(snapshot, dict):
        fail(f"{case_id}: input payload.snapshot must be an object")
    actual = classify(snapshot, case_id)
    expected = expected_event.get("payload")
    if actual != expected:
        fail(
            f"{case_id}: classification mismatch\n"
            f"expected={json.dumps(expected, sort_keys=True)}\n"
            f"actual={json.dumps(actual, sort_keys=True)}"
        )
    resolved_source_refs += count_resolved_dashboard_source_refs(snapshot)

if selftest:
    expect_fail_closed_selftest(
        rows,
        "bad_path_selftest",
        lambda snapshot: set_component_source_ref(
            snapshot,
            "monitoring",
            "tests/golden/v250_missing_monitoring_observability_contract.jsonl#healthy",
        ),
        "unresolved_source_ref:monitoring:missing_path:",
    )
    expect_fail_closed_selftest(
        rows,
        "bad_jsonl_anchor_selftest",
        lambda snapshot: set_component_source_ref(
            snapshot,
            "monitoring",
            "tests/golden/v250_monitoring_observability_contract.jsonl#missing-anchor",
        ),
        "unresolved_source_ref:monitoring:unresolved_jsonl_anchor:",
    )
    expect_fail_closed_selftest(
        rows,
        "bad_markdown_anchor_selftest",
        lambda snapshot: set_component_source_ref(
            snapshot,
            "runbook",
            "docs/rust-cutover/release/v0_25_0_runbook_audit_evidence.md#missing-anchor",
        ),
        "unresolved_source_ref:runbook:unresolved_markdown_anchor:",
    )
    expect_fail_closed_selftest(
        rows,
        "empty_source_ref_selftest",
        lambda snapshot: set_component_source_ref(snapshot, "alert", " "),
        "missing_source_provenance:alert",
    )
    expect_fail_closed_selftest(
        rows,
        "cross_version_ref_selftest",
        lambda snapshot: set_component_source_ref(
            snapshot,
            "alert",
            "docs/rust-cutover/release/v0_24_0_order_control_contract.md#dashboard-and-workbench-boundary",
        ),
        "unresolved_source_ref:alert:unexpected_path:",
    )
    expect_fail_closed_selftest(
        rows,
        "forbidden_control_selftest",
        lambda snapshot: snapshot["components"]["incident"].__setitem__(
            "dashboard_trading_control_allowed",
            True,
        ),
        "forbidden_control:incident:dashboard_trading_control_allowed",
    )

assert_dashboard_source_markers()
release_contract_refs = assert_release_contract_ref_resolution()

print(
    "v25_dashboard_monitoring_surface status=ok "
    f"trace={trace_path} cases={len(rows)} components={len(REQUIRED_COMPONENTS)} "
    f"source_refs_resolved={resolved_source_refs} "
    f"release_contract_refs={release_contract_refs} "
    f"negative_selftest={6 if selftest else 0} "
    f"bad_path_selftest={1 if selftest else 0} "
    f"bad_jsonl_anchor_selftest={1 if selftest else 0} "
    f"bad_markdown_anchor_selftest={1 if selftest else 0} "
    f"empty_source_ref_selftest={1 if selftest else 0} "
    f"cross_version_ref_selftest={1 if selftest else 0} "
    f"forbidden_control_selftest={1 if selftest else 0}"
)
PY
