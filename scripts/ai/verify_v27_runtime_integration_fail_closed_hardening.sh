#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

TRACE_PATH="${NTPRO_V270_FAIL_CLOSED_TRACE:-tests/golden/v270_runtime_integration_fail_closed_hardening.jsonl}"
TASK_PATH="${NTPRO_V270_FAIL_CLOSED_TASK:-docs/rust-cutover/tasks/V270-007.md}"
EVIDENCE_PATH="${NTPRO_V270_FAIL_CLOSED_EVIDENCE:-docs/rust-cutover/evidence/V270-007.md}"
CONTRACT_PATH="${NTPRO_V270_FAIL_CLOSED_CONTRACT:-docs/rust-cutover/release/v0_27_0_runtime_integration_fail_closed_hardening.md}"
REPLAY_SCOPE_PATH="${NTPRO_V270_FAIL_CLOSED_REPLAY_SCOPE:-docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"
SELFTEST="${NTPRO_V270_FAIL_CLOSED_SELFTEST:-1}"

fail() {
  echo "v27 runtime integration fail-closed hardening failed: $*" >&2
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
  docs/rust-cutover/release/v0_27_0_long_run_telemetry_slo_runtime_evidence.md \
  docs/rust-cutover/release/v0_27_0_admin_workbench_runtime_state_bridge.md; do
  require_file "$dependency"
done

require_contains "$TASK_PATH" "GitHub issue: \`#860\`"
require_contains "$EVIDENCE_PATH" "Task: \`V270-007\` / GitHub issue \`#860\`"
require_contains "$CONTRACT_PATH" "runtime_integration_scope = product_operations_runtime_integration_fail_closed_hardening"
require_contains "$CONTRACT_PATH" "required_artifacts = identity_permission,audit_storage,deployment_orchestration,telemetry_slo,admin_workbench_bridge"
require_contains "$CONTRACT_PATH" "partial_runtime_integration_product_ready_allowed = false"
require_contains "$CONTRACT_PATH" "product_grade_trading_ready_allowed = false"
require_contains "$CONTRACT_PATH" "any submit/mutation/adapter/remediation/control field true => fail_closed_forbidden_control"
require_contains "$CONTRACT_PATH" "any required-false boundary field missing => fail_closed_missing_required_false_boundary"

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
    "read_model.runtime_integration_fail_closed_hardening.ready_readonly.001",
    "read_model.runtime_integration_fail_closed_hardening.partial_identity_degraded.001",
    "read_model.runtime_integration_fail_closed_hardening.stale_telemetry_degraded.001",
    "read_model.runtime_integration_fail_closed_hardening.missing_audit_fail_closed.001",
    "read_model.runtime_integration_fail_closed_hardening.forbidden_submit_control_fail_closed.001",
    "read_model.runtime_integration_fail_closed_hardening.forbidden_remediation_adapter_fail_closed.001",
    "read_model.runtime_integration_fail_closed_hardening.product_ready_claim_fail_closed.001",
    "read_model.runtime_integration_fail_closed_hardening.missing_required_false_boundary_fail_closed.001",
]
CONTRACT_VERSION = "ntpro.v270.runtime_integration_fail_closed_hardening.v1"
SCHEMA_VERSION = "ntpro.v270.runtime_integration_fail_closed_hardening.schema.v1"
INTEGRATION_SCOPE = "product_operations_runtime_integration_fail_closed_hardening"
DEPENDENCIES = ["V270-001", "V270-002", "V270-003", "V270-004", "V270-005", "V270-006"]
REQUIRED_ARTIFACTS = [
    "identity_permission",
    "audit_storage",
    "deployment_orchestration",
    "telemetry_slo",
    "admin_workbench_bridge",
]
ALLOWED_SOURCE_PREFIX = {
    "identity_permission": "docs/rust-cutover/release/v0_27_0_external_identity_permission_foundation.md",
    "audit_storage": "docs/rust-cutover/release/v0_27_0_persistent_operation_audit_storage_foundation.md",
    "deployment_orchestration": "docs/rust-cutover/release/v0_27_0_deployment_orchestration_foundation.md",
    "telemetry_slo": "docs/rust-cutover/release/v0_27_0_long_run_telemetry_slo_runtime_evidence.md",
    "admin_workbench_bridge": "docs/rust-cutover/release/v0_27_0_admin_workbench_runtime_state_bridge.md",
}
BOUNDARY_FALSE_FLAGS = [
    "submit_order_allowed",
    "cancel_order_allowed",
    "retry_order_allowed",
    "replace_order_allowed",
    "amend_order_allowed",
    "flatten_position_allowed",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "order_ticket_enabled",
    "manual_operation_entry_enabled",
    "manual_operation_submit_allowed",
    "dashboard_operation_controls_enabled",
    "dashboard_trading_controls_enabled",
    "admin_workbench_operation_controls_enabled",
    "admin_workbench_trading_controls_enabled",
    "automatic_remediation_allowed",
    "retry_scheduler_enabled",
    "adapter_send_allowed",
    "live_exchange_request_allowed",
    "product_grade_trading_terminal_claim",
    "product_grade_trading_ready",
    "display_product_ready_badge",
]
PRODUCT_READY_FLAGS = {
    "product_grade_trading_terminal_claim",
    "product_grade_trading_ready",
    "display_product_ready_badge",
}
HARNESS = "scripts/ai/verify_release.sh v27-runtime-integration-fail-closed-hardening"
TRACE_REF = "tests/golden/v270_runtime_integration_fail_closed_hardening.jsonl"
VALIDATOR_ENTRYPOINT = "scripts/ai/verify_v27_runtime_integration_fail_closed_hardening.sh::classify"


