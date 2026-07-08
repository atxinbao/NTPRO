#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

TRACE_PATH="${NTPRO_V270_AUDIT_STORAGE_TRACE:-tests/golden/v270_persistent_audit_storage_foundation.jsonl}"
TASK_PATH="${NTPRO_V270_AUDIT_STORAGE_TASK:-docs/rust-cutover/tasks/V270-003.md}"
EVIDENCE_PATH="${NTPRO_V270_AUDIT_STORAGE_EVIDENCE:-docs/rust-cutover/evidence/V270-003.md}"
CONTRACT_PATH="${NTPRO_V270_AUDIT_STORAGE_CONTRACT:-docs/rust-cutover/release/v0_27_0_persistent_operation_audit_storage_foundation.md}"
BOUNDARY_PATH="${NTPRO_V270_AUDIT_STORAGE_BOUNDARY:-docs/rust-cutover/release/v0_27_0_product_operations_runtime_integration_boundary_contract.md}"
IDENTITY_PATH="${NTPRO_V270_AUDIT_STORAGE_IDENTITY:-docs/rust-cutover/release/v0_27_0_external_identity_permission_foundation.md}"
V26_AUDIT_PATH="${NTPRO_V270_AUDIT_STORAGE_V26_AUDIT:-docs/rust-cutover/release/v0_26_0_operation_audit_trail.md}"
REPLAY_SCOPE_PATH="${NTPRO_V270_AUDIT_STORAGE_REPLAY_SCOPE:-docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"
SELFTEST="${NTPRO_V270_AUDIT_STORAGE_SELFTEST:-1}"

fail() {
  echo "v27 persistent audit storage foundation failed: $*" >&2
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

for path in "$TRACE_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$CONTRACT_PATH" "$BOUNDARY_PATH" "$IDENTITY_PATH" "$V26_AUDIT_PATH" "$REPLAY_SCOPE_PATH"; do
  require_file "$path"
done

require_contains "$TASK_PATH" "GitHub issue: \`#856\`"
require_contains "$EVIDENCE_PATH" "Task: \`V270-003\` / GitHub issue \`#856\`"
require_contains "$BOUNDARY_PATH" "release_scope = product_operations_runtime_integration_foundation_only"
require_contains "$IDENTITY_PATH" "identity_permission_integration_scope = external_identity_permission_foundation_only"
require_contains "$V26_AUDIT_PATH" "audit_artifact_scope = operation_audit_evidence_only"
require_contains "$CONTRACT_PATH" "persistent_audit_storage_scope = operation_audit_storage_foundation_only"
require_contains "$CONTRACT_PATH" "dependency_contracts = V270-001,V270-002,V260-003"
require_contains "$CONTRACT_PATH" "append_only_audit_sink_required = true"
require_contains "$CONTRACT_PATH" "storage_provenance_required = true"
require_contains "$CONTRACT_PATH" "sequence_hash_lineage_required = true"
require_contains "$CONTRACT_PATH" "retention_metadata_required = true"
require_contains "$CONTRACT_PATH" "store_source_reconciliation_required = true"
require_contains "$CONTRACT_PATH" "operation_execution_allowed = false"
require_contains "$CONTRACT_PATH" "automatic_remediation_allowed = false"
require_contains "$CONTRACT_PATH" "adapter_send_allowed = false"
require_contains "$CONTRACT_PATH" "live_exchange_request_allowed = false"
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
    "read_model.persistent_audit_storage.valid_append_only_sink.001",
    "read_model.persistent_audit_storage.missing_lineage_fail_closed.001",
    "read_model.persistent_audit_storage.mutable_storage_claim_fail_closed.001",
    "read_model.persistent_audit_storage.missing_retention_fail_closed.001",
    "read_model.persistent_audit_storage.unredacted_payload_fail_closed.001",
    "read_model.persistent_audit_storage.store_source_drift_fail_closed.001",
    "read_model.persistent_audit_storage.forbidden_operation_trigger_fail_closed.001",
]
CONTRACT_VERSION = "ntpro.v270.persistent_audit_storage_foundation.v1"
SCHEMA_VERSION = "ntpro.v270.persistent_audit_storage_foundation.schema.v1"
STORAGE_SCOPE = "operation_audit_storage_foundation_only"
HARNESS = "scripts/ai/verify_release.sh v27-persistent-audit-storage-foundation"
DEPENDENCIES = ["V270-001", "V270-002", "V260-003"]
BOUNDARY_FALSE_FLAGS = [
    "operation_execution_allowed",
    "automatic_remediation_allowed",
    "adapter_send_allowed",
    "live_exchange_request_allowed",
    "retry_scheduler_enabled",
    "dashboard_operation_controls_enabled",
    "dashboard_trading_controls_enabled",
    "manual_operation_submit_allowed",
    "product_grade_trading_terminal_claim",
]
FORBIDDEN_DECISIONS = {
    "executed",
    "sent_to_adapter",
    "sent_to_exchange",
    "retry_scheduled",
    "automatic_remediation_triggered",
    "dashboard_control_rendered",
}
FORBIDDEN_INTENTS = {
    "submit_order",
    "cancel_order",
    "replace_order",
    "amend_order",
    "flatten_position",
    "adapter_send",
    "live_exchange_request",
    "retry_scheduler",
    "automatic_remediation",
}


