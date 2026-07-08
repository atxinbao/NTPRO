#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

TRACE_PATH="${NTPRO_V270_IDENTITY_PERMISSION_TRACE:-tests/golden/v270_external_identity_permission_foundation.jsonl}"
TASK_PATH="${NTPRO_V270_IDENTITY_PERMISSION_TASK:-docs/rust-cutover/tasks/V270-002.md}"
EVIDENCE_PATH="${NTPRO_V270_IDENTITY_PERMISSION_EVIDENCE:-docs/rust-cutover/evidence/V270-002.md}"
CONTRACT_PATH="${NTPRO_V270_IDENTITY_PERMISSION_CONTRACT:-docs/rust-cutover/release/v0_27_0_external_identity_permission_foundation.md}"
BOUNDARY_PATH="${NTPRO_V270_IDENTITY_PERMISSION_BOUNDARY:-docs/rust-cutover/release/v0_27_0_product_operations_runtime_integration_boundary_contract.md}"
V26_PERMISSION_PATH="${NTPRO_V270_IDENTITY_PERMISSION_V26:-docs/rust-cutover/release/v0_26_0_operator_permission_model.md}"
REPLAY_SCOPE_PATH="${NTPRO_V270_IDENTITY_PERMISSION_REPLAY_SCOPE:-docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"
SELFTEST="${NTPRO_V270_IDENTITY_PERMISSION_SELFTEST:-1}"

