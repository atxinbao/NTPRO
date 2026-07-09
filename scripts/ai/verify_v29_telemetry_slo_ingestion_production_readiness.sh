#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

ARTIFACT_PATH="${NTPRO_V290_TELEMETRY_SLO_ARTIFACT:-docs/rust-cutover/release/v0_29_0_telemetry_slo_ingestion_production_readiness_artifact.json}"
CONTRACT_PATH="${NTPRO_V290_TELEMETRY_SLO_CONTRACT:-docs/rust-cutover/release/v0_29_0_telemetry_slo_ingestion_production_readiness.md}"
TASK_PATH="${NTPRO_V290_TELEMETRY_SLO_TASK:-docs/rust-cutover/tasks/V290-003.md}"
EVIDENCE_PATH="${NTPRO_V290_TELEMETRY_SLO_EVIDENCE:-docs/rust-cutover/evidence/V290-003.md}"
MATRIX_PATH="${NTPRO_V290_TELEMETRY_SLO_MATRIX:-docs/rust-cutover/release/v0_29_0_backend_production_readiness_matrix.json}"
BOUNDARY_CONTRACT_PATH="${NTPRO_V290_TELEMETRY_SLO_BOUNDARY_CONTRACT:-docs/rust-cutover/release/v0_29_0_backend_production_readiness_boundary_contract.md}"
V280_ARTIFACT_PATH="${NTPRO_V290_TELEMETRY_SLO_V280_ARTIFACT:-docs/rust-cutover/release/v0_28_0_telemetry_slo_ingestion_runtime_artifact.json}"
AUDIT_ARTIFACT_PATH="${NTPRO_V290_TELEMETRY_SLO_AUDIT_ARTIFACT:-docs/rust-cutover/release/v0_29_0_persistent_audit_storage_production_readiness_artifact.json}"
INTAKE_PATH="${NTPRO_V290_TELEMETRY_SLO_INTAKE:-docs/rust-cutover/release/v0_29_0_intake_gate.md}"
SELFTEST="${NTPRO_V290_TELEMETRY_SLO_SELFTEST:-1}"

