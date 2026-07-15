#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

SCOPE_PATH="${NTPRO_V241_SCHEMA_REPLAY_SCOPE:-docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"
MANIFEST_PATH="${NTPRO_V241_SCHEMA_MANIFEST:-docs/rust-cutover/release/v0_24_0_release_manifest.json}"
REPORT_PATH="${NTPRO_V241_SCHEMA_REPORT:-docs/rust-cutover/release/v0_24_1_schema_replay_classification.md}"
TASK_PATH="${NTPRO_V241_SCHEMA_TASK:-docs/rust-cutover/tasks/V241-004.md}"
EVIDENCE_PATH="${NTPRO_V241_SCHEMA_EVIDENCE:-docs/rust-cutover/evidence/V241-004.md}"
VERIFICATION_PATH="${NTPRO_V241_SCHEMA_VERIFICATION:-verification.md}"

TRACE_FILES=(
  tests/golden/v240_order_intent_execution_policy.jsonl
  tests/golden/v240_rate_limit_throttle_gate.jsonl
  tests/golden/v240_order_slicing_preview.jsonl
  tests/golden/v240_cancel_replace_amend_preview.jsonl
  tests/golden/v240_retry_policy_ledger.jsonl
  tests/golden/v240_readback_audit_evidence.jsonl
)

