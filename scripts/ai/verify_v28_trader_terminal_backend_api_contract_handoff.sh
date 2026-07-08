#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

ARTIFACT_PATH="${NTPRO_V280_TRADER_TERMINAL_API_ARTIFACT:-docs/rust-cutover/release/v0_28_0_trader_terminal_backend_api_contract_artifact.json}"
CONTRACT_PATH="${NTPRO_V280_TRADER_TERMINAL_API_CONTRACT:-docs/rust-cutover/release/v0_28_0_trader_terminal_backend_api_contract_handoff.md}"
TASK_PATH="${NTPRO_V280_TRADER_TERMINAL_API_TASK:-docs/rust-cutover/tasks/V280-007.md}"
EVIDENCE_PATH="${NTPRO_V280_TRADER_TERMINAL_API_EVIDENCE:-docs/rust-cutover/evidence/V280-007.md}"
MATRIX_PATH="${NTPRO_V280_TRADER_TERMINAL_API_MATRIX:-docs/rust-cutover/release/v0_28_0_backend_closure_readiness_matrix.json}"
SELFTEST="${NTPRO_V280_TRADER_TERMINAL_API_SELFTEST:-1}"

fail() {
  echo "v28 Trader Terminal backend API contract handoff failed: $*" >&2
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

for path in "$ARTIFACT_PATH" "$CONTRACT_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$MATRIX_PATH"; do
  require_file "$path"
done

require_contains "$TASK_PATH" "GitHub issue: \`#900\`"
require_contains "$EVIDENCE_PATH" "Task: \`V280-007\` / GitHub issue \`#900\`"
require_contains "$CONTRACT_PATH" "contract_version = ntpro.v280.trader_terminal_backend_api_contract_handoff.v1"
require_contains "$CONTRACT_PATH" "schema_version = ntpro.v280.trader_terminal_backend_api_contract_artifact.v1"
require_contains "$CONTRACT_PATH" "backend_module_status = runtime_closed"
require_contains "$CONTRACT_PATH" "release stage = scripts/ai/verify_release.sh v28-trader-terminal-backend-api-contract-handoff"
require_contains "$CONTRACT_PATH" "GET /api/v28/backend-closure/status"
require_contains "$CONTRACT_PATH" "GET /api/v28/provenance/drilldown"
require_contains "$CONTRACT_PATH" "GET /api/v28/audit/entries"
require_contains "$CONTRACT_PATH" "GET /api/v28/telemetry/health"
require_contains "$CONTRACT_PATH" "GET /api/v28/permissions/snapshot"
require_contains "$CONTRACT_PATH" "GET /api/v28/deployment/state"

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

SCHEMA_VERSION = "ntpro.v280.trader_terminal_backend_api_contract_artifact.v1"
CONTRACT_VERSION = "ntpro.v280.trader_terminal_backend_api_contract_handoff.v1"
DEPENDENCIES = {
    "V280-001",
    "V280-002",
    "V280-003",
    "V280-004",
    "V280-005",
    "V280-006",
    "V220-007",
    "V250-006",
    "V260-007",
}
EXPECTED_API_CONTRACTS = {
    "backend_closure_status": "GET /api/v28/backend-closure/status",
    "provenance_drilldown": "GET /api/v28/provenance/drilldown",
    "audit_entries": "GET /api/v28/audit/entries",
    "telemetry_health": "GET /api/v28/telemetry/health",
    "permission_snapshot": "GET /api/v28/permissions/snapshot",
    "deployment_state": "GET /api/v28/deployment/state",
}
EXPECTED_CASES = [
    "trader_terminal_backend_api_contract.ready.allowed.001",
    "trader_terminal_backend_api_contract.stale.degraded.001",
    "trader_terminal_backend_api_contract.partial.degraded.001",
    "trader_terminal_backend_api_contract.missing_source.fail_closed.001",
    "trader_terminal_backend_api_contract.malformed_response.fail_closed.001",
    "trader_terminal_backend_api_contract.unredacted_payload.fail_closed.001",
    "trader_terminal_backend_api_contract.forbidden_controls.fail_closed.001",
]
BOUNDARY_FALSE_FLAGS = [
    "default_submit_allowed",
    "submit_order_allowed",
    "cancel_order_allowed",
    "retry_order_allowed",
    "replace_order_allowed",
    "amend_order_allowed",
    "flatten_position_allowed",
    "order_ticket_enabled",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "execution_adapter_call_allowed",
    "adapter_send_allowed",
    "live_exchange_request_allowed",
    "network_attempted",
    "dashboard_operation_controls_enabled",
    "dashboard_trading_controls_enabled",
    "admin_workbench_operation_controls_enabled",
    "admin_workbench_trading_controls_enabled",
    "trader_terminal_order_ticket_enabled",
    "trader_terminal_submit_controls_enabled",
    "manual_operation_submit_allowed",
    "automatic_remediation_allowed",
    "retry_scheduler_enabled",
    "product_grade_trading_terminal_claim",
]
REQUIRED_FAILURE_SEMANTICS = {
    "missing_source": "fail_closed_missing_source",
    "malformed_response": "fail_closed_malformed_response",
    "stale_source": "degraded_stale_response",
    "forbidden_controls": "fail_closed_forbidden_controls",
}


def fail(message: str) -> None:
    raise SystemExit(f"v28 Trader Terminal backend API contract handoff failed: {message}")


def non_empty(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


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
    if case.get("api_surface_override"):
        result["api_surface"] = merge(result["api_surface"], case["api_surface_override"])
    return result


def require_existing_path(value: Any, reason: str, reasons: list[str]) -> None:
    if not non_empty(value):
        reasons.append(reason)
        return
    path = Path(str(value).split("#", 1)[0])
    if path.is_absolute() or not path.is_file():
        reasons.append(reason)


def classify_status(reasons: list[str]) -> str:
    if any(reason.startswith("missing_source") for reason in reasons):
        return "fail_closed_missing_source"
    if any(reason.startswith("unredacted_payload") for reason in reasons):
        return "fail_closed_unredacted_payload"
    if any(reason.startswith("forbidden_controls") for reason in reasons):
        return "fail_closed_forbidden_controls"
    if any(reason.startswith("malformed_response") for reason in reasons):
        return "fail_closed_malformed_response"
    if any(reason.startswith("stale_response") for reason in reasons):
        return "degraded_stale_response"
    if any(reason.startswith("degraded_response") for reason in reasons):
        return "degraded_partial_response"
    if reasons:
        return "fail_closed_malformed_response"
    return "trader_terminal_backend_api_contract_ready"


def classify_artifact(artifact: dict[str, Any]) -> dict[str, Any]:
    reasons: list[str] = []
    if artifact.get("schema_version") != SCHEMA_VERSION:
        reasons.append("malformed_response:schema_version_mismatch")
    if artifact.get("contract_version") != CONTRACT_VERSION:
        reasons.append("malformed_response:contract_version_mismatch")
    if artifact.get("task_id") != "V280-007" or artifact.get("github_issue") != 900:
        reasons.append("malformed_response:task_identity_mismatch")
    if artifact.get("backend_module") != "trader_terminal_backend_api_contract_handoff":
        reasons.append("malformed_response:backend_module_mismatch")
    if artifact.get("backend_module_status") != "runtime_closed":
        reasons.append("malformed_response:backend_module_not_runtime_closed")
    if set(artifact.get("dependency_contracts") or []) != DEPENDENCIES:
        reasons.append("malformed_response:dependency_contracts_mismatch")
    if artifact.get("handoff_mode") != "deterministic_read_only_backend_api_contract":
        reasons.append(f"malformed_response:handoff_mode:{artifact.get('handoff_mode')}")

    api_surface = artifact.get("api_surface")
    if not isinstance(api_surface, dict):
        reasons.append("malformed_response:missing_api_surface")
        api_surface = {}
    if api_surface.get("read_only") is not True or api_surface.get("artifact_driven") is not True:
        reasons.append("forbidden_controls:api_surface_not_readonly_artifact")
    if api_surface.get("ad_hoc_evidence_file_inspection_required") is not False:
        reasons.append("malformed_response:ad_hoc_evidence_required")
    for key in (
        "order_ticket_enabled",
        "submit_controls_enabled",
        "operation_controls_enabled",
        "trading_controls_enabled",
        "product_grade_terminal_claim_allowed",
    ):
        if api_surface.get(key) is not False:
            reasons.append(f"forbidden_controls:api_surface:{key}")

    boundary = artifact.get("boundary_flags")
    if not isinstance(boundary, dict):
        reasons.append("forbidden_controls:missing_boundary_flags")
        boundary = {}
    for key in BOUNDARY_FALSE_FLAGS:
        if key not in boundary:
            reasons.append(f"forbidden_controls:missing_required_false:{key}")
        elif boundary.get(key) is not False:
            reasons.append(f"forbidden_controls:boundary:{key}")

    contracts = artifact.get("api_contracts")
    if not isinstance(contracts, dict):
        reasons.append("malformed_response:missing_api_contracts")
        contracts = {}
    if set(contracts) != set(EXPECTED_API_CONTRACTS):
        missing = sorted(set(EXPECTED_API_CONTRACTS) - set(contracts))
        extra = sorted(set(contracts) - set(EXPECTED_API_CONTRACTS))
        if missing:
            reasons.append(f"malformed_response:missing_api_contracts:{missing}")
        if extra:
            reasons.append(f"malformed_response:extra_api_contracts:{extra}")

    for api_id, endpoint_id in EXPECTED_API_CONTRACTS.items():
        contract = contracts.get(api_id)
        if not isinstance(contract, dict):
            reasons.append(f"malformed_response:api_contract:{api_id}")
            continue
        if contract.get("endpoint_id") != endpoint_id:
            reasons.append(f"malformed_response:endpoint:{api_id}:{contract.get('endpoint_id')}")
        if contract.get("contract_status") == "degraded":
            degradation_reasons = contract.get("degradation_reasons")
            if not isinstance(degradation_reasons, list) or not degradation_reasons:
                reasons.append(f"malformed_response:missing_degradation_reasons:{api_id}")
            else:
                reasons.append(f"degraded_response:{api_id}")
        elif contract.get("contract_status") != "ready":
            reasons.append(f"malformed_response:contract_status:{api_id}:{contract.get('contract_status')}")
        if contract.get("allowed_methods") != ["GET"]:
            reasons.append(f"forbidden_controls:methods:{api_id}")
        if contract.get("read_only") is not True or contract.get("mutating_methods_allowed") is not False:
            reasons.append(f"forbidden_controls:mutating_methods:{api_id}")
        for key in (
            "operation_controls_enabled",
            "trading_controls_enabled",
            "order_ticket_enabled",
            "manual_operation_submit_allowed",
        ):
            if contract.get(key) is not False:
                reasons.append(f"forbidden_controls:api:{api_id}:{key}")

        response_schema = contract.get("response_schema")
        if not isinstance(response_schema, dict):
            reasons.append(f"malformed_response:missing_schema:{api_id}")
            response_schema = {}
        if not non_empty(response_schema.get("schema_id")):
            reasons.append(f"malformed_response:schema_id:{api_id}")
        required_fields = response_schema.get("required_fields")
        if not isinstance(required_fields, list) or not required_fields or not all(non_empty(field) for field in required_fields):
            reasons.append(f"malformed_response:required_fields:{api_id}")
        if response_schema.get("breaking_change_policy") != "migration_note_required_before_contract_change":
            reasons.append(f"malformed_response:breaking_change_policy:{api_id}")
        if response_schema.get("migration_note_required_for_breaking_change") is not True:
            reasons.append(f"malformed_response:migration_note_required:{api_id}")

        redaction = contract.get("redaction")
        if not isinstance(redaction, dict):
            reasons.append(f"unredacted_payload:missing_redaction:{api_id}")
            redaction = {}
        if redaction.get("redaction_status") != "redacted":
            reasons.append(f"unredacted_payload:redaction_status:{api_id}")
        if redaction.get("secret_fields_policy") != "omit":
            reasons.append(f"unredacted_payload:secret_policy:{api_id}")
        if redaction.get("account_identifier_policy") != "hash_or_role_only":
            reasons.append(f"unredacted_payload:account_identifier_policy:{api_id}")
        if redaction.get("raw_payload_allowed") is not False:
            reasons.append(f"unredacted_payload:raw_payload:{api_id}")

        freshness = contract.get("freshness")
        if not isinstance(freshness, dict):
            reasons.append(f"malformed_response:missing_freshness:{api_id}")
            freshness = {}
        if not isinstance(freshness.get("max_age_seconds"), int) or freshness.get("max_age_seconds") <= 0:
            reasons.append(f"malformed_response:max_age_seconds:{api_id}")
        if freshness.get("stale_status") != "degraded_stale_response":
            reasons.append(f"malformed_response:stale_status:{api_id}")
        freshness_status = freshness.get("source_freshness_status")
        if freshness_status == "stale":
            degradation_reasons = freshness.get("degradation_reasons")
            if not isinstance(degradation_reasons, list) or not degradation_reasons:
                reasons.append(f"malformed_response:missing_stale_reasons:{api_id}")
            else:
                reasons.append(f"stale_response:{api_id}")
        elif freshness_status != "fresh":
            reasons.append(f"malformed_response:freshness_status:{api_id}:{freshness_status}")

        if contract.get("failure_semantics") != REQUIRED_FAILURE_SEMANTICS:
            reasons.append(f"malformed_response:failure_semantics:{api_id}")
        source_refs = contract.get("source_refs")
        if not isinstance(source_refs, list) or not source_refs:
            reasons.append(f"missing_source:source_refs:{api_id}")
        else:
            for index, source_ref in enumerate(source_refs):
                require_existing_path(source_ref, f"missing_source:source_ref:{api_id}:{index}", reasons)
        verification_commands = contract.get("verification_commands")
        if not isinstance(verification_commands, list) or not verification_commands or not all(non_empty(cmd) for cmd in verification_commands):
            reasons.append(f"malformed_response:verification_commands:{api_id}")

    status = classify_status(reasons)
    return {
        "status": status,
        "fail_closed": status.startswith("fail_closed"),
        "degraded": status.startswith("degraded"),
        "blocking_reasons": reasons,
    }


artifact = json.loads(artifact_path.read_text(encoding="utf-8"))
matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
module = next(
    (item for item in matrix.get("module_readiness", []) if item.get("module_id") == "trader_terminal_backend_api_contract_handoff"),
    None,
)
if not module:
    fail("matrix missing trader_terminal_backend_api_contract_handoff")
if module.get("classification") != "runtime-closed":
    fail("Trader Terminal API contract matrix entry must be runtime-closed")
if module.get("evidence_path") != "docs/rust-cutover/evidence/V280-007.md":
    fail("Trader Terminal API contract matrix evidence path mismatch")
if module.get("verification_command") != "scripts/ai/verify_release.sh v28-trader-terminal-backend-api-contract-handoff":
    fail("Trader Terminal API contract matrix verification command mismatch")

cases = artifact.get("contract_replay_cases")
if not isinstance(cases, list) or [case.get("case_id") for case in cases] != EXPECTED_CASES:
    fail("Trader Terminal API contract replay cases mismatch")
allowed = 0
degraded = 0
fail_closed = 0
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
if allowed != 1 or degraded != 2 or fail_closed != 4:
    fail(f"unexpected case counts: allowed={allowed} degraded={degraded} fail_closed={fail_closed}")

if selftest:
    opened = copy.deepcopy(artifact)
    opened["boundary_flags"]["submit_order_allowed"] = True
    if classify_artifact(opened)["status"] != "fail_closed_forbidden_controls":
        fail("negative self-test unexpectedly allowed submit_order_allowed")

    unredacted = copy.deepcopy(artifact)
    unredacted["api_contracts"]["audit_entries"]["redaction"]["raw_payload_allowed"] = True
    if classify_artifact(unredacted)["status"] != "fail_closed_unredacted_payload":
        fail("negative self-test unexpectedly allowed raw audit payload")

print(
    "v28_trader_terminal_backend_api_contract_handoff=pass "
    f"cases={len(cases)} allowed={allowed} degraded={degraded} fail_closed={fail_closed} "
    f"api_contracts={len(EXPECTED_API_CONTRACTS)} boundary_flags={len(BOUNDARY_FALSE_FLAGS)} negative_selftest={int(selftest)}"
)
PY
