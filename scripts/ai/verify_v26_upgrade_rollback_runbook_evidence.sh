#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

TRACE_PATH="${NTPRO_V260_RUNBOOK_TRACE:-tests/golden/v260_upgrade_rollback_runbook_evidence.jsonl}"
TASK_PATH="${NTPRO_V260_RUNBOOK_TASK:-docs/rust-cutover/tasks/V260-005.md}"
EVIDENCE_PATH="${NTPRO_V260_RUNBOOK_EVIDENCE:-docs/rust-cutover/evidence/V260-005.md}"
CONTRACT_PATH="${NTPRO_V260_RUNBOOK_CONTRACT:-docs/rust-cutover/release/v0_26_0_upgrade_rollback_runbook_evidence.md}"
AUDIT_DEPENDENCY_PATH="${NTPRO_V260_RUNBOOK_AUDIT_DEPENDENCY:-docs/rust-cutover/release/v0_26_0_operation_audit_trail.md}"
DEPLOYMENT_DEPENDENCY_PATH="${NTPRO_V260_RUNBOOK_DEPLOYMENT_DEPENDENCY:-docs/rust-cutover/release/v0_26_0_deployment_provenance_model.md}"
REPLAY_SCOPE_PATH="${NTPRO_V260_RUNBOOK_REPLAY_SCOPE:-docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"
SELFTEST="${NTPRO_V260_RUNBOOK_SELFTEST:-1}"

fail() {
  echo "v26 upgrade rollback runbook evidence failed: $*" >&2
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

for path in "$TRACE_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$CONTRACT_PATH" "$AUDIT_DEPENDENCY_PATH" "$DEPLOYMENT_DEPENDENCY_PATH" "$REPLAY_SCOPE_PATH"; do
  require_file "$path"
done

require_contains "$TASK_PATH" "GitHub issue: \`#817\`"
require_contains "$EVIDENCE_PATH" "Task: \`V260-005\` / GitHub issue \`#817\`"
require_contains "$CONTRACT_PATH" "runbook_artifact_scope = upgrade_rollback_runbook_preview_only"
require_contains "$CONTRACT_PATH" "release_operation_execution_allowed = false"
require_contains "$CONTRACT_PATH" "automatic_deploy_allowed = false"
require_contains "$CONTRACT_PATH" "automatic_rollback_allowed = false"
require_contains "$CONTRACT_PATH" "release_publication_workflow_changed = false"
require_contains "$AUDIT_DEPENDENCY_PATH" "audit_artifact_scope = operation_audit_evidence_only"
require_contains "$DEPLOYMENT_DEPENDENCY_PATH" "deployment_provenance_scope = deployment_provenance_evidence_only"

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
    "read_model.upgrade_rollback_runbook.valid_preview.001",
    "read_model.upgrade_rollback_runbook.missing_approval_blocked_preview.001",
    "read_model.upgrade_rollback_runbook.tag_mismatch_fail_closed.001",
    "read_model.upgrade_rollback_runbook.failed_preflight_fail_closed.001",
    "read_model.upgrade_rollback_runbook.rollback_recommendation_preview.001",
    "read_model.upgrade_rollback_runbook.stale_environment_evidence_fail_closed.001",
]
CONTRACT_VERSION = "ntpro.v260.upgrade_rollback_runbook.v1"
SCHEMA_VERSION = "ntpro.v260.release_operation_runbook.schema.v1"
RUNBOOK_SCOPE = "upgrade_rollback_runbook_preview_only"
EXPECTED_TAG = "ntpro-rust-only-v0.26.0-expected"
HARNESS = "scripts/ai/verify_release.sh v26-upgrade-rollback-runbook-evidence"


def fail(message: str) -> None:
    raise SystemExit(f"v26 upgrade rollback runbook evidence failed: {message}")


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


