#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

ARTIFACT_PATH="${NTPRO_V280_AUDIT_STORAGE_ARTIFACT:-docs/rust-cutover/release/v0_28_0_persistent_audit_storage_runtime_artifact.json}"
CONTRACT_PATH="${NTPRO_V280_AUDIT_STORAGE_CONTRACT:-docs/rust-cutover/release/v0_28_0_persistent_audit_storage_runtime_closure.md}"
TASK_PATH="${NTPRO_V280_AUDIT_STORAGE_TASK:-docs/rust-cutover/tasks/V280-003.md}"
EVIDENCE_PATH="${NTPRO_V280_AUDIT_STORAGE_EVIDENCE:-docs/rust-cutover/evidence/V280-003.md}"
MATRIX_PATH="${NTPRO_V280_AUDIT_STORAGE_MATRIX:-docs/rust-cutover/release/v0_28_0_backend_closure_readiness_matrix.json}"
FOUNDATION_PATH="${NTPRO_V280_AUDIT_STORAGE_FOUNDATION:-docs/rust-cutover/release/v0_27_0_persistent_operation_audit_storage_foundation.md}"
SELFTEST="${NTPRO_V280_AUDIT_STORAGE_SELFTEST:-1}"

fail() {
  echo "v28 persistent audit storage runtime closure failed: $*" >&2
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

for path in "$ARTIFACT_PATH" "$CONTRACT_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$MATRIX_PATH" "$FOUNDATION_PATH"; do
  require_file "$path"
done

require_contains "$TASK_PATH" "GitHub issue: \`#896\`"
require_contains "$EVIDENCE_PATH" "Task: \`V280-003\` / GitHub issue \`#896\`"
require_contains "$FOUNDATION_PATH" "persistent_audit_storage_scope = operation_audit_storage_foundation_only"
require_contains "$CONTRACT_PATH" "contract_version = ntpro.v280.persistent_audit_storage_runtime_closure.v1"
require_contains "$CONTRACT_PATH" "schema_version = ntpro.v280.persistent_audit_storage_runtime_artifact.v1"
require_contains "$CONTRACT_PATH" "backend_module_status = runtime_closed"
require_contains "$CONTRACT_PATH" "release stage = scripts/ai/verify_release.sh v28-persistent-audit-storage-runtime-closure"

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

SCHEMA_VERSION = "ntpro.v280.persistent_audit_storage_runtime_artifact.v1"
CONTRACT_VERSION = "ntpro.v280.persistent_audit_storage_runtime_closure.v1"
DEPENDENCIES = {"V280-001", "V280-002", "V270-003", "V260-003"}
BOUNDARY_FALSE_FLAGS = [
    "operation_execution_allowed",
    "automatic_remediation_allowed",
    "adapter_send_allowed",
    "live_exchange_request_allowed",
    "retry_scheduler_enabled",
    "dashboard_operation_controls_enabled",
    "dashboard_trading_controls_enabled",
    "admin_workbench_trading_controls_enabled",
    "manual_operation_submit_allowed",
    "product_grade_trading_terminal_claim",
]
FORBIDDEN_EFFECTS = {
    "executed",
    "sent_to_adapter",
    "sent_to_exchange",
    "retry_scheduled",
    "automatic_remediation_triggered",
    "dashboard_control_rendered",
}
EXPECTED_CASES = [
    "persistent_audit_storage.write_read.allowed.001",
    "persistent_audit_storage.missing_storage.fail_closed.001",
    "persistent_audit_storage.stale_source.fail_closed.001",
    "persistent_audit_storage.broken_lineage.fail_closed.001",
    "persistent_audit_storage.unredacted_payload.fail_closed.001",
    "persistent_audit_storage.forbidden_operation.fail_closed.001",
]


def fail(message: str) -> None:
    raise SystemExit(f"v28 persistent audit storage runtime closure failed: {message}")


def non_empty(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


def merge(base: dict[str, Any], override: dict[str, Any] | None) -> dict[str, Any]:
    merged = copy.deepcopy(base)
    if override:
        for key, value in override.items():
            merged[key] = value
    return merged


def expected_hash(sequence: int, source_event_hash: str, payload_digest: str) -> str:
    return f"ntpro-audit-runtime:{sequence:06d}:{source_event_hash}:{payload_digest}"


def classify_status(reasons: list[str]) -> str:
    if any(reason.startswith("missing_storage") for reason in reasons):
        return "fail_closed_missing_storage"
    if any(reason.startswith("stale_audit_source") for reason in reasons):
        return "fail_closed_stale_audit_source"
    if any(reason.startswith("broken_lineage") or reason.startswith("sequence") for reason in reasons):
        return "fail_closed_broken_lineage"
    if any(reason.startswith("missing_retention") for reason in reasons):
        return "fail_closed_missing_retention"
    if any(reason.startswith("unredacted_payload") for reason in reasons):
        return "fail_closed_unredacted_payload"
    if any(reason.startswith("store_source_drift") for reason in reasons):
        return "fail_closed_store_source_drift"
    if any(reason.startswith("forbidden") for reason in reasons):
        return "fail_closed_forbidden_operation_trigger"
    if reasons:
        return "fail_closed_audit_storage_violation"
    return "audit_write_read_replay_ready"


def apply_case(artifact: dict[str, Any], case: dict[str, Any]) -> dict[str, Any]:
    result = copy.deepcopy(artifact)
    result["audit_sink"] = merge(result["audit_sink"], case.get("audit_sink_override"))
    result["storage_provenance"] = merge(result["storage_provenance"], case.get("storage_provenance_override"))
    result["boundary_flags"] = merge(result["boundary_flags"], case.get("boundary_flags_override"))
    overrides = case.get("record_overrides") or {}
    for record in result["persistent_records"]:
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
    if artifact.get("task_id") != "V280-003" or artifact.get("github_issue") != 896:
        reasons.append("task_identity_mismatch")
    if artifact.get("backend_module") != "persistent_audit_storage_runtime_closure":
        reasons.append("backend_module_mismatch")
    if artifact.get("backend_module_status") != "runtime_closed":
        reasons.append("backend_module_not_runtime_closed")
    if set(artifact.get("dependency_contracts") or []) != DEPENDENCIES:
        reasons.append("dependency_contracts_mismatch")

    sink = artifact.get("audit_sink")
    if not isinstance(sink, dict):
        reasons.append("missing_storage_sink")
        sink = {}
    for key in ("sink_id", "sink_type", "storage_backend_claim", "segment_id"):
        if not non_empty(sink.get(key)):
            reasons.append(f"missing_storage:{key}")
    for key in ("append_only", "immutable_segments", "idempotency_key_required", "read_after_write_required"):
        if sink.get(key) is not True:
            reasons.append(f"missing_storage:{key}")
    for key in ("mutable_updates_allowed", "delete_before_retention_allowed"):
        if sink.get(key) is not False:
            reasons.append(f"missing_storage:{key}")
    if sink.get("freshness_status") != "fresh":
        reasons.append(f"stale_audit_source:sink:{sink.get('freshness_status')}")

    provenance = artifact.get("storage_provenance")
    if not isinstance(provenance, dict):
        reasons.append("missing_storage_provenance")
        provenance = {}
    for key in ("source_type", "store_id", "backend_class", "collected_at", "config_digest"):
        if not non_empty(provenance.get(key)):
            reasons.append(f"missing_storage:{key}")
    if provenance.get("lineage_status") != "linked":
        reasons.append(f"broken_lineage:storage_provenance:{provenance.get('lineage_status')}")

    boundary = artifact.get("boundary_flags")
    if not isinstance(boundary, dict):
        reasons.append("forbidden:missing_boundary_flags")
        boundary = {}
    for key in BOUNDARY_FALSE_FLAGS:
        if key not in boundary:
            reasons.append(f"forbidden:missing_required_false:{key}")
        elif boundary.get(key) is not False:
            reasons.append(f"forbidden:boundary_flag:{key}")

    records = artifact.get("persistent_records")
    if not isinstance(records, list) or not records:
        fail("persistent_records must be a non-empty list")
    previous_hash = "GENESIS"
    expected_sequence = 1
    for record in records:
        sequence = record.get("sequence")
        if sequence != expected_sequence:
            reasons.append(f"sequence_gap:{expected_sequence}:{sequence}")
        if record.get("previous_store_hash") != previous_hash:
            reasons.append(f"broken_lineage:previous_hash:{sequence}")
        source_hash = record.get("source_event_hash")
        payload_digest = record.get("payload_digest")
        expected = expected_hash(sequence, str(source_hash), str(payload_digest))
        if record.get("store_record_hash") != expected or record.get("readback_record_hash") != expected:
            reasons.append(f"store_source_drift:{sequence}")
        if record.get("redaction_status") != "redacted":
            reasons.append(f"unredacted_payload:{sequence}")
        retention = record.get("retention")
        if not isinstance(retention, dict) or not non_empty(retention.get("policy_id")) or not non_empty(retention.get("expires_at")) or retention.get("mode") != "immutable_until_expiry":
            reasons.append(f"missing_retention:{sequence}")
        lineage = record.get("lineage")
        if not isinstance(lineage, dict) or not non_empty(lineage.get("source_ref")) or lineage.get("lineage_status") != "linked":
            reasons.append(f"broken_lineage:record:{sequence}")
        if not non_empty(record.get("idempotency_key")):
            reasons.append(f"broken_lineage:idempotency:{sequence}")
        if record.get("operation_effect") in FORBIDDEN_EFFECTS:
            reasons.append(f"forbidden:operation_effect:{record.get('operation_effect')}")
        previous_hash = str(record.get("store_record_hash"))
        expected_sequence += 1
    return {"status": classify_status(reasons), "fail_closed": bool(reasons), "blocking_reasons": reasons}


artifact = json.loads(artifact_path.read_text(encoding="utf-8"))
matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
module = next((item for item in matrix.get("module_readiness", []) if item.get("module_id") == "persistent_audit_storage_runtime_closure"), None)
if not module:
    fail("matrix missing persistent_audit_storage_runtime_closure")
if module.get("classification") != "runtime-closed":
    fail("persistent audit storage matrix entry must be runtime-closed")
if module.get("evidence_path") != "docs/rust-cutover/evidence/V280-003.md":
    fail("persistent audit storage matrix evidence path mismatch")
if module.get("verification_command") != "scripts/ai/verify_release.sh v28-persistent-audit-storage-runtime-closure":
    fail("persistent audit storage matrix verification command mismatch")

cases = artifact.get("audit_replay_cases")
if not isinstance(cases, list) or [case.get("case_id") for case in cases] != EXPECTED_CASES:
    fail("audit replay cases mismatch")
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
if allowed != 1 or fail_closed != 5:
    fail(f"unexpected case counts: allowed={allowed} fail_closed={fail_closed}")

if selftest:
    opened = copy.deepcopy(artifact)
    opened["boundary_flags"]["adapter_send_allowed"] = True
    if classify_artifact(opened)["status"] != "fail_closed_forbidden_operation_trigger":
        fail("negative self-test unexpectedly allowed adapter_send_allowed")

print(
    "v28_persistent_audit_storage_runtime_closure=pass "
    f"cases={len(cases)} allowed={allowed} fail_closed={fail_closed} "
    f"records={len(artifact['persistent_records'])} boundary_flags={len(BOUNDARY_FALSE_FLAGS)} negative_selftest={int(selftest)}"
)
PY
