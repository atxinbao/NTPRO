#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

ARTIFACT_PATH="${NTPRO_V280_IDENTITY_PERMISSION_ARTIFACT:-docs/rust-cutover/release/v0_28_0_identity_permission_runtime_artifact.json}"
CONTRACT_PATH="${NTPRO_V280_IDENTITY_PERMISSION_CONTRACT:-docs/rust-cutover/release/v0_28_0_identity_permission_runtime_closure.md}"
TASK_PATH="${NTPRO_V280_IDENTITY_PERMISSION_TASK:-docs/rust-cutover/tasks/V280-002.md}"
EVIDENCE_PATH="${NTPRO_V280_IDENTITY_PERMISSION_EVIDENCE:-docs/rust-cutover/evidence/V280-002.md}"
MATRIX_PATH="${NTPRO_V280_IDENTITY_PERMISSION_MATRIX:-docs/rust-cutover/release/v0_28_0_backend_closure_readiness_matrix.json}"
BOUNDARY_PATH="${NTPRO_V280_IDENTITY_PERMISSION_BOUNDARY:-docs/rust-cutover/release/v0_28_0_backend_closure_boundary_contract.md}"
FOUNDATION_PATH="${NTPRO_V280_IDENTITY_PERMISSION_FOUNDATION:-docs/rust-cutover/release/v0_27_0_external_identity_permission_foundation.md}"
SELFTEST="${NTPRO_V280_IDENTITY_PERMISSION_SELFTEST:-1}"

fail() {
  echo "v28 identity permission runtime closure failed: $*" >&2
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

for path in "$ARTIFACT_PATH" "$CONTRACT_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$MATRIX_PATH" "$BOUNDARY_PATH" "$FOUNDATION_PATH"; do
  require_file "$path"
done

require_contains "$TASK_PATH" "GitHub issue: \`#895\`"
require_contains "$EVIDENCE_PATH" "Task: \`V280-002\` / GitHub issue \`#895\`"
require_contains "$BOUNDARY_PATH" "contract_version = ntpro.v280.backend_closure_boundary.v1"
require_contains "$FOUNDATION_PATH" "identity_permission_integration_scope = external_identity_permission_foundation_only"
require_contains "$CONTRACT_PATH" "contract_version = ntpro.v280.identity_permission_runtime_closure.v1"
require_contains "$CONTRACT_PATH" "schema_version = ntpro.v280.identity_permission_runtime_artifact.v1"
require_contains "$CONTRACT_PATH" "backend_module_status = runtime_closed"
require_contains "$CONTRACT_PATH" "operation_authorization_surface = read_admin_only"
require_contains "$CONTRACT_PATH" "live_operation_authorization = false"
require_contains "$CONTRACT_PATH" "production_trading_authorization = false"
require_contains "$CONTRACT_PATH" "external_idp_sso_runtime_integration = false"
require_contains "$CONTRACT_PATH" "release stage = scripts/ai/verify_release.sh v28-identity-permission-runtime-closure"

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
  "admin_workbench_trading_controls = false" \
  "manual_operation_submit = false" \
  "product_grade_trading_terminal_claim = false"; do
  require_contains "$CONTRACT_PATH" "$marker"
done

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

SCHEMA_VERSION = "ntpro.v280.identity_permission_runtime_artifact.v1"
CONTRACT_VERSION = "ntpro.v280.identity_permission_runtime_closure.v1"
RELEASE_SCOPE = "backend_closure_product_operations_runtime_finalization_only"
DEPENDENCIES = {"V280-001", "V280-000", "V270-002", "V260-002"}
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
    "admin_workbench_trading_controls",
    "manual_operation_submit",
    "product_grade_trading_terminal_claim",
]
UNSUPPORTED_SSO_FLAGS = [
    "live_sso_token_verification",
    "oauth_authorization_code_flow",
    "saml_assertion_validation",
    "external_idp_network_call",
    "dynamic_group_sync",
    "live_operation_authorization_from_idp",
]
EXPECTED_CASES = [
    "identity_permission_runtime.operator_dashboard_read.allowed.001",
    "identity_permission_runtime.admin_audit_read.allowed.001",
    "identity_permission_runtime.missing_provenance.fail_closed.001",
    "identity_permission_runtime.stale_source.fail_closed.001",
    "identity_permission_runtime.permission_drift.fail_closed.001",
    "identity_permission_runtime.forbidden_trading_permission.fail_closed.001",
]
EXPECTED_ALLOWED = {
    "operator": {"scope_prefix": "account:", "permissions": {"dashboard_read", "operation_preview_read", "runbook_read"}},
    "admin": {"scope_prefix": "admin:", "permissions": {"dashboard_read", "operation_preview_read", "runbook_read", "audit_read", "provenance_read"}},
    "auditor": {"scope_prefix": "audit:", "permissions": {"dashboard_read", "audit_read", "provenance_read"}},
    "release_gatekeeper": {"scope_prefix": "release:", "permissions": {"dashboard_read", "release_gate_read", "release_manifest_read"}},
}


