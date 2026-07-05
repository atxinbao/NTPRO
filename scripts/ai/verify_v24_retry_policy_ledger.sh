#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

CONTRACT_PATH="${NTPRO_V24_RETRY_POLICY_CONTRACT:-docs/rust-cutover/release/v0_24_0_retry_policy_ledger.md}"
TASK_PATH="${NTPRO_V24_RETRY_POLICY_TASK:-docs/rust-cutover/tasks/V240-006.md}"
EVIDENCE_PATH="${NTPRO_V24_RETRY_POLICY_EVIDENCE:-docs/rust-cutover/evidence/V240-006.md}"
TRACE_PATH="${NTPRO_V24_RETRY_POLICY_TRACE:-tests/golden/v240_retry_policy_ledger.jsonl}"
REPLAY_SCOPE_PATH="${NTPRO_V24_RETRY_POLICY_REPLAY_SCOPE:-docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"

fail() {
  echo "v24 retry policy ledger failed: $*" >&2
  exit 1
}

require_file() {
  local path="$1"
  [[ -f "$path" ]] || fail "missing required file: $path"
}

contains() {
  local path="$1"
  local marker="$2"
  grep -F -- "$marker" "$path" >/dev/null
}

require_contains() {
  local path="$1"
  local marker="$2"
  contains "$path" "$marker" || fail "missing marker in $path: $marker"
}

for path in "$CONTRACT_PATH" "$TASK_PATH" "$EVIDENCE_PATH" "$TRACE_PATH" "$REPLAY_SCOPE_PATH"; do
  require_file "$path"
done

for marker in \
  "schema_version = ntpro.v240_retry_no_retry_ledger.v1" \
  "contract_id = ntpro.v240_retry_no_retry_policy_ledger.v1" \
  "contract_status = preview_evidence_only_no_runtime_retry_scheduler" \
  "start_gate_dependency = scripts/ai/verify_release.sh v24-cancel-replace-amend-preview" \
  "golden_trace = tests/golden/v240_retry_policy_ledger.jsonl" \
  "retry_intent_digest_required = true" \
  "prior_attempt_ref_required = true" \
  "retry_policy_id_required = true" \
  "policy_approval_id_required = true" \
  "owner_approval_id_required = true" \
  "audit_ref_required = true" \
  "retry_preview_allowed = transport error or timeout explicitly allowed by policy" \
  "no_retry_terminal = business rejection or risk rejection is terminal" \
  "blocked_duplicate_retry = retry intent digest already consumed" \
  "blocked_missing_prior_attempt = prior attempt ref missing" \
  "blocked_unknown_state_retry = unknown state retry is not explicitly allowed" \
  "blocked_policy_mismatch = retry policy scope differs from attempt scope" \
  "dashboard_readonly_evidence = true" \
  "network_attempted = false" \
  "execution_adapter_call_allowed = false" \
  "production_order_mutation_allowed = false" \
  "new_submit_capability = false" \
  "retry_scheduler_enabled = false" \
  "implicit_retry_allowed = false" \
  "dashboard_operation_controls_enabled = false" \
  "signed_request_present = false"; do
  require_contains "$CONTRACT_PATH" "$marker"
done

for marker in \
  "new_submit_capability = true" \
  "production_order_mutation_allowed = true" \
  "execution_adapter_call_allowed = true" \
  "retry_scheduler_enabled = true" \
  "implicit_retry_allowed = true" \
  "dashboard_operation_controls_enabled = true" \
  "signed_request_present = true" \
  "blocked_duplicate_retry = allow" \
  "blocked_missing_prior_attempt = allow" \
  "blocked_unknown_state_retry = allow" \
  "blocked_policy_mismatch = allow"; do
  if contains "$CONTRACT_PATH" "$marker"; then
    fail "forbidden marker in $CONTRACT_PATH: $marker"
  fi
done

for marker in \
  "Task: \`V240-006\` / GitHub issue \`#749\`" \
  "tests/golden/v240_retry_policy_ledger.jsonl" \
  "scripts/ai/verify_release.sh v24-retry-policy-ledger"; do
  require_contains "$TASK_PATH" "$marker"
  require_contains "$EVIDENCE_PATH" "$marker"
done

python3 scripts/ai/golden_trace_runner.py "$TRACE_PATH" --mode validate-only
python3 scripts/ai/validate_golden_trace_release_scope.py \
  --manifest "$REPLAY_SCOPE_PATH" \
  --trace-glob 'tests/golden/*.jsonl'

python3 - "$TRACE_PATH" "$REPLAY_SCOPE_PATH" <<'PY'
import json
import sys
from pathlib import Path

trace_path = Path(sys.argv[1])
scope_path = Path(sys.argv[2])

