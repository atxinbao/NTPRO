#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

ARTIFACT_PATH="${NTPRO_V280_TELEMETRY_SLO_ARTIFACT:-docs/rust-cutover/release/v0_28_0_telemetry_slo_ingestion_runtime_artifact.json}"
CONTRACT_PATH="${NTPRO_V280_TELEMETRY_SLO_CONTRACT:-docs/rust-cutover/release/v0_28_0_telemetry_slo_ingestion_runtime_closure.md}"
TASK_PATH="${NTPRO_V280_TELEMETRY_SLO_TASK:-docs/rust-cutover/tasks/V280-005.md}"
EVIDENCE_PATH="${NTPRO_V280_TELEMETRY_SLO_EVIDENCE:-docs/rust-cutover/evidence/V280-005.md}"
MATRIX_PATH="${NTPRO_V280_TELEMETRY_SLO_MATRIX:-docs/rust-cutover/release/v0_28_0_backend_closure_readiness_matrix.json}"
FOUNDATION_PATH="${NTPRO_V280_TELEMETRY_SLO_FOUNDATION:-docs/rust-cutover/release/v0_27_0_long_run_telemetry_slo_runtime_evidence.md}"
STABILITY_PATH="${NTPRO_V280_TELEMETRY_SLO_STABILITY:-docs/rust-cutover/release/v0_26_0_slo_runbook_stability_evidence.md}"
SELFTEST="${NTPRO_V280_TELEMETRY_SLO_SELFTEST:-1}"

