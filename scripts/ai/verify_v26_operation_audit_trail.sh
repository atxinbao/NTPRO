#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

TRACE_PATH="${NTPRO_V260_OPERATION_AUDIT_TRACE:-tests/golden/v260_operation_audit_trail.jsonl}"
TASK_PATH="${NTPRO_V260_OPERATION_AUDIT_TASK:-docs/rust-cutover/tasks/V260-003.md}"
EVIDENCE_PATH="${NTPRO_V260_OPERATION_AUDIT_EVIDENCE:-docs/rust-cutover/evidence/V260-003.md}"
CONTRACT_PATH="${NTPRO_V260_OPERATION_AUDIT_CONTRACT:-docs/rust-cutover/release/v0_26_0_operation_audit_trail.md}"
PERMISSION_DEPENDENCY_PATH="${NTPRO_V260_OPERATION_AUDIT_PERMISSION_DEPENDENCY:-docs/rust-cutover/release/v0_26_0_operator_permission_model.md}"
BOUNDARY_DEPENDENCY_PATH="${NTPRO_V260_OPERATION_AUDIT_BOUNDARY_DEPENDENCY:-docs/rust-cutover/release/v0_26_0_product_hardening_boundary_contract.md}"
REPLAY_SCOPE_PATH="${NTPRO_V260_OPERATION_AUDIT_REPLAY_SCOPE:-docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"
SELFTEST="${NTPRO_V260_OPERATION_AUDIT_SELFTEST:-1}"

fail() {
  echo "v26 operation audit trail failed: $*" >&2
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

for path in \
  "$TRACE_PATH" \
  "$TASK_PATH" \
  "$EVIDENCE_PATH" \
  "$CONTRACT_PATH" \
  "$PERMISSION_DEPENDENCY_PATH" \
  "$BOUNDARY_DEPENDENCY_PATH" \
  "$REPLAY_SCOPE_PATH"; do
  require_file "$path"
done

require_contains "$TASK_PATH" "GitHub issue: \`#815\`"
require_contains "$EVIDENCE_PATH" "Task: \`V260-003\` / GitHub issue \`#815\`"
require_contains "$CONTRACT_PATH" "audit_artifact_scope = operation_audit_evidence_only"
require_contains "$CONTRACT_PATH" "operation_execution_allowed = false"
require_contains "$CONTRACT_PATH" "live_control_api_added = false"
require_contains "$CONTRACT_PATH" "automatic_remediation_allowed = false"
require_contains "$CONTRACT_PATH" "dashboard_trading_controls_enabled = false"
require_contains "$PERMISSION_DEPENDENCY_PATH" "permission_artifact_scope = operator_permission_evidence_only"
require_contains "$BOUNDARY_DEPENDENCY_PATH" "release_scope = product_hardening_foundation_only"

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
    "read_model.operation_audit_trail.valid_immutable_chain.001",
    "read_model.operation_audit_trail.missing_actor_lineage_hash_fail_closed.001",
    "read_model.operation_audit_trail.sequence_gap_fail_closed.001",
    "read_model.operation_audit_trail.hash_mismatch_fail_closed.001",
    "read_model.operation_audit_trail.unredacted_payload_fail_closed.001",
    "read_model.operation_audit_trail.forbidden_trading_action_fail_closed.001",
]
CONTRACT_VERSION = "ntpro.v260.operation_audit_trail.v1"
SCHEMA_VERSION = "ntpro.v260.operation_audit_event.schema.v1"
AUDIT_SCOPE = "operation_audit_evidence_only"
HARNESS = "scripts/ai/verify_release.sh v26-operation-audit-trail"
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
FORBIDDEN_DECISIONS = {
    "submitted",
    "executed",
    "sent_to_adapter",
    "sent_to_exchange",
    "retry_scheduled",
    "automatic_remediation_triggered",
    "dashboard_control_rendered",
}
SECRET_FIELD_NAMES = {
    "secret",
    "api_key",
    "api_secret",
    "credential",
    "raw_credential",
    "signature",
    "signed_payload",
    "signed_url",
    "private_key",
}


def fail(message: str) -> None:
    raise SystemExit(f"v26 operation audit trail failed: {message}")


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


def parse_ts(value: str) -> datetime:
    parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)


def expected_hash(sequence: int, event_type: str, payload_digest: str) -> str:
    return f"ntpro-audit-hash:{sequence:06d}:{event_type}:{payload_digest}"


def secret_field_reasons(value: Any, seq: int, reasons: list[str]) -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            key_string = str(key)
            if key_string in SECRET_FIELD_NAMES:
                if child not in (False, None, "", [], {}):
                    push_reason(reasons, f"unredacted_field:{seq}:{key_string}")
            secret_field_reasons(child, seq, reasons)
    elif isinstance(value, list):
        for child in value:
            secret_field_reasons(child, seq, reasons)


