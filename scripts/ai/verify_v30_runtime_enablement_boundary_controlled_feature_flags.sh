#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

ARTIFACT_PATH="${NTPRO_V300_RUNTIME_FLAGS_ARTIFACT:-docs/rust-cutover/release/v0_30_0_runtime_enablement_boundary_controlled_feature_flags.json}"
CONTRACT_PATH="${NTPRO_V300_RUNTIME_FLAGS_CONTRACT:-docs/rust-cutover/release/v0_30_0_runtime_enablement_boundary_controlled_feature_flags.md}"
TASK_PATH="${NTPRO_V300_RUNTIME_FLAGS_TASK:-docs/rust-cutover/tasks/V300-003.md}"
EVIDENCE_PATH="${NTPRO_V300_RUNTIME_FLAGS_EVIDENCE:-docs/rust-cutover/evidence/V300-003.md}"
BOUNDARY_PATH="${NTPRO_V300_RUNTIME_FLAGS_BOUNDARY:-docs/rust-cutover/release/v0_30_0_backend_go_live_candidate_boundary_contract.md}"
DEPLOYMENT_READINESS="${NTPRO_V300_RUNTIME_FLAGS_DEPLOYMENT_READINESS:-docs/rust-cutover/release/v0_30_0_production_deployment_plan_environment_readiness.json}"
DEPLOYMENT_CONTRACT="${NTPRO_V300_RUNTIME_FLAGS_DEPLOYMENT_CONTRACT:-docs/rust-cutover/release/v0_30_0_production_deployment_plan_environment_readiness.md}"
RELEASE_INDEX="${NTPRO_V300_RUNTIME_FLAGS_RELEASE_INDEX:-docs/rust-cutover/release/README.md}"
SELFTEST="${NTPRO_V300_RUNTIME_FLAGS_SELFTEST:-1}"

