#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

TRACE_PATH="${NTPRO_V260_DASHBOARD_ADMIN_TRACE:-tests/golden/v260_dashboard_admin_boundary_surface.jsonl}"
TASK_PATH="${NTPRO_V260_DASHBOARD_ADMIN_TASK:-docs/rust-cutover/tasks/V260-007.md}"
EVIDENCE_PATH="${NTPRO_V260_DASHBOARD_ADMIN_EVIDENCE:-docs/rust-cutover/evidence/V260-007.md}"
CONTRACT_PATH="${NTPRO_V260_DASHBOARD_ADMIN_CONTRACT:-docs/rust-cutover/release/v0_26_0_dashboard_admin_boundary_surface.md}"
REPLAY_SCOPE_PATH="${NTPRO_V260_DASHBOARD_ADMIN_REPLAY_SCOPE:-docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"
SELFTEST="${NTPRO_V260_DASHBOARD_ADMIN_SELFTEST:-1}"

fail() {
  echo "v26 Dashboard admin boundary surface failed: $*" >&2
  exit 1
}

require_file() {
  local path="$1"
  [[ -f "$path" ]] || fail "missing required file: $path"
}

require_contains() {
  local path="$1"
  local marker="$2"
  if ! grep -F -- "$marker" "$path" >/dev/null; then
    fail "missing marker in $path: $marker"
  fi
}

for path in "$TRACE_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$CONTRACT_PATH" "$REPLAY_SCOPE_PATH"; do
  require_file "$path"
done

require_contains "$TASK_PATH" "GitHub issue: \`#819\`"
require_contains "$EVIDENCE_PATH" "Task: \`V260-007\` / GitHub issue \`#819\`"
require_contains "$CONTRACT_PATH" "dashboard_admin_surface_scope = product_hardening_read_only_admin_evidence"
require_contains "$CONTRACT_PATH" "dashboard_operation_controls_enabled = false"
require_contains "$CONTRACT_PATH" "dashboard_trading_controls_enabled = false"
require_contains "$CONTRACT_PATH" "trader_terminal_order_ticket_enabled = false"

python3 - "$TRACE_PATH" "$REPLAY_SCOPE_PATH" "$SELFTEST" <<'PY'
from __future__ import annotations

import copy
import json
import re
import sys
from pathlib import Path
from typing import Any

trace_path = Path(sys.argv[1])
replay_scope_path = Path(sys.argv[2])
selftest = sys.argv[3] != "0"
dashboard_rs = Path("crates/cli/src/dashboard.rs")

EXPECTED_CASES = [
    "read_model.dashboard_admin_boundary_surface.ready_readonly.001",
    "read_model.dashboard_admin_boundary_surface.malformed_source_ref_fail_closed.001",
    "read_model.dashboard_admin_boundary_surface.forbidden_control_fail_closed.001",
    "read_model.dashboard_admin_boundary_surface.unredacted_secret_fail_closed.001",
    "read_model.dashboard_admin_boundary_surface.stale_artifact_degraded.001",
    "read_model.dashboard_admin_boundary_surface.missing_component_degraded.001",
]
REQUIRED_COMPONENTS = [
    "permission_boundary",
    "operation_audit",
    "deployment_provenance",
    "upgrade_rollback",
    "stability_slo",
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
    "dashboard_operation_control_allowed",
    "live_exchange_request_allowed",
    "adapter_send_allowed",
    "automatic_remediation_allowed",
    "automatic_actions_allowed",
    "manual_operation_submit_allowed",
]
REQUIRED_SOURCE_FIELDS = ["source_type", "source_ref", "producer", "collected_at"]
COMPONENT_ALLOWED_PATHS = {
    "permission_boundary": {
        "docs/rust-cutover/release/v0_26_0_operator_permission_model.md",
        "tests/golden/v260_operator_permission_model.jsonl",
    },
    "operation_audit": {
        "docs/rust-cutover/release/v0_26_0_operation_audit_trail.md",
        "tests/golden/v260_operation_audit_trail.jsonl",
    },
    "deployment_provenance": {
        "docs/rust-cutover/release/v0_26_0_deployment_provenance_model.md",
        "tests/golden/v260_deployment_provenance_model.jsonl",
    },
    "upgrade_rollback": {
        "docs/rust-cutover/release/v0_26_0_upgrade_rollback_runbook_evidence.md",
        "tests/golden/v260_upgrade_rollback_runbook_evidence.jsonl",
    },
    "stability_slo": {
        "docs/rust-cutover/release/v0_26_0_slo_runbook_stability_evidence.md",
        "tests/golden/v260_slo_runbook_stability_evidence.jsonl",
    },
}
JSONL_ANCHOR_ALIASES = {
    "tests/golden/v260_operator_permission_model.jsonl": {
        "valid": "read_model.operator_permission_model.valid_role_matrix.001",
    },
    "tests/golden/v260_operation_audit_trail.jsonl": {
        "valid": "read_model.operation_audit_trail.valid_immutable_chain.001",
    },
    "tests/golden/v260_deployment_provenance_model.jsonl": {
        "valid": "read_model.deployment_provenance_model.valid_topology_matrix.001",
    },
    "tests/golden/v260_upgrade_rollback_runbook_evidence.jsonl": {
        "valid": "read_model.upgrade_rollback_runbook.valid_preview.001",
    },
    "tests/golden/v260_slo_runbook_stability_evidence.jsonl": {
        "valid": "read_model.slo_runbook_stability.valid_long_run_window.001",
    },
}
HARNESS = "scripts/ai/verify_release.sh v26-dashboard-admin-boundary-surface"


