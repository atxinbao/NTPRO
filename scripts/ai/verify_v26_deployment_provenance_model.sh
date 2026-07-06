#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

TRACE_PATH="${NTPRO_V260_DEPLOYMENT_PROVENANCE_TRACE:-tests/golden/v260_deployment_provenance_model.jsonl}"
TASK_PATH="${NTPRO_V260_DEPLOYMENT_PROVENANCE_TASK:-docs/rust-cutover/tasks/V260-004.md}"
EVIDENCE_PATH="${NTPRO_V260_DEPLOYMENT_PROVENANCE_EVIDENCE:-docs/rust-cutover/evidence/V260-004.md}"
CONTRACT_PATH="${NTPRO_V260_DEPLOYMENT_PROVENANCE_CONTRACT:-docs/rust-cutover/release/v0_26_0_deployment_provenance_model.md}"
DEPENDENCY_PATH="${NTPRO_V260_DEPLOYMENT_PROVENANCE_DEPENDENCY:-docs/rust-cutover/release/v0_26_0_product_hardening_boundary_contract.md}"
REPLAY_SCOPE_PATH="${NTPRO_V260_DEPLOYMENT_PROVENANCE_REPLAY_SCOPE:-docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"
SELFTEST="${NTPRO_V260_DEPLOYMENT_PROVENANCE_SELFTEST:-1}"

