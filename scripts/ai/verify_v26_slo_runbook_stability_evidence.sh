#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

TRACE_PATH="${NTPRO_V260_STABILITY_TRACE:-tests/golden/v260_slo_runbook_stability_evidence.jsonl}"
TASK_PATH="${NTPRO_V260_STABILITY_TASK:-docs/rust-cutover/tasks/V260-006.md}"
EVIDENCE_PATH="${NTPRO_V260_STABILITY_EVIDENCE:-docs/rust-cutover/evidence/V260-006.md}"
CONTRACT_PATH="${NTPRO_V260_STABILITY_CONTRACT:-docs/rust-cutover/release/v0_26_0_slo_runbook_stability_evidence.md}"
BOUNDARY_DEPENDENCY_PATH="${NTPRO_V260_STABILITY_BOUNDARY_DEPENDENCY:-docs/rust-cutover/release/v0_26_0_product_hardening_boundary_contract.md}"
RUNBOOK_DEPENDENCY_PATH="${NTPRO_V260_STABILITY_RUNBOOK_DEPENDENCY:-docs/rust-cutover/release/v0_26_0_upgrade_rollback_runbook_evidence.md}"
REPLAY_SCOPE_PATH="${NTPRO_V260_STABILITY_REPLAY_SCOPE:-docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"
SELFTEST="${NTPRO_V260_STABILITY_SELFTEST:-1}"

