#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

ARTIFACT_PATH="${NTPRO_V300_DEPLOYMENT_READINESS_ARTIFACT:-docs/rust-cutover/release/v0_30_0_production_deployment_plan_environment_readiness.json}"
CONTRACT_PATH="${NTPRO_V300_DEPLOYMENT_READINESS_CONTRACT:-docs/rust-cutover/release/v0_30_0_production_deployment_plan_environment_readiness.md}"
TASK_PATH="${NTPRO_V300_DEPLOYMENT_READINESS_TASK:-docs/rust-cutover/tasks/V300-002.md}"
EVIDENCE_PATH="${NTPRO_V300_DEPLOYMENT_READINESS_EVIDENCE:-docs/rust-cutover/evidence/V300-002.md}"
INTAKE_PATH="${NTPRO_V300_DEPLOYMENT_READINESS_INTAKE:-docs/rust-cutover/release/v0_30_0_intake_gate.md}"
BOUNDARY_PATH="${NTPRO_V300_DEPLOYMENT_READINESS_BOUNDARY:-docs/rust-cutover/release/v0_30_0_backend_go_live_candidate_boundary_contract.md}"
BOUNDARY_JSON="${NTPRO_V300_DEPLOYMENT_READINESS_BOUNDARY_JSON:-docs/rust-cutover/release/v0_30_0_backend_go_live_candidate_boundary_contract.json}"
V291_MANIFEST="${NTPRO_V300_DEPLOYMENT_READINESS_V291_MANIFEST:-docs/rust-cutover/release/v0_29_1_release_manifest.json}"
V291_CLOSEOUT="${NTPRO_V300_DEPLOYMENT_READINESS_V291_CLOSEOUT:-docs/rust-cutover/release/v0_29_1_release_closeout_evidence.md}"
V290_DEPLOYMENT_ARTIFACT="${NTPRO_V300_DEPLOYMENT_READINESS_V290_DEPLOYMENT_ARTIFACT:-docs/rust-cutover/release/v0_29_0_deployment_config_runbook_production_readiness_artifact.json}"
RELEASE_INDEX="${NTPRO_V300_DEPLOYMENT_READINESS_RELEASE_INDEX:-docs/rust-cutover/release/README.md}"
SELFTEST="${NTPRO_V300_DEPLOYMENT_READINESS_SELFTEST:-1}"

