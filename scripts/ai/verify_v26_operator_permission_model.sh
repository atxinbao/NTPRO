#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

TRACE_PATH="${NTPRO_V260_OPERATOR_PERMISSION_TRACE:-tests/golden/v260_operator_permission_model.jsonl}"
TASK_PATH="${NTPRO_V260_OPERATOR_PERMISSION_TASK:-docs/rust-cutover/tasks/V260-002.md}"
EVIDENCE_PATH="${NTPRO_V260_OPERATOR_PERMISSION_EVIDENCE:-docs/rust-cutover/evidence/V260-002.md}"
CONTRACT_PATH="${NTPRO_V260_OPERATOR_PERMISSION_CONTRACT:-docs/rust-cutover/release/v0_26_0_operator_permission_model.md}"
DEPENDENCY_PATH="${NTPRO_V260_OPERATOR_PERMISSION_DEPENDENCY:-docs/rust-cutover/release/v0_26_0_product_hardening_boundary_contract.md}"
REPLAY_SCOPE_PATH="${NTPRO_V260_OPERATOR_PERMISSION_REPLAY_SCOPE:-docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"
SELFTEST="${NTPRO_V260_OPERATOR_PERMISSION_SELFTEST:-1}"

fail() {
  echo "v26 operator permission model failed: $*" >&2
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

for path in "$TRACE_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$CONTRACT_PATH" "$DEPENDENCY_PATH" "$REPLAY_SCOPE_PATH"; do
  require_file "$path"
done

require_contains "$DEPENDENCY_PATH" "release_scope = product_hardening_foundation_only"
require_contains "$TASK_PATH" "GitHub issue: \`#814\`"
require_contains "$EVIDENCE_PATH" "Task: \`V260-002\` / GitHub issue \`#814\`"
require_contains "$CONTRACT_PATH" "permission_artifact_scope = operator_permission_evidence_only"
require_contains "$CONTRACT_PATH" "external_identity_provider_integration = false"
require_contains "$CONTRACT_PATH" "live_operation_authorization = false"
require_contains "$CONTRACT_PATH" "production_trading_authorization = false"
require_contains "$CONTRACT_PATH" "dashboard_trading_controls_enabled = false"

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
    "read_model.operator_permission_model.valid_role_matrix.001",
    "read_model.operator_permission_model.missing_role_scope_provenance_fail_closed.001",
    "read_model.operator_permission_model.missing_permission_fail_closed.001",
    "read_model.operator_permission_model.cross_scope_permission_fail_closed.001",
    "read_model.operator_permission_model.expired_approval_fail_closed.001",
    "read_model.operator_permission_model.role_escalation_fail_closed.001",
    "read_model.operator_permission_model.trading_control_attempt_fail_closed.001",
]
CONTRACT_VERSION = "ntpro.v260.operator_permission_model.v1"
SCHEMA_VERSION = "ntpro.v260.operator_permission_model.schema.v1"
PERMISSION_SCOPE = "operator_permission_evidence_only"
HARNESS = "scripts/ai/verify_release.sh v26-operator-permission-model"
ALL_PERMISSION_KEYS = [
    "dashboard_read",
    "operation_preview_read",
    "runbook_read",
    "release_gate_read",
    "release_manifest_read",
    "incident_ack_review",
    "audit_read",
    "provenance_read",
    "submit_order",
    "cancel_order",
    "replace_order",
    "amend_order",
    "flatten_position",
    "retry_scheduler",
    "automatic_remediation",
    "adapter_send",
    "live_exchange_request",
    "dashboard_trading_controls",
    "role_escalation",
]
TRADING_CONTROL_KEYS = [
    "submit_order",
    "cancel_order",
    "replace_order",
    "amend_order",
    "flatten_position",
    "retry_scheduler",
    "automatic_remediation",
    "adapter_send",
    "live_exchange_request",
    "dashboard_trading_controls",
]
ROLE_POLICIES = {
    "viewer": {
        "required": {"dashboard_read"},
        "allowed": {"dashboard_read"},
        "approval_required": False,
    },
    "operator": {
        "required": {"dashboard_read", "operation_preview_read", "runbook_read"},
        "allowed": {"dashboard_read", "operation_preview_read", "runbook_read"},
        "approval_required": True,
    },
    "release_gatekeeper": {
        "required": {"dashboard_read", "release_gate_read", "release_manifest_read"},
        "allowed": {"dashboard_read", "release_gate_read", "release_manifest_read"},
        "approval_required": True,
    },
    "incident_owner": {
        "required": {"dashboard_read", "incident_ack_review", "runbook_read"},
        "allowed": {"dashboard_read", "incident_ack_review", "runbook_read"},
        "approval_required": True,
    },
    "auditor": {
        "required": {"dashboard_read", "audit_read", "provenance_read"},
        "allowed": {"dashboard_read", "audit_read", "provenance_read"},
        "approval_required": False,
    },
}


