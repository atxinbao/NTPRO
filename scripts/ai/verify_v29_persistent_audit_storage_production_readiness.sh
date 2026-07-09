#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

ARTIFACT_PATH="${NTPRO_V290_AUDIT_STORAGE_ARTIFACT:-docs/rust-cutover/release/v0_29_0_persistent_audit_storage_production_readiness_artifact.json}"
CONTRACT_PATH="${NTPRO_V290_AUDIT_STORAGE_CONTRACT:-docs/rust-cutover/release/v0_29_0_persistent_audit_storage_production_readiness.md}"
TASK_PATH="${NTPRO_V290_AUDIT_STORAGE_TASK:-docs/rust-cutover/tasks/V290-002.md}"
EVIDENCE_PATH="${NTPRO_V290_AUDIT_STORAGE_EVIDENCE:-docs/rust-cutover/evidence/V290-002.md}"
MATRIX_PATH="${NTPRO_V290_AUDIT_STORAGE_MATRIX:-docs/rust-cutover/release/v0_29_0_backend_production_readiness_matrix.json}"
BOUNDARY_CONTRACT_PATH="${NTPRO_V290_AUDIT_STORAGE_BOUNDARY_CONTRACT:-docs/rust-cutover/release/v0_29_0_backend_production_readiness_boundary_contract.md}"
V280_ARTIFACT_PATH="${NTPRO_V290_AUDIT_STORAGE_V280_ARTIFACT:-docs/rust-cutover/release/v0_28_0_persistent_audit_storage_runtime_artifact.json}"
INTAKE_PATH="${NTPRO_V290_AUDIT_STORAGE_INTAKE:-docs/rust-cutover/release/v0_29_0_intake_gate.md}"
SELFTEST="${NTPRO_V290_AUDIT_STORAGE_SELFTEST:-1}"