fail() {
  echo "v27 external identity permission foundation failed: $*" >&2
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

for path in "$TRACE_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$CONTRACT_PATH" "$BOUNDARY_PATH" "$V26_PERMISSION_PATH" "$REPLAY_SCOPE_PATH"; do
  require_file "$path"
done

require_contains "$BOUNDARY_PATH" "release_scope = product_operations_runtime_integration_foundation_only"
require_contains "$V26_PERMISSION_PATH" "permission_artifact_scope = operator_permission_evidence_only"
require_contains "$TASK_PATH" "GitHub issue: \`#855\`"
require_contains "$EVIDENCE_PATH" "Task: \`V270-002\` / GitHub issue \`#855\`"
require_contains "$CONTRACT_PATH" "identity_permission_integration_scope = external_identity_permission_foundation_only"
require_contains "$CONTRACT_PATH" "dependency_contracts = V270-001,V260-002"
require_contains "$CONTRACT_PATH" "operation_authorization_surface = read_admin_only"
require_contains "$CONTRACT_PATH" "external_identity_provider_evidence_required = true"
require_contains "$CONTRACT_PATH" "role_mapping_provenance_required = true"
require_contains "$CONTRACT_PATH" "role_mapping_freshness_required = true"
require_contains "$CONTRACT_PATH" "role_mapping_redaction_required = true"
require_contains "$CONTRACT_PATH" "role_mapping_lineage_required = true"
require_contains "$CONTRACT_PATH" "v26_permission_boundary_alignment_required = true"
require_contains "$CONTRACT_PATH" "live_operation_authorization = false"
require_contains "$CONTRACT_PATH" "production_trading_authorization = false"
require_contains "$CONTRACT_PATH" "runtime_submit_permission_enabled = false"

for marker in \
  "submit_order = false" \
  "cancel_order = false" \
  "replace_order = false" \
  "amend_order = false" \
  "flatten_position = false" \
  "retry_scheduler = false" \
  "automatic_remediation = false" \
  "adapter_send = false" \
  "live_exchange_request = false" \
  "dashboard_trading_controls = false" \
  "manual_operation_submit = false" \
  "product_grade_trading_terminal_claim = false"; do
  require_contains "$CONTRACT_PATH" "$marker"
done

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
    "read_model.external_identity_permission.valid_mapping.001",
    "read_model.external_identity_permission.missing_idp_provenance_fail_closed.001",
    "read_model.external_identity_permission.stale_role_mapping_fail_closed.001",
    "read_model.external_identity_permission.unknown_role_fail_closed.001",
    "read_model.external_identity_permission.cross_scope_action_fail_closed.001",
    "read_model.external_identity_permission.v26_boundary_mismatch_fail_closed.001",
    "read_model.external_identity_permission.trading_permission_fail_closed.001",
]
CONTRACT_VERSION = "ntpro.v270.external_identity_permission_foundation.v1"
SCHEMA_VERSION = "ntpro.v270.external_identity_permission_foundation.schema.v1"
INTEGRATION_SCOPE = "external_identity_permission_foundation_only"
HARNESS = "scripts/ai/verify_release.sh v27-external-identity-permission-foundation"
DEPENDENCIES = {"V270-001", "V260-002"}
PERMISSION_KEYS = [
    "dashboard_read",
    "operation_preview_read",
    "runbook_read",
    "release_gate_read",
    "release_manifest_read",
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
    "manual_operation_submit",
    "product_grade_trading_terminal_claim",
]
FORBIDDEN_PERMISSIONS = [
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
    "manual_operation_submit",
    "product_grade_trading_terminal_claim",
]
BOUNDARY_FALSE_FLAGS = [
    "runtime_submit_permission_enabled",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "cancel_order_permission_enabled",
    "replace_order_permission_enabled",
    "amend_order_permission_enabled",
    "flatten_position_permission_enabled",
    "retry_scheduler_permission_enabled",
    "automatic_remediation_permission_enabled",
    "adapter_send_permission_enabled",
    "live_exchange_request_permission_enabled",
    "dashboard_trading_controls_enabled",
    "manual_operation_submit_allowed",
    "product_grade_trading_terminal_claim",
]
ROLE_POLICIES = {
    "operator": {
        "required": {"dashboard_read", "operation_preview_read", "runbook_read"},
        "allowed": {"dashboard_read", "operation_preview_read", "runbook_read"},
        "scope_prefix": "account:",
    },
    "admin": {
        "required": {"dashboard_read", "operation_preview_read", "runbook_read", "audit_read", "provenance_read"},
        "allowed": {"dashboard_read", "operation_preview_read", "runbook_read", "audit_read", "provenance_read"},
        "scope_prefix": "admin:",
    },
    "auditor": {
        "required": {"dashboard_read", "audit_read", "provenance_read"},
        "allowed": {"dashboard_read", "audit_read", "provenance_read"},
        "scope_prefix": "audit:",
    },
    "release_gatekeeper": {
        "required": {"dashboard_read", "release_gate_read", "release_manifest_read"},
        "allowed": {"dashboard_read", "release_gate_read", "release_manifest_read"},
        "scope_prefix": "release:",
    },
}


def fail(message: str) -> None:
    raise SystemExit(f"v27 external identity permission foundation failed: {message}")


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


def check_idp_provenance(artifact: dict[str, Any], reasons: list[str]) -> bool:
    idp = artifact.get("idp_provenance")
    complete = isinstance(idp, dict)
    if not complete:
        push_reason(reasons, "missing_idp_provenance")
        return False
    for key in ["source_type", "provider", "tenant_id", "issuer", "jwks_fingerprint", "collected_at"]:
        if not non_empty(idp.get(key)):
            complete = False
            push_reason(reasons, f"missing_idp_provenance:{key}")
    if idp.get("source_type") != "external_identity_provider":
        complete = False
        push_reason(reasons, f"idp_source_type_mismatch:{idp.get('source_type')}")
    if idp.get("freshness_status") != "fresh":
        complete = False
        push_reason(reasons, f"idp_freshness_not_fresh:{idp.get('freshness_status')}")
    return complete