def fail(message: str) -> None:
    raise SystemExit(f"v26 operator permission model failed: {message}")


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


def classify(artifact: dict[str, Any], case_id: str) -> dict[str, Any]:
    if artifact.get("contract_version") != CONTRACT_VERSION:
        fail(f"{case_id}: contract_version must be {CONTRACT_VERSION}")
    if artifact.get("schema_version") != SCHEMA_VERSION:
        fail(f"{case_id}: schema_version must be {SCHEMA_VERSION}")
    if artifact.get("dependency_contract") != "V260-001":
        fail(f"{case_id}: dependency_contract must be V260-001")

    reasons: list[str] = list(artifact.get("blocking_reasons") or [])
    missing_required = False
    permission_denied = False
    cross_scope = False
    expired_approval = False
    role_escalation = False
    trading_control = False

    if artifact.get("permission_artifact_scope") != PERMISSION_SCOPE:
        missing_required = True
        push_reason(reasons, "permission_artifact_scope_mismatch")
    if artifact.get("external_identity_provider_integration") is not False:
        missing_required = True
        push_reason(reasons, "external_identity_provider_integration_opened")
    if artifact.get("live_operation_authorization") is not False:
        trading_control = True
        push_reason(reasons, "live_operation_authorization_opened")
    if not isinstance(artifact.get("artifact_provenance"), dict):
        missing_required = True
        push_reason(reasons, "missing_artifact_provenance")

    evaluated_at = parse_ts(str(artifact.get("evaluated_at")))
    operators = artifact.get("operators")
    if not isinstance(operators, list) or not operators:
        fail(f"{case_id}: operators must be a non-empty list")

    roles_checked: list[str] = []
    for operator in operators:
        if not isinstance(operator, dict):
            fail(f"{case_id}: operator entry must be an object")
        role = operator.get("role") if non_empty(operator.get("role")) else "unknown"
        roles_checked.append(str(role))
        role_key = str(role)
        operator_id = str(operator.get("operator_id") or "unknown")

        if not non_empty(operator.get("operator_id")):
            missing_required = True
            push_reason(reasons, f"missing_operator_id:{role_key}")
        if role_key == "unknown":
            missing_required = True
            push_reason(reasons, f"missing_role:{role_key}")
        if not non_empty(operator.get("scope")):
            missing_required = True
            push_reason(reasons, f"missing_scope:{role_key}")
        if not non_empty(operator.get("requested_scope")):
            missing_required = True
            push_reason(reasons, f"missing_requested_scope:{role_key}")
        if not non_empty(operator.get("source_provenance")):
            missing_required = True
            push_reason(reasons, f"missing_operator_source_provenance:{role_key}")

        policy = ROLE_POLICIES.get(role_key)
        if policy is None:
            permission_denied = True
            push_reason(reasons, f"unknown_role:{role_key}")
            policy = {"required": set(), "allowed": set(), "approval_required": False}

        permissions = operator.get("permissions")
        if not isinstance(permissions, dict):
            missing_required = True
            push_reason(reasons, f"missing_permissions:{role_key}")
            permissions = {}
        for key in ALL_PERMISSION_KEYS:
            if key not in permissions:
                missing_required = True
                push_reason(reasons, f"missing_permission_field:{role_key}:{key}")

        for key in policy["required"]:
            if permissions.get(key) is not True:
                permission_denied = True
                push_reason(reasons, f"missing_required_permission:{role_key}:{key}")
        for key, value in permissions.items():
            if value is True and key not in policy["allowed"] and key not in TRADING_CONTROL_KEYS and key != "role_escalation":
                permission_denied = True
                push_reason(reasons, f"permission_not_allowed:{role_key}:{key}")

        for key in TRADING_CONTROL_KEYS:
            if permissions.get(key) is not False:
                trading_control = True
                push_reason(reasons, f"forbidden_trading_control:{role_key}:{key}")

        requested_role = operator.get("requested_role", role_key)
        if requested_role != role_key:
            role_escalation = True
            push_reason(reasons, f"role_escalation_attempt:{role_key}->{requested_role}")
        if permissions.get("role_escalation") is not False:
            role_escalation = True
            push_reason(reasons, f"forbidden_role_capability:{role_key}:role_escalation")

        scope = operator.get("scope")
        requested_scope = operator.get("requested_scope")
        if non_empty(scope) and non_empty(requested_scope) and scope != requested_scope:
            cross_scope = True
            push_reason(reasons, f"cross_scope_permission:{role_key}:{scope}->{requested_scope}")

        approval = operator.get("approval")
        if not isinstance(approval, dict):
            missing_required = True
            push_reason(reasons, f"missing_approval:{role_key}")
            approval = {}
        if policy["approval_required"]:
            if approval.get("required") is not True or approval.get("status") != "approved":
                permission_denied = True
                push_reason(reasons, f"approval_not_approved:{role_key}")
            expires_at = approval.get("expires_at")
            if not non_empty(expires_at):
                missing_required = True
                push_reason(reasons, f"missing_approval_expiry:{role_key}")
            elif parse_ts(str(expires_at)) <= evaluated_at:
                expired_approval = True
                push_reason(reasons, f"expired_approval:{role_key}")
        elif approval.get("required") is not False:
            permission_denied = True
            push_reason(reasons, f"unexpected_approval_requirement:{role_key}")

    if missing_required:
        status = "fail_closed_missing_required_evidence"
    elif role_escalation:
        status = "fail_closed_role_escalation"
    elif trading_control:
        status = "fail_closed_trading_control"
    elif cross_scope:
        status = "fail_closed_cross_scope_permission"
    elif expired_approval:
        status = "fail_closed_expired_approval"
    elif permission_denied:
        status = "fail_closed_permission_denied"
    else:
        status = "permission_evidence_ready"

    return {
        "case_id": case_id,
        "permission_artifact_scope": artifact.get("permission_artifact_scope"),
        "effective_permission_status": status,
        "operator_count": len(operators),
        "roles_checked": roles_checked,
        "permission_evidence_only": artifact.get("permission_artifact_scope") == PERMISSION_SCOPE,
        "live_operation_authorization_allowed": False,
        "external_identity_provider_integration_allowed": False,
        "trading_controls_allowed": False,
        "fail_closed": status != "permission_evidence_ready",
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
    if case_id.endswith("valid_role_matrix.001"):
        healthy_artifact = copy.deepcopy(artifact)

if selftest:
    if healthy_artifact is None:
        fail("negative selftest requires valid role matrix")
    healthy_artifact["operators"][0]["permissions"]["submit_order"] = True
    closed = classify(healthy_artifact, "negative.selftest.submit_order_opened")
    if closed["effective_permission_status"] != "fail_closed_trading_control":
        fail("negative selftest opened submit_order but did not fail closed as trading control")
    if "forbidden_trading_control:viewer:submit_order" not in closed["blocking_reasons"]:
        fail("negative selftest did not surface submit_order boundary reason")

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
        "evidence_id": "V260-002",
        "harness": HARNESS,
        "validator_entrypoint": "scripts/ai/verify_v26_operator_permission_model.sh::classify",
        "replay_type": "validator_executable_operator_permission_model",
        "classification_owner": "V260-002",
        "source_scope_owner": "V260-002",
        "permission_artifact_scope": PERMISSION_SCOPE,
    }
    for key, expected in expected_pairs.items():
        if entry.get(key) != expected:
            fail(f"{case_id}: release scope {key} mismatch: {entry.get(key)!r}")
    for key in (
        "runtime_adapter_integration",
        "complete_executable_order_control_runtime",
        "external_identity_provider_integration",
        "live_operation_authorization",
        "production_trading_authorization",
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
    "v26_operator_permission_model "
    f"status=ok trace={trace_path.as_posix()} "
    f"cases={len(rows)} roles={len(ROLE_POLICIES)} "
    f"negative_selftest={1 if selftest else 0}"
)
PY