def classify(artifact: dict[str, Any], case_id: str) -> dict[str, Any]:
    if artifact.get("contract_version") != CONTRACT_VERSION:
        fail(f"{case_id}: contract_version must be {CONTRACT_VERSION}")
    if artifact.get("schema_version") != SCHEMA_VERSION:
        fail(f"{case_id}: schema_version must be {SCHEMA_VERSION}")
    if artifact.get("dependency_contracts") != ["V260-003", "V260-004"]:
        fail(f"{case_id}: dependency_contracts must be ['V260-003', 'V260-004']")

    reasons: list[str] = list(artifact.get("blocking_reasons") or [])
    missing_required = False
    missing_approval = False
    tag_or_source_mismatch = False
    failed_preflight = False
    stale_environment = False
    forbidden_execution = False
    rollback_recommendation = False

    if artifact.get("runbook_artifact_scope") != RUNBOOK_SCOPE:
        missing_required = True
        push_reason(reasons, "runbook_artifact_scope_mismatch")
    for key in (
        "release_operation_execution_allowed",
        "automatic_deploy_allowed",
        "automatic_rollback_allowed",
        "automatic_remediation_allowed",
        "trading_operation_allowed",
        "dashboard_execution_controls_enabled",
        "release_publication_workflow_changed",
    ):
        if artifact.get(key) is not False:
            forbidden_execution = True
            push_reason(reasons, f"{key}_opened")
    parse_ts(str(artifact.get("evaluated_at")))

    lineage = artifact.get("audit_lineage")
    if not isinstance(lineage, dict):
        missing_required = True
        push_reason(reasons, "missing_audit_lineage")
        lineage = {}
    for field, reason in (
        ("audit_event_ref", "missing_audit_event_ref"),
        ("deployment_provenance_ref", "missing_deployment_provenance_ref"),
    ):
        if not non_empty(lineage.get(field)):
            missing_required = True
            push_reason(reasons, reason)

    expected_tag = str(artifact.get("expected_release_tag") or EXPECTED_TAG)
    runbooks = artifact.get("runbooks")
    if not isinstance(runbooks, list) or not runbooks:
        fail(f"{case_id}: runbooks must be a non-empty list")

    runbook_types_checked: list[str] = []
    for runbook in runbooks:
        if not isinstance(runbook, dict):
            fail(f"{case_id}: runbook entry must be an object")
        runbook_id = str(runbook.get("runbook_id") or "unknown")
        runbook_type = str(runbook.get("runbook_type") or "unknown")
        runbook_types_checked.append(runbook_type)

        for field, reason in (
            ("runbook_id", "missing_runbook_id"),
            ("runbook_type", "missing_runbook_type"),
            ("plan_ref", "missing_plan_ref"),
            ("release_tag", "missing_release_tag"),
            ("source_ref", "missing_source_ref"),
            ("environment_id", "missing_environment_id"),
            ("environment_provenance_ref", "missing_environment_provenance_ref"),
            ("release_gate_evidence_ref", "missing_release_gate_evidence_ref"),
        ):
            if not non_empty(runbook.get(field)):
                missing_required = True
                push_reason(reasons, f"{reason}:{runbook_id}")
        if runbook_type not in {"upgrade", "rollback"}:
            missing_required = True
            push_reason(reasons, f"unknown_runbook_type:{runbook_id}:{runbook_type}")
        if runbook.get("release_tag") != expected_tag and non_empty(runbook.get("release_tag")):
            tag_or_source_mismatch = True
            push_reason(reasons, f"release_tag_mismatch:{runbook_id}:{runbook.get('release_tag')}!={expected_tag}")
        if str(runbook.get("source_ref") or "").endswith(":stale"):
            tag_or_source_mismatch = True
            push_reason(reasons, f"source_ref_stale:{runbook_id}")
        if runbook.get("environment_provenance_fresh") is not True:
            stale_environment = True
            push_reason(reasons, f"stale_environment_evidence:{runbook_id}")

        approval = runbook.get("owner_approval")
        if not isinstance(approval, dict):
            missing_required = True
            push_reason(reasons, f"missing_owner_approval:{runbook_id}")
            approval = {}
        if approval.get("required") is not True or approval.get("status") != "approved":
            missing_approval = True
            push_reason(reasons, f"approval_not_approved:{runbook_id}")
        if approval.get("required") is True and not non_empty(approval.get("approval_ref")):
            missing_approval = True
            push_reason(reasons, f"missing_approval_ref:{runbook_id}")

        checks = runbook.get("preflight_checks")
        if not isinstance(checks, list) or not checks:
            missing_required = True
            push_reason(reasons, f"missing_preflight_checks:{runbook_id}")
            checks = []
        for check in checks:
            if not isinstance(check, dict):
                fail(f"{case_id}: preflight check must be an object")
            name = str(check.get("name") or "unknown")
            if check.get("status") != "pass":
                failed_preflight = True
                push_reason(reasons, f"preflight_failed:{runbook_id}:{name}")
            if not non_empty(check.get("evidence_ref")):
                missing_required = True
                push_reason(reasons, f"missing_preflight_evidence:{runbook_id}:{name}")

        post_check = runbook.get("post_check_evidence")
        if not isinstance(post_check, dict) or not non_empty(post_check.get("evidence_ref")):
            missing_required = True
            push_reason(reasons, f"missing_post_check_evidence:{runbook_id}")

        if runbook.get("dashboard_read_only") is not True:
            forbidden_execution = True
            push_reason(reasons, f"dashboard_not_read_only:{runbook_id}")
        if runbook.get("preview_only") is not True:
            forbidden_execution = True
            push_reason(reasons, f"preview_only_disabled:{runbook_id}")
        if runbook.get("execution_triggered") is not False:
            forbidden_execution = True
            push_reason(reasons, f"execution_triggered:{runbook_id}")
        if runbook_type == "rollback" and runbook.get("decision") == "recommendation_only":
            rollback_recommendation = True

    if forbidden_execution:
        status = "fail_closed_forbidden_execution"
    elif tag_or_source_mismatch:
        status = "fail_closed_tag_or_source_mismatch"
    elif failed_preflight:
        status = "fail_closed_failed_preflight"
    elif stale_environment:
        status = "fail_closed_stale_environment_evidence"
    elif missing_approval:
        status = "blocked_preview_missing_approval"
    elif missing_required:
        status = "fail_closed_missing_required_evidence"
    elif rollback_recommendation:
        status = "rollback_recommendation_preview_only"
    else:
        status = "runbook_preview_ready"

    return {
        "case_id": case_id,
        "runbook_artifact_scope": artifact.get("runbook_artifact_scope"),
        "effective_runbook_status": status,
        "runbook_count": len(runbooks),
        "runbook_types_checked": runbook_types_checked,
        "runbook_preview_only": artifact.get("runbook_artifact_scope") == RUNBOOK_SCOPE,
        "release_operation_execution_allowed": False,
        "automatic_deploy_allowed": False,
        "automatic_rollback_allowed": False,
        "automatic_remediation_allowed": False,
        "trading_operation_allowed": False,
        "dashboard_execution_controls_allowed": False,
        "release_publication_workflow_changed": False,
        "fail_closed": status not in ("runbook_preview_ready", "rollback_recommendation_preview_only"),
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
    if case_id.endswith("valid_preview.001"):
        healthy_artifact = copy.deepcopy(artifact)

if selftest:
    if healthy_artifact is None:
        fail("negative selftest requires valid runbook preview")
    healthy_artifact["automatic_rollback_allowed"] = True
    closed = classify(healthy_artifact, "negative.selftest.automatic_rollback_opened")
    if closed["effective_runbook_status"] != "fail_closed_forbidden_execution":
        fail("negative selftest opened automatic rollback but did not fail closed")
    if "automatic_rollback_allowed_opened" not in closed["blocking_reasons"]:
        fail("negative selftest did not surface automatic rollback boundary reason")

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
        "evidence_id": "V260-005",
        "harness": HARNESS,
        "validator_entrypoint": "scripts/ai/verify_v26_upgrade_rollback_runbook_evidence.sh::classify",
        "replay_type": "validator_executable_upgrade_rollback_runbook_evidence",
        "classification_owner": "V260-005",
        "source_scope_owner": "V260-005",
        "runbook_artifact_scope": RUNBOOK_SCOPE,
    }
    for key, expected in expected_pairs.items():
        if entry.get(key) != expected:
            fail(f"{case_id}: release scope {key} mismatch: {entry.get(key)!r}")
    for key in (
        "runtime_adapter_integration",
        "release_operation_execution_allowed",
        "automatic_deploy_allowed",
        "automatic_rollback_allowed",
        "automatic_remediation_allowed",
        "trading_operation_allowed",
        "dashboard_execution_controls_enabled",
        "release_publication_workflow_changed",
        "new_submit_capability",
        "production_order_mutation_allowed",
        "adapter_send_allowed",
        "live_exchange_request_allowed",
        "product_grade_live_trading_terminal",
    ):
        if entry.get(key) is not False:
            fail(f"{case_id}: release scope {key} must be false")

print(
    "v26_upgrade_rollback_runbook_evidence "
    f"status=ok trace={trace_path.as_posix()} "
    f"cases={len(rows)} negative_selftest={1 if selftest else 0}"
)
PY
