#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

MATRIX_PATH="${NTPRO_V290_BACKEND_READINESS_MATRIX:-docs/rust-cutover/release/v0_29_0_backend_production_readiness_matrix.json}"
CONTRACT_PATH="${NTPRO_V290_BACKEND_READINESS_CONTRACT:-docs/rust-cutover/release/v0_29_0_backend_production_readiness_boundary_contract.md}"
TASK_PATH="${NTPRO_V290_BACKEND_READINESS_TASK:-docs/rust-cutover/tasks/V290-001.md}"
EVIDENCE_PATH="${NTPRO_V290_BACKEND_READINESS_EVIDENCE:-docs/rust-cutover/evidence/V290-001.md}"
INTAKE_PATH="${NTPRO_V290_BACKEND_READINESS_INTAKE:-docs/rust-cutover/release/v0_29_0_intake_gate.md}"
SELFTEST="${NTPRO_V290_BACKEND_READINESS_SELFTEST:-1}"

fail() {
  echo "v29 backend production readiness boundary contract failed: $*" >&2
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
require_contains "$INTAKE_PATH" "v0.29.0 capability track = backend_production_readiness_foundation_only"
require_contains "$TASK_PATH" "GitHub issue: \`#927\`"
require_contains "$EVIDENCE_PATH" "Task: \`V290-001\` / GitHub issue \`#927\`"
require_contains "$CONTRACT_PATH" "contract_version = ntpro.v290.backend_production_readiness_boundary.v1"
require_contains "$CONTRACT_PATH" "schema_version = ntpro.v290.backend_production_readiness_matrix.v1"
require_contains "$CONTRACT_PATH" "release_scope = backend_production_readiness_foundation_only"
require_contains "$CONTRACT_PATH" "production_readiness_terminology = backend_readiness_evidence_only"
require_contains "$CONTRACT_PATH" "backend_production_go_live_claim = false"
require_contains "$CONTRACT_PATH" "product_grade_live_trading_terminal_claim = false"
require_contains "$CONTRACT_PATH" "default_submit_claim = false"
require_contains "$CONTRACT_PATH" "release stage = scripts/ai/verify_release.sh v29-backend-production-readiness-boundary-contract"
require_contains "$CONTRACT_PATH" "production-ready (readiness evidence)"
require_contains "$CONTRACT_PATH" "production-ready go-live/product-ready positive claim => fail_closed_boundary_violation"

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
  "backend_go_live_claim = false" \
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

SCHEMA_VERSION = "ntpro.v290.backend_production_readiness_matrix.v1"
CONTRACT_VERSION = "ntpro.v290.backend_production_readiness_boundary.v1"
RELEASE_SCOPE = "backend_production_readiness_foundation_only"
RELEASE_CLAIM = "backend_production_readiness_boundary_and_matrix"
EXPECTED_CLASSIFICATIONS = {
    "v281_release_publication_dependency": "production-ready",
    "v290_backend_production_readiness_boundary_contract": "production-ready",
    "v280_backend_closure_evidence": "readiness-preview",
    "v281_release_governance_patch_evidence": "readiness-preview",
    "persistent_audit_storage_production_readiness": "production-ready",
    "telemetry_slo_ingestion_production_readiness": "production-ready",
    "permission_source_production_readiness": "production-ready",
    "read_only_backend_api_production_readiness": "production-ready",
    "deployment_config_runbook_production_readiness": "production-ready",
    "monitoring_alert_incident_production_readiness": "production-ready",
    "canary_rollback_dr_preflight_readiness": "production-ready",
    "backend_production_readiness_fail_closed_hardening": "blocked",
    "v29_release_gates_v30_handoff": "deferred",
}
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
REQUIRED_FALSE_TERMINOLOGY_FLAGS = [
    "backend_production_go_live_claim_allowed",
    "product_grade_live_trading_terminal_claim_allowed",
    "production_execution_runtime_claim_allowed",
    "default_submit_claim_allowed",
]
EXPECTED_COUNTS = {
    "production-ready": 9,
    "readiness-preview": 2,
    "blocked": 1,
    "deferred": 1,
}


def fail(message: str) -> None:
    raise SystemExit(f"v29 backend production readiness boundary contract failed: {message}")


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
    if snapshot.get("task_id") != "V290-001" or snapshot.get("github_issue") != 927:
        push_reason(reasons, "task_identity_mismatch")
    if snapshot.get("release_scope") != RELEASE_SCOPE:
        push_reason(reasons, "release_scope_mismatch")
    if snapshot.get("release_claim") != RELEASE_CLAIM:
        push_reason(reasons, "release_claim_mismatch")
    if snapshot.get("dependency_start_gate") != "satisfied":
        push_reason(reasons, "dependency_start_gate_not_satisfied")

    terminology = snapshot.get("terminology")
    if not isinstance(terminology, dict):
        fail("terminology must be an object")
    if terminology.get("production_readiness_meaning") != "backend_readiness_evidence_only":
        push_reason(reasons, "production_readiness_meaning_mismatch")
    if terminology.get("production_readiness_label") != "production-ready (readiness evidence)":
        push_reason(reasons, "production_readiness_label_mismatch")
    does_not_mean = terminology.get("production_readiness_does_not_mean")
    if not isinstance(does_not_mean, list):
        fail("production_readiness_does_not_mean must be a list")
    for value in (
        "backend_production_go_live",
        "product_grade_live_trading_terminal",
        "production_execution_runtime",
        "default_submit_capability",
    ):
        if value not in does_not_mean:
            missing.append(f"terminology.production_readiness_does_not_mean.{value}")
            push_reason(reasons, f"production_readiness_does_not_mean_missing:{value}")
    for key in REQUIRED_FALSE_TERMINOLOGY_FLAGS:
        if key not in terminology:
            missing.append(f"terminology.{key}")
            push_reason(reasons, f"missing_required_false_terminology:{key}")
        elif terminology.get(key) is not False:
            opened.append(f"terminology.{key}")
            push_reason(reasons, f"forbidden_terminology_claim:{key}")

    claim_rules = snapshot.get("claim_rules")
    if not isinstance(claim_rules, dict):
        fail("claim_rules must be an object")
    for key in (
        "backend_production_go_live_claim",
        "product_grade_live_trading_terminal_claim",
        "default_submit_claim",
        "production_execution_runtime_claim",
        "blocked_module_production_ready_claim_allowed",
        "deferred_module_production_ready_claim_allowed",
        "readiness_preview_module_production_ready_claim_allowed",
    ):
        if key not in claim_rules:
            missing.append(f"claim_rules.{key}")
            push_reason(reasons, f"missing_required_false_claim:{key}")
        elif claim_rules.get(key) is not False:
            opened.append(f"claim_rules.{key}")
            push_reason(reasons, f"forbidden_claim:{key}")
    for key in (
        "production_ready_module_requires_evidence_path",
        "production_ready_module_requires_verification_command",
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

        expected = EXPECTED_CLASSIFICATIONS.get(module_id)
        if expected is None:
            push_reason(reasons, f"unexpected_module:{module_id}")
            continue
        classification = item.get("classification")
        if classification != expected:
            push_reason(reasons, f"classification_mismatch:{module_id}:{classification}")
        if classification == "production-ready":
            if item.get("readiness_mode") != "deterministic_readiness_replay":
                push_reason(reasons, f"production_ready_missing_readiness_mode:{module_id}")
            if not item.get("evidence_path"):
                push_reason(reasons, f"production_ready_missing_evidence_path:{module_id}")
            if not item.get("verification_command"):
                push_reason(reasons, f"production_ready_missing_verification_command:{module_id}")
            for key in REQUIRED_FALSE_TERMINOLOGY_FLAGS:
                if item.get(key) is not False:
                    push_reason(reasons, f"production_ready_forbidden_claim:{module_id}:{key}")
            if item.get("production_ready_claim_allowed") is not True:
                push_reason(reasons, f"production_ready_claim_not_allowed:{module_id}")
        elif classification in {"readiness-preview", "blocked", "deferred"}:
            if item.get("production_ready_claim_allowed") is not False:
                push_reason(reasons, f"non_ready_claim_allowed:{module_id}")
            if classification == "blocked" and item.get("blocker_issue") != item.get("issue"):
                push_reason(reasons, f"blocked_missing_matching_issue:{module_id}")
            if classification == "deferred" and item.get("deferred_issue") != item.get("issue"):
                push_reason(reasons, f"deferred_missing_matching_issue:{module_id}")

    if set(by_module) != set(EXPECTED_CLASSIFICATIONS):
        push_reason(reasons, "module_set_mismatch")

    counts = {key: 0 for key in EXPECTED_COUNTS}
    for item in modules:
        classification = item.get("classification")
        if classification in counts:
            counts[classification] += 1
    if counts != EXPECTED_COUNTS:
        push_reason(reasons, f"count_mismatch:{counts}")
    if snapshot.get("expected_counts") != EXPECTED_COUNTS:
        push_reason(reasons, "expected_counts_mismatch")

    return {
        "ok": not reasons,
        "reasons": reasons,
        "missing": missing,
        "opened": opened,
        "counts": counts,
        "required_false_flags": len(BOUNDARY_FALSE_FLAGS),
    }


matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
result = classify(matrix)
if not result["ok"]:
    fail(json.dumps(result, sort_keys=True))

if selftest:
    bad_boundary = copy.deepcopy(matrix)
    bad_boundary["required_false_boundary_flags"]["adapter_send_allowed"] = True
    if classify(bad_boundary)["ok"]:
        fail("negative self-test allowed adapter_send_allowed")

    bad_ready = copy.deepcopy(matrix)
    bad_ready["module_readiness"][11]["production_ready_claim_allowed"] = True
    if classify(bad_ready)["ok"]:
        fail("negative self-test allowed blocked module production-ready claim")

    missing_evidence = copy.deepcopy(matrix)
    missing_evidence["module_readiness"][1].pop("evidence_path", None)
    if classify(missing_evidence)["ok"]:
        fail("negative self-test allowed missing production-ready evidence path")

print(
    "v29_backend_production_readiness_boundary_contract=pass "
    f"modules={len(matrix['module_readiness'])} "
    f"production_ready={result['counts']['production-ready']} "
    f"readiness_preview={result['counts']['readiness-preview']} "
    f"blocked={result['counts']['blocked']} "
    f"deferred={result['counts']['deferred']} "
    f"required_false_flags={result['required_false_flags']} "
    "negative_selftest=1"
)
PY
