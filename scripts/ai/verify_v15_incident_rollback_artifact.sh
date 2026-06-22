#!/usr/bin/env bash
set -euo pipefail

# V150-007: incident, rollback, and emergency-stop artifact contracts.
# This is a local artifact gate only. It validates manual evidence documents and
# rejects any automatic remediation or real exchange mutation fields.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

ARTIFACT_ROOT="${NTPRO_V15_INCIDENT_ROLLBACK_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v15-incident-rollback.XXXXXX")}"
OUTPUT_DIR="$ARTIFACT_ROOT/artifacts"
mkdir -p "$OUTPUT_DIR"

python3 - "$OUTPUT_DIR" <<'PY'
import copy
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])

SAFE_FALSE_FIELDS = {
    "automatic_remediation_allowed",
    "automatic_remediation_attempted",
    "automatic_cancel_allowed",
    "automatic_cancel_attempted",
    "automatic_correction_allowed",
    "automatic_correction_attempted",
    "automatic_retry_allowed",
    "automatic_retry_attempted",
    "production_cancel_allowed",
    "production_cancel_attempted",
    "production_correction_allowed",
    "production_correction_attempted",
    "production_retry_allowed",
    "production_retry_attempted",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "real_exchange_mutation_allowed",
    "real_exchange_mutation_attempted",
    "network_attempted",
    "execution_adapter_called",
    "production_adapter_called",
    "cancel_replace_amend_attempted",
    "dashboard_order_controls_enabled",
    "real_orders_submitted",
    "real_funds",
    "production_trading_enabled",
    "executes_automatically",
}
SAFE_ZERO_FIELDS = {
    "production_orders_submitted",
    "production_order_mutations_attempted",
    "automatic_correction_orders_submitted",
    "production_cancels_attempted",
    "production_corrections_attempted",
    "production_retries_attempted",
    "real_exchange_mutations_attempted",
}
REQUIRED_COMMON_FALSE = {
    "automatic_remediation_allowed",
    "production_cancel_allowed",
    "production_correction_allowed",
    "production_retry_allowed",
    "real_exchange_mutation_allowed",
    "network_attempted",
    "execution_adapter_called",
    "dashboard_order_controls_enabled",
}
REQUIRED_COMMON_ZERO = {
    "production_orders_submitted",
    "production_order_mutations_attempted",
}
EXPECTED = {
    "incident_plan.json": ("ntpro.v150_incident_plan.v1", "manual_incident_plan"),
    "rollback_plan.json": ("ntpro.v150_rollback_plan.v1", "manual_rollback_plan"),
    "emergency_stop.json": ("ntpro.v150_emergency_stop.v1", "manual_emergency_stop"),
}


def write_json(path: Path, value: dict) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def common_fields() -> dict:
    return {
        "created_at": "2026-06-22T00:00:00Z",
        "manual_evidence_only": True,
        "manual_operator_required": True,
        "automatic_remediation_allowed": False,
        "automatic_remediation_attempted": False,
        "automatic_cancel_allowed": False,
        "automatic_cancel_attempted": False,
        "automatic_correction_allowed": False,
        "automatic_correction_attempted": False,
        "automatic_retry_allowed": False,
        "automatic_retry_attempted": False,
        "production_cancel_allowed": False,
        "production_cancel_attempted": False,
        "production_correction_allowed": False,
        "production_correction_attempted": False,
        "production_retry_allowed": False,
        "production_retry_attempted": False,
        "production_order_submission_allowed": False,
        "production_order_mutation_allowed": False,
        "real_exchange_mutation_allowed": False,
        "real_exchange_mutation_attempted": False,
        "network_attempted": False,
        "execution_adapter_called": False,
        "production_adapter_called": False,
        "cancel_replace_amend_attempted": False,
        "dashboard_order_controls_enabled": False,
        "real_orders_submitted": False,
        "real_funds": False,
        "production_trading_enabled": False,
        "production_orders_submitted": 0,
        "production_order_mutations_attempted": 0,
        "automatic_correction_orders_submitted": 0,
        "production_cancels_attempted": 0,
        "production_corrections_attempted": 0,
        "production_retries_attempted": 0,
        "real_exchange_mutations_attempted": 0,
    }


incident = {
    **common_fields(),
    "schema_version": "ntpro.v150_incident_plan.v1",
    "artifact_type": "manual_incident_plan",
    "status": "manual_review_required",
    "incident_id": "incident-v150-007-001",
    "severity": "high",
    "trigger": "mutation_dry_run_preflight_rejected",
    "affected_scope": {
        "venue": "BINANCE",
        "account_label": "production-alpha-dry-run",
        "symbol": "BTCUSDT",
        "strategy_id": "ema_cross_btcusdt_v1",
    },
    "operator_actions": [
        {
            "name": "freeze_strategy_session",
            "mode": "manual_evidence_only",
            "executes_automatically": False,
        },
        {
            "name": "collect_artifacts",
            "mode": "manual_evidence_only",
            "executes_automatically": False,
        },
    ],
    "diagnostic": "manual incident plan only; no exchange mutation is executed",
}

