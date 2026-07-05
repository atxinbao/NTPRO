#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

TRACE_PATH="${NTPRO_V250_DR_PREVIEW_TRACE:-tests/golden/v250_dr_preview_drill_evidence.jsonl}"
SELFTEST="${NTPRO_V250_DR_PREVIEW_SELFTEST:-1}"

if [[ ! -f "$TRACE_PATH" ]]; then
  echo "missing V250 DR preview trace: $TRACE_PATH" >&2
  exit 1
fi

python3 - "$TRACE_PATH" "$SELFTEST" <<'PY'
from __future__ import annotations

import copy
import json
import re
import sys
from pathlib import Path
from typing import Any

trace_path = Path(sys.argv[1])
selftest = sys.argv[2] != "0"

EXPECTED_CASES = [
    "read_model.dr_preview_drill_evidence.valid_preview_matrix.001",
    "read_model.dr_preview_drill_evidence.missing_snapshot_fail_closed.001",
    "read_model.dr_preview_drill_evidence.stale_recovery_point_fail_closed.001",
    "read_model.dr_preview_drill_evidence.scope_mismatch_fail_closed.001",
    "read_model.dr_preview_drill_evidence.unapproved_restore_fail_closed.001",
    "read_model.dr_preview_drill_evidence.actual_execution_claim_fail_closed.001",
    "read_model.dr_preview_drill_evidence.redaction_secret_fail_closed.001",
]
EXPECTED_SCENARIOS = [
    "restart_preview",
    "read_model_rebuild_preview",
    "artifact_replay_preview",
    "release_rollback_recommendation",
]
APPROVED_STATUSES = {"owner_approved", "audit_gate_approved", "blocked_preview"}
BOUNDARY_FALSE_FIELDS = [
    "automatic_restart_allowed",
    "service_restart_execution_allowed",
    "data_restore_execution_allowed",
    "artifact_replay_execution_allowed",
    "release_rollback_execution_allowed",
    "production_order_mutation_allowed",
    "exchange_state_mutation_allowed",
    "live_exchange_request_allowed",
    "adapter_send_allowed",
    "automatic_remediation_allowed",
]
REQUIRED_SOURCE_FIELDS = ["source_type", "source_ref", "producer", "collected_at"]
REQUIRED_SCOPE_FIELDS = ["account_key", "strategy_key", "venue_node_key", "isolation_scope_key"]
REQUIRED_SNAPSHOT_FIELDS = ["snapshot_id", "source_ref", "checksum", "captured_at"]
REQUIRED_READBACK_FIELDS = ["readback_id", "source_ref", "checksum"]
REQUIRED_AUDIT_FIELDS = ["audit_id", "actor", "action", "at", "evidence_ref"]
FORBIDDEN_KEY_PATTERNS = [
    re.compile(pattern, re.IGNORECASE)
    for pattern in (
        r"(^|_)(secret|credential|signature|signed_payload|raw_order_id|raw_credential)(_|$)",
        r"api[_-]?key",
        r"client[_-]?order[_-]?id",
    )
]
FORBIDDEN_VALUE_PATTERNS = [
    re.compile(pattern, re.IGNORECASE)
    for pattern in (
        r"api[_-]?secret\s*=",
        r"credential\s*=",
        r"signed[_-]?payload\s*=",
        r"signature\s*=",
        r"raw[_-]?order[_-]?id\s*=",
        r"client[_-]?order[_-]?id\s*=",
    )
]
CONTRACT_VERSION = "ntpro.v250.dr_preview_drill_evidence.v1"
SCHEMA_VERSION = "ntpro.v250.dr_preview_drill_evidence.schema.v1"


def fail(message: str) -> None:
    raise SystemExit(f"v25 DR preview drill evidence failed: {message}")


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
    events = row.get(section, {}).get("events")
    if not isinstance(events, list) or len(events) != 1 or not isinstance(events[0], dict):
        fail(f"{case_id}: {section}.events must contain exactly one object")
    return events[0]


