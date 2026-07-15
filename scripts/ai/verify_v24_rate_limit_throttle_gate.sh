#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

CONTRACT_PATH="${NTPRO_V24_RATE_LIMIT_THROTTLE_CONTRACT:-docs/rust-cutover/release/v0_24_0_rate_limit_throttle_gate.md}"
TASK_PATH="${NTPRO_V24_RATE_LIMIT_THROTTLE_TASK:-docs/rust-cutover/tasks/V240-003.md}"
EVIDENCE_PATH="${NTPRO_V24_RATE_LIMIT_THROTTLE_EVIDENCE:-docs/rust-cutover/evidence/V240-003.md}"
TRACE_PATH="${NTPRO_V24_RATE_LIMIT_THROTTLE_TRACE:-tests/golden/v240_rate_limit_throttle_gate.jsonl}"
REPLAY_SCOPE_PATH="${NTPRO_V24_RATE_LIMIT_THROTTLE_REPLAY_SCOPE:-docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"

fail() {
  echo "v24 rate-limit throttle gate failed: $*" >&2
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
  "schema_version = ntpro.v240_rate_limit_throttle_gate.v1" \
  "contract_id = ntpro.v240_rate_limit_throttle_gate_preview.v1" \
  "contract_status = preview_evidence_only_no_runtime_throttle_execution" \
  "start_gate_dependency = scripts/ai/verify_release.sh v24-order-intent-policy" \
  "golden_trace = tests/golden/v240_rate_limit_throttle_gate.jsonl" \
  "account_key_required = true" \
  "strategy_key_required = true" \
  "venue_node_key_required = true" \
  "isolation_scope_key_required = true" \
  "allowed_preview = within burst, rolling window, and venue-specific caps" \
  "throttled = burst, rolling window, or venue-specific cap exceeded" \
  "blocked_missing_limit = rate-limit or throttle policy missing" \
  "blocked_scope_mismatch = policy scope differs from intent scope" \
  "dashboard_readonly_evidence = true" \
  "network_attempted = false" \
  "execution_adapter_call_allowed = false" \
  "production_order_mutation_allowed = false" \
  "new_submit_capability = false" \
  "dashboard_operation_controls_enabled = false"; do
  require_contains "$CONTRACT_PATH" "$marker"
done

for marker in \
  "new_submit_capability = true" \
  "production_order_mutation_allowed = true" \
  "execution_adapter_call_allowed = true" \
  "dashboard_operation_controls_enabled = true" \
  "network_attempted = true" \
  "blocked_missing_limit = allow" \
  "blocked_scope_mismatch = allow"; do
  if contains "$CONTRACT_PATH" "$marker"; then
    fail "forbidden marker in $CONTRACT_PATH: $marker"
  fi
done

for marker in \
  "Task: \`V240-003\` / GitHub issue \`#746\`" \
  "tests/golden/v240_rate_limit_throttle_gate.jsonl" \
  "scripts/ai/verify_release.sh v24-rate-limit-throttle-gate"; do
  require_contains "$TASK_PATH" "$marker"
  require_contains "$EVIDENCE_PATH" "$marker"
done

scripts/ai/ntpro_governance.sh golden-trace "$TRACE_PATH" --mode validate-only
scripts/ai/ntpro_governance.sh golden-trace-release-scope \
  --manifest "$REPLAY_SCOPE_PATH" \
  --trace-glob 'tests/golden/*.jsonl'

python3 - "$TRACE_PATH" "$REPLAY_SCOPE_PATH" <<'PY'
import json
import sys
from pathlib import Path

trace_path = Path(sys.argv[1])
scope_path = Path(sys.argv[2])

required = {
    "execution.v240_rate_limit_throttle.allowed_preview.001": ("allowed_preview", "v240_rate_limit_allowed_preview"),
    "execution.v240_rate_limit_throttle.burst_exceeded.001": ("throttled", "v240_rate_limit_burst_exceeded"),
    "execution.v240_rate_limit_throttle.window_exceeded.001": ("throttled", "v240_rate_limit_window_exceeded"),
    "execution.v240_rate_limit_throttle.venue_cap_exceeded.001": ("throttled", "v240_rate_limit_venue_cap_exceeded"),
    "execution.v240_rate_limit_throttle.missing_limit_policy.001": ("blocked_missing_limit", "v240_rate_limit_missing_policy"),
    "execution.v240_rate_limit_throttle.scope_mismatch.001": ("blocked_scope_mismatch", "v240_rate_limit_scope_mismatch"),
}


def fail(message: str) -> None:
    raise SystemExit(message)


def rows(path: Path) -> list[dict]:
    return [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]


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
    payload = input_events[0].get("payload") or {}
    expected = expected_events[0].get("payload") or {}
    status, code = required[case_id]
    if expected.get("status") != status or expected.get("code") != code:
        fail(f"{case_id}: status/code mismatch")
    for key in ("account_key", "strategy_key", "venue_node_key", "isolation_scope_key", "policy_scope_key"):
        if not payload.get(key):
            fail(f"{case_id}: missing {key}")
    if expected.get("dashboard_readonly_evidence") is not True:
        fail(f"{case_id}: dashboard_readonly_evidence must be true")
    for key in (
        "network_attempted",
        "execution_adapter_call_allowed",
        "production_order_mutation_allowed",
        "new_submit_capability",
        "dashboard_operation_controls_enabled",
    ):
        if expected.get(key) is not False:
            fail(f"{case_id}: {key} must be false")
    if status == "allowed_preview":
        if not (
            payload["window_used"] < payload["window_limit"]
            and payload["burst_used"] < payload["burst_limit"]
            and payload["venue_window_used"] < payload["venue_window_limit"]
            and payload["isolation_scope_key"] == payload["policy_scope_key"]
        ):
            fail(f"{case_id}: allowed preview input is not within all limits")
    if code == "v240_rate_limit_burst_exceeded" and payload["burst_used"] < payload["burst_limit"]:
        fail(f"{case_id}: burst exceeded case must exhaust burst")
    if code == "v240_rate_limit_window_exceeded" and payload["window_used"] < payload["window_limit"]:
        fail(f"{case_id}: window exceeded case must exhaust window")
    if code == "v240_rate_limit_venue_cap_exceeded" and payload["venue_window_used"] < payload["venue_window_limit"]:
        fail(f"{case_id}: venue cap case must exhaust venue cap")
    if code == "v240_rate_limit_missing_policy":
        if payload.get("rate_limit_policy_present") is not False or payload.get("throttle_policy_present") is not False:
            fail(f"{case_id}: missing policy case must mark both policies absent")
    if code == "v240_rate_limit_scope_mismatch":
        if payload.get("isolation_scope_key") == payload.get("policy_scope_key"):
            fail(f"{case_id}: scope mismatch case must differ")
    if payload.get("submit_requested") is not False or payload.get("network_requested") is not False:
        fail(f"{case_id}: preview gate must not request submit or network")

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
        fail(f"{case_id}: unsupported V240-003 replay scope status {status!r}")

print("v24 rate-limit throttle trace ok: 6 cases, read-only boundary clean")
PY

tmp_trace="$(mktemp)"
cp "$TRACE_PATH" "$tmp_trace"
python3 - "$tmp_trace" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
text = text.replace('"status":"throttled","code":"v240_rate_limit_burst_exceeded"', '"status":"allowed_preview","code":"v240_rate_limit_allowed_preview"', 1)
path.write_text(text, encoding="utf-8")
PY

if python3 - "$tmp_trace" 2>/dev/null <<'PY'
import json
import sys
from pathlib import Path

for line in Path(sys.argv[1]).read_text(encoding="utf-8").splitlines():
    if not line.strip():
        continue
    row = json.loads(line)
    if row.get("case_id", "").endswith("burst_exceeded.001"):
        payload = row["input"]["events"][0]["payload"]
        expected = row["expected"]["events"][0]["payload"]
        if payload["burst_used"] >= payload["burst_limit"] and expected.get("status") == "allowed_preview":
            raise SystemExit("burst exhaustion cannot be allowed_preview")
PY
then
  rm -f "$tmp_trace"
  fail "negative selftest failed: burst exhaustion was accepted"
else
  rm -f "$tmp_trace"
fi

scripts/ai/verify_release.sh v24-order-intent-policy

echo "v24_rate_limit_throttle_gate=pass"
echo "contract_id=ntpro.v240_rate_limit_throttle_gate_preview.v1"
echo "golden_trace_cases=6"
echo "allowed_preview=pass"
echo "burst_exceeded=throttled"
echo "window_exceeded=throttled"
echo "venue_cap_exceeded=throttled"
echo "missing_limit_policy=blocked_missing_limit"
echo "scope_mismatch=blocked_scope_mismatch"
echo "dashboard_readonly_evidence=true"
echo "network_attempted=false"
echo "execution_adapter_call_allowed=false"
echo "production_order_mutation_allowed=false"
echo "new_submit_capability=false"
echo "dashboard_operation_controls_enabled=false"
echo "negative_selftest=1"
