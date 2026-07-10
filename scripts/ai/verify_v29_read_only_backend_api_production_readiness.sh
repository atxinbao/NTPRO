#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

ARTIFACT_PATH="${NTPRO_V290_READONLY_API_ARTIFACT:-docs/rust-cutover/release/v0_29_0_read_only_backend_api_production_readiness_artifact.json}"
CONTRACT_PATH="${NTPRO_V290_READONLY_API_CONTRACT:-docs/rust-cutover/release/v0_29_0_read_only_backend_api_production_readiness.md}"
TASK_PATH="${NTPRO_V290_READONLY_API_TASK:-docs/rust-cutover/tasks/V290-005.md}"
EVIDENCE_PATH="${NTPRO_V290_READONLY_API_EVIDENCE:-docs/rust-cutover/evidence/V290-005.md}"
MATRIX_PATH="${NTPRO_V290_READONLY_API_MATRIX:-docs/rust-cutover/release/v0_29_0_backend_production_readiness_matrix.json}"
BOUNDARY_CONTRACT_PATH="${NTPRO_V290_READONLY_API_BOUNDARY_CONTRACT:-docs/rust-cutover/release/v0_29_0_backend_production_readiness_boundary_contract.md}"
V280_ARTIFACT_PATH="${NTPRO_V290_READONLY_API_V280_ARTIFACT:-docs/rust-cutover/release/v0_28_0_trader_terminal_backend_api_contract_artifact.json}"
INTAKE_PATH="${NTPRO_V290_READONLY_API_INTAKE:-docs/rust-cutover/release/v0_29_0_intake_gate.md}"
SELFTEST="${NTPRO_V290_READONLY_API_SELFTEST:-1}"