fail() {
  echo "v26 SLO runbook stability evidence failed: $*" >&2
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

for path in "$TRACE_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$CONTRACT_PATH" "$BOUNDARY_DEPENDENCY_PATH" "$RUNBOOK_DEPENDENCY_PATH" "$REPLAY_SCOPE_PATH"; do
  require_file "$path"
done

require_contains "$TASK_PATH" "GitHub issue: \`#818\`"
require_contains "$EVIDENCE_PATH" "Task: \`V260-006\` / GitHub issue \`#818\`"
require_contains "$CONTRACT_PATH" "stability_artifact_scope = slo_runbook_long_run_stability_evidence_only"
require_contains "$CONTRACT_PATH" "sample_provenance_required = true"
require_contains "$CONTRACT_PATH" "sample_freshness_required = true"
require_contains "$CONTRACT_PATH" "sample_redaction_required = true"
require_contains "$CONTRACT_PATH" "automatic_remediation_allowed = false"
require_contains "$CONTRACT_PATH" "restart_execution_allowed = false"
require_contains "$CONTRACT_PATH" "trading_operation_allowed = false"
require_contains "$CONTRACT_PATH" "dashboard_execution_controls_enabled = false"
require_contains "$BOUNDARY_DEPENDENCY_PATH" "release_scope = product_hardening_foundation_only"
require_contains "$RUNBOOK_DEPENDENCY_PATH" "runbook_artifact_scope = upgrade_rollback_runbook_preview_only"

python3 - "$TRACE_PATH" "$REPLAY_SCOPE_PATH" "$SELFTEST" <<'PY'
from __future__ import annotations

import copy
import json
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

trace_path = Path(sys.argv[1])
replay_scope_path = Path(sys.argv[2])
selftest = sys.argv[3] != "0"

EXPECTED_CASES = [
    "read_model.slo_runbook_stability.valid_long_run_window.001",
    "read_model.slo_runbook_stability.stale_sample_degraded.001",
    "read_model.slo_runbook_stability.missing_component_restart_recommended.001",
    "read_model.slo_runbook_stability.error_budget_exhausted_fail_closed.001",
    "read_model.slo_runbook_stability.runbook_stale_degraded.001",
    "read_model.slo_runbook_stability.release_drift_fail_closed.001",
]
CONTRACT_VERSION = "ntpro.v260.slo_runbook_stability.v1"
SCHEMA_VERSION = "ntpro.v260.stability_evidence.schema.v1"
STABILITY_SCOPE = "slo_runbook_long_run_stability_evidence_only"
EXPECTED_TAG = "ntpro-rust-only-v0.26.0-expected"
HARNESS = "scripts/ai/verify_release.sh v26-slo-runbook-stability-evidence"


def fail(message: str) -> None:
    raise SystemExit(f"v26 SLO runbook stability evidence failed: {message}")


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


def parse_ts(value: Any) -> datetime:
    if not non_empty(value):
        fail("timestamp must be a non-empty string")
    parsed = datetime.fromisoformat(str(value).replace("Z", "+00:00"))
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)


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
    if artifact.get("dependency_contracts") != ["V260-001", "V260-004", "V260-005"]:
        fail(f"{case_id}: dependency_contracts must be ['V260-001', 'V260-004', 'V260-005']")

    reasons: list[str] = list(artifact.get("degradation_reasons") or [])
    missing_required = False
    stale_or_unredacted = False
    missing_components = False
    error_budget_exhausted = False
    runbook_stale = False
    release_drift = False
    restart_recommended = False
    forbidden_execution = False

    if artifact.get("stability_artifact_scope") != STABILITY_SCOPE:
        missing_required = True
        push_reason(reasons, "stability_artifact_scope_mismatch")
    for key in (
        "automatic_remediation_allowed",
        "restart_execution_allowed",
        "trading_operation_allowed",
        "dashboard_execution_controls_enabled",
        "strategy_stop_allowed",
        "order_cancel_allowed",
        "order_submit_allowed",
        "live_exchange_request_allowed",
    ):
        if artifact.get(key) is not False:
            forbidden_execution = True
            push_reason(reasons, f"{key}_opened")
    parse_ts(artifact.get("evaluated_at"))

    window = artifact.get("long_run_window")
    if not isinstance(window, dict):
        fail(f"{case_id}: long_run_window must be an object")
    window_id = str(window.get("window_id") or "unknown")
    observed_minutes = as_int(window.get("observed_minutes"), "long_run_window.observed_minutes", case_id)
    min_required_minutes = as_int(window.get("min_required_minutes"), "long_run_window.min_required_minutes", case_id)
    sample_count = as_int(window.get("sample_count"), "long_run_window.sample_count", case_id)
    expected_component_count = as_int(window.get("expected_component_count"), "long_run_window.expected_component_count", case_id)
    present_component_count = as_int(window.get("present_component_count"), "long_run_window.present_component_count", case_id)

    if observed_minutes < min_required_minutes:
        missing_required = True
        push_reason(reasons, f"long_run_window_too_short:{observed_minutes}<{min_required_minutes}")
    if sample_count <= 0:
        missing_required = True
        push_reason(reasons, "missing_stability_samples")
    if present_component_count < expected_component_count:
        missing_components = True
        push_reason(reasons, f"present_component_count_below_expected:{present_component_count}<{expected_component_count}")
    if not non_empty(window.get("sample_provenance_ref")):
        missing_required = True
        push_reason(reasons, "missing_sample_provenance_ref")
    if window.get("freshness_status") != "fresh":
        stale_or_unredacted = True
        push_reason(reasons, f"sample_freshness_not_fresh:{window.get('freshness_status')}")
    if window.get("redaction") != "redacted":
        stale_or_unredacted = True
        push_reason(reasons, f"sample_redaction_not_redacted:{window.get('redaction')}")
    expected_tag = str(window.get("expected_release_tag") or EXPECTED_TAG)
    release_tag = str(window.get("release_tag") or "")
    if release_tag != expected_tag:
        release_drift = True
        push_reason(reasons, f"release_tag_mismatch:{release_tag}!={expected_tag}")

    provenance = artifact.get("sample_provenance")
    if not isinstance(provenance, dict):
        missing_required = True
        push_reason(reasons, "missing_sample_provenance")
        provenance = {}
    for field, reason in (
        ("ref", "missing_sample_provenance_ref"),
        ("source_digest", "missing_sample_source_digest"),
    ):
        if not non_empty(provenance.get(field)):
            missing_required = True
            push_reason(reasons, reason)
    if provenance.get("redaction") != "redacted":
        stale_or_unredacted = True
        push_reason(reasons, f"sample_provenance_redaction_not_redacted:{provenance.get('redaction')}")

    components = artifact.get("components")
    if not isinstance(components, list) or not components:
        missing_components = True
        push_reason(reasons, "missing_component_evidence")
        components = []
    for component in components:
        if not isinstance(component, dict):
            fail(f"{case_id}: component entry must be an object")
        component_id = str(component.get("component_id") or "unknown")
        if component.get("status") != "present":
            missing_components = True
            push_reason(reasons, f"component_missing:{component_id}")
        if component.get("freshness_status") != "fresh":
            stale_or_unredacted = True
            push_reason(reasons, f"component_sample_not_fresh:{component_id}:{component.get('freshness_status')}")
        if not non_empty(component.get("sample_ref")):
            missing_required = True
            push_reason(reasons, f"missing_component_sample_ref:{component_id}")
        if component.get("redaction") != "redacted":
            stale_or_unredacted = True
            push_reason(reasons, f"component_redaction_not_redacted:{component_id}:{component.get('redaction')}")

    objectives = artifact.get("slo_objectives")
    if not isinstance(objectives, list) or not objectives:
        missing_required = True
        push_reason(reasons, "missing_slo_objectives")
        objectives = []
    for objective in objectives:
        if not isinstance(objective, dict):
            fail(f"{case_id}: SLO objective must be an object")
        objective_id = str(objective.get("objective_id") or "unknown")
        target = as_float(objective.get("target"), f"{objective_id}.target", case_id)
        observed = as_float(objective.get("observed"), f"{objective_id}.observed", case_id)
        remaining = as_float(objective.get("error_budget_remaining"), f"{objective_id}.error_budget_remaining", case_id)
        if objective.get("error_budget_exhausted") is True or remaining <= 0:
            error_budget_exhausted = True
            push_reason(reasons, f"objective_error_budget_exhausted:{objective_id}")
        if objective.get("comparison") == "lte" and observed > target and remaining <= 0:
            error_budget_exhausted = True
            push_reason(reasons, f"objective_observed_above_target:{objective_id}:{observed}>{target}")

    runbook = artifact.get("runbook")
    if not isinstance(runbook, dict):
        missing_required = True
        push_reason(reasons, "missing_stability_runbook")
        runbook = {}
    if not non_empty(runbook.get("runbook_ref")):
        missing_required = True
        push_reason(reasons, "missing_runbook_ref")
    if runbook.get("freshness_status") != "current" or runbook.get("stale") is True:
        runbook_stale = True
        push_reason(reasons, f"runbook_stale:{runbook.get('runbook_ref')}")
    if runbook.get("recommendation_only") is not True:
        forbidden_execution = True
        push_reason(reasons, "runbook_recommendation_boundary_opened")

    restart = artifact.get("restart_recommendation")
    if not isinstance(restart, dict):
        missing_required = True
        push_reason(reasons, "missing_restart_recommendation_boundary")
        restart = {}
    if restart.get("recommended") is True:
        restart_recommended = True
        push_reason(reasons, f"restart_recommended:{restart.get('reason')}")
    if restart.get("recommendation_only") is not True:
        forbidden_execution = True
        push_reason(reasons, "restart_recommendation_not_preview_only")
    if restart.get("execution_triggered") is not False:
        forbidden_execution = True
        push_reason(reasons, "restart_execution_triggered")

    release_drift_obj = artifact.get("release_drift")
    if isinstance(release_drift_obj, dict) and release_drift_obj.get("detected") is True:
        release_drift = True
        push_reason(reasons, f"release_drift_detected:{release_drift_obj.get('reason')}")

    if forbidden_execution:
        status = "fail_closed_forbidden_execution_boundary"
    elif release_drift:
        status = "fail_closed_release_drift"
    elif error_budget_exhausted:
        status = "fail_closed_error_budget_exhausted"
    elif missing_required:
        status = "degraded_missing_required_evidence"
    elif stale_or_unredacted:
        status = "degraded_stale_or_unredacted_samples"
    elif missing_components:
        status = "degraded_missing_components_restart_recommended"
    elif runbook_stale:
        status = "degraded_runbook_stale"
    elif restart_recommended:
        status = "degraded_restart_recommended"
    else:
        status = "stability_healthy"

    return {
        "case_id": case_id,
        "stability_artifact_scope": artifact.get("stability_artifact_scope"),
        "effective_stability_status": status,
        "long_run_window_id": window_id,
        "sample_count": sample_count,
        "observed_minutes": observed_minutes,
        "present_component_count": present_component_count,
        "expected_component_count": expected_component_count,
        "long_run_ready": status == "stability_healthy",
        "dashboard_read_only": True,
        "automatic_remediation_allowed": False,
        "restart_execution_allowed": False,
        "trading_operation_allowed": False,
        "fail_closed": status.startswith("fail_closed"),
        "degradation_reasons": reasons,
    }