def fail(message: str) -> None:
    raise SystemExit(f"v27 persistent audit storage foundation failed: {message}")


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


def expected_hash(sequence: int, source_event_hash: str, payload_digest: str) -> str:
    return f"ntpro-audit-storage-hash:{sequence:06d}:{source_event_hash}:{payload_digest}"


def classify(artifact: dict[str, Any], case_id: str) -> dict[str, Any]:
    if artifact.get("contract_version") != CONTRACT_VERSION:
        fail(f"{case_id}: contract_version must be {CONTRACT_VERSION}")
    if artifact.get("schema_version") != SCHEMA_VERSION:
        fail(f"{case_id}: schema_version must be {SCHEMA_VERSION}")

    reasons: list[str] = list(artifact.get("blocking_reasons") or [])
    missing_lineage = False
    mutable_storage = False
    missing_retention = False
    unredacted = False
    drift = False
    forbidden = False

    if artifact.get("persistent_audit_storage_scope") != STORAGE_SCOPE:
        missing_lineage = True
        push_reason(reasons, "persistent_audit_storage_scope_mismatch")
    if artifact.get("dependency_contracts") != DEPENDENCIES:
        missing_lineage = True
        push_reason(reasons, "dependency_contracts_mismatch")

    sink = artifact.get("audit_sink")
    if not isinstance(sink, dict):
        missing_lineage = True
        mutable_storage = True
        push_reason(reasons, "missing_audit_sink")
        sink = {}
    for key in ["sink_id", "sink_type", "storage_backend_claim", "segment_id"]:
        if not non_empty(sink.get(key)):
            missing_lineage = True
            push_reason(reasons, f"missing_audit_sink_field:{key}")
    if sink.get("append_only") is not True:
        mutable_storage = True
        push_reason(reasons, "append_only_not_true")
    if sink.get("immutable_segments") is not True:
        mutable_storage = True
        push_reason(reasons, "immutable_segments_not_true")
    if sink.get("mutable_updates_allowed") is not False:
        mutable_storage = True
        push_reason(reasons, "mutable_updates_allowed")
    if sink.get("delete_before_retention_allowed") is not False:
        mutable_storage = True
        push_reason(reasons, "delete_before_retention_allowed")

    provenance = artifact.get("storage_provenance")
    if not isinstance(provenance, dict):
        missing_lineage = True
        push_reason(reasons, "missing_storage_provenance")
        provenance = {}
    for key in ["source_type", "store_id", "backend_class", "collected_at", "config_digest"]:
        if not non_empty(provenance.get(key)):
            missing_lineage = True
            push_reason(reasons, f"missing_storage_provenance:{key}")

    boundary_flags = artifact.get("boundary_flags")
    if not isinstance(boundary_flags, dict):
        forbidden = True
        push_reason(reasons, "missing_boundary_flags")
        boundary_flags = {}
    for key in BOUNDARY_FALSE_FLAGS:
        if key not in boundary_flags:
            forbidden = True
            push_reason(reasons, f"missing_required_false_boundary:{key}")
        elif boundary_flags.get(key) is not False:
            forbidden = True
            push_reason(reasons, f"forbidden_boundary_flag:{key}")

    records = artifact.get("persistent_records")
    if not isinstance(records, list) or not records:
        fail(f"{case_id}: persistent_records must be a non-empty list")

    previous_hash = "GENESIS"
    expected_sequence = 1
    record_types: list[str] = []
    source_hashes: list[str] = []
    store_hashes: list[str] = []
    for record in records:
        if not isinstance(record, dict):
            fail(f"{case_id}: persistent record must be an object")
        record_type = str(record.get("audit_event_type") or "unknown")
        record_types.append(record_type)
        sequence = record.get("sequence")
        if sequence != expected_sequence:
            missing_lineage = True
            push_reason(reasons, f"sequence_gap:{expected_sequence}->{sequence}")

        actor = record.get("actor")
        if not isinstance(actor, dict):
            missing_lineage = True
            push_reason(reasons, f"missing_actor:{expected_sequence}")
            actor = {}
        for key in ["id", "role", "scope", "provenance_ref"]:
            if not non_empty(actor.get(key)):
                missing_lineage = True
                push_reason(reasons, f"missing_actor_{key}:{expected_sequence}")

        for key in ["record_id", "source_event_hash", "payload_digest", "store_record_hash", "previous_store_hash", "timestamp"]:
            if not non_empty(record.get(key)):
                missing_lineage = True
                push_reason(reasons, f"missing_record_field:{expected_sequence}:{key}")

        source_event_hash = str(record.get("source_event_hash") or "")
        payload_digest = str(record.get("payload_digest") or "")
        store_record_hash = str(record.get("store_record_hash") or "")
        previous_store_hash = str(record.get("previous_store_hash") or "")
        source_hashes.append(source_event_hash)
        store_hashes.append(store_record_hash)
        if previous_store_hash != previous_hash:
            missing_lineage = True
            push_reason(reasons, f"previous_hash_mismatch:{expected_sequence}")
        computed = expected_hash(int(sequence or 0), source_event_hash, payload_digest)
        if store_record_hash != computed:
            missing_lineage = True
            push_reason(reasons, f"store_hash_mismatch:{expected_sequence}")

        if record.get("redaction_status") != "redacted":
            unredacted = True
            push_reason(reasons, f"redaction_not_redacted:{expected_sequence}:{record.get('redaction_status')}")

        retention = record.get("retention")
        if not isinstance(retention, dict):
            missing_retention = True
            push_reason(reasons, f"missing_retention:{expected_sequence}")
            retention = {}
        for key in ["policy_id", "expires_at", "mode"]:
            if not non_empty(retention.get(key)):
                missing_retention = True
                push_reason(reasons, f"missing_retention_field:{expected_sequence}:{key}")
        if retention.get("mode") != "immutable_until_expiry":
            missing_retention = True
            push_reason(reasons, f"retention_mode_mismatch:{expected_sequence}:{retention.get('mode')}")

        lineage = record.get("lineage")
        if not isinstance(lineage, dict):
            missing_lineage = True
            push_reason(reasons, f"missing_lineage:{expected_sequence}")
            lineage = {}
        for key in ["source_ref", "store_ref", "source_event_hash", "store_record_hash"]:
            if not non_empty(lineage.get(key)):
                missing_lineage = True
                push_reason(reasons, f"missing_lineage_field:{expected_sequence}:{key}")
        if non_empty(lineage.get("source_event_hash")) and lineage.get("source_event_hash") != source_event_hash:
            drift = True
            push_reason(reasons, f"source_lineage_drift:{expected_sequence}")
        if non_empty(lineage.get("store_record_hash")) and lineage.get("store_record_hash") != store_record_hash:
            drift = True
            push_reason(reasons, f"store_lineage_drift:{expected_sequence}")

        if record.get("intent") in FORBIDDEN_INTENTS:
            forbidden = True
            push_reason(reasons, f"forbidden_intent:{expected_sequence}:{record.get('intent')}")
        if record.get("decision") in FORBIDDEN_DECISIONS:
            forbidden = True
            push_reason(reasons, f"forbidden_decision:{expected_sequence}:{record.get('decision')}")
        for key in ["operation_execution_triggered", "automatic_remediation_triggered", "adapter_send_triggered", "live_exchange_request_triggered", "dashboard_control_triggered"]:
            if record.get(key) is not False:
                forbidden = True
                push_reason(reasons, f"forbidden_record_trigger:{expected_sequence}:{key}")

        previous_hash = store_record_hash
        expected_sequence += 1

    reconciliation = artifact.get("store_source_reconciliation")
    if not isinstance(reconciliation, dict):
        drift = True
        push_reason(reasons, "missing_store_source_reconciliation")
        reconciliation = {}
    expected_source_head = source_hashes[-1] if source_hashes else ""
    expected_store_head = store_hashes[-1] if store_hashes else ""
    if reconciliation.get("source_head_hash") != expected_source_head:
        drift = True
        push_reason(reasons, "source_head_hash_drift")
    if reconciliation.get("store_head_hash") != expected_store_head:
        drift = True
        push_reason(reasons, "store_head_hash_drift")
    if reconciliation.get("drift_status") != "in_sync":
        drift = True
        push_reason(reasons, f"drift_status_not_in_sync:{reconciliation.get('drift_status')}")

    if missing_lineage:
        effective_status = "fail_closed_missing_lineage"
    elif mutable_storage:
        effective_status = "fail_closed_mutable_storage_claim"
    elif missing_retention:
        effective_status = "fail_closed_missing_retention"
    elif unredacted:
        effective_status = "fail_closed_unredacted_payload"
    elif drift:
        effective_status = "fail_closed_store_source_drift"
    elif forbidden:
        effective_status = "fail_closed_forbidden_operation_trigger"
    elif reasons:
        effective_status = "fail_closed_missing_lineage"
    else:
        effective_status = "persistent_audit_storage_ready"

    return {
        "persistent_audit_storage_scope": artifact.get("persistent_audit_storage_scope"),
        "effective_persistent_audit_storage_status": effective_status,
        "record_count": len(records),
        "record_types_checked": record_types,
        "append_only_sink": sink.get("append_only") is True and sink.get("mutable_updates_allowed") is False,
        "storage_provenance_complete": not any(reason.startswith("missing_storage_provenance") for reason in reasons),
        "lineage_complete": not missing_lineage,
        "retention_complete": not missing_retention,
        "redaction_complete": not unredacted,
        "store_source_in_sync": not drift,
        "operation_triggers_allowed": False,
        "fail_closed": effective_status != "persistent_audit_storage_ready",
        "blocking_reasons": reasons,
    }


