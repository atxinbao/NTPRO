#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

TRACE_PATH="${NTPRO_V270_TELEMETRY_SLO_TRACE:-tests/golden/v270_long_run_telemetry_slo_runtime_evidence.jsonl}"
TASK_PATH="${NTPRO_V270_TELEMETRY_SLO_TASK:-docs/rust-cutover/tasks/V270-005.md}"
EVIDENCE_PATH="${NTPRO_V270_TELEMETRY_SLO_EVIDENCE:-docs/rust-cutover/evidence/V270-005.md}"
CONTRACT_PATH="${NTPRO_V270_TELEMETRY_SLO_CONTRACT:-docs/rust-cutover/release/v0_27_0_long_run_telemetry_slo_runtime_evidence.md}"
BOUNDARY_PATH="${NTPRO_V270_TELEMETRY_SLO_BOUNDARY:-docs/rust-cutover/release/v0_27_0_product_operations_runtime_integration_boundary_contract.md}"
V26_STABILITY_PATH="${NTPRO_V270_TELEMETRY_SLO_V26_STABILITY:-docs/rust-cutover/release/v0_26_0_slo_runbook_stability_evidence.md}"
REPLAY_SCOPE_PATH="${NTPRO_V270_TELEMETRY_SLO_REPLAY_SCOPE:-docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"
SELFTEST="${NTPRO_V270_TELEMETRY_SLO_SELFTEST:-1}"

fail() {
  echo "v27 long-run telemetry SLO runtime evidence failed: $*" >&2
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

for path in "$TRACE_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$CONTRACT_PATH" "$BOUNDARY_PATH" "$V26_STABILITY_PATH" "$REPLAY_SCOPE_PATH"; do
  require_file "$path"
done

require_contains "$TASK_PATH" "GitHub issue: \`#858\`"
require_contains "$EVIDENCE_PATH" "Task: \`V270-005\` / GitHub issue \`#858\`"
require_contains "$BOUNDARY_PATH" "long_run_telemetry_ingestion"
require_contains "$V26_STABILITY_PATH" "stability_artifact_scope = slo_runbook_long_run_stability_evidence_only"
require_contains "$CONTRACT_PATH" "telemetry_ingestion_scope = long_run_telemetry_ingestion_slo_runtime_evidence_only"
require_contains "$CONTRACT_PATH" "ingestion_mode = observational_non_remediating"
require_contains "$CONTRACT_PATH" "source_contract_required = true"
require_contains "$CONTRACT_PATH" "source_freshness_required = true"
require_contains "$CONTRACT_PATH" "source_redaction_required = true"
require_contains "$CONTRACT_PATH" "sampling_window_required = true"
require_contains "$CONTRACT_PATH" "sampling_gap_detection_required = true"
require_contains "$CONTRACT_PATH" "slo_rollup_required = true"
require_contains "$CONTRACT_PATH" "admin_read_model_degradation_reasons_required = true"
require_contains "$CONTRACT_PATH" "dashboard_degradation_reasons_required = true"
require_contains "$CONTRACT_PATH" "automatic_remediation_allowed = false"
require_contains "$CONTRACT_PATH" "retry_scheduler_enabled = false"
require_contains "$CONTRACT_PATH" "adapter_send_allowed = false"
require_contains "$CONTRACT_PATH" "live_exchange_request_allowed = false"
require_contains "$CONTRACT_PATH" "telemetry_event_triggers_trading_control = false"
require_contains "$CONTRACT_PATH" "dashboard_trading_controls_enabled = false"

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
    "read_model.long_run_telemetry_slo.ready_ingestion.001",
    "read_model.long_run_telemetry_slo.gap_degraded.001",
    "read_model.long_run_telemetry_slo.stale_source_degraded.001",
    "read_model.long_run_telemetry_slo.missing_source_fail_closed.001",
    "read_model.long_run_telemetry_slo.redaction_breach_fail_closed.001",
    "read_model.long_run_telemetry_slo.release_source_drift_fail_closed.001",
    "read_model.long_run_telemetry_slo.forbidden_operation_boundary_fail_closed.001",
]
CONTRACT_VERSION = "ntpro.v270.long_run_telemetry_slo_runtime_evidence.v1"
SCHEMA_VERSION = "ntpro.v270.long_run_telemetry_slo_runtime_evidence.schema.v1"
TELEMETRY_SCOPE = "long_run_telemetry_ingestion_slo_runtime_evidence_only"
INGESTION_MODE = "observational_non_remediating"
DEPENDENCIES = ["V270-001", "V260-006"]
HARNESS = "scripts/ai/verify_release.sh v27-long-run-telemetry-slo-runtime-evidence"
TRACE_REF = "tests/golden/v270_long_run_telemetry_slo_runtime_evidence.jsonl"
VALIDATOR_ENTRYPOINT = "scripts/ai/verify_v27_long_run_telemetry_slo_runtime_evidence.sh::classify"
REQUIRED_SOURCE_FIELDS = [
    "source_id",
    "source_type",
    "source_ref",
    "source_digest",
    "source_snapshot_id",
    "release_tag",
    "expected_release_tag",
]
BOUNDARY_FALSE_FLAGS = [
    "automatic_remediation_allowed",
    "retry_scheduler_enabled",
    "adapter_send_allowed",
    "live_exchange_request_allowed",
    "dashboard_operation_controls_enabled",
    "dashboard_trading_controls_enabled",
    "admin_workbench_operation_controls_enabled",
    "admin_workbench_trading_controls_enabled",
    "trading_operation_allowed",
    "telemetry_event_triggers_remediation",
    "telemetry_event_triggers_retry",
    "telemetry_event_triggers_adapter_send",
    "telemetry_event_triggers_trading_control",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "manual_operation_submit_allowed",
    "product_grade_trading_terminal_claim",
]