fail() {
  echo "v24.1 schema replay classification failed: $*" >&2
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

require_not_contains() {
  local path="$1"
  local marker="$2"
  if grep -F -- "$marker" "$path" >/dev/null; then
    fail "forbidden runtime claim in $path: $marker"
  fi
}

for path in \
  "$SCOPE_PATH" \
  "$MANIFEST_PATH" \
  "$REPORT_PATH" \
  "$TASK_PATH" \
  "$EVIDENCE_PATH" \
  "$VERIFICATION_PATH" \
  scripts/ai/ntpro_governance.sh \
  crates/governance/src/golden_trace.rs; do
  require_file "$path"
done

for trace in "${TRACE_FILES[@]}"; do
  require_file "$trace"
  scripts/ai/ntpro_governance.sh golden-trace "$trace" --mode validate-only >/dev/null
done

for marker in \
  "validator_executable_replay = 39" \
  "schema_only_scoped v240 rows = 0" \
  "runtime adapter integration = 0" \
  "complete executable order-control runtime = false" \
  "production order submission allowed = false" \
  "execution adapter call allowed = false" \
  "Dashboard operation controls enabled = false" \
  "schema-only traces are not executable runtime"; do
  require_contains "$REPORT_PATH" "$marker"
  require_contains "$EVIDENCE_PATH" "$marker"
done

for path in "$REPORT_PATH" "$EVIDENCE_PATH" "$TASK_PATH" "$VERIFICATION_PATH"; do
  for marker in \
    "complete executable order-control runtime = true" \
    "runtime adapter integration = true" \
    "production order submission allowed = true" \
    "production order mutation allowed = true" \
    "execution adapter call allowed = true" \
    "live exchange request allowed = true" \
    "Dashboard operation controls enabled = true" \
    "product-grade live trading terminal = true"; do
    require_not_contains "$path" "$marker"
  done
done

scripts/ai/ntpro_governance.sh golden-trace-release-scope \
  --manifest "$SCOPE_PATH" \
  --trace-glob 'tests/golden/*.jsonl' >/tmp/ntpro-v241-schema-release-scope.log

SCOPE_PATH="$SCOPE_PATH" \
MANIFEST_PATH="$MANIFEST_PATH" \
REPORT_PATH="$REPORT_PATH" \
TASK_PATH="$TASK_PATH" \
EVIDENCE_PATH="$EVIDENCE_PATH" \
python3 <<'PY'
import json
import os
from pathlib import Path

TRACE_CASES = {
    "execution.v240_order_intent_policy.valid_intent.001": ("tests/golden/v240_order_intent_execution_policy.jsonl", "preview_ready"),
    "execution.v240_order_intent_policy.missing_scope.001": ("tests/golden/v240_order_intent_execution_policy.jsonl", "identity_blocked"),
    "execution.v240_order_intent_policy.policy_mismatch.001": ("tests/golden/v240_order_intent_execution_policy.jsonl", "policy_blocked"),
    "execution.v240_order_intent_policy.forbidden_operation.001": ("tests/golden/v240_order_intent_execution_policy.jsonl", "blocked"),
    "execution.v240_rate_limit_throttle.allowed_preview.001": ("tests/golden/v240_rate_limit_throttle_gate.jsonl", "allowed_preview"),
    "execution.v240_rate_limit_throttle.burst_exceeded.001": ("tests/golden/v240_rate_limit_throttle_gate.jsonl", "throttled"),
    "execution.v240_rate_limit_throttle.window_exceeded.001": ("tests/golden/v240_rate_limit_throttle_gate.jsonl", "throttled"),
    "execution.v240_rate_limit_throttle.venue_cap_exceeded.001": ("tests/golden/v240_rate_limit_throttle_gate.jsonl", "throttled"),
    "execution.v240_rate_limit_throttle.missing_limit_policy.001": ("tests/golden/v240_rate_limit_throttle_gate.jsonl", "blocked_missing_limit"),
    "execution.v240_rate_limit_throttle.scope_mismatch.001": ("tests/golden/v240_rate_limit_throttle_gate.jsonl", "blocked_scope_mismatch"),
    "execution.v240_order_slicing.valid_plan.001": ("tests/golden/v240_order_slicing_preview.jsonl", "preview_ready"),
    "execution.v240_order_slicing.invalid_size.001": ("tests/golden/v240_order_slicing_preview.jsonl", "blocked_invalid_size"),
    "execution.v240_order_slicing.precision_mismatch.001": ("tests/golden/v240_order_slicing_preview.jsonl", "blocked_precision_mismatch"),
    "execution.v240_order_slicing.scope_mismatch.001": ("tests/golden/v240_order_slicing_preview.jsonl", "blocked_scope_mismatch"),
    "execution.v240_order_slicing.policy_missing.001": ("tests/golden/v240_order_slicing_preview.jsonl", "blocked_missing_policy"),
    "execution.v240_order_slicing.forbidden_market_limit_combo.001": ("tests/golden/v240_order_slicing_preview.jsonl", "blocked_forbidden_order_combo"),
    "execution.v240_cancel_replace_amend.cancel_preview.001": ("tests/golden/v240_cancel_replace_amend_preview.jsonl", "cancel_preview_ready"),
    "execution.v240_cancel_replace_amend.replace_preview.001": ("tests/golden/v240_cancel_replace_amend_preview.jsonl", "replace_preview_ready"),
    "execution.v240_cancel_replace_amend.amend_preview.001": ("tests/golden/v240_cancel_replace_amend_preview.jsonl", "amend_preview_ready"),
    "execution.v240_cancel_replace_amend.missing_lineage.001": ("tests/golden/v240_cancel_replace_amend_preview.jsonl", "blocked_missing_lineage"),
    "execution.v240_cancel_replace_amend.scope_mismatch.001": ("tests/golden/v240_cancel_replace_amend_preview.jsonl", "blocked_scope_mismatch"),
    "execution.v240_cancel_replace_amend.expired_approval.001": ("tests/golden/v240_cancel_replace_amend_preview.jsonl", "blocked_expired_approval"),
    "execution.v240_cancel_replace_amend.forbidden_operation.001": ("tests/golden/v240_cancel_replace_amend_preview.jsonl", "blocked_forbidden_operation"),
    "execution.v240_retry_policy.transport_retry_allowed.001": ("tests/golden/v240_retry_policy_ledger.jsonl", "retry_preview_allowed"),
    "execution.v240_retry_policy.timeout_retry_allowed.001": ("tests/golden/v240_retry_policy_ledger.jsonl", "retry_preview_allowed"),
    "execution.v240_retry_policy.business_rejection_terminal.001": ("tests/golden/v240_retry_policy_ledger.jsonl", "no_retry_terminal"),
    "execution.v240_retry_policy.risk_rejection_terminal.001": ("tests/golden/v240_retry_policy_ledger.jsonl", "no_retry_terminal"),
    "execution.v240_retry_policy.duplicate_retry.001": ("tests/golden/v240_retry_policy_ledger.jsonl", "blocked_duplicate_retry"),
    "execution.v240_retry_policy.missing_prior_attempt.001": ("tests/golden/v240_retry_policy_ledger.jsonl", "blocked_missing_prior_attempt"),
    "execution.v240_retry_policy.unknown_state_blocked.001": ("tests/golden/v240_retry_policy_ledger.jsonl", "blocked_unknown_state_retry"),
    "execution.v240_retry_policy.policy_mismatch.001": ("tests/golden/v240_retry_policy_ledger.jsonl", "blocked_policy_mismatch"),
    "execution.v240_readback_audit.ready_preview.001": ("tests/golden/v240_readback_audit_evidence.jsonl", "ready_preview"),
    "execution.v240_readback_audit.missing_readback.001": ("tests/golden/v240_readback_audit_evidence.jsonl", "fail_closed"),
    "execution.v240_readback_audit.missing_audit.001": ("tests/golden/v240_readback_audit_evidence.jsonl", "fail_closed"),
    "execution.v240_readback_audit.missing_provenance.001": ("tests/golden/v240_readback_audit_evidence.jsonl", "fail_closed"),
    "execution.v240_readback_audit.stale_source.001": ("tests/golden/v240_readback_audit_evidence.jsonl", "fail_closed"),
    "execution.v240_readback_audit.redaction_breach.001": ("tests/golden/v240_readback_audit_evidence.jsonl", "fail_closed"),
    "execution.v240_readback_audit.cross_scope_mismatch.001": ("tests/golden/v240_readback_audit_evidence.jsonl", "blocked"),
    "execution.v240_readback_audit.degraded_unavailable.001": ("tests/golden/v240_readback_audit_evidence.jsonl", "degraded_unavailable"),
}

COMMON_FALSE_FLAGS = (
    "new_submit_capability",
    "production_order_mutation_allowed",
    "execution_adapter_call_allowed",
    "dashboard_operation_controls_enabled",
)
OPTIONAL_FALSE_FLAGS = (
    "live_exchange_request_allowed",
    "network_attempted",
    "retry_scheduler_enabled",
    "implicit_retry_allowed",
    "cancel_replace_amend_send_allowed",
    "flatten_allowed",
    "child_order_submission_allowed",
    "child_order_scheduler_enabled",
    "exchange_truth_claimed",
)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def load_jsonl(path: str) -> dict[str, dict]:
    rows = {}
    with Path(path).open(encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, 1):
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            row = json.loads(line)
            rows[row["case_id"]] = row
    return rows


scope = json.loads(Path(os.environ["SCOPE_PATH"]).read_text(encoding="utf-8"))
manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))
scope_entries = {entry["case_id"]: entry for entry in scope["cases"]}

