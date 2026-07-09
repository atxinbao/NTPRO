#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

ARTIFACT_PATH="${NTPRO_V290_PERMISSION_SOURCE_ARTIFACT:-docs/rust-cutover/release/v0_29_0_permission_source_production_readiness_artifact.json}"
CONTRACT_PATH="${NTPRO_V290_PERMISSION_SOURCE_CONTRACT:-docs/rust-cutover/release/v0_29_0_permission_source_production_readiness.md}"
TASK_PATH="${NTPRO_V290_PERMISSION_SOURCE_TASK:-docs/rust-cutover/tasks/V290-004.md}"
EVIDENCE_PATH="${NTPRO_V290_PERMISSION_SOURCE_EVIDENCE:-docs/rust-cutover/evidence/V290-004.md}"
MATRIX_PATH="${NTPRO_V290_PERMISSION_SOURCE_MATRIX:-docs/rust-cutover/release/v0_29_0_backend_production_readiness_matrix.json}"
BOUNDARY_CONTRACT_PATH="${NTPRO_V290_PERMISSION_SOURCE_BOUNDARY_CONTRACT:-docs/rust-cutover/release/v0_29_0_backend_production_readiness_boundary_contract.md}"
V280_ARTIFACT_PATH="${NTPRO_V290_PERMISSION_SOURCE_V280_ARTIFACT:-docs/rust-cutover/release/v0_28_0_identity_permission_runtime_artifact.json}"
INTAKE_PATH="${NTPRO_V290_PERMISSION_SOURCE_INTAKE:-docs/rust-cutover/release/v0_29_0_intake_gate.md}"
SELFTEST="${NTPRO_V290_PERMISSION_SOURCE_SELFTEST:-1}"

fail() {
  echo "v29 permission source production readiness failed: $*" >&2
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

require_contains "$TASK_PATH" "GitHub issue: \`#930\`"
require_contains "$EVIDENCE_PATH" "Task: \`V290-004\` / GitHub issue \`#930\`"
require_contains "$INTAKE_PATH" "start_gate_status = satisfied"
require_contains "$BOUNDARY_CONTRACT_PATH" "contract_version = ntpro.v290.backend_production_readiness_boundary.v1"
require_contains "$CONTRACT_PATH" "contract_version = ntpro.v290.permission_source_production_readiness.v1"
require_contains "$CONTRACT_PATH" "schema_version = ntpro.v290.permission_source_production_readiness_artifact.v1"
require_contains "$CONTRACT_PATH" "backend_module = permission_source_production_readiness"
require_contains "$CONTRACT_PATH" "backend_module_status = production_ready_evidence"
require_contains "$CONTRACT_PATH" "operation_authorization_surface = read_admin_only"
require_contains "$CONTRACT_PATH" "live_operation_authorization = false"
require_contains "$CONTRACT_PATH" "production_trading_authorization = false"
require_contains "$CONTRACT_PATH" "external_idp_sso_runtime_integration = false"
require_contains "$CONTRACT_PATH" "release stage = scripts/ai/verify_release.sh v29-permission-source-production-readiness"
require_contains "$CONTRACT_PATH" "forbidden_trading_permission_true => fail_closed_forbidden_trading_permission"

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

SCHEMA_VERSION = "ntpro.v290.permission_source_production_readiness_artifact.v1"
CONTRACT_VERSION = "ntpro.v290.permission_source_production_readiness.v1"
RELEASE_SCOPE = "backend_production_readiness_foundation_only"
MODULE_ID = "permission_source_production_readiness"
DEPENDENCIES = {"V290-000", "V290-001", "V290-002", "V280-002", "v0.28.1-release-evidence"}
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
    "trader_terminal_order_ticket",
    "manual_operation_submit",
    "backend_go_live_claim",
    "product_grade_trading_terminal_claim",
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
    "permission_source.production_readiness.operator_dashboard_read.allowed.001",
    "permission_source.production_readiness.admin_audit_read.allowed.001",
    "permission_source.production_readiness.missing_provenance.fail_closed.001",
    "permission_source.production_readiness.stale_source.fail_closed.001",
    "permission_source.production_readiness.revoked_subject.fail_closed.001",
    "permission_source.production_readiness.permission_drift.fail_closed.001",
    "permission_source.production_readiness.forbidden_trading_permission.fail_closed.001",
]
EXPECTED_ALLOWED = {
    "operator": {"scope_prefix": "account:", "permissions": {"dashboard_read", "operation_preview_read", "runbook_read"}},
    "admin": {"scope_prefix": "admin:", "permissions": {"dashboard_read", "operation_preview_read", "runbook_read", "audit_read", "provenance_read"}},
    "auditor": {"scope_prefix": "audit:", "permissions": {"dashboard_read", "audit_read", "provenance_read"}},
    "release_gatekeeper": {"scope_prefix": "release:", "permissions": {"dashboard_read", "release_gate_read", "release_manifest_read"}},
}