def classify(artifact: dict[str, Any], case_id: str) -> dict[str, Any]:
    if artifact.get("contract_version") != CONTRACT_VERSION:
        fail(f"{case_id}: contract_version must be {CONTRACT_VERSION}")
    if artifact.get("schema_version") != SCHEMA_VERSION:
        fail(f"{case_id}: schema_version must be {SCHEMA_VERSION}")

    reasons: list[str] = list(artifact.get("blocking_reasons") or [])
    missing_idp = False
    stale_mapping = False
    unknown_role = False
    cross_scope = False
    v26_mismatch = False
    trading_permission = False

    if artifact.get("identity_permission_integration_scope") != INTEGRATION_SCOPE:
        v26_mismatch = True
        push_reason(reasons, "identity_permission_integration_scope_mismatch")
    dependencies = artifact.get("dependency_contracts")
    if not isinstance(dependencies, list) or set(dependencies) != DEPENDENCIES:
        v26_mismatch = True
        push_reason(reasons, "dependency_contracts_mismatch")
    if artifact.get("operation_authorization_surface") != "read_admin_only":
        v26_mismatch = True
        push_reason(reasons, f"operation_authorization_surface_mismatch:{artifact.get('operation_authorization_surface')}")
    if artifact.get("live_operation_authorization") is not False:
        trading_permission = True
        push_reason(reasons, "live_operation_authorization_opened")
    if artifact.get("production_trading_authorization") is not False:
        trading_permission = True
        push_reason(reasons, "production_trading_authorization_opened")
    if artifact.get("v26_permission_boundary_alignment") is not True:
        v26_mismatch = True
        push_reason(reasons, "v26_permission_boundary_alignment_missing")

    boundary_flags = artifact.get("boundary_flags")
    if not isinstance(boundary_flags, dict):
        trading_permission = True
        push_reason(reasons, "missing_boundary_flags")
        boundary_flags = {}
    for key in BOUNDARY_FALSE_FLAGS:
        if key not in boundary_flags:
            trading_permission = True
            push_reason(reasons, f"missing_required_false_boundary:{key}")
        elif boundary_flags.get(key) is not False:
            trading_permission = True
            push_reason(reasons, f"forbidden_boundary_flag:{key}")

    missing_idp = not check_idp_provenance(artifact, reasons)

    role_mapping = artifact.get("role_mapping")
    if not isinstance(role_mapping, dict):
        stale_mapping = True
        push_reason(reasons, "missing_role_mapping")
        role_mapping = {}
    for key in ["mapping_id", "source_ref", "collected_at", "lineage_ref"]:
        if not non_empty(role_mapping.get(key)):
            v26_mismatch = True
            push_reason(reasons, f"missing_role_mapping_field:{key}")
    if role_mapping.get("freshness_status") != "fresh":
        stale_mapping = True
        push_reason(reasons, f"role_mapping_freshness_not_fresh:{role_mapping.get('freshness_status')}")
    if role_mapping.get("redaction_status") != "redacted":
        v26_mismatch = True
        push_reason(reasons, f"role_mapping_redaction_not_redacted:{role_mapping.get('redaction_status')}")

    entries = role_mapping.get("entries")
    if not isinstance(entries, list) or not entries:
        v26_mismatch = True
        push_reason(reasons, "missing_role_mapping_entries")
        entries = []

    roles_checked: list[str] = []
    for entry in entries:
        if not isinstance(entry, dict):
            fail(f"{case_id}: role mapping entry must be an object")
        role = str(entry.get("mapped_role") or "unknown")
        roles_checked.append(role)
        policy = ROLE_POLICIES.get(role)
        if policy is None:
            unknown_role = True
            push_reason(reasons, f"unknown_role:{role}")
            policy = {"required": set(), "allowed": set(), "scope_prefix": ""}

        for key in ["principal_id", "external_subject", "external_group", "scope", "requested_scope", "requested_action", "source_provenance"]:
            if not non_empty(entry.get(key)):
                v26_mismatch = True
                push_reason(reasons, f"missing_mapping_field:{role}:{key}")
        if entry.get("mapping_freshness_status") != "fresh":
            stale_mapping = True
            push_reason(reasons, f"mapping_freshness_not_fresh:{role}:{entry.get('mapping_freshness_status')}")

        scope = str(entry.get("scope") or "")
        requested_scope = str(entry.get("requested_scope") or "")
        scope_prefix = str(policy["scope_prefix"])
        if scope_prefix and not scope.startswith(scope_prefix):
            v26_mismatch = True
            push_reason(reasons, f"scope_prefix_mismatch:{role}:{scope}")
        if scope != requested_scope:
            cross_scope = True
            push_reason(reasons, f"cross_scope_action:{role}:{scope}->{requested_scope}")

        permissions = entry.get("permissions")
        if not isinstance(permissions, dict):
            v26_mismatch = True
            push_reason(reasons, f"missing_permissions:{role}")
            permissions = {}
        for key in PERMISSION_KEYS:
            if key not in permissions:
                v26_mismatch = True
                push_reason(reasons, f"missing_permission_field:{role}:{key}")
        for key in policy["required"]:
            if permissions.get(key) is not True:
                v26_mismatch = True
                push_reason(reasons, f"missing_required_permission:{role}:{key}")
        for key, value in permissions.items():
            if value is not True:
                continue
            if key in FORBIDDEN_PERMISSIONS:
                trading_permission = True
                push_reason(reasons, f"forbidden_trading_permission:{role}:{key}")
            elif key not in policy["allowed"]:
                v26_mismatch = True
                push_reason(reasons, f"permission_not_allowed:{role}:{key}")

    if missing_idp:
        effective_status = "fail_closed_missing_idp_provenance"
    elif stale_mapping:
        effective_status = "fail_closed_stale_role_mapping"
    elif unknown_role:
        effective_status = "fail_closed_unknown_role"
    elif cross_scope:
        effective_status = "fail_closed_cross_scope_action"
    elif v26_mismatch:
        effective_status = "fail_closed_v26_boundary_mismatch"
    elif trading_permission:
        effective_status = "fail_closed_trading_permission"
    elif reasons:
        effective_status = "fail_closed_v26_boundary_mismatch"
    else:
        effective_status = "identity_permission_foundation_ready"

    return {
        "identity_permission_integration_scope": artifact.get("identity_permission_integration_scope"),
        "effective_identity_permission_status": effective_status,
        "mapping_count": len(entries),
        "roles_checked": roles_checked,
        "idp_provenance_complete": not missing_idp,
        "role_mapping_fresh": not stale_mapping,
        "v26_boundary_aligned": not v26_mismatch,
        "read_admin_only": artifact.get("operation_authorization_surface") == "read_admin_only",
        "live_operation_authorization_allowed": False,
        "trading_permissions_allowed": False,
        "fail_closed": effective_status != "identity_permission_foundation_ready",
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
    if input_event.get("event_type") != "read_model.external_identity_permission.input":
        fail(f"{case_id}: unexpected input event_type")
    if expected_event.get("event_type") != "read_model.external_identity_permission.validated":
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
    mutated["role_mapping"]["entries"][0]["permissions"]["submit_order"] = True
    status = classify(mutated, "negative_selftest")["effective_identity_permission_status"]
    if status != "fail_closed_trading_permission":
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
    if entry.get("evidence_id") != "V270-002":
        fail(f"{case_id}: replay scope evidence_id must be V270-002")
    if entry.get("harness") != HARNESS:
        fail(f"{case_id}: replay scope harness mismatch")
    if entry.get("runtime_adapter_integration") is not False:
        fail(f"{case_id}: runtime_adapter_integration must be false")
    if entry.get("live_exchange_request_allowed") is not False:
        fail(f"{case_id}: live_exchange_request_allowed must be false")
    if entry.get("adapter_send_allowed") is not False:
        fail(f"{case_id}: adapter_send_allowed must be false")
    if entry.get("dashboard_trading_controls_enabled") is not False:
        fail(f"{case_id}: dashboard_trading_controls_enabled must be false")

print(
    "v27_external_identity_permission_foundation=pass "
    f"cases={len(rows)} roles=4 required_false_permissions={len(FORBIDDEN_PERMISSIONS)} "
    f"boundary_flags={len(BOUNDARY_FALSE_FLAGS)} negative_selftest={1 if selftest else 0}"
)
PY