fail() {
  echo "v29 read-only backend API production readiness failed: $*" >&2
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

require_contains "$TASK_PATH" "GitHub issue: \`#931\`"
require_contains "$EVIDENCE_PATH" "Task: \`V290-005\` / GitHub issue \`#931\`"
require_contains "$INTAKE_PATH" "start_gate_status = satisfied"
require_contains "$BOUNDARY_CONTRACT_PATH" "contract_version = ntpro.v290.backend_production_readiness_boundary.v1"
require_contains "$CONTRACT_PATH" "contract_version = ntpro.v290.read_only_backend_api_production_readiness.v1"
require_contains "$CONTRACT_PATH" "schema_version = ntpro.v290.read_only_backend_api_production_readiness_artifact.v1"
require_contains "$CONTRACT_PATH" "GET /api/v29/readiness/status"
require_contains "$CONTRACT_PATH" "GET /api/v29/runbooks/status"
require_contains "$CONTRACT_PATH" "release stage = scripts/ai/verify_release.sh v29-read-only-backend-api-production-readiness"

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

SCHEMA_VERSION = "ntpro.v290.read_only_backend_api_production_readiness_artifact.v1"
CONTRACT_VERSION = "ntpro.v290.read_only_backend_api_production_readiness.v1"
MODULE_ID = "read_only_backend_api_production_readiness"
DEPENDENCIES = {"V290-000", "V290-001", "V290-002", "V290-003", "V290-004", "V280-007", "v0.28.1-release-evidence"}
EXPECTED_API_CONTRACTS = {
    "readiness_status": "GET /api/v29/readiness/status",
    "provenance_drilldown": "GET /api/v29/provenance/drilldown",
    "audit_entries": "GET /api/v29/audit/entries",
    "telemetry_health": "GET /api/v29/telemetry/health",
    "permission_snapshot": "GET /api/v29/permissions/snapshot",
    "deployment_state": "GET /api/v29/deployment/state",
    "runbook_status": "GET /api/v29/runbooks/status",
}
EXPECTED_CASES = [
    "read_only_backend_api.production_readiness.ready.allowed.001",
    "read_only_backend_api.production_readiness.stale.degraded.001",
    "read_only_backend_api.production_readiness.partial.degraded.001",
    "read_only_backend_api.production_readiness.unauthorized.fail_closed.001",
    "read_only_backend_api.production_readiness.missing_source.fail_closed.001",
    "read_only_backend_api.production_readiness.malformed_response.fail_closed.001",
    "read_only_backend_api.production_readiness.unredacted_payload.fail_closed.001",
    "read_only_backend_api.production_readiness.forbidden_controls.fail_closed.001",
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


def fail(message: str) -> None:
    raise SystemExit(f"v29 read-only backend API production readiness failed: {message}")


def merge(base: Any, override: Any) -> Any:
    if isinstance(base, dict) and isinstance(override, dict):
        result = copy.deepcopy(base)
        for key, value in override.items():
            result[key] = merge(result.get(key), value)
        return result
    return copy.deepcopy(override)


def apply_case(artifact: dict[str, Any], case: dict[str, Any]) -> dict[str, Any]:
    result = copy.deepcopy(artifact)
    for api_id, override in (case.get("api_overrides") or {}).items():
        result["api_contracts"][api_id] = merge(result["api_contracts"].get(api_id, {}), override)
    if case.get("boundary_flags_override"):
        result["boundary_flags"] = merge(result["boundary_flags"], case["boundary_flags_override"])
    return result


def classify_status(reasons: list[str]) -> str:
    if any(reason.startswith("unauthorized") for reason in reasons):
        return "fail_closed_unauthorized_response"
    if any(reason.startswith("missing_source") for reason in reasons):
        return "fail_closed_missing_source"
    if any(reason.startswith("unredacted") for reason in reasons):
        return "fail_closed_unredacted_payload"
    if any(reason.startswith("forbidden") for reason in reasons):
        return "fail_closed_forbidden_controls"
    if any(reason.startswith("malformed") for reason in reasons):
        return "fail_closed_malformed_response"
    if any(reason.startswith("stale") for reason in reasons):
        return "degraded_stale_response"
    if any(reason.startswith("partial") for reason in reasons):
        return "degraded_partial_response"
    if reasons:
        return "fail_closed_malformed_response"
    return "read_only_backend_api_ready"


def classify_artifact(artifact: dict[str, Any]) -> dict[str, Any]:
    reasons: list[str] = []
    if artifact.get("schema_version") != SCHEMA_VERSION or artifact.get("contract_version") != CONTRACT_VERSION:
        reasons.append("malformed:schema_contract")
    if artifact.get("task_id") != "V290-005" or artifact.get("github_issue") != 931:
        reasons.append("malformed:task_identity")
    if artifact.get("backend_module") != MODULE_ID or artifact.get("backend_module_status") != "production_ready_evidence":
        reasons.append("malformed:module")
    if artifact.get("readiness_mode") != "deterministic_readiness_replay":
        reasons.append("malformed:readiness_mode")
    if set(artifact.get("dependency_contracts") or []) != DEPENDENCIES:
        reasons.append("malformed:dependencies")
    if artifact.get("handoff_mode") != "deterministic_read_only_backend_api_contract":
        reasons.append("malformed:handoff_mode")

    surface = artifact.get("api_surface") or {}
    if surface.get("read_only") is not True or surface.get("artifact_driven") is not True:
        reasons.append("forbidden:surface_not_readonly")
    for key in ("order_ticket_enabled", "submit_controls_enabled", "operation_controls_enabled", "trading_controls_enabled", "product_grade_terminal_claim_allowed"):
        if surface.get(key) is not False:
            reasons.append(f"forbidden:surface:{key}")

    boundary = artifact.get("boundary_flags") or {}
    for key in BOUNDARY_FALSE_FLAGS:
        if boundary.get(key) is not False:
            reasons.append(f"forbidden:boundary:{key}")

    contracts = artifact.get("api_contracts")
    if not isinstance(contracts, dict) or set(contracts) != set(EXPECTED_API_CONTRACTS):
        reasons.append("malformed:api_contract_set")
        contracts = contracts if isinstance(contracts, dict) else {}
    for api_id, endpoint in EXPECTED_API_CONTRACTS.items():
        contract = contracts.get(api_id) or {}
        if contract.get("endpoint_id") != endpoint:
            reasons.append(f"malformed:endpoint:{api_id}")
        if contract.get("authorization") != "read_admin_only":
            reasons.append(f"unauthorized:{api_id}:{contract.get('authorization')}")
        if contract.get("allowed_methods") != ["GET"] or contract.get("read_only") is not True or contract.get("mutating_methods_allowed") is not False:
            reasons.append(f"forbidden:methods:{api_id}")
        schema = contract.get("response_schema") or {}
        if not schema.get("schema_id") or not schema.get("required_fields") or schema.get("migration_note_required_for_breaking_change") is not True:
            reasons.append(f"malformed:schema:{api_id}")
        pagination = contract.get("pagination") or {}
        if not isinstance(pagination.get("max_page_size"), int) or pagination.get("max_page_size") < 1 or not isinstance(pagination.get("max_response_bytes"), int) or pagination.get("max_response_bytes") < 1:
            reasons.append(f"malformed:pagination:{api_id}")
        redaction = contract.get("redaction") or {}
        if redaction.get("redaction_status") != "redacted" or redaction.get("raw_payload_allowed") is not False:
            reasons.append(f"unredacted:{api_id}")
        freshness = contract.get("freshness") or {}
        if freshness.get("source_freshness_status") != "fresh":
            reasons.append(f"stale:{api_id}")
        if contract.get("contract_status") == "partial":
            reasons.append(f"partial:{api_id}")
        semantics = contract.get("failure_semantics") or {}
        for key in ("missing_source", "malformed_response", "stale_source", "unauthorized", "forbidden_controls"):
            if key not in semantics:
                reasons.append(f"malformed:failure_semantics:{api_id}:{key}")
        source_refs = contract.get("source_refs")
        if not isinstance(source_refs, list) or not source_refs or any(not isinstance(ref, str) or not ref.strip() for ref in source_refs):
            reasons.append(f"missing_source:{api_id}")
        for key in ("operation_controls_enabled", "trading_controls_enabled", "order_ticket_enabled"):
            if contract.get(key) is not False:
                reasons.append(f"forbidden:contract:{api_id}:{key}")
    status = classify_status(reasons)
    return {"status": status, "fail_closed": status.startswith("fail_closed"), "degraded": status.startswith("degraded"), "blocking_reasons": reasons}


artifact = json.loads(artifact_path.read_text(encoding="utf-8"))
v280_artifact = json.loads(v280_artifact_path.read_text(encoding="utf-8"))
if v280_artifact.get("schema_version") != "ntpro.v280.trader_terminal_backend_api_contract_artifact.v1":
    fail("v28 backend API artifact schema mismatch")

matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
module = next((item for item in matrix.get("module_readiness", []) if item.get("module_id") == MODULE_ID), None)
if not module:
    fail("matrix missing read_only_backend_api_production_readiness")
if module.get("classification") != "production-ready" or module.get("issue") != 931:
    fail("read-only backend API matrix entry mismatch")
if module.get("evidence_path") != "docs/rust-cutover/evidence/V290-005.md":
    fail("read-only backend API matrix evidence path mismatch")
if module.get("verification_command") != "scripts/ai/verify_release.sh v29-read-only-backend-api-production-readiness":
    fail("read-only backend API matrix verification command mismatch")

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
if allowed != 1 or degraded != 2 or fail_closed != 5:
    fail(f"unexpected case counts: allowed={allowed} degraded={degraded} fail_closed={fail_closed}")

if selftest:
    bad = copy.deepcopy(artifact)
    bad["boundary_flags"]["trader_terminal_order_ticket_enabled"] = True
    if classify_artifact(bad)["status"] != "fail_closed_forbidden_controls":
        fail("negative self-test unexpectedly allowed order ticket")

print(
    "v29_read_only_backend_api_production_readiness=pass "
    f"cases={len(cases)} "
    f"allowed={allowed} "
    f"degraded={degraded} "
    f"fail_closed={fail_closed} "
    f"api_contracts={len(EXPECTED_API_CONTRACTS)} "
    f"boundary_flags={len(BOUNDARY_FALSE_FLAGS)} "
    "negative_selftest=1"
)
PY