def fail(message: str) -> None:
    raise SystemExit(f"v29 permission source production readiness failed: {message}")


def non_empty(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


def merge(base: dict[str, Any], override: dict[str, Any] | None) -> dict[str, Any]:
    merged = copy.deepcopy(base)
    if override:
        for key, value in override.items():
            merged[key] = value
    return merged


def check_required_false(mapping: dict[str, Any], keys: list[str], prefix: str, reasons: list[str]) -> None:
    for key in keys:
        if key not in mapping:
            reasons.append(f"missing_required_false:{prefix}:{key}")
        elif mapping.get(key) is not False:
            reasons.append(f"forbidden_permission_opened:{prefix}:{key}")


def check_provenance(prefix: str, source: dict[str, Any], reasons: list[str]) -> None:
    if source.get("provenance_status") != "linked" and not non_empty(source.get("provenance_id")):
        reasons.append(f"missing_provenance:{prefix}")
    if source.get("provenance_status") not in (None, "linked"):
        reasons.append(f"missing_provenance:{prefix}:{source.get('provenance_status')}")
    if source.get("freshness_status") != "fresh":
        reasons.append(f"stale_source:{prefix}:{source.get('freshness_status')}")
    if source.get("redaction_status") != "redacted":
        reasons.append(f"redaction_violation:{prefix}:{source.get('redaction_status')}")
    if source.get("lineage_status") != "linked":
        reasons.append(f"lineage_violation:{prefix}:{source.get('lineage_status')}")


def classify_status(reasons: list[str]) -> str:
    if any(reason.startswith("forbidden_permission_opened") or reason.startswith("live_operation_authorization") for reason in reasons):
        return "fail_closed_forbidden_trading_permission"
    if any(reason.startswith("missing_provenance") for reason in reasons):
        return "fail_closed_missing_provenance"
    if any(reason.startswith("stale_source") for reason in reasons):
        return "fail_closed_stale_source"
    if any(reason.startswith("redaction_violation") or reason.startswith("lineage_violation") for reason in reasons):
        return "fail_closed_provenance_violation"
    if any(reason.startswith("permission_revoked") for reason in reasons):
        return "fail_closed_permission_revoked"
    if any(reason.startswith("permission_drift") or reason.startswith("unknown_role") for reason in reasons):
        return "fail_closed_permission_drift"
    if reasons:
        return "fail_closed_permission_source_violation"
    return "allowed_read_admin_permission"


def classify_artifact(artifact: dict[str, Any], case: dict[str, Any] | None = None) -> dict[str, Any]:
    case = case or {}
    reasons: list[str] = []
    if artifact.get("schema_version") != SCHEMA_VERSION or artifact.get("contract_version") != CONTRACT_VERSION:
        reasons.append("missing_provenance:schema_contract")
    if artifact.get("task_id") != "V290-004" or artifact.get("github_issue") != 930:
        reasons.append("missing_provenance:task_identity")
    if artifact.get("release_scope") != RELEASE_SCOPE or artifact.get("backend_module") != MODULE_ID:
        reasons.append("missing_provenance:module_scope")
    if artifact.get("backend_module_status") != "production_ready_evidence" or artifact.get("readiness_mode") != "deterministic_readiness_replay":
        reasons.append("missing_provenance:readiness_status")
    if artifact.get("operation_authorization_surface") != "read_admin_only" or artifact.get("permission_source_claim") != "source_controlled_sandbox_fixture":
        reasons.append("permission_drift:authorization_surface")
    if set(artifact.get("dependency_contracts") or []) != DEPENDENCIES:
        reasons.append("missing_provenance:dependency_contracts")
    for key in ("live_operation_authorization", "production_trading_authorization", "external_idp_sso_runtime_integration"):
        if artifact.get(key) is not False:
            reasons.append(f"live_operation_authorization:{key}")

    source = merge(artifact.get("permission_source") or {}, case.get("permission_source_override"))
    mapping = merge(artifact.get("permission_mapping") or {}, case.get("permission_mapping_override"))
    required_false = merge(artifact.get("required_false_permissions") or {}, case.get("required_false_permissions_override"))
    boundary = artifact.get("boundary_flags") or {}
    unsupported = artifact.get("unsupported_external_idp_sso_behavior") or {}
    revocation = artifact.get("revocation_ledger") or {}

    for key in ("source_id", "source_type", "source_ref", "provider", "tenant_id_redacted", "issuer", "jwks_fingerprint_sha256", "config_digest", "collected_at"):
        if not non_empty(source.get(key)):
            reasons.append(f"missing_provenance:source:{key}")
    if source.get("source_ref") != str(v280_artifact_path):
        reasons.append("missing_provenance:source_ref")
    check_provenance("permission_source", source, reasons)
    check_provenance("permission_mapping", mapping, reasons)
    if mapping.get("revocation_status") != "enforced":
        reasons.append("permission_drift:revocation_not_enforced")
    if revocation.get("freshness_status") != "fresh" or revocation.get("provenance_status") != "linked":
        reasons.append("missing_provenance:revocation_ledger")
    check_required_false(required_false, FORBIDDEN_PERMISSIONS, "permission", reasons)
    check_required_false(boundary, BOUNDARY_FALSE_FLAGS, "boundary", reasons)
    for key in UNSUPPORTED_SSO_FLAGS:
        if unsupported.get(key) is not False:
            reasons.append(f"live_operation_authorization:sso:{key}")

    roles = mapping.get("roles")
    if not isinstance(roles, dict) or set(roles) != set(EXPECTED_ALLOWED):
        reasons.append("permission_drift:roles")
        roles = {}
    for role, expected in EXPECTED_ALLOWED.items():
        role_entry = roles.get(role) or {}
        if role_entry.get("scope_prefix") != expected["scope_prefix"]:
            reasons.append(f"permission_drift:scope_prefix:{role}")
        if set(role_entry.get("allowed_permissions") or []) != expected["permissions"]:
            reasons.append(f"permission_drift:allowed_permissions:{role}")

    if case:
        subject = case.get("subject_id")
        role = case.get("role")
        permission = case.get("permission")
        scope = case.get("scope")
        revoked_subjects = set(revocation.get("revoked_subjects") or [])
        if subject in revoked_subjects:
            reasons.append(f"permission_revoked:{subject}")
        role_entry = roles.get(role) if isinstance(roles, dict) else None
        if not isinstance(role_entry, dict):
            reasons.append(f"unknown_role:{role}")
        else:
            if not isinstance(scope, str) or not scope.startswith(str(role_entry.get("scope_prefix"))):
                reasons.append(f"permission_drift:scope:{role}:{scope}")
            if permission not in set(role_entry.get("allowed_permissions") or []):
                reasons.append(f"permission_drift:permission:{role}:{permission}")
        if permission in FORBIDDEN_PERMISSIONS:
            reasons.append(f"forbidden_permission_opened:requested:{permission}")

    status = classify_status(reasons)
    return {"status": status, "fail_closed": status.startswith("fail_closed"), "blocking_reasons": reasons}


artifact = json.loads(artifact_path.read_text(encoding="utf-8"))
v280_artifact = json.loads(v280_artifact_path.read_text(encoding="utf-8"))
if v280_artifact.get("schema_version") != "ntpro.v280.identity_permission_runtime_artifact.v1":
    fail("v28 identity permission artifact schema mismatch")

matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
module = next((item for item in matrix.get("module_readiness", []) if item.get("module_id") == MODULE_ID), None)
if not module:
    fail("matrix missing permission_source_production_readiness")
if module.get("classification") != "production-ready":
    fail("permission source matrix entry must be production-ready")
if module.get("readiness_mode") != "deterministic_readiness_replay":
    fail("permission source matrix readiness mode mismatch")
if module.get("issue") != 930:
    fail("permission source matrix issue mismatch")
if module.get("evidence_path") != "docs/rust-cutover/evidence/V290-004.md":
    fail("permission source matrix evidence path mismatch")
if module.get("verification_command") != "scripts/ai/verify_release.sh v29-permission-source-production-readiness":
    fail("permission source matrix verification command mismatch")
if module.get("production_ready_claim_allowed") is not True:
    fail("permission source production-ready claim flag mismatch")

baseline = classify_artifact(artifact)
if baseline["status"] != "allowed_read_admin_permission":
    fail(f"baseline artifact failed: {baseline}")

cases = artifact.get("permission_readiness_cases")
if not isinstance(cases, list) or [case.get("case_id") for case in cases] != EXPECTED_CASES:
    fail("permission readiness cases mismatch")
allowed = 0
fail_closed = 0
for case in cases:
    actual = classify_artifact(artifact, case)
    if actual["status"] != case.get("expected_status"):
        fail(f"{case.get('case_id')}: expected {case.get('expected_status')} got {actual}")
    if actual["fail_closed"]:
        fail_closed += 1
    else:
        allowed += 1
if allowed != 2 or fail_closed != 5:
    fail(f"unexpected case counts: allowed={allowed} fail_closed={fail_closed}")

if selftest:
    opened = copy.deepcopy(artifact)
    opened["required_false_permissions"]["submit_order"] = True
    if classify_artifact(opened)["status"] != "fail_closed_forbidden_trading_permission":
        fail("negative self-test unexpectedly allowed submit_order")

print(
    "v29_permission_source_production_readiness=pass "
    f"cases={len(cases)} "
    f"allowed={allowed} "
    f"fail_closed={fail_closed} "
    f"required_false_permissions={len(FORBIDDEN_PERMISSIONS)} "
    f"boundary_flags={len(BOUNDARY_FALSE_FLAGS)} "
    "negative_selftest=1"
)
PY
