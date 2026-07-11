#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

CONTRACT_JSON="${NTPRO_V300_GO_LIVE_BOUNDARY_JSON:-docs/rust-cutover/release/v0_30_0_backend_go_live_candidate_boundary_contract.json}"
CONTRACT_MD="${NTPRO_V300_GO_LIVE_BOUNDARY_MD:-docs/rust-cutover/release/v0_30_0_backend_go_live_candidate_boundary_contract.md}"
TASK_PATH="${NTPRO_V300_GO_LIVE_BOUNDARY_TASK:-docs/rust-cutover/tasks/V300-001.md}"
EVIDENCE_PATH="${NTPRO_V300_GO_LIVE_BOUNDARY_EVIDENCE:-docs/rust-cutover/evidence/V300-001.md}"
INTAKE_PATH="${NTPRO_V300_GO_LIVE_BOUNDARY_INTAKE:-docs/rust-cutover/release/v0_30_0_intake_gate.md}"
RELEASE_INDEX="${NTPRO_V300_GO_LIVE_BOUNDARY_RELEASE_INDEX:-docs/rust-cutover/release/README.md}"
SELFTEST="${NTPRO_V300_GO_LIVE_BOUNDARY_SELFTEST:-1}"

fail() {
  echo "v30 backend go-live candidate boundary contract failed: $*" >&2
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

for path in "$CONTRACT_JSON" "$CONTRACT_MD" "$TASK_PATH" "$EVIDENCE_PATH" "$INTAKE_PATH" "$RELEASE_INDEX"; do
  require_file "$path"
done

require_contains "$INTAKE_PATH" "start_gate_status = satisfied"
require_contains "$INTAKE_PATH" "v0.30.0 capability track = backend_production_go_live_candidate_foundation_only"
require_contains "$TASK_PATH" "GitHub issue: \`#970\`"
require_contains "$EVIDENCE_PATH" "Task: \`V300-001\` / GitHub issue \`#970\`"
require_contains "$CONTRACT_MD" "contract_version = ntpro.v300.backend_go_live_candidate_boundary.v1"
require_contains "$CONTRACT_MD" "release_scope = backend_production_go_live_candidate_foundation_only"
require_contains "$CONTRACT_MD" "candidate_claim = backend_go_live_candidate_boundary_contract"
require_contains "$CONTRACT_MD" "backend_go_live_candidate_claim = allowed_candidate_evidence_only"
require_contains "$CONTRACT_MD" "candidate_claim_runtime_effect_allowed = false"
require_contains "$CONTRACT_MD" "backend_go_live_claim = false"
require_contains "$CONTRACT_MD" "ambiguous_backend_go_live_claim = false"
require_contains "$CONTRACT_MD" "actual_backend_production_go_live_allowed = false"
require_contains "$CONTRACT_MD" "backend_go_live=true => fail_closed_ambiguous_go_live_claim"
require_contains "$CONTRACT_MD" "release stage = scripts/ai/verify_release.sh v30-backend-go-live-candidate-boundary-contract"
require_contains "$RELEASE_INDEX" "v0_30_0_backend_go_live_candidate_boundary_contract.md"
require_contains "$RELEASE_INDEX" "../evidence/V300-001.md"

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
  "ambiguous_backend_go_live_claim = false" \
  "actual_backend_production_go_live_allowed = false" \
  "production_runtime_enablement_allowed = false" \
  "product_grade_trading_terminal_claim = false" \
  "product_grade_live_trading_terminal_claim = false" \
  "default_production_execution_allowed = false" \
  "candidate_artifact_runtime_effect_allowed = false" \
  "production_feature_flags_default_enabled = false" \
  "shared_approval_consumption_allowed = false"; do
  require_contains "$CONTRACT_MD" "$marker"
done

CONTRACT_JSON="$CONTRACT_JSON" SELFTEST="$SELFTEST" python3 <<'PY'
from __future__ import annotations

import copy
import json
import os
from pathlib import Path
from typing import Any

payload = json.loads(Path(os.environ["CONTRACT_JSON"]).read_text(encoding="utf-8"))
selftest = os.environ.get("SELFTEST", "1") != "0"

SCHEMA_VERSION = "ntpro.v300.backend_go_live_candidate_boundary.v1"
RELEASE_SCOPE = "backend_production_go_live_candidate_foundation_only"
CANDIDATE_CLAIM = "backend_go_live_candidate_boundary_contract"
EXPECTED_ARTIFACTS = {
    "production_deployment_plan",
    "environment_readiness_record",
    "runtime_enablement_boundary",
    "controlled_feature_flag_inventory",
    "operator_approval_freeze_record",
    "change_window_lifecycle_record",
    "canary_execution_preflight",
    "rollback_disaster_recovery_boundary",
    "production_config_provenance",
    "venue_connectivity_readiness",
    "telemetry_slo_incident_freeze_gate",
    "audit_retention_evidence_export_plan",
    "go_no_go_decision_record",
    "release_gate_v31_handoff",
}
EXPECTED_PREREQUISITES = {
    "owner_operator_approval",
    "freeze_criteria",
    "rollback_readiness",
    "audit_retention",
    "telemetry_slo_gate",
    "risk_gate",
    "release_gate",
    "scoped_enablement_issue",
}
REQUIRED_FALSE_FLAGS = [
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
    "ambiguous_backend_go_live_claim",
    "actual_backend_production_go_live_allowed",
    "production_runtime_enablement_allowed",
    "product_grade_trading_terminal_claim",
    "product_grade_live_trading_terminal_claim",
    "default_production_execution_allowed",
    "candidate_artifact_runtime_effect_allowed",
    "production_feature_flags_default_enabled",
    "shared_approval_consumption_allowed",
]
EXPECTED_REJECTIONS = {
    "backend_go_live=true": "fail_closed_ambiguous_go_live_claim",
    "backend_production_go_live=true": "fail_closed_ambiguous_go_live_claim",
    "production_go_live=true": "fail_closed_ambiguous_go_live_claim",
    "go_live_candidate=production_enabled": "fail_closed_ambiguous_go_live_claim",
    "backend_go_live_candidate=live_enabled": "fail_closed_ambiguous_go_live_claim",
    "product_grade_live_trading_terminal=true": "fail_closed_product_grade_live_trading_claim",
    "default_production_execution=true": "fail_closed_default_execution_claim",
}


def fail(message: str) -> None:
    raise SystemExit(f"v30 backend go-live candidate boundary contract failed: {message}")


def validate(snapshot: dict[str, Any]) -> None:
    if snapshot.get("schema_version") != SCHEMA_VERSION:
        fail("schema version mismatch")
    if snapshot.get("contract_version") != SCHEMA_VERSION:
        fail("contract version mismatch")
    if snapshot.get("task_id") != "V300-001" or snapshot.get("github_issue") != 970:
        fail("task identity mismatch")
    if snapshot.get("product_version") != "v0.30.0":
        fail("product version mismatch")
    if snapshot.get("release_scope") != RELEASE_SCOPE:
        fail("release scope mismatch")
    if snapshot.get("candidate_claim") != CANDIDATE_CLAIM:
        fail("candidate claim mismatch")

    dependency = snapshot.get("dependency_start_gate")
    if not isinstance(dependency, dict):
        fail("dependency_start_gate must be an object")
    expected_dependency = {
        "v300_intake_gate": "satisfied",
        "v291_release_evidence": "published",
        "v291_milestone": "closed",
        "v291_exact_issue_set": "#963-#968",
        "v300_intake_issue": 969,
    }
    for key, expected in expected_dependency.items():
        if dependency.get(key) != expected:
            fail(f"dependency mismatch: {key}")

    terminology = snapshot.get("terminology")
    if not isinstance(terminology, dict):
        fail("terminology must be an object")
    for key in (
        "backend_production_readiness",
        "backend_go_live_candidate",
        "actual_backend_production_go_live",
        "product_grade_live_trading_terminal",
    ):
        if not isinstance(terminology.get(key), str) or not terminology[key]:
            fail(f"terminology missing: {key}")
    does_not_mean = terminology.get("candidate_does_not_mean")
    if not isinstance(does_not_mean, list):
        fail("candidate_does_not_mean must be a list")
    for value in (
        "actual_backend_production_go_live",
        "product_grade_live_trading_terminal",
        "default_production_execution",
        "production_order_submission",
        "production_order_mutation",
        "adapter_send",
        "live_exchange_request",
        "automatic_remediation",
        "dashboard_admin_trader_terminal_trading_controls",
    ):
        if value not in does_not_mean:
            fail(f"candidate_does_not_mean missing: {value}")

    rules = snapshot.get("candidate_claim_rules")
    if not isinstance(rules, dict):
        fail("candidate_claim_rules must be an object")
    if rules.get("backend_go_live_candidate_claim_allowed") is not True:
        fail("candidate claim must be allowed as evidence only")
    for key in (
        "candidate_claim_runtime_effect_allowed",
        "backend_go_live_claim_allowed",
        "ambiguous_backend_go_live_claim_allowed",
        "actual_backend_production_go_live_allowed",
        "product_grade_live_trading_terminal_claim_allowed",
        "production_execution_runtime_claim_allowed",
        "default_submit_claim_allowed",
        "default_production_execution_allowed",
    ):
        if key not in rules:
            fail(f"candidate claim rule missing: {key}")
        if rules.get(key) is not False:
            fail(f"forbidden candidate claim rule opened: {key}")
    for key in (
        "candidate_artifact_requires_scoped_issue",
        "candidate_artifact_requires_later_go_no_go_decision",
        "candidate_artifact_requires_release_gate",
    ):
        if rules.get(key) is not True:
            fail(f"required candidate claim rule not true: {key}")

    artifacts = snapshot.get("allowed_candidate_artifacts")
    if not isinstance(artifacts, list):
        fail("allowed_candidate_artifacts must be a list")
    seen_artifacts: set[str] = set()
    for artifact in artifacts:
        if not isinstance(artifact, dict):
            fail("allowed candidate artifact entries must be objects")
        artifact_id = artifact.get("id")
        if not isinstance(artifact_id, str):
            fail("candidate artifact id missing")
        if artifact_id in seen_artifacts:
            fail(f"duplicate candidate artifact: {artifact_id}")
        seen_artifacts.add(artifact_id)
        if artifact_id not in EXPECTED_ARTIFACTS:
            fail(f"unexpected candidate artifact: {artifact_id}")
        if not isinstance(artifact.get("issue"), int):
            fail(f"candidate artifact issue missing: {artifact_id}")
        for key in ("runtime_effect_allowed", "default_enabled"):
            if artifact.get(key) is not False:
                fail(f"candidate artifact runtime/default opened: {artifact_id}.{key}")
        for key in ("requires_scoped_issue", "requires_later_enablement_decision"):
            if artifact.get(key) is not True:
                fail(f"candidate artifact missing required gate: {artifact_id}.{key}")
    if seen_artifacts != EXPECTED_ARTIFACTS:
        fail(f"candidate artifact set mismatch: {sorted(seen_artifacts)}")

    prerequisites = snapshot.get("required_later_enablement_prerequisites")
    if not isinstance(prerequisites, list):
        fail("required_later_enablement_prerequisites must be a list")
    seen_prerequisites: set[str] = set()
    for item in prerequisites:
        if not isinstance(item, dict):
            fail("later enablement prerequisite entries must be objects")
        item_id = item.get("id")
        if not isinstance(item_id, str):
            fail("later enablement prerequisite id missing")
        if item_id in seen_prerequisites:
            fail(f"duplicate later enablement prerequisite: {item_id}")
        seen_prerequisites.add(item_id)
        if item_id not in EXPECTED_PREREQUISITES:
            fail(f"unexpected later enablement prerequisite: {item_id}")
        if item.get("status") != "required_later":
            fail(f"later enablement prerequisite status mismatch: {item_id}")
        if item.get("current_satisfied") is not False:
            fail(f"later enablement prerequisite must not be satisfied in V300-001: {item_id}")
        if item.get("bypass_allowed") is not False:
            fail(f"later enablement prerequisite bypass opened: {item_id}")
    if seen_prerequisites != EXPECTED_PREREQUISITES:
        fail(f"later enablement prerequisite set mismatch: {sorted(seen_prerequisites)}")

    flags = snapshot.get("required_false_boundary_flags")
    if not isinstance(flags, dict):
        fail("required_false_boundary_flags must be an object")
    for key in REQUIRED_FALSE_FLAGS:
        if key not in flags:
            fail(f"required false boundary flag missing: {key}")
        if flags.get(key) is not False:
            fail(f"required false boundary flag opened: {key}")

    rejections = snapshot.get("ambiguous_claim_rejections")
    if not isinstance(rejections, list):
        fail("ambiguous_claim_rejections must be a list")
    rejection_map = {}
    for item in rejections:
        if not isinstance(item, dict):
            fail("ambiguous claim rejection entries must be objects")
        claim = item.get("claim")
        action = item.get("action")
        if not isinstance(claim, str) or not isinstance(action, str):
            fail("ambiguous claim rejection claim/action missing")
        if claim in rejection_map:
            fail(f"duplicate ambiguous claim rejection: {claim}")
        rejection_map[claim] = action
    if rejection_map != EXPECTED_REJECTIONS:
        fail(f"ambiguous claim rejection set mismatch: {rejection_map}")


validate(payload)

negative_selftests = 0
if selftest:
    opened_flag = copy.deepcopy(payload)
    opened_flag["required_false_boundary_flags"]["backend_go_live_claim"] = True
    try:
        validate(opened_flag)
    except SystemExit:
        negative_selftests += 1
    else:
        fail("negative self-test unexpectedly allowed backend_go_live_claim=true")

    ambiguous_open = copy.deepcopy(payload)
    ambiguous_open["candidate_claim_rules"]["ambiguous_backend_go_live_claim_allowed"] = True
    try:
        validate(ambiguous_open)
    except SystemExit:
        negative_selftests += 1
    else:
        fail("negative self-test unexpectedly allowed ambiguous go-live claims")

print(
    "v30_backend_go_live_candidate_boundary_contract=pass "
    f"allowed_candidate_artifacts={len(EXPECTED_ARTIFACTS)} "
    f"required_later_enablement_prerequisites={len(EXPECTED_PREREQUISITES)} "
    f"required_false_flags={len(REQUIRED_FALSE_FLAGS)} "
    f"ambiguous_claim_rejections={len(EXPECTED_REJECTIONS)} "
    f"negative_selftest={negative_selftests}"
)
PY
