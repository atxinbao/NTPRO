#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

MATRIX_PATH="${NTPRO_V280_BACKEND_CLOSURE_MATRIX:-docs/rust-cutover/release/v0_28_0_backend_closure_readiness_matrix.json}"
CONTRACT_PATH="${NTPRO_V280_BACKEND_CLOSURE_CONTRACT:-docs/rust-cutover/release/v0_28_0_backend_closure_boundary_contract.md}"
TASK_PATH="${NTPRO_V280_BACKEND_CLOSURE_TASK:-docs/rust-cutover/tasks/V280-001.md}"
EVIDENCE_PATH="${NTPRO_V280_BACKEND_CLOSURE_EVIDENCE:-docs/rust-cutover/evidence/V280-001.md}"
INTAKE_PATH="${NTPRO_V280_BACKEND_CLOSURE_INTAKE:-docs/rust-cutover/release/v0_28_0_intake_gate.md}"
SELFTEST="${NTPRO_V280_BACKEND_CLOSURE_SELFTEST:-1}"

fail() {
  echo "v28 backend closure boundary contract failed: $*" >&2
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

for path in "$MATRIX_PATH" "$CONTRACT_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$INTAKE_PATH"; do
  require_file "$path"
done

require_contains "$INTAKE_PATH" "start_gate_status = satisfied"
require_contains "$INTAKE_PATH" "v0.28.0 capability track = backend_closure_product_operations_runtime_finalization_only"
require_contains "$TASK_PATH" "GitHub issue: \`#894\`"
require_contains "$EVIDENCE_PATH" "Task: \`V280-001\` / GitHub issue \`#894\`"
require_contains "$CONTRACT_PATH" "contract_version = ntpro.v280.backend_closure_boundary.v1"
require_contains "$CONTRACT_PATH" "schema_version = ntpro.v280.backend_closure_readiness_matrix.v1"
require_contains "$CONTRACT_PATH" "release_scope = backend_closure_product_operations_runtime_finalization_only"
require_contains "$CONTRACT_PATH" "backend_complete_claim = false"
require_contains "$CONTRACT_PATH" "frontend_product_work_complete_claim = false"
require_contains "$CONTRACT_PATH" "product_grade_live_trading_terminal_claim = false"
require_contains "$CONTRACT_PATH" "release stage = scripts/ai/verify_release.sh v28-backend-closure-boundary-contract"

for marker in \
  "new_submit_capability = false" \
  "production_order_submission_allowed = false" \
  "production_order_mutation_allowed = false" \
  "cancel_order_allowed = false" \
  "replace_order_allowed = false" \
  "amend_order_allowed = false" \
  "flatten_position_allowed = false" \
  "execution_adapter_call_allowed = false" \
  "adapter_send_allowed = false" \
  "live_exchange_request_allowed = false" \
  "network_attempted = false" \
  "retry_scheduler_enabled = false" \
  "automatic_remediation_allowed = false" \
  "automatic_operation_action_allowed = false" \
  "dashboard_operation_controls_enabled = false" \
  "dashboard_trading_controls_enabled = false" \
  "admin_workbench_operation_controls_enabled = false" \
  "admin_workbench_trading_controls_enabled = false" \
  "trader_terminal_order_ticket_enabled = false" \
  "manual_operation_submit_allowed = false" \
  "product_grade_trading_terminal_claim = false"; do
  require_contains "$CONTRACT_PATH" "$marker"
done

MATRIX_PATH="$MATRIX_PATH" SELFTEST="$SELFTEST" python3 <<'PY'
from __future__ import annotations

import copy
import json
import os
from pathlib import Path
from typing import Any

matrix_path = Path(os.environ["MATRIX_PATH"])
selftest = os.environ.get("SELFTEST", "1") != "0"

SCHEMA_VERSION = "ntpro.v280.backend_closure_readiness_matrix.v1"
CONTRACT_VERSION = "ntpro.v280.backend_closure_boundary.v1"
RELEASE_SCOPE = "backend_closure_product_operations_runtime_finalization_only"
RELEASE_CLAIM = "backend_closure_boundary_and_readiness_matrix"
EXPECTED_CLASSIFICATIONS = {
    "v271_release_publication_dependency": "runtime-closed",
    "v280_backend_closure_boundary_contract": "runtime-closed",
    "v270_operations_runtime_foundation_evidence": "evidence-only",
    "v271_release_governance_patch_evidence": "evidence-only",
    "identity_permission_runtime_closure": "blocked",
    "persistent_audit_storage_runtime_closure": "blocked",
    "deployment_upgrade_rollback_orchestration_runtime_closure": "blocked",
    "telemetry_slo_ingestion_runtime_closure": "blocked",
    "admin_workbench_backend_state_bridge_closure": "blocked",
    "trader_terminal_backend_api_contract_handoff": "blocked",
    "backend_closure_fail_closed_hardening": "blocked",
    "v28_release_gates_strict_provenance_handoff": "deferred",
}
EXPECTED_BLOCKERS = {
    "identity_permission_runtime_closure": 895,
    "persistent_audit_storage_runtime_closure": 896,
    "deployment_upgrade_rollback_orchestration_runtime_closure": 897,
    "telemetry_slo_ingestion_runtime_closure": 898,
    "admin_workbench_backend_state_bridge_closure": 899,
    "trader_terminal_backend_api_contract_handoff": 900,
    "backend_closure_fail_closed_hardening": 901,
}
EXPECTED_DEFERRED = {"v28_release_gates_strict_provenance_handoff": 902}
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
    "product_grade_trading_terminal_claim",
]
EXPECTED_COUNTS = {"runtime-closed": 2, "evidence-only": 2, "blocked": 7, "deferred": 1}