def fail(message: str) -> None:
    raise SystemExit(f"v27 long-run telemetry SLO runtime evidence failed: {message}")


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


def as_int(value: Any, field: str, case_id: str) -> int:
    if isinstance(value, bool):
        fail(f"{case_id}: {field} must be an integer")
    try:
        return int(value)
    except (TypeError, ValueError):
        fail(f"{case_id}: {field} must be an integer")


def as_float(value: Any, field: str, case_id: str) -> float:
    if isinstance(value, bool):
        fail(f"{case_id}: {field} must be numeric")
    try:
        return float(value)
    except (TypeError, ValueError):
        fail(f"{case_id}: {field} must be numeric")


def classify(artifact: dict[str, Any], case_id: str) -> dict[str, Any]:
    if artifact.get("contract_version") != CONTRACT_VERSION:
        fail(f"{case_id}: contract_version must be {CONTRACT_VERSION}")
    if artifact.get("schema_version") != SCHEMA_VERSION:
        fail(f"{case_id}: schema_version must be {SCHEMA_VERSION}")
    if artifact.get("telemetry_ingestion_scope") != TELEMETRY_SCOPE:
        fail(f"{case_id}: telemetry_ingestion_scope must be {TELEMETRY_SCOPE}")
    if artifact.get("ingestion_mode") != INGESTION_MODE:
        fail(f"{case_id}: ingestion_mode must be {INGESTION_MODE}")
    if artifact.get("dependency_contracts") != DEPENDENCIES:
        fail(f"{case_id}: dependency_contracts must be {DEPENDENCIES}")

    reasons: list[str] = list(artifact.get("degradation_reasons") or [])
    missing_source = False
    stale_source = False
    redaction_breach = False
    release_source_drift = False
    gap_degraded = False
    slo_degraded = False
    slo_fail_closed = False
    forbidden_boundary = False

    boundary_flags = artifact.get("boundary_flags")
    if not isinstance(boundary_flags, dict):
        forbidden_boundary = True
        push_reason(reasons, "missing_boundary_flags")
        boundary_flags = {}
    for key in BOUNDARY_FALSE_FLAGS:
        if key not in boundary_flags:
            forbidden_boundary = True
            push_reason(reasons, f"missing_required_false_boundary:{key}")
        elif boundary_flags.get(key) is not False:
            forbidden_boundary = True
            push_reason(reasons, f"forbidden_boundary_flag:{key}")

    source = artifact.get("source_contract")
    source_present = isinstance(source, dict)
    if not source_present:
        missing_source = True
        push_reason(reasons, "missing_source_contract")
        source = {}
    if source_present:
        for field in REQUIRED_SOURCE_FIELDS:
            if not non_empty(source.get(field)):
                missing_source = True
                push_reason(reasons, f"missing_source_field:{field}")
        if source.get("source_type") not in ("local_runtime_telemetry_artifact", None):
            release_source_drift = True
            push_reason(reasons, f"source_type_mismatch:{source.get('source_type')}")
        if source.get("freshness_status") != "fresh":
            stale_source = True
            push_reason(reasons, f"source_freshness_not_fresh:{source.get('freshness_status')}")
        observed_age = as_int(source.get("observed_age_ms", 0), "source_contract.observed_age_ms", case_id)
        max_age = as_int(source.get("max_age_ms", 0), "source_contract.max_age_ms", case_id)
        if observed_age > max_age:
            stale_source = True
            push_reason(reasons, f"source_age_exceeded:{observed_age}>{max_age}")
        if source.get("redaction_status") != "redacted":
            redaction_breach = True
            push_reason(reasons, f"source_redaction_not_redacted:{source.get('redaction_status')}")
        for field in ("raw_secret_persisted", "raw_exchange_response_persisted"):
            if source.get(field) is not False:
                redaction_breach = True
                push_reason(reasons, f"{field}_not_false")
        if source.get("release_tag") != source.get("expected_release_tag"):
            release_source_drift = True
            push_reason(reasons, f"release_tag_mismatch:{source.get('release_tag')}!={source.get('expected_release_tag')}")
        if source.get("source_scope") != "artifact_truth_only":
            release_source_drift = True
            push_reason(reasons, f"source_scope_mismatch:{source.get('source_scope')}")
        if source.get("runtime_adapter_integration") is not False:
            release_source_drift = True
            push_reason(reasons, "runtime_adapter_integration_not_false")

    window = artifact.get("sampling_window")
    if not isinstance(window, dict):
        missing_source = True
        push_reason(reasons, "missing_sampling_window")
        window = {}
    duration = as_int(window.get("duration_minutes", 0), "sampling_window.duration_minutes", case_id)
    min_duration = as_int(window.get("min_duration_minutes", 0), "sampling_window.min_duration_minutes", case_id)
    sample_count = as_int(window.get("sample_count", 0), "sampling_window.sample_count", case_id)
    expected_sample_count = as_int(window.get("expected_sample_count", 0), "sampling_window.expected_sample_count", case_id)
    gap_count = as_int(window.get("gap_count", 0), "sampling_window.gap_count", case_id)
    max_gap_ms = as_int(window.get("max_gap_ms", 0), "sampling_window.max_gap_ms", case_id)
    allowed_gap_ms = as_int(window.get("allowed_gap_ms", 0), "sampling_window.allowed_gap_ms", case_id)
    if duration < min_duration:
        gap_degraded = True
        push_reason(reasons, f"sampling_window_too_short:{duration}<{min_duration}")
    if sample_count < expected_sample_count:
        gap_degraded = True
        push_reason(reasons, f"sample_count_below_expected:{sample_count}<{expected_sample_count}")
    if gap_count > 0:
        gap_degraded = True
        push_reason(reasons, f"sampling_gap_count:{gap_count}")
    if max_gap_ms > allowed_gap_ms:
        gap_degraded = True
        push_reason(reasons, f"sampling_gap_exceeded:{max_gap_ms}>{allowed_gap_ms}")
    if window.get("window_complete") is not True:
        gap_degraded = True
        push_reason(reasons, "sampling_window_incomplete")

    rollup = artifact.get("slo_rollup")
    if not isinstance(rollup, dict):
        slo_fail_closed = True
        push_reason(reasons, "missing_slo_rollup")
        rollup = {}
    objective_id = rollup.get("objective_id")
    if not non_empty(objective_id):
        slo_fail_closed = True
        push_reason(reasons, "missing_slo_objective_id")
    target = as_float(rollup.get("availability_target", 0), "slo_rollup.availability_target", case_id)
    observed = as_float(rollup.get("observed_availability", 0), "slo_rollup.observed_availability", case_id)
    error_budget = as_float(rollup.get("error_budget_remaining", 0), "slo_rollup.error_budget_remaining", case_id)
    if observed < target:
        slo_degraded = True
        push_reason(reasons, f"slo_availability_below_target:{observed}<{target}")
    if error_budget < 0:
        slo_fail_closed = True
        push_reason(reasons, f"slo_error_budget_exhausted:{error_budget}")

    fail_closed = bool(missing_source or redaction_breach or release_source_drift or forbidden_boundary or slo_fail_closed)
    degraded = bool(stale_source or gap_degraded or slo_degraded)

    if forbidden_boundary:
        telemetry_status = "fail_closed_forbidden_operation_boundary"
    elif missing_source:
        telemetry_status = "fail_closed_missing_source_contract"
    elif redaction_breach:
        telemetry_status = "fail_closed_redaction_breach"
    elif release_source_drift:
        telemetry_status = "fail_closed_release_source_drift"
    elif slo_fail_closed:
        telemetry_status = "fail_closed_slo_rollup"
    elif stale_source:
        telemetry_status = "degraded_stale_source"
    elif gap_degraded:
        telemetry_status = "degraded_sampling_gap"
    elif slo_degraded:
        telemetry_status = "degraded_slo_rollup"
    else:
        telemetry_status = "healthy"

    surface_status = "fail_closed" if fail_closed else ("degraded" if degraded else "healthy")
    if surface_status != "healthy" and not reasons:
        fail(f"{case_id}: degraded/fail_closed cases must expose degradation reasons")

    surfaces = artifact.get("read_model_surfaces")
    if not isinstance(surfaces, dict):
        fail(f"{case_id}: read_model_surfaces must be an object")
    for surface_name in ("admin_read_model", "dashboard"):
        surface = surfaces.get(surface_name)
        if not isinstance(surface, dict):
            fail(f"{case_id}: missing read_model_surfaces.{surface_name}")
        if surface.get("read_only") is not True or surface.get("display_only") is not True:
            fail(f"{case_id}: {surface_name} must be read-only display evidence")
        if surface.get("operation_controls_enabled") is not False or surface.get("trading_controls_enabled") is not False:
            fail(f"{case_id}: {surface_name} controls must be disabled")
        surface_reasons = surface.get("degradation_reasons")
        if surface_status == "healthy":
            if surface_reasons not in ([], None):
                fail(f"{case_id}: healthy {surface_name} must not carry degradation reasons")
        elif not isinstance(surface_reasons, list) or not surface_reasons:
            fail(f"{case_id}: {surface_name} must surface degradation reasons")

    return {
        "telemetry_status": telemetry_status,
        "slo_status": "fail_closed" if slo_fail_closed else ("degraded" if slo_degraded else "healthy"),
        "admin_read_model_status": surface_status,
        "dashboard_status": surface_status,
        "fail_closed": fail_closed,
        "degraded": degraded and not fail_closed,
        "read_only": True,
        "no_automatic_remediation": not forbidden_boundary,
        "no_retry_scheduler": not forbidden_boundary,
        "no_adapter_send": not forbidden_boundary,
        "no_trading_control": not forbidden_boundary,
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
            "classification_owner": "V270-005",
            "source_scope_owner": "V270-005",
            "evidence_id": "V270-005",
            "replay_type": "validator_executable_long_run_telemetry_slo_runtime_evidence",
            "release_decision": "validator_executable_scope_recorded",
            "telemetry_ingestion_scope": TELEMETRY_SCOPE,
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
            "trading_operation_allowed",
            "product_grade_live_trading_terminal",
        ):
            if item.get(key) is not False:
                fail(f"{case_id}: replay scope {key} must be false")