missing = sorted(set(TRACE_CASES) - set(scope_entries))
extra = sorted(case_id for case_id in scope_entries if case_id.startswith("execution.v240_") and case_id not in TRACE_CASES)
require(not missing, "missing v240 release scope cases: " + ", ".join(missing))
require(not extra, "unexpected v240 release scope cases: " + ", ".join(extra))

rows_by_trace = {}
for trace, _ in set(TRACE_CASES.values()):
    rows_by_trace[trace] = load_jsonl(trace)

for case_id, (trace, decision) in TRACE_CASES.items():
    entry = scope_entries[case_id]
    require(entry.get("trace") == trace, f"{case_id}: trace path mismatch")
    require(entry.get("category") == "execution", f"{case_id}: category must remain execution")
    require(entry.get("status") == "validator_executable_replay", f"{case_id}: must be validator_executable_replay")
    require(entry.get("release_decision") == "validator_executable_scope_recorded", f"{case_id}: release decision mismatch")
    require(entry.get("evidence_id") == "V241-004", f"{case_id}: evidence id mismatch")
    require(entry.get("harness") == "scripts/ai/verify_release.sh v24.1-schema-replay-classification", f"{case_id}: harness mismatch")
    require(entry.get("validator_entrypoint") == "scripts/ai/verify_v24_1_schema_replay_classification.sh::validate_v240_order_control_decision_envelopes", f"{case_id}: validator entrypoint mismatch")
    require("rust_entrypoint" not in entry, f"{case_id}: validator replay must not claim rust_entrypoint")
    require(entry.get("runtime_adapter_integration") is False, f"{case_id}: runtime adapter integration must be false")
    require(entry.get("complete_executable_order_control_runtime") is False, f"{case_id}: complete runtime flag must be false")
    for flag in COMMON_FALSE_FLAGS:
        require(entry.get(flag) is False, f"{case_id}: manifest boundary flag must be false: {flag}")

    row = rows_by_trace[trace][case_id]
    events = row.get("expected", {}).get("events", [])
    require(len(events) == 1, f"{case_id}: expected exactly one decision event")
    payload = events[0].get("payload", {})
    actual_decision = payload.get("decision") or payload.get("status")
    require(actual_decision == decision, f"{case_id}: expected decision {decision}, got {actual_decision}")
    for flag in COMMON_FALSE_FLAGS:
        require(payload.get(flag) is False, f"{case_id}: payload boundary flag must be false: {flag}")
    for flag in OPTIONAL_FALSE_FLAGS:
        if flag in payload:
            require(payload[flag] is False, f"{case_id}: payload optional boundary flag must be false: {flag}")