rows = load_rows(trace_path)
case_ids = [str(row.get("case_id")) for row in rows]
if case_ids != EXPECTED_CASES:
    fail(f"case order mismatch: {case_ids}")

healthy_artifact: dict[str, Any] | None = None
for row in rows:
    case_id = str(row.get("case_id"))
    if row.get("schema_version") != "golden-trace-v1":
        fail(f"{case_id}: schema_version must be golden-trace-v1")
    if row.get("category") != "read_model":
        fail(f"{case_id}: category must be read_model")
    input_event = single_event(row, "input", case_id)
    expected_event = single_event(row, "expected", case_id)
    if input_event.get("event_type") != "read_model.persistent_audit_storage.input":
        fail(f"{case_id}: unexpected input event_type")
    if expected_event.get("event_type") != "read_model.persistent_audit_storage.validated":
        fail(f"{case_id}: unexpected expected event_type")
    artifact = input_event.get("payload", {}).get("artifact")
    if not isinstance(artifact, dict):
        fail(f"{case_id}: input artifact must be an object")
    actual = classify(artifact, case_id)
    actual["case_id"] = case_id
    expected_payload = expected_event.get("payload")
    if actual != expected_payload:
        fail(
            f"{case_id}: expected payload mismatch\n"
            f"actual={json.dumps(actual, sort_keys=True)}\n"
            f"expected={json.dumps(expected_payload, sort_keys=True)}"
        )
    if case_id == EXPECTED_CASES[0]:
        healthy_artifact = artifact