def non_empty(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


def push_reason(reasons: list[str], reason: str) -> None:
    if reason not in reasons:
        reasons.append(reason)


def source_provenance_complete(value: Any) -> bool:
    return isinstance(value, dict) and all(non_empty(value.get(field)) for field in REQUIRED_SOURCE_FIELDS)


def list_of_complete_dicts(value: Any, required_fields: list[str]) -> bool:
    return (
        isinstance(value, list)
        and bool(value)
        and all(isinstance(entry, dict) and all(non_empty(entry.get(field)) for field in required_fields) for entry in value)
    )


def audit_trace_complete(value: Any) -> bool:
    return list_of_complete_dicts(value, REQUIRED_AUDIT_FIELDS)


def source_lineage_complete(value: Any) -> bool:
    return (
        isinstance(value, list)
        and bool(value)
        and all(source_provenance_complete(entry) for entry in value)
    )


def scope_complete(value: Any) -> bool:
    return isinstance(value, dict) and all(non_empty(value.get(field)) for field in REQUIRED_SCOPE_FIELDS)


def scan_redaction(value: Any, reasons: list[str], path: str = "dr_preview") -> bool:
    breached = False
    if isinstance(value, dict):
        for key, nested in value.items():
            if any(pattern.search(str(key)) for pattern in FORBIDDEN_KEY_PATTERNS):
                push_reason(reasons, f"forbidden_sensitive_key:{path}.{key}")
                breached = True
            if scan_redaction(nested, reasons, f"{path}.{key}"):
                breached = True
    elif isinstance(value, list):
        for index, nested in enumerate(value):
            if scan_redaction(nested, reasons, f"{path}[{index}]"):
                breached = True
    elif isinstance(value, str):
        if any(pattern.search(value) for pattern in FORBIDDEN_VALUE_PATTERNS):
            push_reason(reasons, f"forbidden_sensitive_value:{path}")
            breached = True
    return breached


def classify(snapshot: dict[str, Any], case_id: str) -> dict[str, Any]:
    if snapshot.get("contract_version") != CONTRACT_VERSION:
        fail(f"{case_id}: contract_version must be {CONTRACT_VERSION}")
    if snapshot.get("schema_version") != SCHEMA_VERSION:
        fail(f"{case_id}: schema_version must be {SCHEMA_VERSION}")

    reasons: list[str] = list(snapshot.get("blocking_reasons") or [])
    if any(not non_empty(reason) for reason in reasons):
        fail(f"{case_id}: blocking_reasons must contain non-empty strings")

    boundary = snapshot.get("dr_boundary")
    if not isinstance(boundary, dict):
        fail(f"{case_id}: dr_boundary must be an object")
    artifacts = snapshot.get("artifacts")
    if not isinstance(artifacts, list) or not artifacts:
        fail(f"{case_id}: artifacts must be a non-empty array")

    missing_required = False
    stale_recovery_point = False
    scope_mismatch = False
    unapproved_restore = False
    actual_execution_claim = False
    redaction_breach = False
    automatic_allowed = False
    scenarios: set[str] = set()
    snapshot_refs_count = 0
    readback_refs_count = 0
    audit_events_checked = 0

    if boundary.get("dr_mode") != "preview_only":
        actual_execution_claim = True
        push_reason(reasons, "dr_mode_not_preview_only")
    for field in BOUNDARY_FALSE_FIELDS:
        if boundary.get(field) is not False:
            actual_execution_claim = True
            if boundary.get(field) is True:
                automatic_allowed = True
            push_reason(reasons, f"boundary_true:{field}")

    for index, artifact in enumerate(artifacts):
        if not isinstance(artifact, dict):
            missing_required = True
            push_reason(reasons, f"artifact_not_object:{index}")
            continue

        for field_name in ("drill_id", "scenario", "owner"):
            if not non_empty(artifact.get(field_name)):
                missing_required = True
                push_reason(reasons, f"missing_{field_name}:artifact:{index}")

        scenario = artifact.get("scenario")
        if non_empty(scenario):
            if scenario in EXPECTED_SCENARIOS:
                scenarios.add(str(scenario))
            else:
                missing_required = True
                push_reason(reasons, f"invalid_scenario:{scenario}:artifact:{index}")

        affected_scope = artifact.get("affected_scope")
        if not scope_complete(affected_scope):
            missing_required = True
            push_reason(reasons, f"missing_affected_scope:artifact:{index}")

        expected_scope = artifact.get("scope_consistency", {}).get("expected_isolation_scope_key") if isinstance(artifact.get("scope_consistency"), dict) else None
        if not non_empty(expected_scope):
            missing_required = True
            push_reason(reasons, f"missing_expected_scope:artifact:{index}")
        elif isinstance(affected_scope, dict) and affected_scope.get("isolation_scope_key") != expected_scope:
            scope_mismatch = True
            push_reason(reasons, f"scope_mismatch:artifact:{index}")

        snapshot_refs = artifact.get("snapshot_refs")
        if not list_of_complete_dicts(snapshot_refs, REQUIRED_SNAPSHOT_FIELDS):
            missing_required = True
            push_reason(reasons, f"missing_snapshot_refs:artifact:{index}")
        else:
            snapshot_refs_count += len(snapshot_refs)

        recovery_point = artifact.get("expected_recovery_point")
        if not isinstance(recovery_point, dict) or not non_empty(recovery_point.get("rpo_ref")) or not non_empty(recovery_point.get("as_of")):
            missing_required = True
            push_reason(reasons, f"missing_expected_recovery_point:artifact:{index}")
        elif recovery_point.get("freshness_status") != "fresh":
            stale_recovery_point = True
            push_reason(reasons, f"stale_recovery_point:artifact:{index}")

        readback_refs = artifact.get("readback_refs")
        if not list_of_complete_dicts(readback_refs, REQUIRED_READBACK_FIELDS):
            missing_required = True
            push_reason(reasons, f"missing_readback_refs:artifact:{index}")
        else:
            readback_refs_count += len(readback_refs)

        audit_trace = artifact.get("audit_trace")
        if not audit_trace_complete(audit_trace):
            missing_required = True
            push_reason(reasons, f"missing_audit_trace:artifact:{index}")
        else:
            audit_events_checked += len(audit_trace)

        if not source_provenance_complete(artifact.get("source_provenance")):
            missing_required = True
            push_reason(reasons, f"missing_source_provenance:artifact:{index}")

        if not source_lineage_complete(artifact.get("snapshot_lineage")):
            missing_required = True
            push_reason(reasons, f"missing_snapshot_lineage:artifact:{index}")

        approval = artifact.get("operator_approval")
        if not isinstance(approval, dict) or not non_empty(approval.get("status")) or not non_empty(approval.get("owner")):
            missing_required = True
            push_reason(reasons, f"missing_operator_approval:artifact:{index}")
        else:
            status = approval.get("status")
            if status == "unapproved":
                unapproved_restore = True
                if approval.get("visible_result") != "blocked_preview":
                    push_reason(reasons, f"unapproved_restore_not_blocked:artifact:{index}")
                else:
                    push_reason(reasons, f"unapproved_restore_blocked:artifact:{index}")
            elif status not in APPROVED_STATUSES:
                missing_required = True
                push_reason(reasons, f"invalid_operator_approval:{status}:artifact:{index}")

        preview_output = artifact.get("preview_output")
        if not isinstance(preview_output, dict):
            missing_required = True
            push_reason(reasons, f"missing_preview_output:artifact:{index}")
        else:
            if preview_output.get("side_effect") != "none":
                actual_execution_claim = True
                automatic_allowed = True
                push_reason(reasons, f"preview_side_effect:{preview_output.get('side_effect')}:artifact:{index}")
            if preview_output.get("execution_claim") is not False:
                actual_execution_claim = True
                if preview_output.get("execution_claim") is True:
                    automatic_allowed = True
                push_reason(reasons, f"preview_execution_claim:{preview_output.get('execution_claim')}:artifact:{index}")

        if artifact.get("redaction_state") != "redacted":
            redaction_breach = True
            push_reason(reasons, f"redaction_state_not_redacted:artifact:{index}")
        if scan_redaction(artifact, reasons, f"artifact:{index}"):
            redaction_breach = True

    if redaction_breach:
        status = "fail_closed_redaction_breach"
    elif actual_execution_claim:
        status = "fail_closed_actual_execution_claim"
    elif unapproved_restore:
        status = "fail_closed_unapproved_restore"
    elif scope_mismatch:
        status = "fail_closed_scope_mismatch"
    elif stale_recovery_point:
        status = "fail_closed_stale_recovery_point"
    elif missing_required:
        status = "fail_closed_missing_required"
    else:
        status = "dr_preview_readonly"

    return {
        "case_id": case_id,
        "dr_preview_status": status,
        "fail_closed": status.startswith("fail_closed"),
        "artifact_count": len(artifacts),
        "scenarios": [scenario for scenario in EXPECTED_SCENARIOS if scenario in scenarios],
        "snapshot_refs_count": snapshot_refs_count,
        "readback_refs_count": readback_refs_count,
        "audit_events_checked": audit_events_checked,
        "approval_complete": not any(reason.startswith("missing_operator_approval:") or reason.startswith("invalid_operator_approval:") for reason in reasons),
        "snapshot_lineage_complete": not any(reason.startswith("missing_snapshot_refs:") or reason.startswith("missing_snapshot_lineage:") for reason in reasons),
        "scope_consistent": not scope_mismatch and not any(reason.startswith("missing_affected_scope:") or reason.startswith("missing_expected_scope:") for reason in reasons),
        "freshness_fresh": not stale_recovery_point and not any(reason.startswith("missing_expected_recovery_point:") for reason in reasons),
        "redaction_clean": not redaction_breach,
        "preview_boundary_readonly": not actual_execution_claim,
        "actual_execution_allowed": automatic_allowed,
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

valid_snapshot: dict[str, Any] | None = None
for row in rows:
    case_id = str(row.get("case_id"))
    if row.get("schema_version") != "golden-trace-v1":
        fail(f"{case_id}: schema_version must be golden-trace-v1")
    if row.get("category") != "read_model":
        fail(f"{case_id}: category must be read_model")
    if not non_empty(row.get("description")):
        fail(f"{case_id}: description is required")

    input_event = single_event(row, "input", case_id)
    expected_event = single_event(row, "expected", case_id)
    expected_event_type = input_event.get("event_type", "").replace(".input", ".validated")
    if expected_event.get("event_type") != expected_event_type:
        fail(f"{case_id}: expected event_type must be {expected_event_type}")
    for key in ("ts_event", "ts_init", "instrument_id", "venue", "correlation_id"):
        if expected_event.get(key) != input_event.get(key):
            fail(f"{case_id}: expected.{key} must match input.{key}")

    payload = input_event.get("payload")
    if not isinstance(payload, dict) or not isinstance(payload.get("snapshot"), dict):
        fail(f"{case_id}: input payload snapshot is required")
    snapshot = payload["snapshot"]
    computed = classify(snapshot, case_id)
    if computed != expected_event.get("payload"):
        fail(
            f"{case_id}: computed payload mismatch\n"
            f"expected={json.dumps(expected_event.get('payload'), sort_keys=True)}\n"
            f"actual={json.dumps(computed, sort_keys=True)}"
        )
    if case_id.endswith("valid_preview_matrix.001"):
        if computed["scenarios"] != EXPECTED_SCENARIOS:
            fail(f"{case_id}: valid matrix must cover all expected scenarios")
        valid_snapshot = copy.deepcopy(snapshot)

if selftest:
    if valid_snapshot is None:
        fail("negative selftest requires valid DR preview fixture")
    valid_snapshot["artifacts"][0]["snapshot_refs"] = []
    failed = classify(valid_snapshot, "negative.selftest.missing_snapshot")
    if failed["dr_preview_status"] != "fail_closed_missing_required":
        fail("negative selftest removed snapshot_refs but did not fail closed")
    if "missing_snapshot_refs:artifact:0" not in failed["blocking_reasons"]:
        fail("negative selftest did not report missing_snapshot_refs:artifact:0")

print(
    "v25_dr_preview_drill_evidence "
    f"status=ok trace={trace_path.as_posix()} "
    f"cases={len(rows)} scenarios={len(EXPECTED_SCENARIOS)} "
    f"negative_selftest={1 if selftest else 0}"
)
PY