def fail(message: str) -> None:
    raise SystemExit(f"v28 identity permission runtime closure failed: {message}")


def non_empty(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


def merge(base: dict[str, Any], override: dict[str, Any] | None) -> dict[str, Any]:
    merged = copy.deepcopy(base)
    if override:
        for key, value in override.items():
            merged[key] = value
    return merged


def check_required_false(mapping: dict[str, Any], reasons: list[str]) -> None:
    for key in FORBIDDEN_PERMISSIONS:
        if key not in mapping:
            reasons.append(f"missing_required_false_permission:{key}")
        elif mapping.get(key) is not False:
            reasons.append(f"forbidden_permission_opened:{key}")


def check_provenance(prefix: str, source: dict[str, Any], reasons: list[str]) -> None:
    if not non_empty(source.get("provenance_id")):
        reasons.append(f"missing_provenance:{prefix}")
    if source.get("freshness_status") != "fresh":
        reasons.append(f"stale_source:{prefix}:{source.get('freshness_status')}")
    if source.get("redaction_status") != "redacted":
        reasons.append(f"redaction_violation:{prefix}:{source.get('redaction_status')}")
    if source.get("lineage_status") != "linked":
        reasons.append(f"lineage_violation:{prefix}:{source.get('lineage_status')}")


def classify_status(reasons: list[str]) -> str:
    if any(reason.startswith("forbidden_permission_opened") for reason in reasons):
        return "fail_closed_forbidden_trading_permission"
    if any(reason.startswith("live_operation_authorization") for reason in reasons):
        return "fail_closed_forbidden_trading_permission"
    if any(reason.startswith("missing_provenance") for reason in reasons):
        return "fail_closed_missing_provenance"
    if any(reason.startswith("stale_source") for reason in reasons):
        return "fail_closed_stale_source"
    if any(reason.startswith("redaction_violation") or reason.startswith("lineage_violation") for reason in reasons):
        return "fail_closed_provenance_violation"
    if any(reason.startswith("permission_drift") or reason.startswith("unknown_role") for reason in reasons):
        return "fail_closed_permission_drift"
    if reasons:
        return "fail_closed_permission_runtime_violation"
    return "allowed_read_admin_permission"


def validate_artifact_baseline(artifact: dict[str, Any]) -> None:
    if artifact.get("schema_version") != SCHEMA_VERSION:
        fail("schema version mismatch")
    if artifact.get("contract_version") != CONTRACT_VERSION:
        fail("contract version mismatch")
    if artifact.get("task_id") != "V280-002" or artifact.get("github_issue") != 895:
        fail("task identity mismatch")
    if artifact.get("release_scope") != RELEASE_SCOPE:
        fail("release scope mismatch")
    if artifact.get("backend_module") != "identity_permission_runtime_closure":
        fail("backend module mismatch")
    if artifact.get("backend_module_status") != "runtime_closed":
        fail("backend module must be runtime_closed")
    if artifact.get("operation_authorization_surface") != "read_admin_only":
        fail("operation authorization surface must be read_admin_only")
    if set(artifact.get("dependency_contracts") or []) != DEPENDENCIES:
        fail("dependency contracts mismatch")
    for key in ("live_operation_authorization", "production_trading_authorization", "external_idp_sso_runtime_integration"):
        if artifact.get(key) is not False:
            fail(f"{key} must be false")
    unsupported = artifact.get("unsupported_external_idp_sso_behavior")
    if not isinstance(unsupported, dict):
        fail("unsupported_external_idp_sso_behavior must be an object")
    for key in UNSUPPORTED_SSO_FLAGS:
        if unsupported.get(key) is not False:
            fail(f"unsupported SSO flag must be false: {key}")
    source = artifact.get("identity_source")
    if not isinstance(source, dict):
        fail("identity_source must be an object")
    for key in ("provider", "tenant_id_redacted", "issuer", "jwks_fingerprint_sha256", "collected_at", "provenance_id"):
        if not non_empty(source.get(key)):
            fail(f"identity_source.{key} is required")
    mapping = artifact.get("permission_mapping")
    if not isinstance(mapping, dict):
        fail("permission_mapping must be an object")
    roles = mapping.get("roles")
    if not isinstance(roles, dict) or set(roles) != set(EXPECTED_ALLOWED):
        fail("permission mapping roles mismatch")
    for role, expected in EXPECTED_ALLOWED.items():
        role_entry = roles.get(role) or {}
        if role_entry.get("scope_prefix") != expected["scope_prefix"]:
            fail(f"{role}: scope prefix mismatch")
        if set(role_entry.get("allowed_permissions") or []) != expected["permissions"]:
            fail(f"{role}: allowed permissions mismatch")
    required_false = artifact.get("required_false_permissions")
    if not isinstance(required_false, dict):
        fail("required_false_permissions must be an object")
    reasons: list[str] = []
    check_required_false(required_false, reasons)
    check_provenance("identity_source", source, reasons)
    check_provenance("permission_mapping", mapping, reasons)
    if reasons:
        fail("baseline artifact violates runtime closure: " + ",".join(reasons))


def replay_case(artifact: dict[str, Any], case: dict[str, Any]) -> dict[str, Any]:
    source = merge(artifact["identity_source"], case.get("identity_source_override"))
    mapping = merge(artifact["permission_mapping"], case.get("permission_mapping_override"))
    required_false = merge(artifact["required_false_permissions"], case.get("required_false_permissions_override"))
    reasons: list[str] = []
    check_provenance("identity_source", source, reasons)
    check_provenance("permission_mapping", mapping, reasons)
    check_required_false(required_false, reasons)
    if artifact.get("live_operation_authorization") is not False:
        reasons.append("live_operation_authorization_opened")
    if artifact.get("production_trading_authorization") is not False:
        reasons.append("live_operation_authorization_opened")

    role = case.get("role")
    permission = case.get("permission")
    scope = case.get("scope")
    roles = mapping.get("roles") if isinstance(mapping, dict) else {}
    role_entry = roles.get(role) if isinstance(roles, dict) else None
    if not isinstance(role_entry, dict):
        reasons.append(f"unknown_role:{role}")
    else:
        if not isinstance(scope, str) or not scope.startswith(str(role_entry.get("scope_prefix"))):
            reasons.append(f"permission_drift:scope:{role}:{scope}")
        allowed = set(role_entry.get("allowed_permissions") or [])
        if permission not in allowed:
            reasons.append(f"permission_drift:permission:{role}:{permission}")
    if permission in FORBIDDEN_PERMISSIONS:
        reasons.append(f"forbidden_permission_opened:{permission}")

    return {
        "case_id": case.get("case_id"),
        "status": classify_status(reasons),
        "fail_closed": bool(reasons),
        "blocking_reasons": reasons,
    }


artifact = json.loads(artifact_path.read_text(encoding="utf-8"))
validate_artifact_baseline(artifact)
matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
module = next((item for item in matrix.get("module_readiness", []) if item.get("module_id") == "identity_permission_runtime_closure"), None)
if not module:
    fail("matrix missing identity_permission_runtime_closure")
if module.get("classification") != "runtime-closed":
    fail("identity_permission_runtime_closure must be runtime-closed in matrix")
if module.get("closure_claim_allowed") is not True:
    fail("identity_permission_runtime_closure matrix claim must be allowed")
if module.get("evidence_path") != "docs/rust-cutover/evidence/V280-002.md":
    fail("identity_permission_runtime_closure evidence path mismatch")
if module.get("verification_command") != "scripts/ai/verify_release.sh v28-identity-permission-runtime-closure":
    fail("identity_permission_runtime_closure verification command mismatch")

cases = artifact.get("permission_replay_cases")
if not isinstance(cases, list):
    fail("permission_replay_cases must be a list")
if [case.get("case_id") for case in cases] != EXPECTED_CASES:
    fail("permission replay case order mismatch")

allowed = 0
fail_closed = 0
for case in cases:
    actual = replay_case(artifact, case)
    expected = case.get("expected_status")
    if actual["status"] != expected:
        fail(f"{case.get('case_id')}: expected {expected}, got {actual}")
    if actual["fail_closed"]:
        fail_closed += 1
    else:
        allowed += 1

if allowed != 2 or fail_closed != 4:
    fail(f"unexpected replay counts: allowed={allowed} fail_closed={fail_closed}")

if selftest:
    opened = copy.deepcopy(artifact)
    opened["required_false_permissions"]["adapter_send"] = True
    case = copy.deepcopy(cases[0])
    case["permission"] = "adapter_send"
    if replay_case(opened, case)["status"] != "fail_closed_forbidden_trading_permission":
        fail("negative self-test unexpectedly allowed adapter_send")

    stale = copy.deepcopy(artifact)
    stale["identity_source"]["freshness_status"] = "stale"
    if replay_case(stale, cases[0])["status"] != "fail_closed_stale_source":
        fail("negative self-test unexpectedly allowed stale identity source")

print(
    "v28_identity_permission_runtime_closure=pass "
    f"cases={len(cases)} allowed={allowed} fail_closed={fail_closed} "
    f"required_false_permissions={len(FORBIDDEN_PERMISSIONS)} negative_selftest={int(selftest)}"
)
PY