fail() {
  echo "v30 runtime enablement boundary controlled feature flags failed: $*" >&2
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

for path in "$ARTIFACT_PATH" "$CONTRACT_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$BOUNDARY_PATH" "$DEPLOYMENT_READINESS" "$DEPLOYMENT_CONTRACT" "$RELEASE_INDEX"; do
  require_file "$path"
done

require_contains "$TASK_PATH" "GitHub issue: \`#972\`"
require_contains "$EVIDENCE_PATH" "Task: \`V300-003\` / GitHub issue \`#972\`"
require_contains "$BOUNDARY_PATH" "runtime_enablement_boundary = #972"
require_contains "$BOUNDARY_PATH" "controlled_feature_flag_inventory = #972"
require_contains "$DEPLOYMENT_CONTRACT" "Task: \`V300-002\` / GitHub issue \`#971\`"
require_contains "$CONTRACT_PATH" "contract_version = ntpro.v300.runtime_enablement_boundary_controlled_feature_flags.v1"
require_contains "$CONTRACT_PATH" "runtime_enablement_allowed = false"
require_contains "$CONTRACT_PATH" "production_feature_flags_default_enabled = false"
require_contains "$CONTRACT_PATH" "missing_approval_for_enabled_switch => fail_closed_missing_approval"
require_contains "$CONTRACT_PATH" "stale_flag_provenance => fail_closed_stale_flag_provenance"
require_contains "$CONTRACT_PATH" "unsupported_flag_combination => fail_closed_unsupported_flag_combination"
require_contains "$RELEASE_INDEX" "v0_30_0_runtime_enablement_boundary_controlled_feature_flags.md"
require_contains "$RELEASE_INDEX" "../evidence/V300-003.md"

ARTIFACT_PATH="$ARTIFACT_PATH" SELFTEST="$SELFTEST" python3 <<'PY'
from __future__ import annotations

import copy
import json
import os
from pathlib import Path
from typing import Any

payload = json.loads(Path(os.environ["ARTIFACT_PATH"]).read_text(encoding="utf-8"))
selftest = os.environ.get("SELFTEST", "1") != "0"

SCHEMA_VERSION = "ntpro.v300.runtime_enablement_boundary_controlled_feature_flags.v1"
RELEASE_SCOPE = "backend_production_go_live_candidate_foundation_only"
READY_STATUS = "runtime_enablement_boundary_controlled_feature_flags_ready"
DEPENDENCIES = {"V300-001", "V300-002", "v0.29.1-release-evidence"}
EXPECTED_PREVIEW_FLAGS = {
    "backend_read_api_runtime_bridge",
    "audit_export_pipeline_preview",
    "telemetry_slo_ingestion_runtime_bridge",
    "config_provenance_runtime_bridge",
    "canary_preflight_runtime_bridge",
    "rollback_dr_runtime_bridge",
    "operator_freeze_lifecycle_runtime_bridge",
}
FORBIDDEN_FLAGS = {
    "production_order_submit",
    "production_order_mutation",
    "execution_adapter_send",
    "live_exchange_request",
    "retry_scheduler_runtime",
    "automatic_remediation_runtime",
    "dashboard_trading_controls",
    "admin_workbench_trading_controls",
    "trader_terminal_order_ticket",
    "manual_operation_submit",
}
EXPECTED_FLAGS = EXPECTED_PREVIEW_FLAGS | FORBIDDEN_FLAGS
EXPECTED_COMBINATIONS = {
    ("production_order_submit", "execution_adapter_send"),
    ("production_order_submit", "live_exchange_request"),
    ("production_order_mutation", "execution_adapter_send"),
    ("production_order_mutation", "live_exchange_request"),
    ("retry_scheduler_runtime", "automatic_remediation_runtime"),
    ("dashboard_trading_controls", "trader_terminal_order_ticket"),
    ("manual_operation_submit", "execution_adapter_send"),
}
EXPECTED_CASES = {
    "runtime_enablement_flags.preview_inventory.allowed.001",
    "runtime_enablement_flags.default_enabled.fail_closed.001",
    "runtime_enablement_flags.missing_approval.fail_closed.001",
    "runtime_enablement_flags.missing_audit.fail_closed.001",
    "runtime_enablement_flags.stale_provenance.fail_closed.001",
    "runtime_enablement_flags.unsupported_submit_adapter.fail_closed.001",
    "runtime_enablement_flags.forbidden_trading_flag.fail_closed.001",
    "runtime_enablement_flags.missing_scoped_issue.fail_closed.001",
    "runtime_enablement_flags.forbidden_boundary.fail_closed.001",
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
]


def fail(message: str) -> None:
    raise SystemExit(f"v30 runtime enablement boundary controlled feature flags failed: {message}")


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
    if case.get("approval_gate_override"):
        result["approval_gate"] = merge(result["approval_gate"], case["approval_gate_override"])
    if case.get("audit_gate_override"):
        result["audit_gate"] = merge(result["audit_gate"], case["audit_gate_override"])
    if case.get("boundary_flags_override"):
        result["boundary_flags"] = merge(result["boundary_flags"], case["boundary_flags_override"])
    if case.get("runtime_switch_overrides"):
        result["runtime_switches"] = apply_indexed_overrides(
            result["runtime_switches"],
            "flag_id",
            case["runtime_switch_overrides"],
        )
    return result


def classify_status(reasons: list[str]) -> str:
    if any(reason.startswith("forbidden_boundary") for reason in reasons):
        return "fail_closed_forbidden_boundary"
    if any(reason.startswith("default_enabled") for reason in reasons):
        return "fail_closed_default_enabled"
    if any(reason.startswith("stale_flag_provenance") for reason in reasons):
        return "fail_closed_stale_flag_provenance"
    if any(reason.startswith("unsupported_flag_combination") for reason in reasons):
        return "fail_closed_unsupported_flag_combination"
    if any(reason.startswith("forbidden_trading_flag") for reason in reasons):
        return "fail_closed_forbidden_trading_flag"
    if any(reason.startswith("missing_approval") for reason in reasons):
        return "fail_closed_missing_approval"
    if any(reason.startswith("missing_audit") for reason in reasons):
        return "fail_closed_missing_audit_evidence"
    if any(reason.startswith("missing_scoped_issue") for reason in reasons):
        return "fail_closed_missing_scoped_enablement_issue"
    if reasons:
        return "fail_closed_forbidden_boundary"
    return READY_STATUS


def collect_reasons(artifact: dict[str, Any]) -> list[str]:
    reasons: list[str] = []
    if artifact.get("schema_version") != SCHEMA_VERSION:
        reasons.append("forbidden_boundary:schema_version")
    if artifact.get("contract_version") != SCHEMA_VERSION:
        reasons.append("forbidden_boundary:contract_version")
    if artifact.get("task_id") != "V300-003" or artifact.get("github_issue") != 972:
        reasons.append("forbidden_boundary:task_identity")
    if artifact.get("milestone") != "v0.30.0" or artifact.get("release_scope") != RELEASE_SCOPE:
        reasons.append("forbidden_boundary:release_scope")
    if artifact.get("candidate_claim") != "runtime_enablement_boundary_controlled_feature_flags":
        reasons.append("forbidden_boundary:candidate_claim")
    if artifact.get("runtime_enablement_mode") != "preview_inventory_only":
        reasons.append("forbidden_boundary:runtime_enablement_mode")
    if artifact.get("controlled_feature_flag_mode") != "default_disabled_inventory":
        reasons.append("default_enabled:mode")
    if artifact.get("runtime_enablement_allowed") is not False:
        reasons.append("forbidden_boundary:runtime_enablement_allowed")
    if artifact.get("production_feature_flags_default_enabled") is not False:
        reasons.append("default_enabled:production_feature_flags")
    if set(artifact.get("dependency_contracts") or []) != DEPENDENCIES:
        reasons.append("forbidden_boundary:dependency_contracts")

    approval_gate = artifact.get("approval_gate") or {}
    if approval_gate.get("owner_operator_approval_required_before_enablement") is not True:
        reasons.append("missing_approval:required")
    if approval_gate.get("approval_bypass_allowed") is not False:
        reasons.append("missing_approval:bypass")
    if approval_gate.get("scoped_enablement_issue_required") is not True:
        reasons.append("missing_scoped_issue:required")

    audit_gate = artifact.get("audit_gate") or {}
    if audit_gate.get("audit_evidence_required_before_enablement") is not True:
        reasons.append("missing_audit:required")
    if audit_gate.get("shared_approval_consumption_allowed") is not False:
        reasons.append("missing_audit:shared_approval")
    if audit_gate.get("audit_bypass_allowed") is not False:
        reasons.append("missing_audit:bypass")

    flags = artifact.get("boundary_flags")
    if not isinstance(flags, dict):
        reasons.append("forbidden_boundary:missing_boundary_flags")
    else:
        for key in REQUIRED_FALSE_FLAGS:
            if key not in flags:
                reasons.append(f"forbidden_boundary:missing:{key}")
            elif flags.get(key) is not False:
                reasons.append(f"forbidden_boundary:opened:{key}")

    switches_raw = artifact.get("runtime_switches")
    if not isinstance(switches_raw, list):
        reasons.append("forbidden_boundary:runtime_switches")
        switches_raw = []
    switches: dict[str, dict[str, Any]] = {}
    enabled_flags: set[str] = set()
    for switch in switches_raw:
        if not isinstance(switch, dict):
            reasons.append("forbidden_boundary:switch_type")
            continue
        flag_id = switch.get("flag_id")
        if not isinstance(flag_id, str):
            reasons.append("forbidden_boundary:switch_id")
            continue
        if flag_id in switches:
            reasons.append(f"forbidden_boundary:duplicate_switch:{flag_id}")
        switches[flag_id] = switch
        if flag_id not in EXPECTED_FLAGS:
            reasons.append(f"forbidden_boundary:unexpected_switch:{flag_id}")

        expected_class = "candidate_preview_switch" if flag_id in EXPECTED_PREVIEW_FLAGS else None
        if flag_id in FORBIDDEN_FLAGS:
            expected_class = {
                "production_order_submit": "forbidden_trading_switch",
                "production_order_mutation": "forbidden_trading_switch",
                "execution_adapter_send": "forbidden_adapter_switch",
                "live_exchange_request": "forbidden_exchange_switch",
                "retry_scheduler_runtime": "forbidden_automation_switch",
                "automatic_remediation_runtime": "forbidden_automation_switch",
                "dashboard_trading_controls": "forbidden_trading_control_switch",
                "admin_workbench_trading_controls": "forbidden_trading_control_switch",
                "trader_terminal_order_ticket": "forbidden_trading_control_switch",
                "manual_operation_submit": "forbidden_trading_control_switch",
            }[flag_id]
        if switch.get("flag_class") != expected_class:
            reasons.append(f"forbidden_boundary:flag_class:{flag_id}")

        expected_state = "preview" if flag_id in EXPECTED_PREVIEW_FLAGS else "disabled"
        if switch.get("current_state") != expected_state:
            reasons.append(f"forbidden_boundary:state:{flag_id}")
        if switch.get("default_enabled") is not False:
            reasons.append(f"default_enabled:{flag_id}")
        if switch.get("runtime_effect_allowed") is not False:
            reasons.append(f"forbidden_boundary:runtime_effect:{flag_id}")
        if switch.get("provenance_status") != "fresh":
            reasons.append(f"stale_flag_provenance:{flag_id}")
        for key in (
            "requires_later_scoped_enablement_issue",
            "requires_approval_before_enablement",
            "requires_audit_before_enablement",
        ):
            if switch.get(key) is not True:
                reasons.append(f"forbidden_boundary:{key}:{flag_id}")
        if switch.get("enabled") is True:
            enabled_flags.add(flag_id)
        elif switch.get("enabled") is not False:
            reasons.append(f"forbidden_boundary:enabled_type:{flag_id}")
    if set(switches) != EXPECTED_FLAGS:
        reasons.append("forbidden_boundary:switch_set")

    combinations_raw = artifact.get("unsupported_combinations")
    if not isinstance(combinations_raw, list):
        reasons.append("unsupported_flag_combination:missing_list")
        combinations_raw = []
    combinations = {tuple(item) for item in combinations_raw if isinstance(item, list) and len(item) == 2}
    if combinations != EXPECTED_COMBINATIONS:
        reasons.append("unsupported_flag_combination:set")
    for first, second in EXPECTED_COMBINATIONS:
        if first in enabled_flags and second in enabled_flags:
            reasons.append(f"unsupported_flag_combination:{first}+{second}")

    for flag_id in sorted(enabled_flags):
        if flag_id in FORBIDDEN_FLAGS:
            reasons.append(f"forbidden_trading_flag:{flag_id}")
        if approval_gate.get("approval_evidence_present") is not True:
            reasons.append(f"missing_approval:{flag_id}")
        if audit_gate.get("audit_evidence_present") is not True:
            reasons.append(f"missing_audit:{flag_id}")
        if approval_gate.get("scoped_enablement_issue_present") is not True:
            reasons.append(f"missing_scoped_issue:{flag_id}")

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
    default_enabled = copy.deepcopy(payload)
    default_enabled["runtime_switches"][0]["default_enabled"] = True
    if classify_status(collect_reasons(default_enabled)) == "fail_closed_default_enabled":
        negative_selftests += 1
    else:
        fail("negative self-test unexpectedly allowed default-enabled flag")

    missing_approval = copy.deepcopy(payload)
    missing_approval["runtime_switches"][0]["enabled"] = True
    if classify_status(collect_reasons(missing_approval)) == "fail_closed_missing_approval":
        negative_selftests += 1
    else:
        fail("negative self-test unexpectedly allowed missing approval")

    stale_provenance = copy.deepcopy(payload)
    stale_provenance["runtime_switches"][2]["provenance_status"] = "stale"
    if classify_status(collect_reasons(stale_provenance)) == "fail_closed_stale_flag_provenance":
        negative_selftests += 1
    else:
        fail("negative self-test unexpectedly allowed stale flag provenance")

    unsupported = copy.deepcopy(payload)
    unsupported["approval_gate"]["approval_evidence_present"] = True
    unsupported["approval_gate"]["scoped_enablement_issue_present"] = True
    unsupported["audit_gate"]["audit_evidence_present"] = True
    for switch in unsupported["runtime_switches"]:
        if switch["flag_id"] in {"production_order_submit", "execution_adapter_send"}:
            switch["enabled"] = True
    if classify_status(collect_reasons(unsupported)) == "fail_closed_unsupported_flag_combination":
        negative_selftests += 1
    else:
        fail("negative self-test unexpectedly allowed unsupported flag combination")

print(
    "v30_runtime_enablement_boundary_controlled_feature_flags=pass "
    f"runtime_switches={len(EXPECTED_FLAGS)} "
    f"preview_switches={len(EXPECTED_PREVIEW_FLAGS)} "
    f"forbidden_flags={len(FORBIDDEN_FLAGS)} "
    f"unsupported_combinations={len(EXPECTED_COMBINATIONS)} "
    f"readiness_cases={len(EXPECTED_CASES)} "
    f"required_false_flags={len(REQUIRED_FALSE_FLAGS)} "
    f"negative_selftest={negative_selftests}"
)
PY