fail() {
  echo "v26 deployment provenance model failed: $*" >&2
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

for path in "$TRACE_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$CONTRACT_PATH" "$DEPENDENCY_PATH" "$REPLAY_SCOPE_PATH"; do
  require_file "$path"
done

require_contains "$TASK_PATH" "GitHub issue: \`#816\`"
require_contains "$EVIDENCE_PATH" "Task: \`V260-004\` / GitHub issue \`#816\`"
require_contains "$CONTRACT_PATH" "deployment_provenance_scope = deployment_provenance_evidence_only"
require_contains "$CONTRACT_PATH" "production_deploy_automation = false"
require_contains "$CONTRACT_PATH" "real_production_trading_ready = false"
require_contains "$CONTRACT_PATH" "adapter_send_allowed = false"
require_contains "$CONTRACT_PATH" "live_exchange_request_allowed = false"
require_contains "$DEPENDENCY_PATH" "release_scope = product_hardening_foundation_only"

python3 - "$TRACE_PATH" "$REPLAY_SCOPE_PATH" "$SELFTEST" <<'PY'
from __future__ import annotations

import copy
import json
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

trace_path = Path(sys.argv[1])
replay_scope_path = Path(sys.argv[2])
selftest = sys.argv[3] != "0"

EXPECTED_CASES = [
    "read_model.deployment_provenance_model.valid_topology_matrix.001",
    "read_model.deployment_provenance_model.missing_digest_fail_closed.001",
    "read_model.deployment_provenance_model.unredacted_config_fail_closed.001",
    "read_model.deployment_provenance_model.unknown_environment_truth_fail_closed.001",
    "read_model.deployment_provenance_model.tag_mismatch_fail_closed.001",
    "read_model.deployment_provenance_model.cross_node_scope_mismatch_fail_closed.001",
]
CONTRACT_VERSION = "ntpro.v260.deployment_provenance_model.v1"
SCHEMA_VERSION = "ntpro.v260.deployment_topology.schema.v1"
DEPLOYMENT_SCOPE = "deployment_provenance_evidence_only"
EXPECTED_TAG = "ntpro-rust-only-v0.26.0-expected"
HARNESS = "scripts/ai/verify_release.sh v26-deployment-provenance-model"
ALLOWED_CLASSIFICATIONS = {"local", "dev", "staging", "prod_like"}
SECRET_FIELD_NAMES = {
    "secret",
    "api_key",
    "api_secret",
    "credential",
    "raw_credential",
    "signature",
    "signed_payload",
    "signed_url",
    "private_key",
}


def fail(message: str) -> None:
    raise SystemExit(f"v26 deployment provenance model failed: {message}")


def non_empty(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


def push_reason(reasons: list[str], reason: str) -> None:
    if reason not in reasons:
        reasons.append(reason)


def load_rows(path: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    with path.open(encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, 1):
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            try:
                row = json.loads(line)
            except json.JSONDecodeError as exc:
                fail(f"{path}:{line_number}: invalid JSON: {exc}")
            if not isinstance(row, dict):
                fail(f"{path}:{line_number}: row must be an object")
            rows.append(row)
    return rows


def single_event(row: dict[str, Any], section: str, case_id: str) -> dict[str, Any]:
    try:
        events = row[section]["events"]
    except KeyError as exc:
        fail(f"{case_id}: missing {section}.events: {exc}")
    if not isinstance(events, list) or len(events) != 1 or not isinstance(events[0], dict):
        fail(f"{case_id}: {section}.events must contain exactly one object")
    return events[0]


def parse_ts(value: str) -> datetime:
    parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)


def secret_field_reasons(value: Any, env_id: str, reasons: list[str]) -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            key_string = str(key)
            if key_string in SECRET_FIELD_NAMES and child not in (False, None, "", [], {}):
                push_reason(reasons, f"unredacted_config_field:{env_id}:{key_string}")
            secret_field_reasons(child, env_id, reasons)
    elif isinstance(value, list):
        for child in value:
            secret_field_reasons(child, env_id, reasons)


def classify(artifact: dict[str, Any], case_id: str) -> dict[str, Any]:
    if artifact.get("contract_version") != CONTRACT_VERSION:
        fail(f"{case_id}: contract_version must be {CONTRACT_VERSION}")
    if artifact.get("schema_version") != SCHEMA_VERSION:
        fail(f"{case_id}: schema_version must be {SCHEMA_VERSION}")
    if artifact.get("dependency_contract") != "V260-001":
        fail(f"{case_id}: dependency_contract must be V260-001")

    reasons: list[str] = list(artifact.get("blocking_reasons") or [])
    missing_required = False
    unredacted_config = False
    unknown_environment_truth = False
    tag_mismatch = False
    cross_node_scope_mismatch = False
    forbidden_runtime_boundary = False

    if artifact.get("deployment_provenance_scope") != DEPLOYMENT_SCOPE:
        missing_required = True
        push_reason(reasons, "deployment_provenance_scope_mismatch")
    for key in (
        "production_deploy_automation",
        "external_deployment_system_added",
        "real_production_trading_ready",
        "adapter_send_allowed",
        "live_exchange_request_allowed",
        "dashboard_trading_controls_enabled",
    ):
        if artifact.get(key) is not False:
            if key in ("adapter_send_allowed", "live_exchange_request_allowed", "dashboard_trading_controls_enabled"):
                forbidden_runtime_boundary = True
            else:
                missing_required = True
            push_reason(reasons, f"{key}_opened")
    if not isinstance(artifact.get("artifact_provenance"), dict):
        missing_required = True
        push_reason(reasons, "missing_artifact_provenance")
    parse_ts(str(artifact.get("evaluated_at")))

    expected_tag = str(artifact.get("expected_release_tag") or EXPECTED_TAG)
    environments = artifact.get("environments")
    if not isinstance(environments, list) or not environments:
        fail(f"{case_id}: environments must be a non-empty list")

    classifications_checked: list[str] = []
    for env in environments:
        if not isinstance(env, dict):
            fail(f"{case_id}: environment entry must be an object")
        env_id = str(env.get("environment_id") or "unknown")
        classification = str(env.get("environment_classification") or "unknown")
        classifications_checked.append(classification)

        for field, reason in (
            ("environment_id", "missing_environment_id"),
            ("environment_truth_source", "missing_environment_truth_source"),
            ("release_tag", "missing_release_tag"),
            ("runtime_boundary", "missing_runtime_boundary"),
            ("artifact_digest", "missing_artifact_digest"),
        ):
            if not non_empty(env.get(field)):
                missing_required = True
                push_reason(reasons, f"{reason}:{env_id}")

        if classification not in ALLOWED_CLASSIFICATIONS:
            unknown_environment_truth = True
            push_reason(reasons, f"unknown_environment_classification:{env_id}:{classification}")
        if env.get("environment_truth_source") in ("unknown", "", None):
            unknown_environment_truth = True
            push_reason(reasons, f"unknown_environment_truth_source:{env_id}")
        release_tag = env.get("release_tag")
        if non_empty(release_tag) and release_tag != expected_tag:
            tag_mismatch = True
            push_reason(reasons, f"release_tag_mismatch:{env_id}:{release_tag}!={expected_tag}")

        config = env.get("config_source")
        if not isinstance(config, dict):
            missing_required = True
            push_reason(reasons, f"missing_config_source:{env_id}")
            config = {}
        for field, reason in (
            ("source_type", "missing_config_source_type"),
            ("source_ref", "missing_config_source_ref"),
            ("provenance_ref", "missing_config_provenance_ref"),
        ):
            if not non_empty(config.get(field)):
                missing_required = True
                push_reason(reasons, f"{reason}:{env_id}")
        if config.get("redaction") != "redacted":
            unredacted_config = True
            push_reason(reasons, f"config_not_redacted:{env_id}")
        secret_field_reasons(config, env_id, reasons)
        if any(reason.startswith(f"unredacted_config_field:{env_id}:") for reason in reasons):
            unredacted_config = True

        expected_scope = f"environment:{env_id}"
        nodes = env.get("nodes")
        if not isinstance(nodes, list) or not nodes:
            missing_required = True
            push_reason(reasons, f"missing_nodes:{env_id}")
            nodes = []
        for node in nodes:
            if not isinstance(node, dict):
                fail(f"{case_id}: node entry must be an object")
            node_id = str(node.get("node_id") or "unknown")
            for field, reason in (
                ("node_id", "missing_node_id"),
                ("node_role", "missing_node_role"),
                ("scope", "missing_node_scope"),
                ("artifact_digest", "missing_node_artifact_digest"),
                ("config_source_ref", "missing_node_config_source_ref"),
                ("runtime_boundary", "missing_node_runtime_boundary"),
            ):
                if not non_empty(node.get(field)):
                    missing_required = True
                    push_reason(reasons, f"{reason}:{node_id}")
            if non_empty(node.get("scope")) and node.get("scope") != expected_scope:
                cross_node_scope_mismatch = True
                push_reason(reasons, f"cross_node_scope_mismatch:{node_id}:{node.get('scope')}!={expected_scope}")
            if node.get("artifact_digest") != env.get("artifact_digest") and non_empty(node.get("artifact_digest")):
                missing_required = True
                push_reason(reasons, f"node_artifact_digest_mismatch:{node_id}")

    if missing_required:
        status = "fail_closed_missing_required_evidence"
    elif unredacted_config:
        status = "fail_closed_unredacted_config"
    elif unknown_environment_truth:
        status = "fail_closed_unknown_environment_truth"
    elif tag_mismatch:
        status = "fail_closed_tag_mismatch"
    elif cross_node_scope_mismatch:
        status = "fail_closed_cross_node_scope_mismatch"
    elif forbidden_runtime_boundary:
        status = "fail_closed_forbidden_runtime_boundary"
    else:
        status = "deployment_provenance_ready"

    return {
        "case_id": case_id,
        "deployment_provenance_scope": artifact.get("deployment_provenance_scope"),
        "effective_deployment_status": status,
        "environment_count": len(environments),
        "environment_classifications_checked": classifications_checked,
        "deployment_evidence_only": artifact.get("deployment_provenance_scope") == DEPLOYMENT_SCOPE,
        "production_deploy_automation_allowed": False,
        "real_production_trading_ready": False,
        "adapter_send_allowed": False,
        "live_exchange_request_allowed": False,
        "dashboard_trading_controls_allowed": False,
        "fail_closed": status != "deployment_provenance_ready",
        "blocking_reasons": reasons,
    }


rows = load_rows(trace_path)
if [row.get("case_id") for row in rows] != EXPECTED_CASES:
    fail(
        "case order mismatch: expected "
        + ", ".join(EXPECTED_CASES)
        + " got "
        + ", ".join(str(row.get("case_id")) for row in rows)
    )

healthy_artifact: dict[str, Any] | None = None
for row in rows:
    case_id = str(row.get("case_id"))
    if row.get("schema_version") != "golden-trace-v1":
        fail(f"{case_id}: schema_version must be golden-trace-v1")
    if row.get("category") != "read_model":
        fail(f"{case_id}: category must be read_model")

    input_event = single_event(row, "input", case_id)
    expected_event = single_event(row, "expected", case_id)
    expected_event_type = input_event.get("event_type", "").replace(".input", ".validated")
    if expected_event.get("event_type") != expected_event_type:
        fail(f"{case_id}: expected event_type must be {expected_event_type}")
    for key in ("ts_event", "ts_init", "instrument_id", "venue", "correlation_id"):
        if expected_event.get(key) != input_event.get(key):
            fail(f"{case_id}: expected.{key} must match input.{key}")

    payload = input_event.get("payload")
    if not isinstance(payload, dict) or not isinstance(payload.get("artifact"), dict):
        fail(f"{case_id}: input payload artifact is required")
    artifact = payload["artifact"]
    computed = classify(artifact, case_id)
    expected_payload = expected_event.get("payload")
    if computed != expected_payload:
        fail(
            f"{case_id}: computed payload mismatch\n"
            f"expected={json.dumps(expected_payload, sort_keys=True)}\n"
            f"actual={json.dumps(computed, sort_keys=True)}"
        )
    if case_id.endswith("valid_topology_matrix.001"):
        healthy_artifact = copy.deepcopy(artifact)

if selftest:
    if healthy_artifact is None:
        fail("negative selftest requires valid topology matrix")
    healthy_artifact["adapter_send_allowed"] = True
    closed = classify(healthy_artifact, "negative.selftest.adapter_send_opened")
    if closed["effective_deployment_status"] != "fail_closed_forbidden_runtime_boundary":
        fail("negative selftest opened adapter_send_allowed but did not fail closed")
    if "adapter_send_allowed_opened" not in closed["blocking_reasons"]:
        fail("negative selftest did not surface adapter_send_allowed boundary reason")

scope = json.loads(replay_scope_path.read_text(encoding="utf-8"))
cases = {case.get("case_id"): case for case in scope.get("cases", [])}
for case_id in EXPECTED_CASES:
    entry = cases.get(case_id)
    if not isinstance(entry, dict):
        fail(f"missing release replay scope entry: {case_id}")
    expected_pairs = {
        "trace": trace_path.as_posix(),
        "category": "read_model",
        "status": "validator_executable_replay",
        "evidence_id": "V260-004",
        "harness": HARNESS,
        "validator_entrypoint": "scripts/ai/verify_v26_deployment_provenance_model.sh::classify",
        "replay_type": "validator_executable_deployment_provenance_model",
        "classification_owner": "V260-004",
        "source_scope_owner": "V260-004",
        "deployment_provenance_scope": DEPLOYMENT_SCOPE,
    }
    for key, expected in expected_pairs.items():
        if entry.get(key) != expected:
            fail(f"{case_id}: release scope {key} mismatch: {entry.get(key)!r}")
    for key in (
        "runtime_adapter_integration",
        "complete_executable_order_control_runtime",
        "production_deploy_automation",
        "external_deployment_system_added",
        "real_production_trading_ready",
        "new_submit_capability",
        "production_order_mutation_allowed",
        "adapter_send_allowed",
        "live_exchange_request_allowed",
        "retry_scheduler_enabled",
        "automatic_remediation_allowed",
        "dashboard_trading_controls_enabled",
        "product_grade_live_trading_terminal",
    ):
        if entry.get(key) is not False:
            fail(f"{case_id}: release scope {key} must be false")

print(
    "v26_deployment_provenance_model "
    f"status=ok trace={trace_path.as_posix()} "
    f"cases={len(rows)} environments=4 negative_selftest={1 if selftest else 0}"
)
PY