rollback = {
    **common_fields(),
    "schema_version": "ntpro.v150_rollback_plan.v1",
    "artifact_type": "manual_rollback_plan",
    "status": "manual_rollback_required",
    "rollback_id": "rollback-v150-007-001",
    "source_incident_id": incident["incident_id"],
    "rollback_steps": [
        {
            "name": "restore_previous_config",
            "mode": "manual_evidence_only",
            "executes_automatically": False,
        },
        {
            "name": "keep_kill_switch_active",
            "mode": "manual_evidence_only",
            "executes_automatically": False,
        },
    ],
    "diagnostic": "manual rollback plan only; cancel/correction/retry remains disabled",
}

emergency_stop = {
    **common_fields(),
    "schema_version": "ntpro.v150_emergency_stop.v1",
    "artifact_type": "manual_emergency_stop",
    "status": "emergency_stop_required",
    "emergency_stop_id": "emergency-stop-v150-007-001",
    "source_incident_id": incident["incident_id"],
    "kill_switch_target_state": "active",
    "stop_scope": {
        "node_id": "node-live-alpha-dry-run-001",
        "strategy_id": "ema_cross_btcusdt_v1",
        "symbol": "BTCUSDT",
    },
    "stop_actions": [
        {
            "name": "operator_confirms_stop",
            "mode": "manual_evidence_only",
            "executes_automatically": False,
        }
    ],
    "diagnostic": "manual emergency-stop evidence only; no production cancel is executed",
}

fixtures = {
    "incident_plan.json": incident,
    "rollback_plan.json": rollback,
    "emergency_stop.json": emergency_stop,
}

for name, artifact in fixtures.items():
    write_json(root / name, artifact)


def walk(value, path="$"):
    if isinstance(value, dict):
        for key, child in value.items():
            yield f"{path}.{key}", key, child
            yield from walk(child, f"{path}.{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            yield from walk(child, f"{path}[{index}]")


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def validate_artifact(name: str, artifact: dict) -> None:
    expected_schema, expected_type = EXPECTED[name]
    require(artifact.get("schema_version") == expected_schema, f"{name}: schema mismatch")
    require(artifact.get("artifact_type") == expected_type, f"{name}: artifact_type mismatch")
    require(artifact.get("manual_evidence_only") is True, f"{name}: manual_evidence_only required")
    require(artifact.get("manual_operator_required") is True, f"{name}: manual operator required")
    for key in REQUIRED_COMMON_FALSE:
        require(artifact.get(key) is False, f"{name}: {key} must be false")
    for key in REQUIRED_COMMON_ZERO:
        require(artifact.get(key) == 0, f"{name}: {key} must be zero")
    for path, key, value in walk(artifact):
        if key in SAFE_FALSE_FIELDS:
            require(value is False, f"{name}: {path} must be false, got {value!r}")
        if key in SAFE_ZERO_FIELDS:
            require(value == 0, f"{name}: {path} must be zero, got {value!r}")
        if key == "mode":
            require(value == "manual_evidence_only", f"{name}: {path} must be manual_evidence_only")


for name, artifact in fixtures.items():
    validate_artifact(name, artifact)


def assert_rejected(field: str, value) -> None:
    mutated = copy.deepcopy(incident)
    mutated[field] = value
    try:
        validate_artifact("incident_plan.json", mutated)
    except AssertionError:
        return
    raise SystemExit(f"validator accepted forbidden field {field}={value!r}")


for field in [
    "automatic_remediation_allowed",
    "production_cancel_allowed",
    "production_correction_allowed",
    "production_retry_allowed",
    "real_exchange_mutation_allowed",
    "network_attempted",
    "execution_adapter_called",
    "dashboard_order_controls_enabled",
    "cancel_replace_amend_attempted",
]:
    assert_rejected(field, True)

for field in [
    "production_orders_submitted",
    "production_order_mutations_attempted",
    "automatic_correction_orders_submitted",
    "production_cancels_attempted",
    "production_corrections_attempted",
    "production_retries_attempted",
    "real_exchange_mutations_attempted",
]:
    assert_rejected(field, 1)

nested = copy.deepcopy(incident)
nested["operator_actions"][0]["executes_automatically"] = True
try:
    validate_artifact("incident_plan.json", nested)
except AssertionError:
    pass
else:
    raise SystemExit("validator accepted nested executes_automatically=true")

print(
    "v15_incident_rollback_artifact status=ok "
    f"root={root} "
    "incident_plan=true rollback_plan=true emergency_stop=true "
    "automatic_remediation_allowed=false production_cancel_allowed=false "
    "production_correction_allowed=false production_retry_allowed=false "
    "real_exchange_mutation_allowed=false production_orders_submitted=0 "
    "production_order_mutations_attempted=0 network_attempted=false"
)
PY