fail() {
  echo "v29 telemetry SLO ingestion production readiness failed: $*" >&2
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

for path in "$ARTIFACT_PATH" "$CONTRACT_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$MATRIX_PATH" "$BOUNDARY_CONTRACT_PATH" "$V280_ARTIFACT_PATH" "$AUDIT_ARTIFACT_PATH" "$INTAKE_PATH"; do
  require_file "$path"
done

require_contains "$TASK_PATH" "GitHub issue: \`#929\`"
require_contains "$EVIDENCE_PATH" "Task: \`V290-003\` / GitHub issue \`#929\`"
require_contains "$INTAKE_PATH" "start_gate_status = satisfied"
require_contains "$BOUNDARY_CONTRACT_PATH" "contract_version = ntpro.v290.backend_production_readiness_boundary.v1"
require_contains "$CONTRACT_PATH" "contract_version = ntpro.v290.telemetry_slo_ingestion_production_readiness.v1"
require_contains "$CONTRACT_PATH" "schema_version = ntpro.v290.telemetry_slo_ingestion_production_readiness_artifact.v1"
require_contains "$CONTRACT_PATH" "backend_module = telemetry_slo_ingestion_production_readiness"
require_contains "$CONTRACT_PATH" "backend_module_status = production_ready_evidence"
require_contains "$CONTRACT_PATH" "telemetry_observability_only = true"
require_contains "$CONTRACT_PATH" "production_telemetry_transport_required = false"
require_contains "$CONTRACT_PATH" "external_observability_backend_required = false"
require_contains "$CONTRACT_PATH" "release stage = scripts/ai/verify_release.sh v29-telemetry-slo-ingestion-production-readiness"
require_contains "$CONTRACT_PATH" "stale_telemetry => fail_closed_stale_telemetry"
require_contains "$CONTRACT_PATH" "malformed_metrics => fail_closed_malformed_metrics"
require_contains "$CONTRACT_PATH" "forbidden_operation_trigger => fail_closed_forbidden_operation_trigger"

ARTIFACT_PATH="$ARTIFACT_PATH" MATRIX_PATH="$MATRIX_PATH" V280_ARTIFACT_PATH="$V280_ARTIFACT_PATH" AUDIT_ARTIFACT_PATH="$AUDIT_ARTIFACT_PATH" SELFTEST="$SELFTEST" python3 <<'PY'
from __future__ import annotations

import copy
import json
import os
from pathlib import Path
from typing import Any

artifact_path = Path(os.environ["ARTIFACT_PATH"])
matrix_path = Path(os.environ["MATRIX_PATH"])
v280_artifact_path = Path(os.environ["V280_ARTIFACT_PATH"])
audit_artifact_path = Path(os.environ["AUDIT_ARTIFACT_PATH"])
selftest = os.environ.get("SELFTEST", "1") != "0"

SCHEMA_VERSION = "ntpro.v290.telemetry_slo_ingestion_production_readiness_artifact.v1"
CONTRACT_VERSION = "ntpro.v290.telemetry_slo_ingestion_production_readiness.v1"
RELEASE_SCOPE = "backend_production_readiness_foundation_only"
MODULE_ID = "telemetry_slo_ingestion_production_readiness"
DEPENDENCIES = {"V290-000", "V290-001", "V290-002", "V280-005", "v0.28.1-release-evidence"}
EXPECTED_CASES = [
    "telemetry_slo_ingestion.production_readiness.healthy.allowed.001",
    "telemetry_slo_ingestion.production_readiness.slo_breach.degraded.001",
    "telemetry_slo_ingestion.production_readiness.stale_telemetry.fail_closed.001",
    "telemetry_slo_ingestion.production_readiness.missing_provenance.fail_closed.001",
    "telemetry_slo_ingestion.production_readiness.malformed_metrics.fail_closed.001",
    "telemetry_slo_ingestion.production_readiness.unredacted_payload.fail_closed.001",
    "telemetry_slo_ingestion.production_readiness.forbidden_operation.fail_closed.001",
]
BOUNDARY_FALSE_FLAGS = [
    "new_submit_capability",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "cancel_order_allowed",
    "replace_order_allowed",
    "amend_order_allowed",
    "flatten_position_allowed",
    "execution_adapter_call_allowed",
    "adapter_send_allowed",
    "live_exchange_request_allowed",
    "network_attempted",
    "retry_scheduler_enabled",
    "automatic_remediation_allowed",
    "automatic_operation_action_allowed",
    "dashboard_operation_controls_enabled",
    "dashboard_trading_controls_enabled",
    "admin_workbench_operation_controls_enabled",
    "admin_workbench_trading_controls_enabled",
    "trader_terminal_order_ticket_enabled",
    "manual_operation_submit_allowed",
    "backend_go_live_claim",
    "product_grade_trading_terminal_claim",
    "telemetry_event_triggers_remediation",
    "telemetry_event_triggers_retry",
    "telemetry_event_triggers_adapter_send",
    "telemetry_event_triggers_trading_control",
]


def fail(message: str) -> None:
    raise SystemExit(f"v29 telemetry SLO ingestion production readiness failed: {message}")


def non_empty(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


def merge(base: dict[str, Any], override: dict[str, Any] | None) -> dict[str, Any]:
    merged = copy.deepcopy(base)
    if override:
        for key, value in override.items():
            merged[key] = value
    return merged


def as_int(value: Any) -> int | None:
    if isinstance(value, bool):
        return None
    try:
        return int(value)
    except (TypeError, ValueError):
        return None


def as_float(value: Any) -> float | None:
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
        return "fail_closed_telemetry_slo_readiness_violation"
    return "telemetry_slo_production_readiness_ready"


def apply_case(artifact: dict[str, Any], case: dict[str, Any]) -> dict[str, Any]:
    result = copy.deepcopy(artifact)
    result["telemetry_source"] = merge(result["telemetry_source"], case.get("telemetry_source_override"))
    result["sampling_policy"] = merge(result["sampling_policy"], case.get("sampling_policy_override"))
    result["slo_rollup"] = merge(result["slo_rollup"], case.get("slo_rollup_override"))
    result["redaction_policy"] = merge(result["redaction_policy"], case.get("redaction_policy_override"))
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
    if artifact.get("task_id") != "V290-003" or artifact.get("github_issue") != 929:
        reasons.append("missing_provenance:task_identity_mismatch")
    if artifact.get("release_scope") != RELEASE_SCOPE:
        reasons.append("missing_provenance:release_scope_mismatch")
    if artifact.get("backend_module") != MODULE_ID or artifact.get("backend_module_status") != "production_ready_evidence":
        reasons.append("missing_provenance:backend_module_mismatch")
    if artifact.get("readiness_mode") != "deterministic_readiness_replay":
        reasons.append("missing_provenance:readiness_mode_mismatch")
    if set(artifact.get("dependency_contracts") or []) != DEPENDENCIES:
        reasons.append("missing_provenance:dependency_contracts_mismatch")
    if artifact.get("ingestion_backend_claim") != "source_controlled_sandbox_fixture":
        reasons.append("missing_provenance:ingestion_backend_claim")
    if artifact.get("telemetry_observability_only") is not True:
        reasons.append("forbidden_operation:telemetry_observability_only")

    source = artifact.get("telemetry_source")
    if not isinstance(source, dict):
        reasons.append("missing_provenance:telemetry_source")
        source = {}
    for key in ("source_id", "source_type", "source_ref", "source_digest", "pipeline_id", "sampling_profile_id", "environment", "schema_name", "schema_version"):
        if not non_empty(source.get(key)):
            reasons.append(f"missing_provenance:source:{key}")
    if source.get("source_ref") != str(v280_artifact_path):
        reasons.append("missing_provenance:source_ref")
    if source.get("source_type") != "production_readiness_sandbox_telemetry_fixture":
        reasons.append(f"missing_provenance:source_type:{source.get('source_type')}")
    if source.get("environment") != "production_readiness_sandbox":
        reasons.append(f"missing_provenance:environment:{source.get('environment')}")
    if source.get("provenance_status") != "linked":
        reasons.append(f"missing_provenance:source_lineage:{source.get('provenance_status')}")
    observed_age = as_int(source.get("observed_age_ms"))
    max_age = as_int(source.get("max_age_ms"))
    if observed_age is None or max_age is None:
        reasons.append("malformed_metrics:source_age_not_integer")
    elif observed_age > max_age:
        reasons.append(f"stale_telemetry:age:{observed_age}>{max_age}")
    if source.get("freshness_status") != "fresh":
        reasons.append(f"stale_telemetry:freshness:{source.get('freshness_status')}")

    sampling = artifact.get("sampling_policy")
    if not isinstance(sampling, dict):
        reasons.append("malformed_metrics:missing_sampling_policy")
        sampling = {}
    numeric: dict[str, int | None] = {}
    for key in ("duration_minutes", "min_duration_minutes", "sample_count", "expected_sample_count", "gap_count", "max_gap_ms", "allowed_gap_ms", "malformed_sample_count"):
        numeric[key] = as_int(sampling.get(key))
        if numeric[key] is None:
            reasons.append(f"malformed_metrics:not_integer:{key}")
    if numeric.get("duration_minutes") is not None and numeric.get("min_duration_minutes") is not None and numeric["duration_minutes"] < numeric["min_duration_minutes"]:
        reasons.append("malformed_metrics:duration_below_minimum")
    if numeric.get("sample_count") is not None and numeric.get("expected_sample_count") is not None and numeric["sample_count"] < numeric["expected_sample_count"]:
        reasons.append("malformed_metrics:sample_count_below_expected")
    if numeric.get("gap_count") not in (0, None):
        reasons.append(f"malformed_metrics:gap_count:{numeric.get('gap_count')}")
    if numeric.get("max_gap_ms") is not None and numeric.get("allowed_gap_ms") is not None and numeric["max_gap_ms"] > numeric["allowed_gap_ms"]:
        reasons.append("malformed_metrics:max_gap_exceeded")
    if numeric.get("malformed_sample_count") not in (0, None):
        reasons.append(f"malformed_metrics:malformed_sample_count:{numeric.get('malformed_sample_count')}")
    if sampling.get("sampling_strategy") != "fixed_interval_fixture" or sampling.get("sampling_drift_status") != "aligned":
        reasons.append("malformed_metrics:sampling_policy_mismatch")

    retention = artifact.get("retention_policy")
    if not isinstance(retention, dict):
        reasons.append("missing_provenance:retention_policy")
        retention = {}
    if not non_empty(retention.get("retention_policy_id")) or retention.get("mode") != "immutable_until_expiry":
        reasons.append("missing_provenance:retention_policy")
    if not isinstance(retention.get("min_days"), int) or retention.get("min_days") < 365:
        reasons.append("missing_provenance:retention_min_days")
    if retention.get("delete_before_retention_allowed") is not False:
        reasons.append("forbidden_operation:delete_before_retention")

    redaction = artifact.get("redaction_policy")
    if not isinstance(redaction, dict):
        reasons.append("unredacted_payload:missing_redaction_policy")
        redaction = {}
    if redaction.get("redaction_status") != "redacted":
        reasons.append(f"unredacted_payload:redaction:{redaction.get('redaction_status')}")
    for key in ("raw_secret_persisted", "raw_exchange_response_persisted", "raw_account_identifier_persisted"):
        if redaction.get(key) is not False:
            reasons.append(f"unredacted_payload:{key}")
    unredacted_count = as_int(redaction.get("unredacted_payload_count"))
    if unredacted_count is None or unredacted_count != 0:
        reasons.append(f"unredacted_payload:count:{unredacted_count}")

    slo = artifact.get("slo_rollup")
    if not isinstance(slo, dict):
        reasons.append("missing_provenance:missing_slo_rollup")
        slo = {}
    for key in ("slo_id", "status", "breach_semantics"):
        if not non_empty(slo.get(key)):
            reasons.append(f"missing_provenance:slo:{key}")
    availability = as_float(slo.get("observed_availability"))
    target = as_float(slo.get("availability_target"))
    error_budget = as_float(slo.get("error_budget_remaining"))
    breach_count = as_int(slo.get("breach_count"))
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
    if slo.get("breach_semantics") != "degraded_observability_only":
        reasons.append("forbidden_operation:slo_breach_semantics")

    alert = artifact.get("alert_handoff_policy")
    if not isinstance(alert, dict):
        reasons.append("missing_provenance:missing_alert_handoff_policy")
        alert = {}
    if alert.get("handoff_mode") != "audit_only_manual_review" or alert.get("alert_lineage_status") != "linked":
        reasons.append("missing_provenance:alert_handoff")
    for key in ("automatic_remediation_allowed", "retry_scheduler_enabled", "adapter_send_allowed", "live_exchange_request_allowed", "trading_control_allowed"):
        if alert.get(key) is not False:
            reasons.append(f"forbidden_operation:alert_handoff:{key}")

    audit = artifact.get("audit_lineage")
    if not isinstance(audit, dict):
        reasons.append("missing_provenance:audit_lineage")
        audit = {}
    if audit.get("lineage_status") != "linked":
        reasons.append(f"missing_provenance:audit_lineage_status:{audit.get('lineage_status')}")
    if audit.get("audit_sink_ref") != str(audit_artifact_path):
        reasons.append("missing_provenance:audit_sink_ref")
    for key in ("transition_ledger_ref", "source_event_hash", "store_record_hash"):
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
            reasons.append(f"forbidden_operation:boundary_flag:{key}")

    transitions = artifact.get("slo_status_transitions")
    if not isinstance(transitions, list) or len(transitions) != 4:
        fail("slo_status_transitions must contain 4 transitions")
    for transition in transitions:
        tid = transition.get("transition_id")
        if not non_empty(tid):
            reasons.append("missing_provenance:transition_id")
        if transition.get("transition_auditable") is not True:
            reasons.append(f"missing_provenance:transition_not_auditable:{tid}")
        if transition.get("alert_handoff") != "audit_only_manual_review":
            reasons.append(f"missing_provenance:transition_alert_handoff:{tid}")
        if transition.get("operation_effect") != "observability_only":
            reasons.append(f"forbidden_operation:transition_effect:{tid}:{transition.get('operation_effect')}")
        for key in ("remediation_triggered", "retry_scheduled", "adapter_send_requested", "live_exchange_request_requested", "trading_operation_triggered"):
            if transition.get(key) is not False:
                reasons.append(f"forbidden_operation:transition:{tid}:{key}")

    return {"status": classify_status(reasons), "fail_closed": classify_status(reasons).startswith("fail_closed"), "degraded": classify_status(reasons) == "degraded_slo_breach", "blocking_reasons": reasons}


artifact = json.loads(artifact_path.read_text(encoding="utf-8"))
v280_artifact = json.loads(v280_artifact_path.read_text(encoding="utf-8"))
if v280_artifact.get("schema_version") != "ntpro.v280.telemetry_slo_ingestion_runtime_artifact.v1":
    fail("v28 telemetry SLO artifact schema mismatch")
audit_artifact = json.loads(audit_artifact_path.read_text(encoding="utf-8"))
if audit_artifact.get("schema_version") != "ntpro.v290.persistent_audit_storage_production_readiness_artifact.v1":
    fail("v29 persistent audit artifact schema mismatch")

matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
module = next((item for item in matrix.get("module_readiness", []) if item.get("module_id") == MODULE_ID), None)
if not module:
    fail("matrix missing telemetry_slo_ingestion_production_readiness")
if module.get("classification") != "production-ready":
    fail("telemetry SLO matrix entry must be production-ready")
if module.get("readiness_mode") != "deterministic_readiness_replay":
    fail("telemetry SLO matrix readiness mode mismatch")
if module.get("issue") != 929:
    fail("telemetry SLO matrix issue mismatch")
if module.get("evidence_path") != "docs/rust-cutover/evidence/V290-003.md":
    fail("telemetry SLO matrix evidence path mismatch")
if module.get("verification_command") != "scripts/ai/verify_release.sh v29-telemetry-slo-ingestion-production-readiness":
    fail("telemetry SLO matrix verification command mismatch")
if module.get("production_ready_claim_allowed") is not True:
    fail("telemetry SLO production-ready claim flag mismatch")

cases = artifact.get("readiness_cases")
if not isinstance(cases, list) or [case.get("case_id") for case in cases] != EXPECTED_CASES:
    fail("readiness cases mismatch")
allowed = degraded = fail_closed = 0
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
    opened["boundary_flags"]["adapter_send_allowed"] = True
    if classify_artifact(opened)["status"] != "fail_closed_forbidden_operation_trigger":
        fail("negative self-test unexpectedly allowed adapter_send_allowed")

    stale = copy.deepcopy(artifact)
    stale["telemetry_source"]["observed_age_ms"] = 120000
    if classify_artifact(stale)["status"] != "fail_closed_stale_telemetry":
        fail("negative self-test unexpectedly allowed stale telemetry")

print(
    "v29_telemetry_slo_ingestion_production_readiness=pass "
    f"cases={len(cases)} "
    f"allowed={allowed} "
    f"degraded={degraded} "
    f"fail_closed={fail_closed} "
    f"transitions={len(artifact['slo_status_transitions'])} "
    f"boundary_flags={len(BOUNDARY_FALSE_FLAGS)} "
    "negative_selftest=1"
)
PY