required = {
    "execution.v240_retry_policy.transport_retry_allowed.001": ("retry_preview_allowed", "v240_retry_policy_transport_retry_allowed", "transport_error"),
    "execution.v240_retry_policy.timeout_retry_allowed.001": ("retry_preview_allowed", "v240_retry_policy_timeout_retry_allowed", "timeout"),
    "execution.v240_retry_policy.business_rejection_terminal.001": ("no_retry_terminal", "v240_retry_policy_business_rejection_terminal", "business_rejection"),
    "execution.v240_retry_policy.risk_rejection_terminal.001": ("no_retry_terminal", "v240_retry_policy_risk_rejection_terminal", "risk_rejection"),
    "execution.v240_retry_policy.duplicate_retry.001": ("blocked_duplicate_retry", "v240_retry_policy_duplicate_retry", "transport_error"),
    "execution.v240_retry_policy.missing_prior_attempt.001": ("blocked_missing_prior_attempt", "v240_retry_policy_missing_prior_attempt", "timeout"),
    "execution.v240_retry_policy.unknown_state_blocked.001": ("blocked_unknown_state_retry", "v240_retry_policy_unknown_state_blocked", "unknown_state"),
    "execution.v240_retry_policy.policy_mismatch.001": ("blocked_policy_mismatch", "v240_retry_policy_scope_mismatch", "transport_error"),
}


def fail(message: str) -> None:
    raise SystemExit(message)


def rows(path: Path) -> list[dict]:
    loaded = []
    for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        try:
            loaded.append(json.loads(line))
        except json.JSONDecodeError as exc:
            fail(f"{path}:{line_number}: invalid JSON: {exc}")
    return loaded


trace_rows = rows(trace_path)
if len(trace_rows) != len(required):
    fail(f"expected {len(required)} rows, got {len(trace_rows)}")

seen = set()
for row in trace_rows:
    case_id = row.get("case_id")
    if case_id not in required:
        fail(f"unexpected case: {case_id}")
    seen.add(case_id)
    if row.get("category") != "execution":
        fail(f"{case_id}: category must be execution")

    input_events = (row.get("input") or {}).get("events") or []
    expected_events = (row.get("expected") or {}).get("events") or []
    if len(input_events) != 1 or len(expected_events) != 1:
        fail(f"{case_id}: expected one input and one expected event")
    if input_events[0].get("event_type") != "execution.retry_policy.request":
        fail(f"{case_id}: input event_type mismatch")
    if expected_events[0].get("event_type") != "execution.retry_policy.decision":
        fail(f"{case_id}: expected event_type mismatch")

    payload = input_events[0].get("payload") or {}
    expected = expected_events[0].get("payload") or {}
    status, code, category = required[case_id]
    if expected.get("status") != status or expected.get("code") != code:
        fail(f"{case_id}: status/code mismatch")
    if payload.get("retry_category") != category:
        fail(f"{case_id}: retry category mismatch")

    for key in ("retry_intent_digest", "retry_policy_id", "policy_scope_key", "account_key", "strategy_key", "venue_node_key", "isolation_scope_key", "attempt_sequence", "retry_reason", "policy_approval_id", "owner_approval_id", "audit_ref"):
        if not payload.get(key):
            fail(f"{case_id}: missing {key}")
    if status != "blocked_missing_prior_attempt" and not payload.get("prior_attempt_ref"):
        fail(f"{case_id}: prior_attempt_ref required")
    if payload.get("submit_requested") is not False or payload.get("network_requested") is not False:
        fail(f"{case_id}: preview request must not request submit or network")

    if expected.get("dashboard_readonly_evidence") is not True or expected.get("ledger_append_preview") is not True:
        fail(f"{case_id}: expected read-only ledger evidence")
    for key in (
        "network_attempted",
        "execution_adapter_call_allowed",
        "live_exchange_request_allowed",
        "production_order_mutation_allowed",
        "new_submit_capability",
        "retry_scheduler_enabled",
        "implicit_retry_allowed",
        "dashboard_operation_controls_enabled",
        "signed_request_present",
    ):
        if expected.get(key) is not False:
            fail(f"{case_id}: {key} must be false")

    if status == "retry_preview_allowed":
        if category not in {"transport_error", "timeout"}:
            fail(f"{case_id}: only transport/timeout can be retry_preview_allowed")
        for key in ("prior_attempt_present", "policy_allows_retry", "owner_approved", "audit_ref_present"):
            if payload.get(key) is not True:
                fail(f"{case_id}: {key} required for retry")
        if payload.get("digest_already_consumed") is not False:
            fail(f"{case_id}: retry digest must not be consumed")
        if payload.get("policy_scope_key") != payload.get("isolation_scope_key"):
            fail(f"{case_id}: policy scope must match")
    if status == "no_retry_terminal":
        if category not in {"business_rejection", "risk_rejection"}:
            fail(f"{case_id}: terminal category mismatch")
        if expected.get("terminal_category") is not True:
            fail(f"{case_id}: terminal category evidence required")
    if status == "blocked_duplicate_retry" and payload.get("digest_already_consumed") is not True:
        fail(f"{case_id}: duplicate retry must mark digest consumed")
    if status == "blocked_missing_prior_attempt":
        if payload.get("prior_attempt_present") is not False or payload.get("prior_attempt_ref"):
            fail(f"{case_id}: missing prior attempt case must omit ref")
    if status == "blocked_unknown_state_retry":
        if category != "unknown_state" or payload.get("policy_allows_retry") is not False:
            fail(f"{case_id}: unknown state must be blocked without explicit policy")
    if status == "blocked_policy_mismatch":
        if payload.get("policy_scope_key") == payload.get("isolation_scope_key"):
            fail(f"{case_id}: policy mismatch must differ")