class SourceRefError(Exception):
    def __init__(self, reason: str):
        super().__init__(reason)
        self.reason = reason


def fail(message: str) -> None:
    raise SystemExit(f"v26 Dashboard admin boundary surface failed: {message}")


def non_empty(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


def push_reason(reasons: list[str], reason: str) -> None:
    if reason not in reasons:
        reasons.append(reason)


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


def complete_source(value: Any) -> bool:
    return isinstance(value, dict) and all(non_empty(value.get(field)) for field in REQUIRED_SOURCE_FIELDS)


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


def resolve_source_ref(source_ref: str, component_name: str) -> None:
    if not non_empty(source_ref):
        raise SourceRefError("empty_source_ref")
    if source_ref.startswith(("http://", "https://")):
        raise SourceRefError("remote_source_ref_not_allowed")

    file_path, anchor = split_ref(source_ref.strip())
    if Path(file_path).is_absolute():
        raise SourceRefError(f"absolute_path_not_allowed:{file_path}")
    if not Path(file_path).is_file():
        raise SourceRefError(f"missing_path:{file_path}")
    if file_path not in COMPONENT_ALLOWED_PATHS.get(component_name, set()):
        raise SourceRefError(f"unexpected_path:{file_path}")
    if not anchor:
        raise SourceRefError(f"missing_anchor:{file_path}")

    if file_path.endswith(".jsonl"):
        target = JSONL_ANCHOR_ALIASES.get(file_path, {}).get(anchor, anchor)
        if target not in jsonl_case_ids(file_path):
            raise SourceRefError(f"unresolved_jsonl_anchor:{file_path}#{anchor}")
        return

    if file_path.endswith(".md"):
        if anchor not in markdown_anchors(file_path):
            raise SourceRefError(f"unresolved_markdown_anchor:{file_path}#{anchor}")
        return

    raise SourceRefError(f"unsupported_ref_type:{file_path}")


def classify(snapshot: dict[str, Any], case_id: str) -> dict[str, Any]:
    components = snapshot.get("components")
    if not isinstance(components, dict):
        fail(f"{case_id}: components must be an object")

    reasons: list[str] = list(snapshot.get("blocking_reasons") or [])
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
            push_reason(reasons, f"missing_component:{component_name}")
            continue

        status = component.get("component_status")
        if status in {"degraded", "partial", "stale"}:
            freshness_complete = False
            push_reason(reasons, f"component_not_healthy:{component_name}:{status}")
        elif status in {"fail_closed", "error"}:
            forbidden_control = True
            operation_controls_absent = False
            push_reason(reasons, f"component_fail_closed:{component_name}:{status}")
        elif status != "healthy":
            source_complete = False
            push_reason(reasons, f"component_status_unexpected:{component_name}:{status}")

        source = component.get("source_provenance")
        if not complete_source(source):
            source_complete = False
            push_reason(reasons, f"missing_source_provenance:{component_name}")
        else:
            try:
                resolve_source_ref(str(source["source_ref"]), component_name)
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
        "render_smoke_covered": True,
        "artifact_ingestion_covered": True,
        "forbidden_control_fail_closed": forbidden_control,
        "fail_closed": fail_closed,
        "blocking_reasons": reasons,
    }


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


def assert_dashboard_source_markers() -> None:
    text = dashboard_rs.read_text(encoding="utf-8")
    required = [
        "V26_PERMISSION_BOUNDARY_COMPONENT",
        "V26_OPERATION_AUDIT_COMPONENT",
        "V26_DEPLOYMENT_PROVENANCE_COMPONENT",
        "V26_UPGRADE_ROLLBACK_COMPONENT",
        "V26_STABILITY_SLO_COMPONENT",
        "workbench-panel-v26-admin-surface",
        "v26_dashboard_admin_surface_status",
        "v26_admin_surface_blocking_reasons",
        "dashboard_v26_admin_surface_renderer_stays_readonly",
    ]
    for marker in required:
        if marker not in text:
            fail(f"dashboard source missing marker: {marker}")
    renderer = dashboard_js_function_body(text, "renderTraderTerminalWorkbench")
    renderer += "\n" + dashboard_js_function_body(text, "renderReadModelRuntime")
    forbidden = [
        "<button",
        "<form",
        "<input",
        "fetch(",
        "data-workbench-action",
        "/api/order",
        "/api/orders",
        "/actions/submit",
        "/actions/cancel",
        "/actions/retry",
        "/actions/replace",
        "/actions/amend",
        "/actions/flatten",
    ]
    for marker in forbidden:
        if marker in renderer:
            fail(f"dashboard source exposes forbidden control marker: {marker}")