def fail(message: str) -> None:
    raise SystemExit(f"v27 runtime integration fail-closed hardening failed: {message}")


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


def validate_source_ref(artifact_id: str, source_ref: Any) -> bool:
    if not non_empty(source_ref):
        return False
    value = source_ref.strip()
    if value.startswith(("http://", "https://", "/")):
        return False
    expected = ALLOWED_SOURCE_PREFIX.get(artifact_id)
    if expected is None or not value.startswith(expected):
        return False
    path = value.split("#", 1)[0]
    return Path(path).is_file()


def classify(snapshot: dict[str, Any], case_id: str) -> dict[str, Any]:
    if snapshot.get("contract_version") != CONTRACT_VERSION:
        fail(f"{case_id}: contract_version must be {CONTRACT_VERSION}")
    if snapshot.get("schema_version") != SCHEMA_VERSION:
        fail(f"{case_id}: schema_version must be {SCHEMA_VERSION}")
    if snapshot.get("runtime_integration_scope") != INTEGRATION_SCOPE:
        fail(f"{case_id}: runtime_integration_scope must be {INTEGRATION_SCOPE}")
    if snapshot.get("dependency_contracts") != DEPENDENCIES:
        fail(f"{case_id}: dependency_contracts must be {DEPENDENCIES}")

    downgrade_reasons: list[str] = []
    fail_closed_reasons: list[str] = list(snapshot.get("fail_closed_reasons") or [])
    missing_artifacts: list[str] = []
    ready_artifacts: list[str] = []
    missing_required_false = False
    forbidden_control = False
    product_ready_claim = False
    malformed_provenance = False
    redaction_breach = False
    stale_runtime = False
    partial_runtime = False

    boundary_flags = snapshot.get("boundary_flags")
    if not isinstance(boundary_flags, dict):
        missing_required_false = True
        push_reason(fail_closed_reasons, "missing_boundary_flags")
        boundary_flags = {}
    for key in BOUNDARY_FALSE_FLAGS:
        if key not in boundary_flags:
            missing_required_false = True
            push_reason(fail_closed_reasons, f"missing_required_false_boundary:{key}")
            continue
        if boundary_flags.get(key) is not False:
            if key in PRODUCT_READY_FLAGS:
                product_ready_claim = True
                push_reason(fail_closed_reasons, f"product_ready_claim:{key}")
            else:
                forbidden_control = True
                push_reason(fail_closed_reasons, f"forbidden_control:{key}")

    readiness = snapshot.get("runtime_readiness")
    if not isinstance(readiness, dict):
        fail(f"{case_id}: runtime_readiness must be an object")
    for key in ("product_grade_trading_ready", "display_product_ready_badge"):
        if readiness.get(key) is not False:
            product_ready_claim = True
            push_reason(fail_closed_reasons, f"product_ready_claim:{key}")
    if readiness.get("product_ready_claim") not in (False, None):
        product_ready_claim = True
        push_reason(fail_closed_reasons, "product_ready_claim:runtime_readiness")

    artifacts = snapshot.get("artifacts")
    if not isinstance(artifacts, dict):
        fail(f"{case_id}: artifacts must be an object")
    for artifact_id in REQUIRED_ARTIFACTS:
        artifact = artifacts.get(artifact_id)
        if not isinstance(artifact, dict):
            missing_artifacts.append(artifact_id)
            push_reason(fail_closed_reasons, f"missing_artifact:{artifact_id}")
            continue

        artifact_status = artifact.get("artifact_status")
        if artifact_status == "ready":
            ready_artifacts.append(artifact_id)
        elif artifact_status == "partial":
            partial_runtime = True
            push_reason(downgrade_reasons, f"partial_runtime_integration:{artifact_id}")
        elif artifact_status in ("missing", "unavailable"):
            missing_artifacts.append(artifact_id)
            push_reason(fail_closed_reasons, f"missing_artifact:{artifact_id}")
        else:
            fail(f"{case_id}: unsupported artifact_status for {artifact_id}: {artifact_status!r}")

        if artifact.get("provenance_status") != "verified" or not validate_source_ref(artifact_id, artifact.get("source_ref")):
            malformed_provenance = True
            push_reason(fail_closed_reasons, f"malformed_provenance:{artifact_id}")
        if not non_empty(artifact.get("source_digest")):
            malformed_provenance = True
            push_reason(fail_closed_reasons, f"missing_source_digest:{artifact_id}")
        if artifact.get("freshness_status") != "fresh":
            stale_runtime = True
            push_reason(downgrade_reasons, f"stale_artifact:{artifact_id}:{artifact.get('freshness_status')}")
        if artifact.get("redaction_status") != "redacted":
            redaction_breach = True
            push_reason(fail_closed_reasons, f"redaction_breach:{artifact_id}:{artifact.get('redaction_status')}")
        if artifact.get("raw_secret_persisted") is not False or artifact.get("raw_exchange_response_persisted") is not False:
            redaction_breach = True
            push_reason(fail_closed_reasons, f"raw_sensitive_payload_persisted:{artifact_id}")
        runtime_state = artifact.get("runtime_integration_state")
        if runtime_state == "partial":
            partial_runtime = True
            push_reason(downgrade_reasons, f"partial_runtime_integration:{artifact_id}")
        elif runtime_state != "aligned":
            malformed_provenance = True
            push_reason(fail_closed_reasons, f"runtime_integration_state_invalid:{artifact_id}:{runtime_state}")
        if artifact.get("read_only") is not True:
            forbidden_control = True
            push_reason(fail_closed_reasons, f"artifact_not_read_only:{artifact_id}")
        if artifact.get("exposes_operation_controls") is not False or artifact.get("exposes_trading_controls") is not False:
            forbidden_control = True
            push_reason(fail_closed_reasons, f"artifact_exposes_controls:{artifact_id}")

    surfaces = snapshot.get("surfaces")
    if not isinstance(surfaces, dict):
        fail(f"{case_id}: surfaces must be an object")
    for surface_name in ("admin_workbench", "dashboard"):
        surface = surfaces.get(surface_name)
        if not isinstance(surface, dict):
            fail(f"{case_id}: missing surface {surface_name}")
        if surface.get("read_only") is not True or surface.get("display_only") is not True:
            forbidden_control = True
            push_reason(fail_closed_reasons, f"{surface_name}_not_read_only")
        for field in ("operation_controls_enabled", "trading_controls_enabled", "submit_controls_enabled", "mutation_controls_enabled"):
            if surface.get(field) is not False:
                forbidden_control = True
                push_reason(fail_closed_reasons, f"{surface_name}_{field}")
        if surface.get("product_ready_badge_enabled") is not False:
            product_ready_claim = True
            push_reason(fail_closed_reasons, f"product_ready_claim:{surface_name}_badge")

    if product_ready_claim:
        status = "fail_closed_product_ready_claim"
    elif missing_required_false:
        status = "fail_closed_missing_required_false_boundary"
    elif forbidden_control:
        status = "fail_closed_forbidden_control"
    elif missing_artifacts:
        status = "fail_closed_missing_required_artifact"
    elif redaction_breach:
        status = "fail_closed_redaction_breach"
    elif malformed_provenance:
        status = "fail_closed_malformed_provenance"
    elif partial_runtime:
        status = "degraded_partial_runtime_integration"
    elif stale_runtime:
        status = "degraded_stale_runtime_integration"
    else:
        status = "healthy_readonly"

    fail_closed = status.startswith("fail_closed")
    degraded = status.startswith("degraded")
    all_controls_absent = not (
        missing_required_false or forbidden_control or product_ready_claim
    )
    operation_controls_absent = all(boundary_flags.get(key) is False for key in [
        "submit_order_allowed",
        "cancel_order_allowed",
        "retry_order_allowed",
        "replace_order_allowed",
        "amend_order_allowed",
        "flatten_position_allowed",
        "production_order_submission_allowed",
        "production_order_mutation_allowed",
        "manual_operation_entry_enabled",
        "manual_operation_submit_allowed",
        "dashboard_operation_controls_enabled",
        "admin_workbench_operation_controls_enabled",
        "automatic_remediation_allowed",
        "retry_scheduler_enabled",
    ])
    trading_controls_absent = all(boundary_flags.get(key) is False for key in [
        "order_ticket_enabled",
        "dashboard_trading_controls_enabled",
        "admin_workbench_trading_controls_enabled",
        "product_grade_trading_terminal_claim",
        "product_grade_trading_ready",
        "display_product_ready_badge",
    ])
    adapter_and_exchange_absent = (
        boundary_flags.get("adapter_send_allowed") is False
        and boundary_flags.get("live_exchange_request_allowed") is False
    )

    if status == "healthy_readonly" and readiness.get("runtime_integration_status") != "ready_readonly":
        fail(f"{case_id}: healthy case must declare ready_readonly readiness")
    return {
        "runtime_integration_status": status,
        "artifact_count": len(REQUIRED_ARTIFACTS),
        "ready_artifact_count": len(ready_artifacts),
        "missing_artifacts": missing_artifacts,
        "fail_closed": fail_closed,
        "degraded": degraded,
        "product_grade_trading_ready": False if not product_ready_claim else bool(readiness.get("product_grade_trading_ready") or boundary_flags.get("product_grade_trading_ready")),
        "display_product_ready_badge": False if not product_ready_claim else bool(readiness.get("display_product_ready_badge") or boundary_flags.get("display_product_ready_badge")),
        "operation_controls_absent": operation_controls_absent,
        "trading_controls_absent": trading_controls_absent,
        "adapter_and_exchange_absent": adapter_and_exchange_absent,
        "all_required_false_boundaries_present": not missing_required_false,
        "downgrade_reasons": downgrade_reasons,
        "fail_closed_reasons": fail_closed_reasons,
    }