fail() {
  echo "v28 telemetry SLO ingestion runtime closure failed: $*" >&2
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

for path in "$ARTIFACT_PATH" "$CONTRACT_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$MATRIX_PATH" "$FOUNDATION_PATH" "$STABILITY_PATH"; do
  require_file "$path"
done

require_contains "$TASK_PATH" "GitHub issue: \`#898\`"
require_contains "$EVIDENCE_PATH" "Task: \`V280-005\` / GitHub issue \`#898\`"
require_contains "$FOUNDATION_PATH" "telemetry_ingestion_scope = long_run_telemetry_ingestion_slo_runtime_evidence_only"
require_contains "$STABILITY_PATH" "stability_artifact_scope = slo_runbook_long_run_stability_evidence_only"
require_contains "$CONTRACT_PATH" "contract_version = ntpro.v280.telemetry_slo_ingestion_runtime_closure.v1"
require_contains "$CONTRACT_PATH" "schema_version = ntpro.v280.telemetry_slo_ingestion_runtime_artifact.v1"
require_contains "$CONTRACT_PATH" "backend_module_status = runtime_closed"
require_contains "$CONTRACT_PATH" "release stage = scripts/ai/verify_release.sh v28-telemetry-slo-ingestion-runtime-closure"

ARTIFACT_PATH="$ARTIFACT_PATH" MATRIX_PATH="$MATRIX_PATH" SELFTEST="$SELFTEST" python3 <<'PY'
from __future__ import annotations

import copy
import json
import os
from pathlib import Path
from typing import Any

artifact_path = Path(os.environ["ARTIFACT_PATH"])
matrix_path = Path(os.environ["MATRIX_PATH"])
selftest = os.environ.get("SELFTEST", "1") != "0"

SCHEMA_VERSION = "ntpro.v280.telemetry_slo_ingestion_runtime_artifact.v1"
CONTRACT_VERSION = "ntpro.v280.telemetry_slo_ingestion_runtime_closure.v1"
DEPENDENCIES = {"V280-001", "V280-003", "V270-005", "V260-006", "V271-006"}
EXPECTED_CASES = [
    "telemetry_slo_ingestion.healthy.allowed.001",
    "telemetry_slo_ingestion.slo_breach.degraded.001",
    "telemetry_slo_ingestion.stale_telemetry.fail_closed.001",
    "telemetry_slo_ingestion.missing_provenance.fail_closed.001",
    "telemetry_slo_ingestion.malformed_metrics.fail_closed.001",
    "telemetry_slo_ingestion.unredacted_payload.fail_closed.001",
    "telemetry_slo_ingestion.forbidden_operation.fail_closed.001",
]
EXPECTED_TRANSITIONS = ["healthy", "degraded", "breached", "fail_closed"]
BOUNDARY_FALSE_FLAGS = [
    "default_submit_allowed",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
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
    "manual_operation_submit_allowed",
    "product_grade_trading_terminal_claim",
]


def fail(message: str) -> None:
    raise SystemExit(f"v28 telemetry SLO ingestion runtime closure failed: {message}")


def non_empty(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


def merge(base: dict[str, Any], override: dict[str, Any] | None) -> dict[str, Any]:
    merged = copy.deepcopy(base)
    if override:
        for key, value in override.items():
            merged[key] = value
    return merged


def as_int(value: Any, field: str) -> int | None:
    if isinstance(value, bool):
        return None
    try:
        return int(value)
    except (TypeError, ValueError):
        return None


def as_float(value: Any, field: str) -> float | None:
    if isinstance(value, bool):
        return None
    try:
        return float(value)
    except (TypeError, ValueError):
        return None


def classify_status(reasons: list[str]) -> str:
    if any(reason.startswith("missing_provenance") for reason in reasons):
        return "fail_closed_missing_provenance"
    if any(reason.startswith("stale_telemetry") for reason in reasons):
        return "fail_closed_stale_telemetry"
    if any(reason.startswith("malformed_metrics") for reason in reasons):
        return "fail_closed_malformed_metrics"
    if any(reason.startswith("unredacted_payload") for reason in reasons):
        return "fail_closed_unredacted_payload"
    if any(reason.startswith("forbidden_operation") for reason in reasons):
        return "fail_closed_forbidden_operation_trigger"
    if any(reason.startswith("slo_breach") for reason in reasons):
        return "degraded_slo_breach"
    if reasons:
        return "fail_closed_telemetry_slo_violation"
    return "telemetry_slo_ingestion_replay_ready"


def apply_case(artifact: dict[str, Any], case: dict[str, Any]) -> dict[str, Any]:
    result = copy.deepcopy(artifact)
    result["telemetry_source"] = merge(result["telemetry_source"], case.get("telemetry_source_override"))
    result["sampling_window"] = merge(result["sampling_window"], case.get("sampling_window_override"))
    result["slo_rollup"] = merge(result["slo_rollup"], case.get("slo_rollup_override"))
    result["payload_policy"] = merge(result["payload_policy"], case.get("payload_policy_override"))
    result["audit_lineage"] = merge(result["audit_lineage"], case.get("audit_lineage_override"))
    result["boundary_flags"] = merge(result["boundary_flags"], case.get("boundary_flags_override"))
    transition_overrides = case.get("transition_overrides") or {}
    for transition in result["slo_status_transitions"]:
        override = transition_overrides.get(transition["transition_id"])
        if override:
            transition.update(override)
    return result


def classify_artifact(artifact: dict[str, Any]) -> dict[str, Any]:
    reasons: list[str] = []
    if artifact.get("schema_version") != SCHEMA_VERSION:
        reasons.append("missing_provenance:schema_version_mismatch")
    if artifact.get("contract_version") != CONTRACT_VERSION:
        reasons.append("missing_provenance:contract_version_mismatch")
    if artifact.get("task_id") != "V280-005" or artifact.get("github_issue") != 898:
        reasons.append("missing_provenance:task_identity_mismatch")
    if artifact.get("backend_module") != "telemetry_slo_ingestion_runtime_closure":
        reasons.append("missing_provenance:backend_module_mismatch")
    if artifact.get("backend_module_status") != "runtime_closed":
        reasons.append("missing_provenance:backend_module_not_runtime_closed")
    if set(artifact.get("dependency_contracts") or []) != DEPENDENCIES:
        reasons.append("missing_provenance:dependency_contracts_mismatch")
    if artifact.get("ingestion_mode") != "deterministic_observability_replay":
        reasons.append(f"missing_provenance:ingestion_mode:{artifact.get('ingestion_mode')}")

    source = artifact.get("telemetry_source")
    if not isinstance(source, dict):
        reasons.append("missing_provenance:telemetry_source")
        source = {}
    for key in ("source_id", "source_type", "source_ref", "source_digest", "source_snapshot_id", "release_tag", "expected_release_tag", "schema_name", "schema_version"):
        if not non_empty(source.get(key)):
            reasons.append(f"missing_provenance:source:{key}")
    if source.get("source_type") != "local_runtime_telemetry_artifact":
        reasons.append(f"missing_provenance:source_type:{source.get('source_type')}")
    if source.get("release_tag") != source.get("expected_release_tag") or source.get("release_tag") != artifact.get("dependency_release_tag"):
        reasons.append("missing_provenance:release_tag_mismatch")
    if source.get("lineage_status") != "linked":
        reasons.append(f"missing_provenance:source_lineage:{source.get('lineage_status')}")
    observed_age = as_int(source.get("observed_age_ms"), "telemetry_source.observed_age_ms")
    max_age = as_int(source.get("max_age_ms"), "telemetry_source.max_age_ms")
    if observed_age is None or max_age is None:
        reasons.append("malformed_metrics:source_age_not_integer")
    elif observed_age > max_age:
        reasons.append(f"stale_telemetry:age:{observed_age}>{max_age}")
    if source.get("freshness_status") != "fresh":
        reasons.append(f"stale_telemetry:freshness:{source.get('freshness_status')}")

    window = artifact.get("sampling_window")
    if not isinstance(window, dict):
        reasons.append("malformed_metrics:missing_sampling_window")
        window = {}
    numeric_window: dict[str, int | None] = {}
    for key in ("duration_minutes", "min_duration_minutes", "sample_count", "expected_sample_count", "gap_count", "max_gap_ms", "allowed_gap_ms", "malformed_sample_count"):
        numeric_window[key] = as_int(window.get(key), f"sampling_window.{key}")
        if numeric_window[key] is None:
            reasons.append(f"malformed_metrics:not_integer:{key}")
    if numeric_window.get("duration_minutes") is not None and numeric_window.get("min_duration_minutes") is not None and numeric_window["duration_minutes"] < numeric_window["min_duration_minutes"]:
        reasons.append("malformed_metrics:duration_below_minimum")
    if numeric_window.get("sample_count") is not None and numeric_window.get("expected_sample_count") is not None and numeric_window["sample_count"] < numeric_window["expected_sample_count"]:
        reasons.append("malformed_metrics:sample_count_below_expected")
    if numeric_window.get("gap_count") not in (0, None):
        reasons.append(f"malformed_metrics:gap_count:{numeric_window.get('gap_count')}")
    if numeric_window.get("max_gap_ms") is not None and numeric_window.get("allowed_gap_ms") is not None and numeric_window["max_gap_ms"] > numeric_window["allowed_gap_ms"]:
        reasons.append("malformed_metrics:max_gap_exceeded")
    if numeric_window.get("malformed_sample_count") not in (0, None):
        reasons.append(f"malformed_metrics:malformed_sample_count:{numeric_window.get('malformed_sample_count')}")

    slo = artifact.get("slo_rollup")
    if not isinstance(slo, dict):
        reasons.append("missing_provenance:missing_slo_rollup")
        slo = {}
    for key in ("slo_id", "status"):
        if not non_empty(slo.get(key)):
            reasons.append(f"missing_provenance:slo:{key}")
    availability = as_float(slo.get("observed_availability"), "slo_rollup.observed_availability")
    target = as_float(slo.get("availability_target"), "slo_rollup.availability_target")
    error_budget = as_float(slo.get("error_budget_remaining"), "slo_rollup.error_budget_remaining")
    breach_count = as_int(slo.get("breach_count"), "slo_rollup.breach_count")
    if availability is None or target is None or error_budget is None or breach_count is None:
        reasons.append("malformed_metrics:slo_numeric_field")
    else:
        if availability < target:
            reasons.append(f"slo_breach:availability:{availability}<{target}")
        if breach_count > 0:
            reasons.append(f"slo_breach:breach_count:{breach_count}")
        if error_budget < 0:
            reasons.append(f"slo_breach:error_budget:{error_budget}")
    if slo.get("transition_audit_required") is not True:
        reasons.append("missing_provenance:slo_transition_audit_not_required")

    payload = artifact.get("payload_policy")
    if not isinstance(payload, dict):
        reasons.append("unredacted_payload:missing_payload_policy")
        payload = {}
    if payload.get("redaction_status") != "redacted":
        reasons.append(f"unredacted_payload:redaction:{payload.get('redaction_status')}")
    if payload.get("raw_secret_persisted") is not False:
        reasons.append("unredacted_payload:raw_secret_persisted")
    if payload.get("raw_exchange_response_persisted") is not False:
        reasons.append("unredacted_payload:raw_exchange_response_persisted")
    unredacted_count = as_int(payload.get("unredacted_payload_count"), "payload_policy.unredacted_payload_count")
    if unredacted_count is None or unredacted_count != 0:
        reasons.append(f"unredacted_payload:count:{unredacted_count}")

    audit = artifact.get("audit_lineage")
    if not isinstance(audit, dict):
        reasons.append("missing_provenance:missing_audit_lineage")
        audit = {}
    if audit.get("lineage_status") != "linked":
        reasons.append(f"missing_provenance:audit_lineage:{audit.get('lineage_status')}")
    for key in ("audit_sink_ref", "transition_ledger_ref", "source_event_hash", "store_record_hash"):
        if not non_empty(audit.get(key)):
            reasons.append(f"missing_provenance:audit:{key}")

    boundary = artifact.get("boundary_flags")
    if not isinstance(boundary, dict):
        reasons.append("forbidden_operation:missing_boundary_flags")
        boundary = {}
    for key in BOUNDARY_FALSE_FLAGS:
        if key not in boundary:
            reasons.append(f"forbidden_operation:missing_required_false:{key}")
        elif boundary.get(key) is not False:
            reasons.append(f"forbidden_operation:boundary:{key}")

    transitions = artifact.get("slo_status_transitions")
    if not isinstance(transitions, list) or not transitions:
        fail("slo_status_transitions must be a non-empty list")
    if [transition.get("to_status") for transition in transitions if isinstance(transition, dict)] != EXPECTED_TRANSITIONS:
        reasons.append("missing_provenance:transition_sequence_mismatch")
    previous_to_status: str | None = None
    for transition in transitions:
        if not isinstance(transition, dict):
            fail("slo status transition must be an object")
        transition_id = str(transition.get("transition_id") or "unknown")
        if transition.get("transition_auditable") is not True:
            reasons.append(f"missing_provenance:transition_not_auditable:{transition_id}")
        if not non_empty(transition.get("from_status")) or not non_empty(transition.get("to_status")):
            reasons.append(f"missing_provenance:transition_missing_status:{transition_id}")
        if previous_to_status is not None and transition.get("from_status") != previous_to_status:
            reasons.append(f"missing_provenance:transition_chain_mismatch:{transition_id}")
        previous_to_status = str(transition.get("to_status"))
        for key in ("remediation_triggered", "retry_scheduled", "adapter_send_requested", "live_exchange_request_requested", "trading_operation_triggered"):
            if transition.get(key) is not False:
                reasons.append(f"forbidden_operation:transition:{transition_id}:{key}")
        if transition.get("operation_effect") != "observability_only":
            reasons.append(f"forbidden_operation:effect:{transition_id}:{transition.get('operation_effect')}")

    status = classify_status(reasons)
    return {
        "status": status,
        "fail_closed": status.startswith("fail_closed"),
        "degraded": status.startswith("degraded"),
        "blocking_reasons": reasons,
    }


artifact = json.loads(artifact_path.read_text(encoding="utf-8"))
matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
module = next((item for item in matrix.get("module_readiness", []) if item.get("module_id") == "telemetry_slo_ingestion_runtime_closure"), None)
if not module:
    fail("matrix missing telemetry_slo_ingestion_runtime_closure")
if module.get("classification") != "runtime-closed":
    fail("telemetry SLO matrix entry must be runtime-closed")
if module.get("evidence_path") != "docs/rust-cutover/evidence/V280-005.md":
    fail("telemetry SLO matrix evidence path mismatch")
if module.get("verification_command") != "scripts/ai/verify_release.sh v28-telemetry-slo-ingestion-runtime-closure":
    fail("telemetry SLO matrix verification command mismatch")

cases = artifact.get("telemetry_replay_cases")
if not isinstance(cases, list) or [case.get("case_id") for case in cases] != EXPECTED_CASES:
    fail("telemetry replay cases mismatch")
allowed = 0
degraded = 0
fail_closed = 0
for case in cases:
    actual = classify_artifact(apply_case(artifact, case))
    if actual["status"] != case.get("expected_status"):
        fail(f"{case.get('case_id')}: expected {case.get('expected_status')} got {actual}")
    if actual["fail_closed"]:
        fail_closed += 1
    elif actual["degraded"]:
        degraded += 1
    else:
        allowed += 1
if allowed != 1 or degraded != 1 or fail_closed != 5:
    fail(f"unexpected case counts: allowed={allowed} degraded={degraded} fail_closed={fail_closed}")

if selftest:
    opened = copy.deepcopy(artifact)
    opened["boundary_flags"]["telemetry_event_triggers_remediation"] = True
    if classify_artifact(opened)["status"] != "fail_closed_forbidden_operation_trigger":
        fail("negative self-test unexpectedly allowed telemetry_event_triggers_remediation")

print(
    "v28_telemetry_slo_ingestion_runtime_closure=pass "
    f"cases={len(cases)} allowed={allowed} degraded={degraded} fail_closed={fail_closed} "
    f"transitions={len(artifact['slo_status_transitions'])} "
    f"boundary_flags={len(BOUNDARY_FALSE_FLAGS)} negative_selftest={int(selftest)}"
)
PY