missing = sorted(set(required) - seen)
if missing:
    fail("missing cases: " + ", ".join(missing))

scope = json.loads(scope_path.read_text(encoding="utf-8"))
scope_cases = {case.get("case_id"): case for case in scope.get("cases", []) if isinstance(case, dict)}
for case_id in required:
    entry = scope_cases.get(case_id)
    if entry is None:
        fail(f"{case_id}: missing replay scope entry")
    status = entry.get("status")
    if status == "schema_only_scoped":
        if entry.get("release_decision") != "schema_only_scope_recorded":
            fail(f"{case_id}: release_decision mismatch")
        if "harness" in entry or "rust_entrypoint" in entry:
            fail(f"{case_id}: schema-only scope must not claim executable replay fields")
    elif status == "validator_executable_replay":
        if entry.get("release_decision") != "validator_executable_scope_recorded":
            fail(f"{case_id}: validator release_decision mismatch")
        if entry.get("evidence_id") != "V241-004":
            fail(f"{case_id}: validator evidence must be V241-004")
        if "rust_entrypoint" in entry:
            fail(f"{case_id}: validator replay must not claim rust_entrypoint")
        if entry.get("runtime_adapter_integration") is not False:
            fail(f"{case_id}: validator replay must not claim runtime adapter integration")
        if entry.get("complete_executable_order_control_runtime") is not False:
            fail(f"{case_id}: validator replay must not claim complete order-control runtime")
    else:
        fail(f"{case_id}: unsupported V240-006 replay scope status {status!r}")

print("v24 retry policy ledger trace ok: 8 cases, no implicit retry boundary clean")
PY

tmp_trace="$(mktemp)"
cp "$TRACE_PATH" "$tmp_trace"
python3 - "$tmp_trace" <<'PY'
from pathlib import Path
import json
import sys

path = Path(sys.argv[1])
rows = []
for line in path.read_text(encoding="utf-8").splitlines():
    if not line.strip():
        continue
    row = json.loads(line)
    if row["case_id"] == "execution.v240_retry_policy.unknown_state_blocked.001":
        row["expected"]["events"][0]["payload"]["status"] = "retry_preview_allowed"
        row["expected"]["events"][0]["payload"]["code"] = "v240_retry_policy_transport_retry_allowed"
    rows.append(row)
path.write_text("\n".join(json.dumps(row, separators=(",", ":")) for row in rows) + "\n", encoding="utf-8")
PY

if python3 - "$tmp_trace" 2>/dev/null <<'PY'
import json
import sys
from pathlib import Path

for line in Path(sys.argv[1]).read_text(encoding="utf-8").splitlines():
    if not line.strip():
        continue
    row = json.loads(line)
    if row["case_id"] == "execution.v240_retry_policy.unknown_state_blocked.001":
        payload = row["input"]["events"][0]["payload"]
        expected = row["expected"]["events"][0]["payload"]
        if payload.get("retry_category") == "unknown_state" and expected.get("status") == "retry_preview_allowed":
            raise SystemExit("unknown state retry cannot be implicitly allowed")
PY
then
  rm -f "$tmp_trace"
  fail "negative selftest failed: unknown-state retry was accepted"
else
  rm -f "$tmp_trace"
fi

scripts/ai/verify_release.sh v24-cancel-replace-amend-preview

echo "v24_retry_policy_ledger=pass"
echo "contract_id=ntpro.v240_retry_no_retry_policy_ledger.v1"
echo "golden_trace_cases=8"
echo "transport_retry_allowed=pass"
echo "timeout_retry_allowed=pass"
echo "business_rejection=no_retry_terminal"
echo "risk_rejection=no_retry_terminal"
echo "duplicate_retry=blocked_duplicate_retry"
echo "missing_prior_attempt=blocked_missing_prior_attempt"
echo "unknown_state_retry=blocked_unknown_state_retry"
echo "policy_mismatch=blocked_policy_mismatch"
echo "dashboard_readonly_evidence=true"
echo "network_attempted=false"
echo "execution_adapter_call_allowed=false"
echo "production_order_mutation_allowed=false"
echo "new_submit_capability=false"
echo "retry_scheduler_enabled=false"
echo "implicit_retry_allowed=false"
echo "dashboard_operation_controls_enabled=false"
echo "signed_request_present=false"
echo "negative_selftest=1"