fail() {
  echo "v29 persistent audit storage production readiness failed: $*" >&2
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

for path in "$ARTIFACT_PATH" "$CONTRACT_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$MATRIX_PATH" "$BOUNDARY_CONTRACT_PATH" "$V280_ARTIFACT_PATH" "$INTAKE_PATH"; do
  require_file "$path"
done

require_contains "$TASK_PATH" "GitHub issue: \`#928\`"
require_contains "$EVIDENCE_PATH" "Task: \`V290-002\` / GitHub issue \`#928\`"
require_contains "$INTAKE_PATH" "start_gate_status = satisfied"
require_contains "$BOUNDARY_CONTRACT_PATH" "contract_version = ntpro.v290.backend_production_readiness_boundary.v1"
require_contains "$CONTRACT_PATH" "contract_version = ntpro.v290.persistent_audit_storage_production_readiness.v1"
require_contains "$CONTRACT_PATH" "schema_version = ntpro.v290.persistent_audit_storage_production_readiness_artifact.v1"
require_contains "$CONTRACT_PATH" "backend_module = persistent_audit_storage_production_readiness"
require_contains "$CONTRACT_PATH" "backend_module_status = production_ready_evidence"
require_contains "$CONTRACT_PATH" "production_storage_mutation_required = false"
require_contains "$CONTRACT_PATH" "external_network_required = false"
require_contains "$CONTRACT_PATH" "release stage = scripts/ai/verify_release.sh v29-persistent-audit-storage-production-readiness"
require_contains "$CONTRACT_PATH" "missing_storage_config => fail_closed_missing_storage_config"
require_contains "$CONTRACT_PATH" "schema_drift => fail_closed_schema_drift"
require_contains "$CONTRACT_PATH" "forbidden_operation_or_control => fail_closed_forbidden_operation_boundary"

for marker in \
  "new_submit_capability = false" \
  "production_order_submission_allowed = false" \
  "production_order_mutation_allowed = false" \
  "cancel_order_allowed = false" \
  "replace_order_allowed = false" \
  "amend_order_allowed = false" \
  "flatten_position_allowed = false" \
  "execution_adapter_call_allowed = false" \
  "adapter_send_allowed = false" \
  "live_exchange_request_allowed = false" \
  "network_attempted = false" \
  "retry_scheduler_enabled = false" \
  "automatic_remediation_allowed = false" \
  "automatic_operation_action_allowed = false" \
  "dashboard_operation_controls_enabled = false" \
  "dashboard_trading_controls_enabled = false" \
  "admin_workbench_operation_controls_enabled = false" \
  "admin_workbench_trading_controls_enabled = false" \
  "trader_terminal_order_ticket_enabled = false" \
  "manual_operation_submit_allowed = false" \
  "backend_go_live_claim = false" \
  "product_grade_trading_terminal_claim = false"; do
  require_contains "$CONTRACT_PATH" "$marker"
done

ARTIFACT_PATH="$ARTIFACT_PATH" MATRIX_PATH="$MATRIX_PATH" V280_ARTIFACT_PATH="$V280_ARTIFACT_PATH" SELFTEST="$SELFTEST" python3 <<'PY'
from __future__ import annotations

import copy
import json
import os
from pathlib import Path
from typing import Any

artifact_path = Path(os.environ["ARTIFACT_PATH"])
matrix_path = Path(os.environ["MATRIX_PATH"])
v280_artifact_path = Path(os.environ["V280_ARTIFACT_PATH"])
selftest = os.environ.get("SELFTEST", "1") != "0"

SCHEMA_VERSION = "ntpro.v290.persistent_audit_storage_production_readiness_artifact.v1"
CONTRACT_VERSION = "ntpro.v290.persistent_audit_storage_production_readiness.v1"
RELEASE_SCOPE = "backend_production_readiness_foundation_only"
MODULE_ID = "persistent_audit_storage_production_readiness"
CURRENT_AUDIT_SCHEMA = "ntpro.audit.storage.v29.production_readiness.v1"
DEPENDENCIES = {"V290-000", "V290-001", "V280-003", "v0.28.1-release-evidence"}
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
]
FORBIDDEN_EFFECTS = {
    "executed",
    "sent_to_adapter",
    "sent_to_exchange",
    "retry_scheduled",
    "automatic_remediation_triggered",
    "dashboard_control_rendered",
    "trader_terminal_order_ticket_rendered",
}
EXPECTED_CASES = [
    "persistent_audit_storage.production_readiness.write_read.allowed.001",
    "persistent_audit_storage.production_readiness.missing_config.fail_closed.001",
    "persistent_audit_storage.production_readiness.stale_source.fail_closed.001",
    "persistent_audit_storage.production_readiness.schema_drift.fail_closed.001",
    "persistent_audit_storage.production_readiness.broken_lineage.fail_closed.001",
    "persistent_audit_storage.production_readiness.unredacted_payload.fail_closed.001",
    "persistent_audit_storage.production_readiness.forbidden_operation.fail_closed.001",
]


def fail(message: str) -> None:
    raise SystemExit(f"v29 persistent audit storage production readiness failed: {message}")


