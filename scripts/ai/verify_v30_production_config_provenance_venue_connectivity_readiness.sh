#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

ARTIFACT_PATH="${NTPRO_V300_CONFIG_VENUE_ARTIFACT:-docs/rust-cutover/release/v0_30_0_production_config_provenance_venue_connectivity_readiness.json}"
CONTRACT_PATH="${NTPRO_V300_CONFIG_VENUE_CONTRACT:-docs/rust-cutover/release/v0_30_0_production_config_provenance_venue_connectivity_readiness.md}"
TASK_PATH="${NTPRO_V300_CONFIG_VENUE_TASK:-docs/rust-cutover/tasks/V300-007.md}"
EVIDENCE_PATH="${NTPRO_V300_CONFIG_VENUE_EVIDENCE:-docs/rust-cutover/evidence/V300-007.md}"
DEPLOYMENT_READINESS="${NTPRO_V300_CONFIG_VENUE_DEPLOYMENT:-docs/rust-cutover/release/v0_30_0_production_deployment_plan_environment_readiness.json}"
RUNTIME_FLAGS="${NTPRO_V300_CONFIG_VENUE_RUNTIME_FLAGS:-docs/rust-cutover/release/v0_30_0_runtime_enablement_boundary_controlled_feature_flags.json}"
BOUNDARY_PATH="${NTPRO_V300_CONFIG_VENUE_BOUNDARY:-docs/rust-cutover/release/v0_30_0_backend_go_live_candidate_boundary_contract.md}"
V29_DEPLOYMENT="${NTPRO_V300_CONFIG_VENUE_V29_DEPLOYMENT:-docs/rust-cutover/release/v0_29_0_deployment_config_runbook_production_readiness_artifact.json}"
RELEASE_INDEX="${NTPRO_V300_CONFIG_VENUE_RELEASE_INDEX:-docs/rust-cutover/release/README.md}"
SELFTEST="${NTPRO_V300_CONFIG_VENUE_SELFTEST:-1}"