def validate_replay_scope() -> None:
    try:
        scope = json.loads(replay_scope_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        fail(f"{replay_scope_path}: invalid JSON: {exc}")
    cases = scope.get("cases")
    if not isinstance(cases, list):
        fail(f"{replay_scope_path}: cases must be a list")
    by_case = {case.get("case_id"): case for case in cases if isinstance(case, dict)}
    for case_id in EXPECTED_CASES:
        entry = by_case.get(case_id)
        if not isinstance(entry, dict):
            fail(f"{case_id}: missing release replay scope entry")
        expected = {
            "evidence_id": "V270-007",
            "harness": HARNESS,
            "status": "validator_executable_replay",
            "classification_owner": "V270-007",
            "source_scope_owner": "V270-007",
            "trace": TRACE_REF,
            "validator_entrypoint": VALIDATOR_ENTRYPOINT,
            "replay_type": "validator_executable_runtime_integration_fail_closed_hardening",
            "release_decision": "validator_executable_scope_recorded",
        }
        for key, value in expected.items():
            if entry.get(key) != value:
                fail(f"{case_id}: release scope {key} must be {value!r}, got {entry.get(key)!r}")


rows = load_rows(trace_path)
case_ids = [row.get("case_id") for row in rows]
if case_ids != EXPECTED_CASES:
    fail(f"case order mismatch: expected {EXPECTED_CASES}, got {case_ids}")

for row in rows:
    case_id = row["case_id"]
    if row.get("schema_version") != "golden-trace-v1":
        fail(f"{case_id}: schema_version must be golden-trace-v1")
    if row.get("category") != "read_model":
        fail(f"{case_id}: category must be read_model")
    input_event = single_event(row, "input", case_id)
    expected_event = single_event(row, "expected", case_id)
    if input_event.get("event_type") != "read_model.runtime_integration_fail_closed_hardening.input":
        fail(f"{case_id}: unexpected input event_type {input_event.get('event_type')!r}")
    if expected_event.get("event_type") != "read_model.runtime_integration_fail_closed_hardening.validated":
        fail(f"{case_id}: unexpected expected event_type {expected_event.get('event_type')!r}")
    snapshot = input_event.get("payload", {}).get("snapshot")
    if not isinstance(snapshot, dict):
        fail(f"{case_id}: missing payload.snapshot")
    actual_payload = classify(snapshot, case_id)
    expected_payload = expected_event.get("payload")
    if actual_payload != expected_payload:
        fail(
            f"{case_id}: expected payload mismatch\n"
            f"expected={json.dumps(expected_payload, sort_keys=True)}\n"
            f"actual={json.dumps(actual_payload, sort_keys=True)}"
        )

negative_selftest = 0
if selftest:
    ready_snapshot = single_event(rows[0], "input", rows[0]["case_id"])["payload"]["snapshot"]
    mutated = copy.deepcopy(ready_snapshot)
    mutated["boundary_flags"]["submit_order_allowed"] = True
    if classify(mutated, "selftest:submit")["runtime_integration_status"] != "fail_closed_forbidden_control":
        fail("selftest failed: submit_order_allowed=true must fail closed")
    negative_selftest += 1

    mutated = copy.deepcopy(ready_snapshot)
    del mutated["boundary_flags"]["manual_operation_submit_allowed"]
    if classify(mutated, "selftest:missing_false")["runtime_integration_status"] != "fail_closed_missing_required_false_boundary":
        fail("selftest failed: missing required-false boundary must fail closed")
    negative_selftest += 1

    mutated = copy.deepcopy(ready_snapshot)
    mutated["runtime_readiness"]["product_grade_trading_ready"] = True
    if classify(mutated, "selftest:product_ready")["runtime_integration_status"] != "fail_closed_product_ready_claim":
        fail("selftest failed: product-ready claim must fail closed")
    negative_selftest += 1

    mutated = copy.deepcopy(ready_snapshot)
    mutated["artifacts"]["telemetry_slo"]["freshness_status"] = "stale"
    if classify(mutated, "selftest:stale")["runtime_integration_status"] != "degraded_stale_runtime_integration":
        fail("selftest failed: stale artifact must degrade")
    negative_selftest += 1

validate_replay_scope()

print(
    "v27_runtime_integration_fail_closed_hardening=pass "
    f"cases={len(rows)} artifacts={len(REQUIRED_ARTIFACTS)} "
    f"boundary_flags={len(BOUNDARY_FALSE_FLAGS)} negative_selftest={negative_selftest}"
)
PY