schema_only_v240 = [
    case_id
    for case_id, entry in scope_entries.items()
    if case_id.startswith("execution.v240_") and entry.get("status") == "schema_only_scoped"
]
require(not schema_only_v240, "v240 schema-only cases remain: " + ", ".join(sorted(schema_only_v240)))

classification = manifest.get("post_release_schema_replay_classification") or {}
require(classification.get("task_id") == "V241-004", "classification task mismatch")
require(classification.get("issue") == 773, "classification issue mismatch")
require(classification.get("task_path") == os.environ["TASK_PATH"], "classification task path mismatch")
require(classification.get("evidence_path") == os.environ["EVIDENCE_PATH"], "classification evidence path mismatch")
require(classification.get("classification_report_path") == os.environ["REPORT_PATH"], "classification report path mismatch")
require(classification.get("golden_trace_scope_path") == os.environ["SCOPE_PATH"], "classification scope path mismatch")
require(classification.get("v240_total_cases") == len(TRACE_CASES), "classification total count mismatch")
require(classification.get("v240_validator_executable_replay_cases") == len(TRACE_CASES), "classification validator count mismatch")
require(classification.get("v240_schema_only_scoped_cases") == 0, "classification schema-only count mismatch")
require(classification.get("runtime_adapter_integration") is False, "runtime adapter integration must be false")
require(classification.get("complete_executable_order_control_runtime") is False, "complete runtime must be false")
boundary = classification.get("boundary") or {}
for key in (
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "execution_adapter_call_allowed",
    "live_exchange_request_allowed",
    "retry_scheduler_enabled",
    "dashboard_operation_controls_enabled",
    "product_grade_live_trading_terminal",
):
    require(boundary.get(key) is False, f"classification boundary must be false: {key}")

status_taxonomy = classification.get("status_taxonomy") or {}
require("schema_only_scoped" in status_taxonomy, "schema-only taxonomy missing")
require("validator_executable_replay" in status_taxonomy, "validator taxonomy missing")
require("runtime_adapter_integration" in status_taxonomy, "runtime adapter taxonomy missing")
PY

if [[ "${NTPRO_V241_SCHEMA_SELFTEST:-1}" == "1" ]]; then
  tmp_scope="$(mktemp "${TMPDIR:-/tmp}/ntpro-v241-schema-scope.XXXXXX.json")"
  python3 - "$SCOPE_PATH" "$tmp_scope" <<'PY'
import json
import sys
from pathlib import Path

source, target = map(Path, sys.argv[1:])
data = json.loads(source.read_text(encoding="utf-8"))
for entry in data["cases"]:
    if entry.get("case_id") == "execution.v240_order_intent_policy.valid_intent.001":
        entry["status"] = "schema_only_scoped"
        entry["release_decision"] = "schema_only_scope_recorded"
        entry.pop("validator_entrypoint", None)
        entry.pop("runtime_adapter_integration", None)
        entry["scope_owner"] = "negative-selftest"
        entry["reason"] = "negative self-test"
        entry["follow_up"] = "negative self-test"
        break
target.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
PY
  if NTPRO_V241_SCHEMA_SELFTEST=0 NTPRO_V241_SCHEMA_REPLAY_SCOPE="$tmp_scope" "$0" >/tmp/ntpro-v241-schema-negative.log 2>&1; then
    rm -f "$tmp_scope"
    fail "schema-only negative self-test unexpectedly passed"
  fi
  rm -f "$tmp_scope"
fi

echo "v24_1_schema_replay_classification status=ok v240_total=39 validator_executable_replay=39 schema_only_scoped=0 runtime_adapter_integration=false selftest=${NTPRO_V241_SCHEMA_SELFTEST:-1}"