def mark_surface_degraded(artifact: dict[str, Any]) -> dict[str, Any]:
    surfaces = artifact.setdefault("read_model_surfaces", {})
    if isinstance(surfaces, dict):
        for surface in surfaces.values():
            if isinstance(surface, dict):
                surface["degradation_reasons"] = ["selftest_degradation"]
    return artifact


rows = load_rows(trace_path)
case_ids = [str(row.get("case_id")) for row in rows]
if case_ids != EXPECTED_CASES:
    fail(f"unexpected case order: {case_ids}")

status_counts: dict[str, int] = {}
for row in rows:
    case_id = str(row["case_id"])
    input_event = single_event(row, "input", case_id)
    expected_event = single_event(row, "expected", case_id)
    artifact = input_event.get("payload", {}).get("artifact")
    if not isinstance(artifact, dict):
        fail(f"{case_id}: input payload artifact must be an object")
    expected_payload = expected_event.get("payload")
    if not isinstance(expected_payload, dict):
        fail(f"{case_id}: expected payload must be an object")
    actual = classify(artifact, case_id)
    if actual != expected_payload:
        fail(f"{case_id}: expected payload mismatch\nactual={json.dumps(actual, sort_keys=True)}\nexpected={json.dumps(expected_payload, sort_keys=True)}")
    status_counts[actual["telemetry_status"]] = status_counts.get(actual["telemetry_status"], 0) + 1