fail() {
  echo "v30 production deployment plan environment readiness failed: $*" >&2
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

for path in "$ARTIFACT_PATH" "$CONTRACT_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$INTAKE_PATH" "$BOUNDARY_PATH" "$BOUNDARY_JSON" "$V291_MANIFEST" "$V291_CLOSEOUT" "$V290_DEPLOYMENT_ARTIFACT" "$RELEASE_INDEX"; do
  require_file "$path"
done

require_contains "$TASK_PATH" "GitHub issue: \`#971\`"
require_contains "$EVIDENCE_PATH" "Task: \`V300-002\` / GitHub issue \`#971\`"
require_contains "$INTAKE_PATH" "start_gate_status = satisfied"
require_contains "$BOUNDARY_PATH" "contract_version = ntpro.v300.backend_go_live_candidate_boundary.v1"
require_contains "$BOUNDARY_PATH" "production_deployment_plan = #971"
require_contains "$BOUNDARY_PATH" "environment_readiness_record = #971"
require_contains "$CONTRACT_PATH" "contract_version = ntpro.v300.production_deployment_plan_environment_readiness.v1"
require_contains "$CONTRACT_PATH" "schema_version = ntpro.v300.production_deployment_plan_environment_readiness.v1"
require_contains "$CONTRACT_PATH" "deployment_mode = source_controlled_preview_only"
require_contains "$CONTRACT_PATH" "production_deployment_execution_allowed = false"
require_contains "$CONTRACT_PATH" "production_deployment_executed = false"
require_contains "$CONTRACT_PATH" "missing_environment_evidence => fail_closed_missing_environment_evidence"
require_contains "$CONTRACT_PATH" "stale_environment_evidence => fail_closed_stale_environment_evidence"
require_contains "$CONTRACT_PATH" "mismatched_environment_evidence => fail_closed_mismatched_environment_evidence"
require_contains "$CONTRACT_PATH" "release stage = scripts/ai/verify_release.sh v30-production-deployment-plan-environment-readiness"
require_contains "$RELEASE_INDEX" "v0_30_0_production_deployment_plan_environment_readiness.md"
require_contains "$RELEASE_INDEX" "../evidence/V300-002.md"

ARTIFACT_PATH="$ARTIFACT_PATH" SELFTEST="$SELFTEST" python3 <<'PY'
from __future__ import annotations

import copy
import json
import os
from pathlib import Path
from typing import Any

payload = json.loads(Path(os.environ["ARTIFACT_PATH"]).read_text(encoding="utf-8"))
selftest = os.environ.get("SELFTEST", "1") != "0"

SCHEMA_VERSION = "ntpro.v300.production_deployment_plan_environment_readiness.v1"
RELEASE_SCOPE = "backend_production_go_live_candidate_foundation_only"
READY_STATUS = "production_deployment_plan_environment_readiness_ready"
DEPENDENCIES = {"V300-000", "V300-001", "v0.29.1-release-evidence"}
EXPECTED_TARGETS = {
    "prod-control-plane": "prod-candidate-primary",
    "prod-read-api": "prod-candidate-primary",
    "prod-audit-storage": "prod-candidate-primary",
    "prod-telemetry-slo": "prod-candidate-primary",
    "prod-canary-sandbox": "prod-candidate-canary",
    "prod-dr-preview": "prod-candidate-dr",
}
EXPECTED_TARGET_CLASSES = {
    "prod-control-plane": "backend_control_plane",
    "prod-read-api": "read_only_backend_api",
    "prod-audit-storage": "persistent_audit_storage",
    "prod-telemetry-slo": "telemetry_slo_pipeline",
    "prod-canary-sandbox": "canary_preview_lane",
    "prod-dr-preview": "disaster_recovery_preview_lane",
}
EXPECTED_ENVIRONMENTS = {
    "prod-candidate-primary",
    "prod-candidate-canary",
    "prod-candidate-dr",
}
EXPECTED_CONFIGS = {
    "config-prod-primary": "prod-candidate-primary",
    "config-prod-canary": "prod-candidate-canary",
    "config-prod-dr": "prod-candidate-dr",
}
EXPECTED_MIGRATIONS = {
    "schema_migration_preview",
    "config_upgrade_preview",
    "artifact_compatibility_preview",
    "operator_handoff_preview",
}
EXPECTED_ROLLBACKS = {
    "pre_deploy_snapshot_checkpoint",
    "artifact_revert_checkpoint",
    "config_revert_checkpoint",
    "schema_rollback_preview_checkpoint",
    "traffic_revert_checkpoint",
}
EXPECTED_CASES = {
    "deployment_plan_environment_readiness.preview.allowed.001",
    "deployment_plan_environment_readiness.missing_environment.fail_closed.001",
    "deployment_plan_environment_readiness.stale_environment.fail_closed.001",
    "deployment_plan_environment_readiness.mismatched_environment.fail_closed.001",
    "deployment_plan_environment_readiness.missing_artifact_provenance.fail_closed.001",
    "deployment_plan_environment_readiness.missing_config_provenance.fail_closed.001",
    "deployment_plan_environment_readiness.missing_migration_prerequisite.fail_closed.001",
    "deployment_plan_environment_readiness.missing_rollback_checkpoint.fail_closed.001",
    "deployment_plan_environment_readiness.forbidden_execution.fail_closed.001",
    "deployment_plan_environment_readiness.forbidden_boundary.fail_closed.001",
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
    "production_deployment_execution_allowed",
    "production_deployment_executed",
    "live_environment_mutation_allowed",
]


def fail(message: str) -> None:
    raise SystemExit(f"v30 production deployment plan environment readiness failed: {message}")


def merge(base: Any, override: Any) -> Any:
    if isinstance(base, dict) and isinstance(override, dict):
        result = copy.deepcopy(base)
        for key, value in override.items():
            result[key] = merge(result.get(key), value)
        return result
    return copy.deepcopy(override)


def apply_indexed_overrides(items: list[dict[str, Any]], id_key: str, overrides: dict[str, Any]) -> list[dict[str, Any]]:
    result = copy.deepcopy(items)
    for item_id, override in overrides.items():
        for index, item in enumerate(result):
            if item.get(id_key) == item_id:
                result[index] = merge(item, override)
                break
        else:
            result.append({id_key: item_id, **copy.deepcopy(override)})
    return result


def apply_case(artifact: dict[str, Any], case: dict[str, Any]) -> dict[str, Any]:
    result = copy.deepcopy(artifact)
    if case.get("artifact_provenance_override"):
        result["artifact_provenance"] = merge(result["artifact_provenance"], case["artifact_provenance_override"])
    if case.get("boundary_flags_override"):
        result["boundary_flags"] = merge(result["boundary_flags"], case["boundary_flags_override"])
    if case.get("deployment_target_overrides"):
        result["deployment_targets"] = apply_indexed_overrides(
            result["deployment_targets"],
            "target_id",
            case["deployment_target_overrides"],
        )
    if case.get("environment_inventory_overrides"):
        result["environment_inventory"] = apply_indexed_overrides(
            result["environment_inventory"],
            "environment_id",
            case["environment_inventory_overrides"],
        )
    if case.get("config_provenance_overrides"):
        result["config_provenance"] = apply_indexed_overrides(
            result["config_provenance"],
            "config_id",
            case["config_provenance_overrides"],
        )
    if case.get("migration_prerequisite_overrides"):
        result["migration_upgrade_prerequisites"] = apply_indexed_overrides(
            result["migration_upgrade_prerequisites"],
            "id",
            case["migration_prerequisite_overrides"],
        )
    if case.get("rollback_checkpoint_overrides"):
        result["rollback_checkpoints"] = apply_indexed_overrides(
            result["rollback_checkpoints"],
            "id",
            case["rollback_checkpoint_overrides"],
        )
    return result


def classify_status(reasons: list[str]) -> str:
    if any(reason.startswith("forbidden_boundary") for reason in reasons):
        return "fail_closed_forbidden_boundary"
    if any(reason.startswith("forbidden_execution") for reason in reasons):
        return "fail_closed_forbidden_execution"
    if any(reason.startswith("missing_environment") for reason in reasons):
        return "fail_closed_missing_environment_evidence"
    if any(reason.startswith("stale_environment") for reason in reasons):
        return "fail_closed_stale_environment_evidence"
    if any(reason.startswith("mismatched_environment") for reason in reasons):
        return "fail_closed_mismatched_environment_evidence"
    if any(reason.startswith("missing_artifact_provenance") for reason in reasons):
        return "fail_closed_missing_artifact_provenance"
    if any(reason.startswith("missing_config_provenance") for reason in reasons):
        return "fail_closed_missing_config_provenance"
    if any(reason.startswith("missing_migration_prerequisite") for reason in reasons):
        return "fail_closed_missing_migration_prerequisite"
    if any(reason.startswith("missing_rollback_checkpoint") for reason in reasons):
        return "fail_closed_missing_rollback_checkpoint"
    if reasons:
        return "fail_closed_missing_environment_evidence"
    return READY_STATUS


def collect_reasons(artifact: dict[str, Any]) -> list[str]:
    reasons: list[str] = []
    if artifact.get("schema_version") != SCHEMA_VERSION:
        reasons.append("missing_artifact_provenance:schema_version")
    if artifact.get("contract_version") != SCHEMA_VERSION:
        reasons.append("missing_artifact_provenance:contract_version")
    if artifact.get("task_id") != "V300-002" or artifact.get("github_issue") != 971:
        reasons.append("missing_artifact_provenance:task_identity")
    if artifact.get("milestone") != "v0.30.0" or artifact.get("release_scope") != RELEASE_SCOPE:
        reasons.append("missing_artifact_provenance:release_scope")
    if artifact.get("candidate_claim") != "production_deployment_plan_environment_readiness":
        reasons.append("missing_artifact_provenance:candidate_claim")
    if artifact.get("deployment_mode") != "source_controlled_preview_only":
        reasons.append("forbidden_execution:deployment_mode")
    if artifact.get("environment_readiness_mode") != "deterministic_readiness_replay":
        reasons.append("missing_environment:readiness_mode")
    if artifact.get("dry_run_or_preview_evidence_only") is not True:
        reasons.append("forbidden_execution:dry_run_or_preview")
    if artifact.get("production_deployment_execution_allowed") is not False:
        reasons.append("forbidden_execution:production_deployment_execution_allowed")
    if artifact.get("production_deployment_executed") is not False:
        reasons.append("forbidden_execution:production_deployment_executed")
    if set(artifact.get("dependency_contracts") or []) != DEPENDENCIES:
        reasons.append("missing_artifact_provenance:dependency_contracts")

    flags = artifact.get("boundary_flags")
    if not isinstance(flags, dict):
        reasons.append("forbidden_boundary:missing_boundary_flags")
    else:
        for key in REQUIRED_FALSE_FLAGS:
            if key not in flags:
                reasons.append(f"forbidden_boundary:missing:{key}")
            elif flags.get(key) is not False:
                reasons.append(f"forbidden_boundary:opened:{key}")

    environments_raw = artifact.get("environment_inventory")
    if not isinstance(environments_raw, list):
        reasons.append("missing_environment:environment_inventory")
        environments_raw = []
    environments: dict[str, dict[str, Any]] = {}
    for environment in environments_raw:
        if not isinstance(environment, dict):
            reasons.append("missing_environment:entry_type")
            continue
        environment_id = environment.get("environment_id")
        if not isinstance(environment_id, str):
            reasons.append("missing_environment:id")
            continue
        if environment_id in environments:
            reasons.append(f"mismatched_environment:duplicate:{environment_id}")
        environments[environment_id] = environment
        if environment_id not in EXPECTED_ENVIRONMENTS:
            reasons.append(f"mismatched_environment:unexpected:{environment_id}")
        if environment.get("required_evidence_present") is not True:
            reasons.append(f"missing_environment:required_evidence:{environment_id}")
        if environment.get("inventory_source") != "source_controlled_manifest":
            reasons.append(f"missing_environment:inventory_source:{environment_id}")
        if not environment.get("source_ref"):
            reasons.append(f"missing_environment:source_ref:{environment_id}")
        if environment.get("freshness_status") != "fresh":
            reasons.append(f"stale_environment:freshness:{environment_id}")
        if environment.get("provenance_status") != "linked":
            reasons.append(f"missing_environment:provenance:{environment_id}")
        if environment.get("artifact_binding_status") != "matched":
            reasons.append(f"mismatched_environment:artifact_binding:{environment_id}")
        if environment.get("config_binding_status") != "matched":
            reasons.append(f"mismatched_environment:config_binding:{environment_id}")
        if environment.get("live_secret_material_present") is not False:
            reasons.append(f"mismatched_environment:live_secret_material:{environment_id}")
        if environment.get("deployment_execution_allowed") is not False:
            reasons.append(f"forbidden_execution:environment_execution:{environment_id}")
    if set(environments) != EXPECTED_ENVIRONMENTS:
        reasons.append("missing_environment:environment_set")

    targets_raw = artifact.get("deployment_targets")
    if not isinstance(targets_raw, list):
        reasons.append("missing_environment:deployment_targets")
        targets_raw = []
    targets: dict[str, dict[str, Any]] = {}
    for target in targets_raw:
        if not isinstance(target, dict):
            reasons.append("missing_environment:target_type")
            continue
        target_id = target.get("target_id")
        if not isinstance(target_id, str):
            reasons.append("missing_environment:target_id")
            continue
        if target_id in targets:
            reasons.append(f"mismatched_environment:duplicate_target:{target_id}")
        targets[target_id] = target
        expected_environment = EXPECTED_TARGETS.get(target_id)
        if expected_environment is None:
            reasons.append(f"mismatched_environment:unexpected_target:{target_id}")
        elif target.get("environment_id") != expected_environment:
            reasons.append(f"mismatched_environment:target_environment:{target_id}")
        if target.get("environment_id") not in environments:
            reasons.append(f"missing_environment:target_environment_missing:{target_id}")
        if target.get("target_class") != EXPECTED_TARGET_CLASSES.get(target_id):
            reasons.append(f"mismatched_environment:target_class:{target_id}")
        if target.get("artifact_ref") != "artifact-v291-release":
            reasons.append(f"missing_artifact_provenance:target_artifact:{target_id}")
        if target.get("config_ref") not in EXPECTED_CONFIGS:
            reasons.append(f"missing_config_provenance:target_config:{target_id}")
        if target.get("dry_run_or_preview_only") is not True:
            reasons.append(f"forbidden_execution:target_preview_only:{target_id}")
        if target.get("execution_allowed") is not False:
            reasons.append(f"forbidden_execution:target_execution:{target_id}")
        if target.get("owner_operator_approval_required_later") is not True:
            reasons.append(f"missing_config_provenance:target_approval:{target_id}")
        if target.get("network_attempted") is not False:
            reasons.append(f"forbidden_execution:target_network:{target_id}")
        if target.get("live_exchange_request_allowed") is not False:
            reasons.append(f"forbidden_execution:target_exchange:{target_id}")
    if set(targets) != set(EXPECTED_TARGETS):
        reasons.append("missing_environment:target_set")

    provenance = artifact.get("artifact_provenance")
    if not isinstance(provenance, dict):
        reasons.append("missing_artifact_provenance:object")
        provenance = {}
    expected_artifact_fields = {
        "artifact_id": "artifact-v291-release",
        "release_tag": "ntpro-rust-only-v0.29.1",
        "release_commit": "a831d802e4321f50ed6e10481aea35b15a74b01e",
        "release_manifest": "docs/rust-cutover/release/v0_29_1_release_manifest.json",
        "release_closeout": "docs/rust-cutover/release/v0_29_1_release_closeout_evidence.md",
        "hosted_gate_run": "29130876713",
        "boundary_contract": "docs/rust-cutover/release/v0_30_0_backend_go_live_candidate_boundary_contract.md",
        "artifact_digest_status": "matched",
    }
    for key, expected in expected_artifact_fields.items():
        if provenance.get(key) != expected:
            reasons.append(f"missing_artifact_provenance:{key}")
    if provenance.get("source_tree_reconstructable") is not True:
        reasons.append("missing_artifact_provenance:source_tree_reconstructable")
    if provenance.get("stale_artifact_allowed") is not False:
        reasons.append("missing_artifact_provenance:stale_allowed")

    configs_raw = artifact.get("config_provenance")
    if not isinstance(configs_raw, list):
        reasons.append("missing_config_provenance:config_provenance")
        configs_raw = []
    configs: dict[str, dict[str, Any]] = {}
    for config in configs_raw:
        if not isinstance(config, dict):
            reasons.append("missing_config_provenance:entry_type")
            continue
        config_id = config.get("config_id")
        if not isinstance(config_id, str):
            reasons.append("missing_config_provenance:id")
            continue
        if config_id in configs:
            reasons.append(f"missing_config_provenance:duplicate:{config_id}")
        configs[config_id] = config
        expected_environment = EXPECTED_CONFIGS.get(config_id)
        if expected_environment is None:
            reasons.append(f"missing_config_provenance:unexpected:{config_id}")
        elif config.get("target_environment_id") != expected_environment:
            reasons.append(f"mismatched_environment:config_environment:{config_id}")
        if config.get("target_environment_id") not in environments:
            reasons.append(f"missing_environment:config_environment_missing:{config_id}")
        if not config.get("source_ref"):
            reasons.append(f"missing_config_provenance:source_ref:{config_id}")
        if config.get("freshness_status") != "fresh":
            reasons.append(f"missing_config_provenance:freshness:{config_id}")
        if config.get("provenance_status") != "linked":
            reasons.append(f"missing_config_provenance:provenance:{config_id}")
        if config.get("digest_status") != "matched":
            reasons.append(f"missing_config_provenance:digest:{config_id}")
        if config.get("unsafe_defaults_allowed") is not False:
            reasons.append(f"missing_config_provenance:unsafe_defaults:{config_id}")
        if config.get("production_secret_value_embedded") is not False:
            reasons.append(f"missing_config_provenance:secret:{config_id}")
        if config.get("execution_allowed") is not False:
            reasons.append(f"forbidden_execution:config_execution:{config_id}")
    if set(configs) != set(EXPECTED_CONFIGS):
        reasons.append("missing_config_provenance:config_set")

    migrations_raw = artifact.get("migration_upgrade_prerequisites")
    if not isinstance(migrations_raw, list):
        reasons.append("missing_migration_prerequisite:list")
        migrations_raw = []
    migrations: dict[str, dict[str, Any]] = {}
    for item in migrations_raw:
        if not isinstance(item, dict):
            reasons.append("missing_migration_prerequisite:entry_type")
            continue
        item_id = item.get("id")
        if not isinstance(item_id, str):
            reasons.append("missing_migration_prerequisite:id")
            continue
        migrations[item_id] = item
        if item_id not in EXPECTED_MIGRATIONS:
            reasons.append(f"missing_migration_prerequisite:unexpected:{item_id}")
        if item.get("status") != "preview_ready":
            reasons.append(f"missing_migration_prerequisite:status:{item_id}")
        if item.get("required_before_execution") is not True:
            reasons.append(f"missing_migration_prerequisite:required:{item_id}")
        if item.get("execution_performed") is not False:
            reasons.append(f"forbidden_execution:migration_execution:{item_id}")
        if item.get("blocking_if_missing") is not True:
            reasons.append(f"missing_migration_prerequisite:blocking:{item_id}")
    if set(migrations) != EXPECTED_MIGRATIONS:
        reasons.append("missing_migration_prerequisite:set")

    rollbacks_raw = artifact.get("rollback_checkpoints")
    if not isinstance(rollbacks_raw, list):
        reasons.append("missing_rollback_checkpoint:list")
        rollbacks_raw = []
    rollbacks: dict[str, dict[str, Any]] = {}
    for item in rollbacks_raw:
        if not isinstance(item, dict):
            reasons.append("missing_rollback_checkpoint:entry_type")
            continue
        item_id = item.get("id")
        if not isinstance(item_id, str):
            reasons.append("missing_rollback_checkpoint:id")
            continue
        rollbacks[item_id] = item
        if item_id not in EXPECTED_ROLLBACKS:
            reasons.append(f"missing_rollback_checkpoint:unexpected:{item_id}")
        if item.get("status") != "documented":
            reasons.append(f"missing_rollback_checkpoint:status:{item_id}")
        if item.get("dry_run_or_preview_only") is not True:
            reasons.append(f"forbidden_execution:rollback_preview:{item_id}")
        if item.get("execution_triggered") is not False:
            reasons.append(f"forbidden_execution:rollback_execution:{item_id}")
        if item.get("approval_required_later") is not True:
            reasons.append(f"missing_rollback_checkpoint:approval:{item_id}")
    if set(rollbacks) != EXPECTED_ROLLBACKS:
        reasons.append("missing_rollback_checkpoint:set")

    return reasons


base_status = classify_status(collect_reasons(payload))
if base_status != READY_STATUS:
    fail(f"base artifact status mismatch: {base_status}")

cases = payload.get("readiness_cases")
if not isinstance(cases, list):
    fail("readiness_cases must be a list")
seen_cases: set[str] = set()
for case in cases:
    if not isinstance(case, dict):
        fail("readiness case entries must be objects")
    case_id = case.get("case_id")
    expected = case.get("expected_status")
    if not isinstance(case_id, str) or not isinstance(expected, str):
        fail("readiness case missing id/status")
    if case_id in seen_cases:
        fail(f"duplicate readiness case: {case_id}")
    seen_cases.add(case_id)
    if case_id not in EXPECTED_CASES:
        fail(f"unexpected readiness case: {case_id}")
    snapshot = apply_case(payload, case)
    actual = classify_status(collect_reasons(snapshot))
    if actual != expected:
        fail(f"case {case_id} expected {expected}, got {actual}")
if seen_cases != EXPECTED_CASES:
    fail(f"readiness case set mismatch: {sorted(seen_cases)}")

negative_selftests = 0
if selftest:
    missing_environment = copy.deepcopy(payload)
    missing_environment["environment_inventory"][0]["required_evidence_present"] = False
    if classify_status(collect_reasons(missing_environment)) == "fail_closed_missing_environment_evidence":
        negative_selftests += 1
    else:
        fail("negative self-test unexpectedly allowed missing environment evidence")

    stale_environment = copy.deepcopy(payload)
    stale_environment["environment_inventory"][1]["freshness_status"] = "stale"
    if classify_status(collect_reasons(stale_environment)) == "fail_closed_stale_environment_evidence":
        negative_selftests += 1
    else:
        fail("negative self-test unexpectedly allowed stale environment evidence")

    mismatched_environment = copy.deepcopy(payload)
    mismatched_environment["deployment_targets"][5]["environment_id"] = "prod-candidate-primary"
    if classify_status(collect_reasons(mismatched_environment)) == "fail_closed_mismatched_environment_evidence":
        negative_selftests += 1
    else:
        fail("negative self-test unexpectedly allowed mismatched environment evidence")

    forbidden_execution = copy.deepcopy(payload)
    forbidden_execution["deployment_targets"][0]["execution_allowed"] = True
    if classify_status(collect_reasons(forbidden_execution)) == "fail_closed_forbidden_execution":
        negative_selftests += 1
    else:
        fail("negative self-test unexpectedly allowed production deployment execution")

print(
    "v30_production_deployment_plan_environment_readiness=pass "
    f"deployment_targets={len(EXPECTED_TARGETS)} "
    f"environment_inventory={len(EXPECTED_ENVIRONMENTS)} "
    f"config_provenance={len(EXPECTED_CONFIGS)} "
    f"migration_upgrade_prerequisites={len(EXPECTED_MIGRATIONS)} "
    f"rollback_checkpoints={len(EXPECTED_ROLLBACKS)} "
    f"readiness_cases={len(EXPECTED_CASES)} "
    f"required_false_flags={len(REQUIRED_FALSE_FLAGS)} "
    f"negative_selftest={negative_selftests}"
)
PY