def non_empty(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


def merge(base: dict[str, Any], override: dict[str, Any] | None) -> dict[str, Any]:
    merged = copy.deepcopy(base)
    if override:
        for key, value in override.items():
            merged[key] = value
    return merged


def expected_hash(sequence: int, source_event_hash: str, payload_digest: str) -> str:
    return f"ntpro-audit-prod-ready:{sequence:06d}:{source_event_hash}:{payload_digest}"


def classify_status(reasons: list[str]) -> str:
    if any(reason.startswith("missing_storage_config") for reason in reasons):
        return "fail_closed_missing_storage_config"
    if any(reason.startswith("stale_audit_source") for reason in reasons):
        return "fail_closed_stale_audit_source"
    if any(reason.startswith("schema_drift") for reason in reasons):
        return "fail_closed_schema_drift"
    if any(reason.startswith("broken_lineage") or reason.startswith("sequence") for reason in reasons):
        return "fail_closed_broken_lineage"
    if any(reason.startswith("unredacted_payload") for reason in reasons):
        return "fail_closed_unredacted_payload"
    if any(reason.startswith("forbidden") for reason in reasons):
        return "fail_closed_forbidden_operation_boundary"
    if reasons:
        return "fail_closed_audit_storage_readiness_violation"
    return "audit_storage_production_readiness_ready"


def apply_case(artifact: dict[str, Any], case: dict[str, Any]) -> dict[str, Any]:
    result = copy.deepcopy(artifact)
    result["storage_config"] = merge(result["storage_config"], case.get("storage_config_override"))
    result["schema_migration_policy"] = merge(result["schema_migration_policy"], case.get("schema_migration_policy_override"))
    result["redaction_policy"] = merge(result["redaction_policy"], case.get("redaction_policy_override"))
    result["boundary_flags"] = merge(result["boundary_flags"], case.get("boundary_flags_override"))
    overrides = case.get("record_overrides") or {}
    for record in result["readiness_records"]:
        override = overrides.get(record["record_id"])
        if override:
            record.update(override)
    return result


def classify_artifact(artifact: dict[str, Any]) -> dict[str, Any]:
    reasons: list[str] = []
    if artifact.get("schema_version") != SCHEMA_VERSION:
        reasons.append("schema_version_mismatch")
    if artifact.get("contract_version") != CONTRACT_VERSION:
        reasons.append("contract_version_mismatch")
    if artifact.get("task_id") != "V290-002" or artifact.get("github_issue") != 928:
        reasons.append("task_identity_mismatch")
    if artifact.get("release_scope") != RELEASE_SCOPE:
        reasons.append("release_scope_mismatch")
    if artifact.get("backend_module") != MODULE_ID:
        reasons.append("backend_module_mismatch")
    if artifact.get("backend_module_status") != "production_ready_evidence":
        reasons.append("backend_module_status_mismatch")
    if artifact.get("readiness_mode") != "deterministic_readiness_replay":
        reasons.append("readiness_mode_mismatch")
    if set(artifact.get("dependency_contracts") or []) != DEPENDENCIES:
        reasons.append("dependency_contracts_mismatch")

    config = artifact.get("storage_config")
    if not isinstance(config, dict):
        reasons.append("missing_storage_config")
        config = {}
    for key in ("config_id", "config_source", "environment", "backend_class", "storage_namespace", "segment_prefix", "kms_key_ref", "config_digest", "mutation_scope"):
        if not non_empty(config.get(key)):
            reasons.append(f"missing_storage_config:{key}")
    if config.get("environment") != "production_readiness_sandbox":
        reasons.append(f"missing_storage_config:environment:{config.get('environment')}")
    if config.get("mutation_scope") != "non_production_fixture_only":
        reasons.append(f"missing_storage_config:mutation_scope:{config.get('mutation_scope')}")
    for key in ("append_only", "immutable_segments"):
        if config.get(key) is not True:
            reasons.append(f"missing_storage_config:{key}")
    for key in ("mutable_updates_allowed", "delete_before_retention_allowed", "production_storage_mutation_required", "external_network_required"):
        if config.get(key) is not False:
            reasons.append(f"forbidden:storage_config:{key}")
    if config.get("config_status") != "fresh" or config.get("source_freshness_status") != "fresh":
        reasons.append("stale_audit_source:storage_config")

    schema = artifact.get("schema_migration_policy")
    if not isinstance(schema, dict):
        reasons.append("schema_drift:missing_schema_policy")
        schema = {}
    if schema.get("current_schema") != CURRENT_AUDIT_SCHEMA:
        reasons.append("schema_drift:current_schema")
    if schema.get("previous_schema") != "ntpro.v280.persistent_audit_storage_runtime_artifact.v1":
        reasons.append("schema_drift:previous_schema")
    if schema.get("migration_policy") != "forward_only":
        reasons.append("schema_drift:migration_policy")
    if schema.get("destructive_migration_allowed") is not False:
        reasons.append("schema_drift:destructive_migration_allowed")
    if schema.get("schema_drift_status") != "aligned":
        reasons.append(f"schema_drift:{schema.get('schema_drift_status')}")
    if not non_empty(schema.get("migration_digest")):
        reasons.append("schema_drift:migration_digest")

    retention = artifact.get("retention_policy")
    if not isinstance(retention, dict):
        reasons.append("missing_storage_config:retention_policy")
        retention = {}
    if not non_empty(retention.get("retention_policy_id")):
        reasons.append("missing_storage_config:retention_policy_id")
    if retention.get("mode") != "immutable_until_expiry":
        reasons.append("missing_storage_config:retention_mode")
    if not isinstance(retention.get("min_days"), int) or retention.get("min_days") < 365:
        reasons.append("missing_storage_config:retention_min_days")
    if retention.get("delete_before_retention_allowed") is not False:
        reasons.append("forbidden:delete_before_retention_allowed")

    redaction = artifact.get("redaction_policy")
    if not isinstance(redaction, dict):
        reasons.append("unredacted_payload:missing_redaction_policy")
        redaction = {}
    if redaction.get("redaction_status") != "redacted":
        reasons.append("unredacted_payload:redaction_status")
    for key in ("raw_secret_material_allowed", "unredacted_payload_allowed"):
        if redaction.get(key) is not False:
            reasons.append(f"unredacted_payload:{key}")
    fields = redaction.get("required_redaction_fields")
    if not isinstance(fields, list) or not {"api_key", "secret", "account_id", "order_payload", "exchange_response"}.issubset(set(fields)):
        reasons.append("unredacted_payload:required_redaction_fields")

    idempotency = artifact.get("idempotency_policy")
    if not isinstance(idempotency, dict):
        reasons.append("broken_lineage:missing_idempotency_policy")
        idempotency = {}
    if idempotency.get("idempotency_required") is not True or not non_empty(idempotency.get("key_strategy")):
        reasons.append("broken_lineage:idempotency_policy")

    lineage_policy = artifact.get("lineage_policy")
    if not isinstance(lineage_policy, dict):
        reasons.append("broken_lineage:missing_lineage_policy")
        lineage_policy = {}
    if lineage_policy.get("source_chain_required") is not True:
        reasons.append("broken_lineage:source_chain_required")
    if lineage_policy.get("source_artifact") != str(v280_artifact_path):
        reasons.append("broken_lineage:source_artifact")
    if lineage_policy.get("storage_lineage_status") != "linked" or lineage_policy.get("config_lineage_status") != "linked":
        reasons.append("broken_lineage:lineage_policy")
    if lineage_policy.get("store_source_drift_allowed") is not False:
        reasons.append("broken_lineage:store_source_drift_allowed")

    boundary = artifact.get("boundary_flags")
    if not isinstance(boundary, dict):
        reasons.append("forbidden:missing_boundary_flags")
        boundary = {}
    for key in BOUNDARY_FALSE_FLAGS:
        if key not in boundary:
            reasons.append(f"forbidden:missing_required_false:{key}")
        elif boundary.get(key) is not False:
            reasons.append(f"forbidden:boundary_flag:{key}")

    records = artifact.get("readiness_records")
    if not isinstance(records, list) or not records:
        fail("readiness_records must be a non-empty list")
    previous_hash = "GENESIS"
    expected_sequence = 1
    config_digest = config.get("config_digest")
    retention_policy_id = retention.get("retention_policy_id")
    for record in records:
        sequence = record.get("sequence")
        if sequence != expected_sequence:
            reasons.append(f"sequence_gap:{expected_sequence}:{sequence}")
        if record.get("schema_version") != CURRENT_AUDIT_SCHEMA:
            reasons.append(f"schema_drift:record:{sequence}")
        if record.get("config_digest") != config_digest:
            reasons.append(f"missing_storage_config:record_config_digest:{sequence}")
        if record.get("previous_store_hash") != previous_hash:
            reasons.append(f"broken_lineage:previous_hash:{sequence}")
        source_hash = record.get("source_event_hash")
        payload_digest = record.get("payload_digest")
        expected = expected_hash(sequence, str(source_hash), str(payload_digest))
        if record.get("store_record_hash") != expected or record.get("readback_record_hash") != expected:
            reasons.append(f"broken_lineage:store_readback_hash:{sequence}")
        if not non_empty(record.get("idempotency_key")):
            reasons.append(f"broken_lineage:idempotency:{sequence}")
        if record.get("redaction_status") != "redacted":
            reasons.append(f"unredacted_payload:{sequence}")
        if record.get("storage_write_scope") != "sandboxed_fixture":
            reasons.append(f"forbidden:storage_write_scope:{sequence}")
        record_retention = record.get("retention")
        if not isinstance(record_retention, dict) or record_retention.get("policy_id") != retention_policy_id or record_retention.get("mode") != "immutable_until_expiry" or not non_empty(record_retention.get("expires_at")):
            reasons.append(f"missing_storage_config:record_retention:{sequence}")
        lineage = record.get("lineage")
        if not isinstance(lineage, dict) or not non_empty(lineage.get("source_ref")) or not non_empty(lineage.get("store_ref")) or lineage.get("lineage_status") != "linked":
            reasons.append(f"broken_lineage:record:{sequence}")
        if record.get("operation_effect") in FORBIDDEN_EFFECTS:
            reasons.append(f"forbidden:operation_effect:{record.get('operation_effect')}")
        previous_hash = str(record.get("store_record_hash"))
        expected_sequence += 1

    return {"status": classify_status(reasons), "fail_closed": bool(reasons), "blocking_reasons": reasons}


artifact = json.loads(artifact_path.read_text(encoding="utf-8"))
v280_artifact = json.loads(v280_artifact_path.read_text(encoding="utf-8"))
if v280_artifact.get("schema_version") != "ntpro.v280.persistent_audit_storage_runtime_artifact.v1":
    fail("v28 persistent audit storage artifact schema mismatch")

matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
module = next((item for item in matrix.get("module_readiness", []) if item.get("module_id") == MODULE_ID), None)
if not module:
    fail("matrix missing persistent_audit_storage_production_readiness")
if module.get("classification") != "production-ready":
    fail("persistent audit storage matrix entry must be production-ready")
if module.get("readiness_mode") != "deterministic_readiness_replay":
    fail("persistent audit storage matrix readiness mode mismatch")
if module.get("issue") != 928:
    fail("persistent audit storage matrix issue mismatch")
if module.get("evidence_path") != "docs/rust-cutover/evidence/V290-002.md":
    fail("persistent audit storage matrix evidence path mismatch")
if module.get("verification_command") != "scripts/ai/verify_release.sh v29-persistent-audit-storage-production-readiness":
    fail("persistent audit storage matrix verification command mismatch")
if module.get("production_ready_claim_allowed") is not True:
    fail("persistent audit storage production-ready claim flag mismatch")
for key in (
    "backend_production_go_live_claim_allowed",
    "product_grade_live_trading_terminal_claim_allowed",
    "production_execution_runtime_claim_allowed",
    "default_submit_claim_allowed",
):
    if module.get(key) is not False:
        fail(f"persistent audit storage matrix forbidden claim flag open: {key}")

cases = artifact.get("readiness_cases")
if not isinstance(cases, list) or [case.get("case_id") for case in cases] != EXPECTED_CASES:
    fail("readiness cases mismatch")
allowed = 0
fail_closed = 0
for case in cases:
    actual = classify_artifact(apply_case(artifact, case))
    if actual["status"] != case.get("expected_status"):
        fail(f"{case.get('case_id')}: expected {case.get('expected_status')} got {actual}")
    if actual["fail_closed"]:
        fail_closed += 1
    else:
        allowed += 1
if allowed != 1 or fail_closed != 6:
    fail(f"unexpected case counts: allowed={allowed} fail_closed={fail_closed}")

if selftest:
    opened = copy.deepcopy(artifact)
    opened["boundary_flags"]["adapter_send_allowed"] = True
    if classify_artifact(opened)["status"] != "fail_closed_forbidden_operation_boundary":
        fail("negative self-test unexpectedly allowed adapter_send_allowed")

    drifted = copy.deepcopy(artifact)
    drifted["schema_migration_policy"]["schema_drift_status"] = "drifted"
    if classify_artifact(drifted)["status"] != "fail_closed_schema_drift":
        fail("negative self-test unexpectedly allowed schema drift")

print(
    "v29_persistent_audit_storage_production_readiness=pass "
    f"cases={len(cases)} "
    f"allowed={allowed} "
    f"fail_closed={fail_closed} "
    f"records={len(artifact['readiness_records'])} "
    f"boundary_flags={len(BOUNDARY_FALSE_FLAGS)} "
    "negative_selftest=1"
)
PY