rows = load_rows(trace_path)
if [row.get("case_id") for row in rows] != EXPECTED_CASES:
    fail(
        "case order mismatch: expected "
        + ", ".join(EXPECTED_CASES)
        + " got "
        + ", ".join(str(row.get("case_id")) for row in rows)
    )

healthy_artifact: dict[str, Any] | None = None
for row in rows:
    case_id = str(row.get("case_id"))
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

    payload = input_event.get("payload")
    if not isinstance(payload, dict) or not isinstance(payload.get("artifact"), dict):
        fail(f"{case_id}: input payload artifact is required")
    artifact = payload["artifact"]
    computed = classify(artifact, case_id)
    expected_payload = expected_event.get("payload")
    if computed != expected_payload:
        fail(
            f"{case_id}: computed payload mismatch\n"
            f"expected={json.dumps(expected_payload, sort_keys=True)}\n"
            f"actual={json.dumps(computed, sort_keys=True)}"
        )
    if case_id.endswith("valid_long_run_window.001"):
        healthy_artifact = copy.deepcopy(artifact)

if selftest:
    if healthy_artifact is None:
        fail("negative selftest requires valid stability artifact")
    healthy_artifact["restart_execution_allowed"] = True
    healthy_artifact["restart_recommendation"]["execution_triggered"] = True
    closed = classify(healthy_artifact, "negative.selftest.restart_execution_opened")
    if closed["effective_stability_status"] != "fail_closed_forbidden_execution_boundary":
        fail("negative selftest opened restart execution but did not fail closed")
    if "restart_execution_allowed_opened" not in closed["degradation_reasons"]:
        fail("negative selftest did not surface restart execution boundary reason")

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
        "evidence_id": "V260-006",
        "harness": HARNESS,
        "validator_entrypoint": "scripts/ai/verify_v26_slo_runbook_stability_evidence.sh::classify",
        "replay_type": "validator_executable_slo_runbook_stability_evidence",
        "classification_owner": "V260-006",
        "source_scope_owner": "V260-006",
        "stability_artifact_scope": STABILITY_SCOPE,
    }
    for key, expected in expected_pairs.items():
        if entry.get(key) != expected:
            fail(f"{case_id}: release scope {key} mismatch: {entry.get(key)!r}")
    for key in (
        "runtime_adapter_integration",
        "automatic_remediation_allowed",
        "restart_execution_allowed",
        "trading_operation_allowed",
        "dashboard_execution_controls_enabled",
        "strategy_stop_allowed",
        "order_cancel_allowed",
        "order_submit_allowed",
        "live_exchange_request_allowed",
        "new_submit_capability",
        "production_order_mutation_allowed",
        "adapter_send_allowed",
        "product_grade_live_trading_terminal",
    ):
        if entry.get(key) is not False:
            fail(f"{case_id}: release scope {key} must be false")

print(
    "v26_slo_runbook_stability_evidence "
    f"status=ok trace={trace_path.as_posix()} "
    f"cases={len(rows)} negative_selftest={1 if selftest else 0}"
)
PY