fail() {
  echo "v30 production config provenance venue connectivity readiness failed: $*" >&2
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

for path in "$ARTIFACT_PATH" "$CONTRACT_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$DEPLOYMENT_READINESS" "$RUNTIME_FLAGS" "$BOUNDARY_PATH" "$V29_DEPLOYMENT" "$RELEASE_INDEX"; do
  require_file "$path"
done

require_contains "$TASK_PATH" "GitHub issue: \`#976\`"
require_contains "$EVIDENCE_PATH" "Task: \`V300-007\` / GitHub issue \`#976\`"
require_contains "$BOUNDARY_PATH" "production_config_provenance = #976"
require_contains "$BOUNDARY_PATH" "venue_connectivity_readiness = #976"
require_contains "$CONTRACT_PATH" "contract_version = ntpro.v300.production_config_provenance_venue_connectivity_readiness.v1"
require_contains "$CONTRACT_PATH" "adapter_send_allowed = false"
require_contains "$CONTRACT_PATH" "live_exchange_request_allowed = false"
require_contains "$CONTRACT_PATH" "unredacted_sensitive_fields => fail_closed_unredacted_sensitive_fields"
require_contains "$CONTRACT_PATH" "live_exchange_request_attempt => fail_closed_live_exchange_request_attempt"
require_contains "$RELEASE_INDEX" "v0_30_0_production_config_provenance_venue_connectivity_readiness.md"
require_contains "$RELEASE_INDEX" "../evidence/V300-007.md"

ARTIFACT_PATH="$ARTIFACT_PATH" SELFTEST="$SELFTEST" python3 <<'PY'
from __future__ import annotations

import copy
import json
import os
from pathlib import Path
from typing import Any

payload = json.loads(Path(os.environ["ARTIFACT_PATH"]).read_text(encoding="utf-8"))
selftest = os.environ.get("SELFTEST", "1") != "0"

SCHEMA_VERSION = "ntpro.v300.production_config_provenance_venue_connectivity_readiness.v1"
RELEASE_SCOPE = "backend_production_go_live_candidate_foundation_only"
READY_STATUS = "production_config_provenance_venue_connectivity_readiness_ready"
DEPENDENCIES = {"V300-001", "V300-002", "V300-003", "v0.29.1-release-evidence"}
EXPECTED_CONFIGS = {
    "prod-primary-config": "prod-candidate-primary",
    "prod-canary-config": "prod-candidate-canary",
    "prod-dr-config": "prod-candidate-dr",
}
EXPECTED_VENUES = {
    "primary-venue-marketdata-readiness": "prod-candidate-primary",
    "primary-venue-execution-disabled-readiness": "prod-candidate-primary",
    "dr-venue-connectivity-reference": "prod-candidate-dr",
}
EXPECTED_CREDENTIALS = {
    "api_key_reference",
    "api_secret_reference",
    "session_token_reference",
    "credential_rotation_runbook",
}
EXPECTED_CASES = {
    "config_venue_readiness.preview.allowed.001",
    "config_venue_readiness.unredacted_sensitive.fail_closed.001",
    "config_venue_readiness.missing_provenance.fail_closed.001",
    "config_venue_readiness.stale_config.fail_closed.001",
    "config_venue_readiness.environment_binding_mismatch.fail_closed.001",
    "config_venue_readiness.credential_material.fail_closed.001",
    "config_venue_readiness.adapter_send_attempt.fail_closed.001",
    "config_venue_readiness.live_exchange_request.fail_closed.001",
    "config_venue_readiness.order_send_permission.fail_closed.001",
    "config_venue_readiness.forbidden_boundary.fail_closed.001",
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
    "runtime_switch_enablement_allowed",
    "candidate_operation_execution_allowed",
    "approval_lifecycle_authorizes_trading_operations",
    "canary_execution_allowed",
    "default_canary_execution_allowed",
    "production_canary_action_executed",
    "live_exchange_side_effect_allowed",
    "rollback_execution_allowed",
    "production_rollback_execution_allowed",
    "dr_restore_execution_allowed",
    "data_restore_execution_allowed",
    "service_restart_execution_allowed",
    "ambiguous_rollback_execution_claim_allowed",
    "unredacted_sensitive_fields_present",
    "credential_material_present",
    "adapter_send_attempted",
    "live_exchange_request_attempted",
    "order_send_permission_allowed",
    "connectivity_probe_network_attempted",
]


def fail(message: str) -> None:
    raise SystemExit(f"v30 production config provenance venue connectivity readiness failed: {message}")


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
    if case.get("production_config_overrides"):
        result["production_configs"] = apply_indexed_overrides(
            result["production_configs"],
            "config_id",
            case["production_config_overrides"],
        )
    if case.get("venue_readiness_overrides"):
        result["venue_connectivity_readiness"] = apply_indexed_overrides(
            result["venue_connectivity_readiness"],
            "venue_id",
            case["venue_readiness_overrides"],
        )
    if case.get("credential_boundary_overrides"):
        result["credential_boundaries"] = apply_indexed_overrides(
            result["credential_boundaries"],
            "credential_id",
            case["credential_boundary_overrides"],
        )
    if case.get("boundary_flags_override"):
        result["boundary_flags"] = merge(result["boundary_flags"], case["boundary_flags_override"])
    return result


def classify_status(reasons: list[str]) -> str:
    if any(reason.startswith("forbidden_boundary") for reason in reasons):
        return "fail_closed_forbidden_boundary"
    if any(reason.startswith("unredacted_sensitive_fields") for reason in reasons):
        return "fail_closed_unredacted_sensitive_fields"
    if any(reason.startswith("missing_provenance") for reason in reasons):
        return "fail_closed_missing_provenance"
    if any(reason.startswith("stale_config") for reason in reasons):
        return "fail_closed_stale_config"
    if any(reason.startswith("environment_binding_mismatch") for reason in reasons):
        return "fail_closed_environment_binding_mismatch"
    if any(reason.startswith("credential_material_boundary") for reason in reasons):
        return "fail_closed_credential_material_boundary"
    if any(reason.startswith("adapter_send_attempt") for reason in reasons):
        return "fail_closed_adapter_send_attempt"
    if any(reason.startswith("live_exchange_request_attempt") for reason in reasons):
        return "fail_closed_live_exchange_request_attempt"
    if any(reason.startswith("order_send_permission") for reason in reasons):
        return "fail_closed_order_send_permission"
    if reasons:
        return "fail_closed_forbidden_boundary"
    return READY_STATUS


def collect_reasons(artifact: dict[str, Any]) -> list[str]:
    reasons: list[str] = []
    if artifact.get("schema_version") != SCHEMA_VERSION:
        reasons.append("forbidden_boundary:schema_version")
    if artifact.get("contract_version") != SCHEMA_VERSION:
        reasons.append("forbidden_boundary:contract_version")
    if artifact.get("task_id") != "V300-007" or artifact.get("github_issue") != 976:
        reasons.append("forbidden_boundary:task_identity")
    if artifact.get("milestone") != "v0.30.0" or artifact.get("release_scope") != RELEASE_SCOPE:
        reasons.append("forbidden_boundary:release_scope")
    if artifact.get("candidate_claim") != "production_config_provenance_venue_connectivity_readiness":
        reasons.append("forbidden_boundary:candidate_claim")
    if artifact.get("config_readiness_mode") != "source_controlled_readonly_probe_plan_only":
        reasons.append("forbidden_boundary:config_mode")
    if artifact.get("venue_connectivity_mode") != "readiness_without_live_exchange_mutation":
        reasons.append("forbidden_boundary:venue_mode")
    for key in ("adapter_send_allowed", "live_exchange_request_allowed", "order_send_permission_allowed"):
        if artifact.get(key) is not False:
            reasons.append(f"forbidden_boundary:top_level:{key}")
    if set(artifact.get("dependency_contracts") or []) != DEPENDENCIES:
        reasons.append("forbidden_boundary:dependency_contracts")

    flags = artifact.get("boundary_flags")
    if not isinstance(flags, dict):
        reasons.append("forbidden_boundary:missing_boundary_flags")
    else:
        for key in REQUIRED_FALSE_FLAGS:
            if key not in flags:
                reasons.append(f"forbidden_boundary:missing:{key}")
            elif flags.get(key) is not False:
                reasons.append(f"forbidden_boundary:opened:{key}")

    configs_raw = artifact.get("production_configs")
    if not isinstance(configs_raw, list):
        reasons.append("missing_provenance:configs")
        configs_raw = []
    configs: dict[str, dict[str, Any]] = {}
    for config in configs_raw:
        if not isinstance(config, dict):
            reasons.append("missing_provenance:config_type")
            continue
        config_id = config.get("config_id")
        if not isinstance(config_id, str):
            reasons.append("missing_provenance:config_id")
            continue
        configs[config_id] = config
        expected_environment = EXPECTED_CONFIGS.get(config_id)
        if expected_environment is None:
            reasons.append(f"missing_provenance:unexpected_config:{config_id}")
        elif config.get("environment_id") != expected_environment:
            reasons.append(f"environment_binding_mismatch:config:{config_id}")
        if not config.get("source_ref"):
            reasons.append(f"missing_provenance:source:{config_id}")
        if config.get("provenance_status") != "linked":
            reasons.append(f"missing_provenance:status:{config_id}")
        if config.get("redaction_status") != "redacted":
            reasons.append(f"unredacted_sensitive_fields:redaction:{config_id}")
        if config.get("digest_status") != "matched":
            reasons.append(f"missing_provenance:digest:{config_id}")
        if config.get("environment_binding_status") != "matched":
            reasons.append(f"environment_binding_mismatch:binding:{config_id}")
        if config.get("freshness_status") != "fresh":
            reasons.append(f"stale_config:{config_id}")
        if config.get("credential_material_handling") != "reference_only_no_secret_material":
            reasons.append(f"credential_material_boundary:handling:{config_id}")
        if config.get("credential_material_present") is not False:
            reasons.append(f"credential_material_boundary:config:{config_id}")
        if config.get("unredacted_sensitive_fields_present") is not False:
            reasons.append(f"unredacted_sensitive_fields:config:{config_id}")
    if set(configs) != set(EXPECTED_CONFIGS):
        reasons.append("missing_provenance:config_set")

    venues_raw = artifact.get("venue_connectivity_readiness")
    if not isinstance(venues_raw, list):
        reasons.append("missing_provenance:venues")
        venues_raw = []
    venues: dict[str, dict[str, Any]] = {}
    for venue in venues_raw:
        if not isinstance(venue, dict):
            reasons.append("missing_provenance:venue_type")
            continue
        venue_id = venue.get("venue_id")
        if not isinstance(venue_id, str):
            reasons.append("missing_provenance:venue_id")
            continue
        venues[venue_id] = venue
        expected_environment = EXPECTED_VENUES.get(venue_id)
        if expected_environment is None:
            reasons.append(f"missing_provenance:unexpected_venue:{venue_id}")
        elif venue.get("environment_binding") != expected_environment:
            reasons.append(f"environment_binding_mismatch:venue:{venue_id}")
        if not venue.get("endpoint_class"):
            reasons.append(f"missing_provenance:endpoint_class:{venue_id}")
        if venue.get("provenance_status") != "linked":
            reasons.append(f"missing_provenance:venue:{venue_id}")
        if venue.get("freshness_status") != "fresh":
            reasons.append(f"stale_config:venue:{venue_id}")
        if venue.get("probe_mode") != "source_controlled_probe_plan_only":
            reasons.append(f"forbidden_boundary:probe_mode:{venue_id}")
        if venue.get("credential_material_status") != "not_present_reference_only":
            reasons.append(f"credential_material_boundary:venue:{venue_id}")
        if venue.get("network_attempted") is not False:
            reasons.append(f"live_exchange_request_attempt:network:{venue_id}")
        if venue.get("adapter_send_attempted") is not False:
            reasons.append(f"adapter_send_attempt:{venue_id}")
        if venue.get("live_exchange_request_attempted") is not False:
            reasons.append(f"live_exchange_request_attempt:{venue_id}")
        if venue.get("order_send_permission_allowed") is not False:
            reasons.append(f"order_send_permission:{venue_id}")
    if set(venues) != set(EXPECTED_VENUES):
        reasons.append("missing_provenance:venue_set")

    credentials_raw = artifact.get("credential_boundaries")
    if not isinstance(credentials_raw, list):
        reasons.append("credential_material_boundary:credentials")
        credentials_raw = []
    credentials: dict[str, dict[str, Any]] = {}
    for credential in credentials_raw:
        if not isinstance(credential, dict):
            reasons.append("credential_material_boundary:credential_type")
            continue
        credential_id = credential.get("credential_id")
        if not isinstance(credential_id, str):
            reasons.append("credential_material_boundary:credential_id")
            continue
        credentials[credential_id] = credential
        if credential_id not in EXPECTED_CREDENTIALS:
            reasons.append(f"credential_material_boundary:unexpected:{credential_id}")
        if not credential.get("handling_status"):
            reasons.append(f"credential_material_boundary:handling:{credential_id}")
        if credential.get("material_present") is not False:
            reasons.append(f"credential_material_boundary:material:{credential_id}")
        if not credential.get("source_ref"):
            reasons.append(f"missing_provenance:credential_source:{credential_id}")
    if set(credentials) != EXPECTED_CREDENTIALS:
        reasons.append("credential_material_boundary:credential_set")

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
    unredacted = copy.deepcopy(payload)
    unredacted["production_configs"][0]["unredacted_sensitive_fields_present"] = True
    if classify_status(collect_reasons(unredacted)) == "fail_closed_unredacted_sensitive_fields":
        negative_selftests += 1
    else:
        fail("negative self-test unexpectedly allowed unredacted sensitive fields")

    stale = copy.deepcopy(payload)
    stale["production_configs"][2]["freshness_status"] = "stale"
    if classify_status(collect_reasons(stale)) == "fail_closed_stale_config":
        negative_selftests += 1
    else:
        fail("negative self-test unexpectedly allowed stale config")

    live_request = copy.deepcopy(payload)
    live_request["venue_connectivity_readiness"][0]["live_exchange_request_attempted"] = True
    if classify_status(collect_reasons(live_request)) == "fail_closed_live_exchange_request_attempt":
        negative_selftests += 1
    else:
        fail("negative self-test unexpectedly allowed live exchange request attempt")

    order_permission = copy.deepcopy(payload)
    order_permission["venue_connectivity_readiness"][0]["order_send_permission_allowed"] = True
    if classify_status(collect_reasons(order_permission)) == "fail_closed_order_send_permission":
        negative_selftests += 1
    else:
        fail("negative self-test unexpectedly allowed order send permission")

print(
    "v30_production_config_provenance_venue_connectivity_readiness=pass "
    f"production_configs={len(EXPECTED_CONFIGS)} "
    f"venue_readiness={len(EXPECTED_VENUES)} "
    f"credential_boundaries={len(EXPECTED_CREDENTIALS)} "
    f"readiness_cases={len(EXPECTED_CASES)} "
    f"required_false_flags={len(REQUIRED_FALSE_FLAGS)} "
    f"negative_selftest={negative_selftests}"
)
PY