def ready_snapshot(rows: list[dict[str, Any]]) -> dict[str, Any]:
    snapshot = single_event(rows[0], "input", str(rows[0]["case_id"]))["payload"]["snapshot"]
    if not isinstance(snapshot, dict):
        fail("ready fixture snapshot missing")
    return copy.deepcopy(snapshot)


def expect_fail_closed_selftest(rows: list[dict[str, Any]], name: str, mutate, expected_prefix: str) -> None:
    snapshot = ready_snapshot(rows)
    mutate(snapshot)
    actual = classify(snapshot, f"selftest.{name}")
    if actual["surface_status"] != "fail_closed_surface_artifact":
        fail(f"{name}: did not fail closed: {actual['surface_status']}")
    if not any(reason.startswith(expected_prefix) for reason in actual["blocking_reasons"]):
        fail(f"{name}: missing expected reason {expected_prefix}: {actual['blocking_reasons']}")


rows = load_rows(trace_path)
case_ids = [str(row.get("case_id")) for row in rows]
if case_ids != EXPECTED_CASES:
    fail(f"case ordering mismatch: {case_ids}")

for row in rows:
    case_id = str(row["case_id"])
    if row.get("schema_version") != "golden-trace-v1":
        fail(f"{case_id}: schema_version must be golden-trace-v1")
    if row.get("category") != "read_model":
        fail(f"{case_id}: category must be read_model")
    input_event = single_event(row, "input", case_id)
    expected_event = single_event(row, "expected", case_id)
    expected_event_type = input_event.get("event_type", "").replace(".input", ".validated")
    if expected_event.get("event_type") != expected_event_type:
        fail(f"{case_id}: expected event_type must be {expected_event_type}")
    for key in ("ts_event", "ts_init", "instrument_id", "venue", "correlation_id"):
        if expected_event.get(key) != input_event.get(key):
            fail(f"{case_id}: expected.{key} must match input.{key}")
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

if selftest:
    expect_fail_closed_selftest(
        rows,
        "remote_source_ref",
        lambda snapshot: snapshot["components"]["permission_boundary"]["source_provenance"].__setitem__(
            "source_ref",
            "https://example.invalid/source.json",
        ),
        "unresolved_source_ref:permission_boundary:remote_source_ref_not_allowed",
    )
    expect_fail_closed_selftest(
        rows,
        "forbidden_control",
        lambda snapshot: snapshot["components"]["stability_slo"].__setitem__(
            "automatic_remediation_allowed",
            True,
        ),
        "forbidden_control:stability_slo:automatic_remediation_allowed",
    )
    expect_fail_closed_selftest(
        rows,
        "unredacted",
        lambda snapshot: snapshot["components"]["operation_audit"].__setitem__(
            "redaction_state",
            "raw_secret",
        ),
        "redaction_not_ready:operation_audit",
    )

assert_dashboard_source_markers()

scope = json.loads(replay_scope_path.read_text(encoding="utf-8"))
cases = {case.get("case_id"): case for case in scope.get("cases", [])}
for case_id in EXPECTED_CASES:
    entry = cases.get(case_id)
    if not isinstance(entry, dict):
        fail(f"missing release replay scope entry: {case_id}")
    expected_pairs = {
        "trace": trace_path.as_posix(),
        "category": "read_model",
        "status": "validator_executable_replay",
        "evidence_id": "V260-007",
        "harness": HARNESS,
        "validator_entrypoint": "scripts/ai/verify_v26_dashboard_admin_boundary_surface.sh::classify",
        "replay_type": "validator_executable_dashboard_admin_boundary_surface",
        "classification_owner": "V260-007",
        "source_scope_owner": "V260-007",
        "dashboard_admin_surface_scope": "product_hardening_read_only_admin_evidence",
    }
    for key, expected in expected_pairs.items():
        if entry.get(key) != expected:
            fail(f"{case_id}: release scope {key} mismatch: {entry.get(key)!r}")
    for key in (
        "runtime_adapter_integration",
        "dashboard_operation_controls_enabled",
        "dashboard_trading_controls_enabled",
        "trader_terminal_order_ticket_enabled",
        "manual_operation_submit_allowed",
        "automatic_remediation_allowed",
        "new_submit_capability",
        "production_order_mutation_allowed",
        "adapter_send_allowed",
        "live_exchange_request_allowed",
        "product_grade_live_trading_terminal",
    ):
        if entry.get(key) is not False:
            fail(f"{case_id}: release scope {key} must be false")

print(
    "v26_dashboard_admin_boundary_surface "
    f"status=ok trace={trace_path.as_posix()} "
    f"cases={len(rows)} components={len(REQUIRED_COMPONENTS)} "
    f"negative_selftest={3 if selftest else 0}"
)
PY