def fail(message: str) -> None:
    raise SystemExit(f"v28 backend closure boundary contract failed: {message}")


def push_reason(reasons: list[str], reason: str) -> None:
    if reason not in reasons:
        reasons.append(reason)


def classify(snapshot: dict[str, Any]) -> dict[str, Any]:
    reasons: list[str] = []
    missing: list[str] = []
    opened: list[str] = []

    if snapshot.get("schema_version") != SCHEMA_VERSION:
        push_reason(reasons, "schema_version_mismatch")
    if snapshot.get("contract_version") != CONTRACT_VERSION:
        push_reason(reasons, "contract_version_mismatch")
    if snapshot.get("task_id") != "V280-001" or snapshot.get("github_issue") != 894:
        push_reason(reasons, "task_identity_mismatch")
    if snapshot.get("release_scope") != RELEASE_SCOPE:
        push_reason(reasons, "release_scope_mismatch")
    if snapshot.get("release_claim") != RELEASE_CLAIM:
        push_reason(reasons, "release_claim_mismatch")
    if snapshot.get("dependency_start_gate") != "satisfied":
        push_reason(reasons, "dependency_start_gate_not_satisfied")

    claim_rules = snapshot.get("claim_rules")
    if not isinstance(claim_rules, dict):
        fail("claim_rules must be an object")
    for key in (
        "backend_complete_claim",
        "frontend_product_work_complete_claim",
        "production_execution_runtime_claim",
        "product_grade_live_trading_terminal_claim",
        "blocked_module_closure_claim_allowed",
        "deferred_module_closure_claim_allowed",
        "evidence_only_module_runtime_closure_claim_allowed",
    ):
        if key not in claim_rules:
            missing.append(f"claim_rules.{key}")
            push_reason(reasons, f"missing_required_false_claim:{key}")
        elif claim_rules.get(key) is not False:
            opened.append(f"claim_rules.{key}")
            push_reason(reasons, f"forbidden_claim:{key}")
    for key in (
        "runtime_closed_module_requires_evidence_path",
        "runtime_closed_module_requires_verification_command",
    ):
        if claim_rules.get(key) is not True:
            missing.append(f"claim_rules.{key}")
            push_reason(reasons, f"missing_required_true_claim_rule:{key}")

    boundary_flags = snapshot.get("required_false_boundary_flags")
    if not isinstance(boundary_flags, dict):
        fail("required_false_boundary_flags must be an object")
    for key in BOUNDARY_FALSE_FLAGS:
        if key not in boundary_flags:
            missing.append(f"required_false_boundary_flags.{key}")
            push_reason(reasons, f"missing_required_false_boundary:{key}")
        elif boundary_flags.get(key) is not False:
            opened.append(f"required_false_boundary_flags.{key}")
            push_reason(reasons, f"forbidden_boundary_flag:{key}")

    modules = snapshot.get("module_readiness")
    if not isinstance(modules, list):
        fail("module_readiness must be a list")
    by_module: dict[str, dict[str, Any]] = {}
    for item in modules:
        if not isinstance(item, dict):
            fail("module_readiness entries must be objects")
        module_id = item.get("module_id")
        if not isinstance(module_id, str) or not module_id:
            fail("module_id must be a non-empty string")
        if module_id in by_module:
            fail(f"duplicate module_id: {module_id}")
        by_module[module_id] = item

    if set(by_module) != set(EXPECTED_CLASSIFICATIONS):
        missing_modules = sorted(set(EXPECTED_CLASSIFICATIONS) - set(by_module))
        extra_modules = sorted(set(by_module) - set(EXPECTED_CLASSIFICATIONS))
        if missing_modules:
            missing.extend([f"module_readiness.{module_id}" for module_id in missing_modules])
        if extra_modules:
            opened.extend([f"module_readiness.{module_id}" for module_id in extra_modules])
        push_reason(reasons, "module_set_mismatch")

    counts = {"runtime-closed": 0, "evidence-only": 0, "blocked": 0, "deferred": 0}
    for module_id, expected_classification in EXPECTED_CLASSIFICATIONS.items():
        item = by_module.get(module_id)
        if not item:
            continue
        classification = item.get("classification")
        if classification != expected_classification:
            opened.append(f"module_readiness.{module_id}.classification")
            push_reason(
                reasons,
                f"classification_mismatch:{module_id}:{classification}:{expected_classification}",
            )
            continue
        counts[classification] += 1

        closure_claim_allowed = item.get("closure_claim_allowed")
        evidence_path = item.get("evidence_path")
        verification_command = item.get("verification_command")
        if classification == "runtime-closed":
            if closure_claim_allowed is not True:
                missing.append(f"module_readiness.{module_id}.closure_claim_allowed")
                push_reason(reasons, f"runtime_closed_claim_not_allowed:{module_id}")
            if not isinstance(evidence_path, str) or not evidence_path:
                missing.append(f"module_readiness.{module_id}.evidence_path")
                push_reason(reasons, f"runtime_closed_missing_evidence_path:{module_id}")
            elif not Path(evidence_path).is_file():
                missing.append(f"module_readiness.{module_id}.evidence_path")
                push_reason(reasons, f"runtime_closed_evidence_path_not_found:{module_id}")
            if not isinstance(verification_command, str) or not verification_command:
                missing.append(f"module_readiness.{module_id}.verification_command")
                push_reason(reasons, f"runtime_closed_missing_verification_command:{module_id}")
        elif classification == "evidence-only":
            if closure_claim_allowed is not False:
                opened.append(f"module_readiness.{module_id}.closure_claim_allowed")
                push_reason(reasons, f"evidence_only_claim_opened:{module_id}")
            if not isinstance(evidence_path, str) or not Path(evidence_path).is_file():
                missing.append(f"module_readiness.{module_id}.evidence_path")
                push_reason(reasons, f"evidence_only_missing_evidence_path:{module_id}")
            if not isinstance(verification_command, str) or not verification_command:
                missing.append(f"module_readiness.{module_id}.verification_command")
                push_reason(reasons, f"evidence_only_missing_verification_command:{module_id}")
        elif classification == "blocked":
            if closure_claim_allowed is not False:
                opened.append(f"module_readiness.{module_id}.closure_claim_allowed")
                push_reason(reasons, f"blocked_claim_opened:{module_id}")
            if item.get("blocked_by_issue") != EXPECTED_BLOCKERS.get(module_id):
                missing.append(f"module_readiness.{module_id}.blocked_by_issue")
                push_reason(reasons, f"blocked_issue_mismatch:{module_id}")
        elif classification == "deferred":
            if closure_claim_allowed is not False:
                opened.append(f"module_readiness.{module_id}.closure_claim_allowed")
                push_reason(reasons, f"deferred_claim_opened:{module_id}")
            if item.get("deferred_until_issue") != EXPECTED_DEFERRED.get(module_id):
                missing.append(f"module_readiness.{module_id}.deferred_until_issue")
                push_reason(reasons, f"deferred_issue_mismatch:{module_id}")

    if snapshot.get("expected_counts") != EXPECTED_COUNTS:
        push_reason(reasons, "expected_counts_mismatch")
    if counts != EXPECTED_COUNTS:
        push_reason(reasons, f"actual_counts_mismatch:{counts}")

    fail_closed = bool(reasons or missing or opened)
    return {
        "effective_backend_closure_status": "fail_closed_boundary_violation"
        if fail_closed
        else "boundary_ready",
        "readiness_matrix_complete": not fail_closed,
        "runtime_closed_count": counts["runtime-closed"],
        "evidence_only_count": counts["evidence-only"],
        "blocked_count": counts["blocked"],
        "deferred_count": counts["deferred"],
        "backend_complete_claim_allowed": False,
        "frontend_product_claim_allowed": False,
        "product_grade_terminal_claim_allowed": False,
        "required_false_boundary_flags": len(BOUNDARY_FALSE_FLAGS),
        "fail_closed": fail_closed,
        "forbidden_fields_opened": opened,
        "missing_required_fields": missing,
        "blocking_reasons": reasons,
    }


matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
result = classify(matrix)
if result != {
    "effective_backend_closure_status": "boundary_ready",
    "readiness_matrix_complete": True,
    "runtime_closed_count": 2,
    "evidence_only_count": 2,
    "blocked_count": 7,
    "deferred_count": 1,
    "backend_complete_claim_allowed": False,
    "frontend_product_claim_allowed": False,
    "product_grade_terminal_claim_allowed": False,
    "required_false_boundary_flags": len(BOUNDARY_FALSE_FLAGS),
    "fail_closed": False,
    "forbidden_fields_opened": [],
    "missing_required_fields": [],
    "blocking_reasons": [],
}:
    fail("baseline matrix did not classify as boundary_ready: " + json.dumps(result, sort_keys=True))

if selftest:
    opened_boundary = copy.deepcopy(matrix)
    opened_boundary["required_false_boundary_flags"]["adapter_send_allowed"] = True
    if not classify(opened_boundary)["fail_closed"]:
        fail("negative self-test unexpectedly allowed adapter_send_allowed")

    blocked_claim = copy.deepcopy(matrix)
    for item in blocked_claim["module_readiness"]:
        if item["module_id"] == "identity_permission_runtime_closure":
            item["closure_claim_allowed"] = True
            break
    if not classify(blocked_claim)["fail_closed"]:
        fail("negative self-test unexpectedly allowed blocked module closure claim")

    missing_evidence = copy.deepcopy(matrix)
    for item in missing_evidence["module_readiness"]:
        if item["module_id"] == "v280_backend_closure_boundary_contract":
            item.pop("evidence_path", None)
            break
    if not classify(missing_evidence)["fail_closed"]:
        fail("negative self-test unexpectedly allowed missing runtime-closed evidence")

print(
    "v28_backend_closure_boundary_contract=pass "
    "modules=12 runtime_closed=2 evidence_only=2 blocked=7 deferred=1 "
    f"required_false_flags={len(BOUNDARY_FALSE_FLAGS)} negative_selftest={int(selftest)}"
)
PY