if healthy_artifact is None:
    fail("missing healthy artifact")

if selftest:
    mutated = copy.deepcopy(healthy_artifact)
    mutated["audit_sink"]["mutable_updates_allowed"] = True
    status = classify(mutated, "negative_selftest")["effective_persistent_audit_storage_status"]
    if status != "fail_closed_mutable_storage_claim":
        fail(f"negative selftest did not fail closed: {status}")

scope = json.loads(replay_scope_path.read_text(encoding="utf-8"))
entries = scope.get("cases")
if not isinstance(entries, list):
    fail("release replay scope cases must be a list")
by_case = {entry.get("case_id"): entry for entry in entries if isinstance(entry, dict)}
for case_id in EXPECTED_CASES:
    entry = by_case.get(case_id)
    if not isinstance(entry, dict):
        fail(f"missing release replay scope entry: {case_id}")
    if entry.get("trace") != str(trace_path):
        fail(f"{case_id}: replay scope trace mismatch")
    if entry.get("status") != "validator_executable_replay":
        fail(f"{case_id}: replay scope status must be validator_executable_replay")
    if entry.get("evidence_id") != "V270-003":
        fail(f"{case_id}: replay scope evidence_id must be V270-003")
    if entry.get("harness") != HARNESS:
        fail(f"{case_id}: replay scope harness mismatch")
    if entry.get("runtime_adapter_integration") is not False:
        fail(f"{case_id}: runtime_adapter_integration must be false")
    if entry.get("automatic_remediation_allowed") is not False:
        fail(f"{case_id}: automatic_remediation_allowed must be false")
    if entry.get("adapter_send_allowed") is not False:
        fail(f"{case_id}: adapter_send_allowed must be false")
    if entry.get("live_exchange_request_allowed") is not False:
        fail(f"{case_id}: live_exchange_request_allowed must be false")
    if entry.get("dashboard_trading_controls_enabled") is not False:
        fail(f"{case_id}: dashboard_trading_controls_enabled must be false")

print(
    "v27_persistent_audit_storage_foundation=pass "
    f"cases={len(rows)} records={len(healthy_artifact['persistent_records'])} "
    f"boundary_flags={len(BOUNDARY_FALSE_FLAGS)} negative_selftest={1 if selftest else 0}"
)
PY
