#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

TRACE_PATH="${NTPRO_V270_ADMIN_BRIDGE_TRACE:-tests/golden/v270_admin_workbench_runtime_state_bridge.jsonl}"
TASK_PATH="${NTPRO_V270_ADMIN_BRIDGE_TASK:-docs/rust-cutover/tasks/V270-006.md}"
EVIDENCE_PATH="${NTPRO_V270_ADMIN_BRIDGE_EVIDENCE:-docs/rust-cutover/evidence/V270-006.md}"
CONTRACT_PATH="${NTPRO_V270_ADMIN_BRIDGE_CONTRACT:-docs/rust-cutover/release/v0_27_0_admin_workbench_runtime_state_bridge.md}"
REPLAY_SCOPE_PATH="${NTPRO_V270_ADMIN_BRIDGE_REPLAY_SCOPE:-docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"
SELFTEST="${NTPRO_V270_ADMIN_BRIDGE_SELFTEST:-1}"

fail() {
  echo "v27 Admin Workbench runtime state bridge failed: $*" >&2
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

for dependency in \
  docs/rust-cutover/release/v0_27_0_product_operations_runtime_integration_boundary_contract.md \
  docs/rust-cutover/release/v0_27_0_external_identity_permission_foundation.md \
  docs/rust-cutover/release/v0_27_0_persistent_operation_audit_storage_foundation.md \
  docs/rust-cutover/release/v0_27_0_deployment_orchestration_foundation.md \
  docs/rust-cutover/release/v0_27_0_long_run_telemetry_slo_runtime_evidence.md; do
  require_file "$dependency"
done

require_contains "$TASK_PATH" "GitHub issue: \`#859\`"
require_contains "$EVIDENCE_PATH" "Task: \`V270-006\` / GitHub issue \`#859\`"
require_contains "$CONTRACT_PATH" "admin_workbench_surface_scope = runtime_state_bridge_read_only_admin_surface"
require_contains "$CONTRACT_PATH" "dashboard_surface_scope = runtime_state_bridge_read_only_dashboard_surface"
require_contains "$CONTRACT_PATH" "required_components = identity_permission,audit_storage,deployment_orchestration,telemetry_slo,runtime_integration_boundary"
require_contains "$CONTRACT_PATH" "operation_controls_enabled = false"
require_contains "$CONTRACT_PATH" "trading_controls_enabled = false"
require_contains "$CONTRACT_PATH" "automatic_remediation_allowed = false"
require_contains "$CONTRACT_PATH" "retry_scheduler_enabled = false"
require_contains "$CONTRACT_PATH" "adapter_send_allowed = false"
require_contains "$CONTRACT_PATH" "live_exchange_request_allowed = false"

python3 - "$TRACE_PATH" "$REPLAY_SCOPE_PATH" "$SELFTEST" <<'PY'
from __future__ import annotations

import copy
import json
import sys
from pathlib import Path
from typing import Any

trace_path = Path(sys.argv[1])
replay_scope_path = Path(sys.argv[2])
selftest = sys.argv[3] != "0"

EXPECTED_CASES = [
    "read_model.admin_workbench_runtime_state_bridge.ready_readonly.001",
    "read_model.admin_workbench_runtime_state_bridge.stale_artifact_degraded.001",
    "read_model.admin_workbench_runtime_state_bridge.missing_component_fail_closed.001",
    "read_model.admin_workbench_runtime_state_bridge.malformed_provenance_fail_closed.001",
    "read_model.admin_workbench_runtime_state_bridge.redaction_breach_fail_closed.001",
    "read_model.admin_workbench_runtime_state_bridge.runtime_state_drift_fail_closed.001",
    "read_model.admin_workbench_runtime_state_bridge.forbidden_controls_fail_closed.001",
]
CONTRACT_VERSION = "ntpro.v270.admin_workbench_runtime_state_bridge.v1"
SCHEMA_VERSION = "ntpro.v270.admin_workbench_runtime_state_bridge.schema.v1"
ADMIN_SCOPE = "runtime_state_bridge_read_only_admin_surface"
DASHBOARD_SCOPE = "runtime_state_bridge_read_only_dashboard_surface"
DEPENDENCIES = ["V270-001", "V270-002", "V270-003", "V270-004", "V270-005"]
REQUIRED_COMPONENTS = [
    "identity_permission",
    "audit_storage",
    "deployment_orchestration",
    "telemetry_slo",
    "runtime_integration_boundary",
]
ALLOWED_SOURCE_PREFIX = {
    "identity_permission": "docs/rust-cutover/release/v0_27_0_external_identity_permission_foundation.md",
    "audit_storage": "docs/rust-cutover/release/v0_27_0_persistent_operation_audit_storage_foundation.md",
    "deployment_orchestration": "docs/rust-cutover/release/v0_27_0_deployment_orchestration_foundation.md",
    "telemetry_slo": "docs/rust-cutover/release/v0_27_0_long_run_telemetry_slo_runtime_evidence.md",
    "runtime_integration_boundary": "docs/rust-cutover/release/v0_27_0_product_operations_runtime_integration_boundary_contract.md",
}
BOUNDARY_FALSE_FLAGS = [
    "submit_order_allowed",
    "cancel_order_allowed",
    "retry_order_allowed",
    "replace_order_allowed",
    "amend_order_allowed",
    "flatten_position_allowed",
    "order_ticket_enabled",
    "dashboard_operation_controls_enabled",
    "dashboard_trading_controls_enabled",
    "admin_workbench_operation_controls_enabled",
    "admin_workbench_trading_controls_enabled",
    "automatic_remediation_allowed",
    "retry_scheduler_enabled",
    "adapter_send_allowed",
    "live_exchange_request_allowed",
    "manual_operation_submit_allowed",
    "product_grade_trading_terminal_claim",
]
HARNESS = "scripts/ai/verify_release.sh v27-admin-workbench-runtime-state-bridge"
TRACE_REF = "tests/golden/v270_admin_workbench_runtime_state_bridge.jsonl"
VALIDATOR_ENTRYPOINT = "scripts/ai/verify_v27_admin_workbench_runtime_state_bridge.sh::classify"


def fail(message: str) -> None:
    raise SystemExit(f"v27 Admin Workbench runtime state bridge failed: {message}")


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
    try:
        events = row[section]["events"]
    except KeyError as exc:
        fail(f"{case_id}: missing {section}.events: {exc}")
    if not isinstance(events, list) or len(events) != 1 or not isinstance(events[0], dict):
        fail(f"{case_id}: {section}.events must contain exactly one object")
    return events[0]


def validate_source_ref(component_id: str, source_ref: Any) -> bool:
    if not non_empty(source_ref):
        return False
    value = source_ref.strip()
    if value.startswith(("http://", "https://", "/")):
        return False
    expected = ALLOWED_SOURCE_PREFIX.get(component_id)
    if expected is None or not value.startswith(expected):
        return False
    path = value.split("#", 1)[0]
    return Path(path).is_file()


def classify(snapshot: dict[str, Any], case_id: str) -> dict[str, Any]:
    if snapshot.get("contract_version") != CONTRACT_VERSION:
        fail(f"{case_id}: contract_version must be {CONTRACT_VERSION}")
    if snapshot.get("schema_version") != SCHEMA_VERSION:
        fail(f"{case_id}: schema_version must be {SCHEMA_VERSION}")
    if snapshot.get("admin_workbench_surface_scope") != ADMIN_SCOPE:
        fail(f"{case_id}: admin_workbench_surface_scope must be {ADMIN_SCOPE}")
    if snapshot.get("dashboard_surface_scope") != DASHBOARD_SCOPE:
        fail(f"{case_id}: dashboard_surface_scope must be {DASHBOARD_SCOPE}")
    if snapshot.get("dependency_contracts") != DEPENDENCIES:
        fail(f"{case_id}: dependency_contracts must be {DEPENDENCIES}")

    reasons: list[str] = list(snapshot.get("blocking_reasons") or [])
    missing_component = False
    stale_artifact = False
    malformed_provenance = False
    redaction_breach = False
    runtime_drift = False
    forbidden_controls = False

    boundary_flags = snapshot.get("boundary_flags")
    if not isinstance(boundary_flags, dict):
        forbidden_controls = True
        push_reason(reasons, "missing_boundary_flags")
        boundary_flags = {}
    for key in BOUNDARY_FALSE_FLAGS:
        if key not in boundary_flags:
            forbidden_controls = True
            push_reason(reasons, f"missing_required_false_boundary:{key}")
        elif boundary_flags.get(key) is not False:
            forbidden_controls = True
            push_reason(reasons, f"forbidden_control:{key}")

    components = snapshot.get("components")
    if not isinstance(components, dict):
        fail(f"{case_id}: components must be an object")
    ready_components: list[str] = []
    missing_components: list[str] = []
    for component_id in REQUIRED_COMPONENTS:
        component = components.get(component_id)
        if not isinstance(component, dict):
            missing_component = True
            missing_components.append(component_id)
            push_reason(reasons, f"missing_component:{component_id}")
            continue
        if component.get("component_status") != "present":
            missing_component = True
            missing_components.append(component_id)
            push_reason(reasons, f"component_not_present:{component_id}:{component.get('component_status')}")
        else:
            ready_components.append(component_id)
        if component.get("provenance_status") != "verified" or not validate_source_ref(component_id, component.get("source_ref")):
            malformed_provenance = True
            push_reason(reasons, f"malformed_provenance:{component_id}")
        if not non_empty(component.get("source_digest")):
            malformed_provenance = True
            push_reason(reasons, f"missing_source_digest:{component_id}")
        if component.get("freshness_status") != "fresh":
            stale_artifact = True
            push_reason(reasons, f"stale_artifact:{component_id}:{component.get('freshness_status')}")
        if component.get("redaction_status") != "redacted":
            redaction_breach = True
            push_reason(reasons, f"redaction_breach:{component_id}:{component.get('redaction_status')}")
        if component.get("raw_secret_persisted") is not False or component.get("raw_exchange_response_persisted") is not False:
            redaction_breach = True
            push_reason(reasons, f"raw_sensitive_payload_persisted:{component_id}")
        if component.get("runtime_state_aligned") is not True:
            runtime_drift = True
            push_reason(reasons, f"runtime_state_drift:{component_id}")
        if component.get("read_only") is not True:
            forbidden_controls = True
            push_reason(reasons, f"component_not_read_only:{component_id}")
        if component.get("operation_controls_enabled") is not False or component.get("trading_controls_enabled") is not False:
            forbidden_controls = True
            push_reason(reasons, f"component_controls_enabled:{component_id}")

    surfaces = snapshot.get("surfaces")
    if not isinstance(surfaces, dict):
        fail(f"{case_id}: surfaces must be an object")
    for surface_name in ("admin_workbench", "dashboard"):
        surface = surfaces.get(surface_name)
        if not isinstance(surface, dict):
            fail(f"{case_id}: missing surface {surface_name}")
        if surface.get("read_only") is not True or surface.get("display_only") is not True:
            forbidden_controls = True
            push_reason(reasons, f"{surface_name}_not_read_only")
        if surface.get("operation_controls_enabled") is not False or surface.get("trading_controls_enabled") is not False:
            forbidden_controls = True
            push_reason(reasons, f"{surface_name}_controls_enabled")

    fail_closed = bool(missing_component or malformed_provenance or redaction_breach or runtime_drift or forbidden_controls)
    degraded = bool(stale_artifact)
    if forbidden_controls:
        bridge_status = "fail_closed_forbidden_controls"
    elif redaction_breach:
        bridge_status = "fail_closed_redaction_breach"
    elif runtime_drift:
        bridge_status = "fail_closed_runtime_state_drift"
    elif malformed_provenance:
        bridge_status = "fail_closed_malformed_provenance"
    elif missing_component:
        bridge_status = "fail_closed_missing_component"
    elif stale_artifact:
        bridge_status = "degraded_read_only_surface"
    else:
        bridge_status = "healthy"
    surface_status = "fail_closed" if fail_closed else ("degraded" if degraded else "healthy")
    if surface_status != "healthy" and not reasons:
        fail(f"{case_id}: degraded/fail_closed cases must expose reasons")

    return {
        "bridge_status": bridge_status,
        "admin_workbench_status": surface_status,
        "dashboard_status": surface_status,
        "ready_component_count": len(ready_components),
        "missing_components": sorted(missing_components),
        "fail_closed": fail_closed,
        "degraded": degraded and not fail_closed,
        "read_only": True,
        "operation_controls_absent": not forbidden_controls,
        "trading_controls_absent": not forbidden_controls,
        "automatic_remediation_absent": not forbidden_controls,
        "adapter_send_absent": not forbidden_controls,
        "live_exchange_request_absent": not forbidden_controls,
        "degradation_reasons": sorted(reasons),
    }


def validate_replay_scope(case_ids: list[str]) -> None:
    try:
        replay_scope = json.loads(replay_scope_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        fail(f"{replay_scope_path}: invalid JSON: {exc}")
    cases = replay_scope.get("cases")
    if not isinstance(cases, list):
        fail(f"{replay_scope_path}: cases must be a list")
    by_id = {case.get("case_id"): case for case in cases if isinstance(case, dict)}
    for case_id in case_ids:
        item = by_id.get(case_id)
        if not isinstance(item, dict):
            fail(f"{case_id}: missing release replay scope entry")
        expected = {
            "status": "validator_executable_replay",
            "trace": TRACE_REF,
            "harness": HARNESS,
            "validator_entrypoint": VALIDATOR_ENTRYPOINT,
            "classification_owner": "V270-006",
            "source_scope_owner": "V270-006",
            "evidence_id": "V270-006",
            "replay_type": "validator_executable_admin_workbench_runtime_state_bridge",
            "release_decision": "validator_executable_scope_recorded",
            "admin_workbench_surface_scope": ADMIN_SCOPE,
            "runtime_adapter_integration": False,
        }
        for key, value in expected.items():
            if item.get(key) != value:
                fail(f"{case_id}: replay scope {key} must be {value!r}")
        for key in (
            "automatic_remediation_allowed",
            "retry_scheduler_enabled",
            "adapter_send_allowed",
            "live_exchange_request_allowed",
            "dashboard_trading_controls_enabled",
            "admin_workbench_trading_controls_enabled",
            "product_grade_live_trading_terminal",
        ):
            if item.get(key) is not False:
                fail(f"{case_id}: replay scope {key} must be false")


def mark_surface_degraded(snapshot: dict[str, Any]) -> dict[str, Any]:
    for surface in snapshot.get("surfaces", {}).values():
        if isinstance(surface, dict):
            surface["degradation_reasons"] = ["selftest_degradation"]
    return snapshot


rows = load_rows(trace_path)
case_ids = [str(row.get("case_id")) for row in rows]
if case_ids != EXPECTED_CASES:
    fail(f"unexpected case order: {case_ids}")

status_counts: dict[str, int] = {}
for row in rows:
    case_id = str(row["case_id"])
    input_event = single_event(row, "input", case_id)
    expected_event = single_event(row, "expected", case_id)
    snapshot = input_event.get("payload", {}).get("snapshot")
    if not isinstance(snapshot, dict):
        fail(f"{case_id}: input payload snapshot must be an object")
    expected_payload = expected_event.get("payload")
    if not isinstance(expected_payload, dict):
        fail(f"{case_id}: expected payload must be an object")
    actual = classify(snapshot, case_id)
    if actual != expected_payload:
        fail(f"{case_id}: expected payload mismatch\nactual={json.dumps(actual, sort_keys=True)}\nexpected={json.dumps(expected_payload, sort_keys=True)}")
    status_counts[actual["bridge_status"]] = status_counts.get(actual["bridge_status"], 0) + 1

validate_replay_scope(case_ids)

if selftest:
    ready = single_event(rows[0], "input", EXPECTED_CASES[0])["payload"]["snapshot"]
    mutated = mark_surface_degraded(copy.deepcopy(ready))
    mutated["boundary_flags"]["adapter_send_allowed"] = True
    if classify(mutated, "selftest.forbidden")["bridge_status"] != "fail_closed_forbidden_controls":
        fail("selftest forbidden controls did not fail closed")
    mutated = mark_surface_degraded(copy.deepcopy(ready))
    del mutated["components"]["telemetry_slo"]
    if classify(mutated, "selftest.missing")["bridge_status"] != "fail_closed_missing_component":
        fail("selftest missing component did not fail closed")
    mutated = mark_surface_degraded(copy.deepcopy(ready))
    mutated["components"]["identity_permission"]["source_ref"] = "https://example.invalid/identity.json"
    if classify(mutated, "selftest.provenance")["bridge_status"] != "fail_closed_malformed_provenance":
        fail("selftest malformed provenance did not fail closed")
    mutated = mark_surface_degraded(copy.deepcopy(ready))
    mutated["components"]["audit_storage"]["redaction_status"] = "unredacted"
    if classify(mutated, "selftest.redaction")["bridge_status"] != "fail_closed_redaction_breach":
        fail("selftest redaction did not fail closed")
    mutated = mark_surface_degraded(copy.deepcopy(ready))
    mutated["components"]["deployment_orchestration"]["freshness_status"] = "stale"
    if classify(mutated, "selftest.stale")["bridge_status"] != "degraded_read_only_surface":
        fail("selftest stale artifact did not degrade")

print(
    "v27_admin_workbench_runtime_state_bridge=pass "
    f"cases={len(rows)} statuses={len(status_counts)} "
    f"components={len(REQUIRED_COMPONENTS)} boundary_flags={len(BOUNDARY_FALSE_FLAGS)} "
    f"negative_selftest={int(selftest)}"
)
PY