def classify(artifact: dict[str, Any], case_id: str) -> dict[str, Any]:
    if artifact.get("contract_version") != CONTRACT_VERSION:
        fail(f"{case_id}: contract_version must be {CONTRACT_VERSION}")
    if artifact.get("schema_version") != SCHEMA_VERSION:
        fail(f"{case_id}: schema_version must be {SCHEMA_VERSION}")
    dependency_contracts = artifact.get("dependency_contracts")
    if dependency_contracts != ["V260-001", "V260-002"]:
        fail(f"{case_id}: dependency_contracts must be ['V260-001', 'V260-002']")

    reasons: list[str] = list(artifact.get("blocking_reasons") or [])
    missing_required = False
    sequence_gap = False
    hash_mismatch = False
    unredacted_payload = False
    forbidden_trading_action = False

    if artifact.get("audit_artifact_scope") != AUDIT_SCOPE:
        missing_required = True
        push_reason(reasons, "audit_artifact_scope_mismatch")
    for key in (
        "external_audit_database_integration",
        "operation_execution_allowed",
        "live_control_api_added",
        "automatic_remediation_allowed",
        "dashboard_trading_controls_enabled",
    ):
        if artifact.get(key) is not False:
            if key in ("operation_execution_allowed", "automatic_remediation_allowed", "dashboard_trading_controls_enabled"):
                forbidden_trading_action = True
            else:
                missing_required = True
            push_reason(reasons, f"{key}_opened")
    if not isinstance(artifact.get("artifact_provenance"), dict):
        missing_required = True
        push_reason(reasons, "missing_artifact_provenance")
    parse_ts(str(artifact.get("evaluated_at")))

    events = artifact.get("audit_events")
    if not isinstance(events, list) or not events:
        fail(f"{case_id}: audit_events must be a non-empty list")

    event_types_checked: list[str] = []
    previous_hash = "GENESIS"
    expected_sequence = 1
    for event in events:
        if not isinstance(event, dict):
            fail(f"{case_id}: audit event must be an object")

        event_type = str(event.get("audit_event_type") or "unknown")
        event_types_checked.append(event_type)

        actor = event.get("actor")
        if not isinstance(actor, dict):
            missing_required = True
            push_reason(reasons, f"missing_actor:{expected_sequence}")
            actor = {}
        for field, reason in (
            ("id", "missing_actor_id"),
            ("role", "missing_actor_role"),
            ("scope", "missing_actor_scope"),
            ("provenance_ref", "missing_actor_provenance"),
        ):
            if not non_empty(actor.get(field)):
                missing_required = True
                push_reason(reasons, f"{reason}:{expected_sequence}")

        if not non_empty(event.get("intent")):
            missing_required = True
            push_reason(reasons, f"missing_intent:{expected_sequence}")
        if not non_empty(event.get("decision")):
            missing_required = True
            push_reason(reasons, f"missing_decision:{expected_sequence}")
        try:
            parse_ts(str(event.get("timestamp")))
        except Exception as exc:
            missing_required = True
            push_reason(reasons, f"invalid_timestamp:{expected_sequence}:{exc}")
        evidence_refs = event.get("evidence_refs")
        if not isinstance(evidence_refs, list) or not evidence_refs or not all(non_empty(ref) for ref in evidence_refs):
            missing_required = True
            push_reason(reasons, f"missing_evidence_refs:{expected_sequence}")

        chain = event.get("chain")
        if not isinstance(chain, dict):
            missing_required = True
            push_reason(reasons, f"missing_chain:{expected_sequence}")
            chain = {}
        sequence = chain.get("sequence")
        if not isinstance(sequence, int):
            missing_required = True
            push_reason(reasons, f"missing_sequence:{expected_sequence}")
            sequence = expected_sequence
        elif sequence != expected_sequence:
            sequence_gap = True
            push_reason(reasons, f"sequence_gap:expected:{expected_sequence}:actual:{sequence}")
        prev_hash = chain.get("prev_hash")
        if not non_empty(prev_hash):
            missing_required = True
            push_reason(reasons, f"missing_prev_hash:{expected_sequence}")
        elif prev_hash != previous_hash:
            hash_mismatch = True
            push_reason(reasons, f"prev_hash_mismatch:{sequence}")

        payload = event.get("payload")
        if not isinstance(payload, dict):
            missing_required = True
            push_reason(reasons, f"missing_payload:{sequence}")
            payload = {}
        payload_digest = payload.get("payload_digest")
        if not non_empty(payload_digest):
            missing_required = True
            push_reason(reasons, f"missing_payload_digest:{sequence}")
            payload_digest = "missing"
        event_hash = chain.get("event_hash")
        if not non_empty(event_hash):
            missing_required = True
            push_reason(reasons, f"missing_event_hash:{sequence}")
        elif event_hash != expected_hash(int(sequence), event_type, str(payload_digest)):
            hash_mismatch = True
            push_reason(reasons, f"event_hash_mismatch:{sequence}")

        if payload.get("redaction") != "redacted":
            unredacted_payload = True
            push_reason(reasons, f"payload_not_redacted:{sequence}")
        for key, reason in (
            ("contains_secret", "payload_contains_secret"),
            ("contains_raw_credential", "payload_contains_raw_credential"),
            ("contains_signed_payload", "payload_contains_signed_payload"),
        ):
            if payload.get(key) is not False:
                unredacted_payload = True
                push_reason(reasons, f"{reason}:{sequence}")
        secret_field_reasons(payload, int(sequence), reasons)
        if any(reason.startswith(f"unredacted_field:{sequence}:") for reason in reasons):
            unredacted_payload = True

        intent = str(event.get("intent") or "")
        decision = str(event.get("decision") or "")
        action_effect = str(payload.get("action_effect") or "")
        if decision in FORBIDDEN_DECISIONS:
            forbidden_trading_action = True
            push_reason(reasons, f"forbidden_trading_decision:{sequence}:{decision}")
        if intent in FORBIDDEN_INTENTS:
            forbidden_trading_action = True
            push_reason(reasons, f"forbidden_intent:{sequence}:{intent}")
        if payload.get("execution_triggered") is not False:
            forbidden_trading_action = True
            push_reason(reasons, f"operation_execution_triggered:{sequence}")
        if payload.get("operation_execution_allowed") is not False:
            forbidden_trading_action = True
            push_reason(reasons, f"operation_execution_allowed:{sequence}")
        if payload.get("automatic_remediation_triggered") is not False:
            forbidden_trading_action = True
            push_reason(reasons, f"automatic_remediation_triggered:{sequence}")
        if payload.get("dashboard_control_rendered") is not False:
            forbidden_trading_action = True
            push_reason(reasons, f"dashboard_control_rendered:{sequence}")
        if action_effect in FORBIDDEN_INTENTS:
            forbidden_trading_action = True
            push_reason(reasons, f"forbidden_action_effect:{sequence}:{action_effect}")

        previous_hash = str(event_hash or previous_hash)
        expected_sequence += 1

    if missing_required:
        status = "fail_closed_missing_required_evidence"
    elif unredacted_payload:
        status = "fail_closed_unredacted_payload"
    elif forbidden_trading_action:
        status = "fail_closed_forbidden_trading_action"
    elif sequence_gap:
        status = "fail_closed_sequence_gap"
    elif hash_mismatch:
        status = "fail_closed_hash_mismatch"
    else:
        status = "audit_trail_ready"

    return {
        "case_id": case_id,
        "audit_artifact_scope": artifact.get("audit_artifact_scope"),
        "effective_audit_status": status,
        "event_count": len(events),
        "event_types_checked": event_types_checked,
        "immutable_chain_evidence_only": artifact.get("audit_artifact_scope") == AUDIT_SCOPE,
        "operation_execution_allowed": False,
        "automatic_remediation_allowed": False,
        "live_control_api_added": False,
        "dashboard_trading_controls_allowed": False,
        "fail_closed": status != "audit_trail_ready",
        "blocking_reasons": reasons,
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
    if case_id.endswith("valid_immutable_chain.001"):
        healthy_artifact = copy.deepcopy(artifact)

if selftest:
    if healthy_artifact is None:
        fail("negative selftest requires valid immutable chain")
    healthy_artifact["audit_events"][0]["payload"]["execution_triggered"] = True
    closed = classify(healthy_artifact, "negative.selftest.execution_triggered")
    if closed["effective_audit_status"] != "fail_closed_forbidden_trading_action":
        fail("negative selftest enabled execution but did not fail closed")
    if "operation_execution_triggered:1" not in closed["blocking_reasons"]:
        fail("negative selftest did not surface execution boundary reason")

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
        "evidence_id": "V260-003",
        "harness": HARNESS,
        "validator_entrypoint": "scripts/ai/verify_v26_operation_audit_trail.sh::classify",
        "replay_type": "validator_executable_operation_audit_trail",
        "classification_owner": "V260-003",
        "source_scope_owner": "V260-003",
        "audit_artifact_scope": AUDIT_SCOPE,
    }
    for key, expected in expected_pairs.items():
        if entry.get(key) != expected:
            fail(f"{case_id}: release scope {key} mismatch: {entry.get(key)!r}")
    for key in (
        "runtime_adapter_integration",
        "complete_executable_order_control_runtime",
        "external_audit_database_integration",
        "operation_execution_allowed",
        "live_control_api_added",
        "new_submit_capability",
        "production_order_mutation_allowed",
        "adapter_send_allowed",
        "live_exchange_request_allowed",
        "retry_scheduler_enabled",
        "automatic_remediation_allowed",
        "dashboard_trading_controls_enabled",
        "product_grade_live_trading_terminal",
    ):
        if entry.get(key) is not False:
            fail(f"{case_id}: release scope {key} must be false")

print(
    "v26_operation_audit_trail "
    f"status=ok trace={trace_path.as_posix()} "
    f"cases={len(rows)} negative_selftest={1 if selftest else 0}"
)
PY