validate_replay_scope(case_ids)

if selftest:
    ready = single_event(rows[0], "input", EXPECTED_CASES[0])["payload"]["artifact"]
    mutated = copy.deepcopy(ready)
    mutated["boundary_flags"]["adapter_send_allowed"] = True
    mark_surface_degraded(mutated)
    if classify(mutated, "selftest.forbidden_boundary")["telemetry_status"] != "fail_closed_forbidden_operation_boundary":
        fail("selftest forbidden boundary did not fail closed")
    mutated = copy.deepcopy(ready)
    del mutated["boundary_flags"]["automatic_remediation_allowed"]
    mark_surface_degraded(mutated)
    if classify(mutated, "selftest.missing_boundary")["telemetry_status"] != "fail_closed_forbidden_operation_boundary":
        fail("selftest missing boundary did not fail closed")
    mutated = copy.deepcopy(ready)
    mutated["source_contract"]["redaction_status"] = "unredacted"
    mark_surface_degraded(mutated)
    if classify(mutated, "selftest.redaction")["telemetry_status"] != "fail_closed_redaction_breach":
        fail("selftest redaction breach did not fail closed")
    mutated = copy.deepcopy(ready)
    mutated["source_contract"]["freshness_status"] = "stale"
    mutated["source_contract"]["observed_age_ms"] = 120000
    mark_surface_degraded(mutated)
    if classify(mutated, "selftest.stale")["telemetry_status"] != "degraded_stale_source":
        fail("selftest stale source did not degrade")
    mutated = copy.deepcopy(ready)
    del mutated["source_contract"]
    mark_surface_degraded(mutated)
    if classify(mutated, "selftest.missing_source")["telemetry_status"] != "fail_closed_missing_source_contract":
        fail("selftest missing source did not fail closed")

print(
    "v27_long_run_telemetry_slo_runtime_evidence=pass "
    f"cases={len(rows)} statuses={len(status_counts)} "
    f"boundary_flags={len(BOUNDARY_FALSE_FLAGS)} negative_selftest={int(selftest)}"
)
PY
