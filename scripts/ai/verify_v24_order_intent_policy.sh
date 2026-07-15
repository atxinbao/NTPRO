#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

CONTRACT_PATH="${NTPRO_V24_ORDER_INTENT_POLICY_CONTRACT:-docs/rust-cutover/release/v0_24_0_order_intent_execution_policy.md}"
TASK_PATH="${NTPRO_V24_ORDER_INTENT_POLICY_TASK:-docs/rust-cutover/tasks/V240-002.md}"
EVIDENCE_PATH="${NTPRO_V24_ORDER_INTENT_POLICY_EVIDENCE:-docs/rust-cutover/evidence/V240-002.md}"
TRACE_PATH="${NTPRO_V24_ORDER_INTENT_POLICY_TRACE:-tests/golden/v240_order_intent_execution_policy.jsonl}"
REPLAY_SCOPE_PATH="${NTPRO_V24_ORDER_INTENT_POLICY_REPLAY_SCOPE:-docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"

fail() {
  echo "v24 order-intent policy failed: $*" >&2
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
  "schema_version = ntpro.v240_order_intent_policy_model.v1" \
  "contract_id = ntpro.v240_order_intent_execution_policy_model.v1" \
  "contract_status = artifact_model_only_no_runtime_adapter_call" \
  "start_gate_dependency = scripts/ai/verify_release.sh v24-order-control-contract" \
  "order_intent_schema_version = ntpro.v240_order_intent.v1" \
  "execution_policy_schema_version = ntpro.v240_execution_policy.v1" \
  "policy_provenance_id_required = true" \
  "owner_approval_id_required = true" \
  "risk_decision_id_required = true" \
  "audit_trace_id_required = true" \
  "source_provenance_id_required = true" \
  "missing_identity = fail_closed" \
  "missing_policy_provenance = fail_closed" \
  "policy_scope_mismatch = fail_closed" \
  "forbidden_operation = fail_closed" \
  "secret_or_signed_payload_present = fail_closed" \
  "new_submit_capability = false" \
  "production_order_mutation_allowed = false" \
  "execution_adapter_call_allowed = false" \
  "dashboard_operation_controls_enabled = false" \
  "valid_intent = execution.v240_order_intent_policy.valid_intent.001" \
  "missing_scope = execution.v240_order_intent_policy.missing_scope.001" \
  "policy_mismatch = execution.v240_order_intent_policy.policy_mismatch.001" \
  "forbidden_operation = execution.v240_order_intent_policy.forbidden_operation.001"; do
  require_contains "$CONTRACT_PATH" "$marker"
done

for marker in \
  "new_submit_capability = true" \
  "production_order_mutation_allowed = true" \
  "execution_adapter_call_allowed = true" \
  "dashboard_operation_controls_enabled = true" \
  "missing_identity = allow" \
  "policy_scope_mismatch = allow" \
  "secret_or_signed_payload_present = allow"; do
  if contains "$CONTRACT_PATH" "$marker"; then
    fail "forbidden marker in $CONTRACT_PATH: $marker"
  fi
done

for marker in \
  "Task: \`V240-002\` / GitHub issue \`#745\`" \
  "tests/golden/v240_order_intent_execution_policy.jsonl" \
  "scripts/ai/verify_release.sh v24-order-intent-policy"; do
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

required_cases = {
    "execution.v240_order_intent_policy.valid_intent.001": {
        "status": "preview_ready",
        "code": "v240_order_intent_policy_ready",
    },
    "execution.v240_order_intent_policy.missing_scope.001": {
        "status": "identity_blocked",
        "code": "v240_order_intent_missing_isolation_scope_key",
    },
    "execution.v240_order_intent_policy.policy_mismatch.001": {
        "status": "policy_blocked",
        "code": "v240_execution_policy_scope_mismatch",
    },
    "execution.v240_order_intent_policy.forbidden_operation.001": {
        "status": "blocked",
        "code": "v240_order_intent_forbidden_operation",
    },
}
forbidden_keys = {
    "api_key",
    "api_secret",
    "raw_credential",
    "signature",
    "signed_payload",
    "signed_query",
    "signed_url",
    "raw_request_body",
    "raw_exchange_response",
}
forbidden_fragments = (
    "X-MBX-APIKEY",
    "apiSecret",
    "signature=",
    "signedPayload",
    "signedQuery",
    "signedUrl",
    "raw request",
    "raw response",
)


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


def walk(value, trail: str = ""):
    if isinstance(value, dict):
        for key, child in value.items():
            child_trail = f"{trail}.{key}" if trail else key
            yield child_trail, key, child
            yield from walk(child, child_trail)
    elif isinstance(value, list):
        for index, child in enumerate(value):
            yield from walk(child, f"{trail}[{index}]")


trace_rows = rows(trace_path)
if len(trace_rows) != 4:
    fail(f"expected 4 V240-002 trace rows, got {len(trace_rows)}")

seen = set()
for row in trace_rows:
    case_id = row.get("case_id")
    if case_id not in required_cases:
        fail(f"unexpected V240-002 case: {case_id}")
    seen.add(case_id)
    if row.get("category") != "execution":
        fail(f"{case_id}: category must be execution")

    input_events = ((row.get("input") or {}).get("events") or [])
    expected_events = ((row.get("expected") or {}).get("events") or [])
    if len(input_events) != 1 or len(expected_events) != 1:
        fail(f"{case_id}: expected one input event and one expected event")
    payload = input_events[0].get("payload") or {}
    intent = payload.get("order_intent") or {}
    policy = payload.get("execution_policy") or {}
    expected = expected_events[0].get("payload") or {}

    if intent.get("schema_version") != "ntpro.v240_order_intent.v1":
        fail(f"{case_id}: order intent schema mismatch")
    if policy.get("schema_version") != "ntpro.v240_execution_policy.v1":
        fail(f"{case_id}: execution policy schema mismatch")
    if not intent.get("policy_provenance_id") or not policy.get("policy_provenance_id"):
        fail(f"{case_id}: missing policy provenance")
    for key in ("owner_approval_id", "risk_decision_id", "audit_trace_id", "source_provenance_id"):
        if not intent.get(key):
            fail(f"{case_id}: missing intent {key}")

    required = required_cases[case_id]
    if expected.get("status") != required["status"] or expected.get("code") != required["code"]:
        fail(f"{case_id}: expected status/code mismatch")
    for key in (
        "new_submit_capability",
        "production_order_mutation_allowed",
        "execution_adapter_call_allowed",
        "dashboard_operation_controls_enabled",
    ):
        if expected.get(key) is not False:
            fail(f"{case_id}: {key} must be false")

    if case_id.endswith("valid_intent.001"):
        for key in ("account_key", "strategy_key", "venue_node_key", "isolation_scope_key"):
            if intent.get(key) != policy.get(key):
                fail(f"{case_id}: {key} must match policy")
    if case_id.endswith("missing_scope.001") and intent.get("isolation_scope_key"):
        fail(f"{case_id}: missing scope case must omit intent isolation_scope_key")
    if case_id.endswith("policy_mismatch.001") and intent.get("strategy_key") == policy.get("strategy_key"):
        fail(f"{case_id}: policy mismatch case must mismatch strategy_key")
    if case_id.endswith("forbidden_operation.001"):
        if intent.get("submit_requested") is not True or intent.get("adapter_call_requested") is not True:
            fail(f"{case_id}: forbidden operation case must request submit and adapter call")
        if expected.get("forbidden_operation") is not True:
            fail(f"{case_id}: forbidden operation expected flag must be true")

    for trail, key, value in walk(row):
        if key in forbidden_keys:
            fail(f"{case_id}: forbidden key present at {trail}")
        if isinstance(value, str):
            for fragment in forbidden_fragments:
                if fragment in value:
                    fail(f"{case_id}: forbidden string fragment at {trail}: {fragment}")

missing = sorted(set(required_cases) - seen)
if missing:
    fail("missing V240-002 trace cases: " + ", ".join(missing))

scope = json.loads(scope_path.read_text(encoding="utf-8"))
scope_cases = {case.get("case_id"): case for case in scope.get("cases", []) if isinstance(case, dict)}
for case_id in required_cases:
    entry = scope_cases.get(case_id)
    if entry is None:
        fail(f"{case_id}: missing release replay scope entry")
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
        fail(f"{case_id}: unsupported V240-002 replay scope status {status!r}")

print("v24 order intent policy trace ok: 4 cases, redaction boundary clean")
PY

tmp_trace="$(mktemp)"
cp "$TRACE_PATH" "$tmp_trace"
python3 - "$tmp_trace" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
text = text.replace('"submit_requested":false', '"submit_requested":true', 1)
path.write_text(text, encoding="utf-8")
PY

if python3 - "$tmp_trace" 2>/dev/null <<'PY'
import json
import sys
from pathlib import Path

rows = [json.loads(line) for line in Path(sys.argv[1]).read_text(encoding="utf-8").splitlines() if line.strip()]
for row in rows:
    if row.get("case_id", "").endswith("valid_intent.001"):
        intent = row["input"]["events"][0]["payload"]["order_intent"]
        expected = row["expected"]["events"][0]["payload"]
        if intent.get("submit_requested") is True and expected.get("code") == "v240_order_intent_policy_ready":
            raise SystemExit("valid intent cannot request submit and still remain ready")
PY
then
  rm -f "$tmp_trace"
  fail "negative selftest failed: forbidden submit request was accepted"
else
  rm -f "$tmp_trace"
fi

scripts/ai/verify_release.sh v24-order-control-contract

echo "v24_order_intent_policy=pass"
echo "contract_id=ntpro.v240_order_intent_execution_policy_model.v1"
echo "golden_trace_cases=4"
echo "valid_intent=preview_ready"
echo "missing_scope=fail_closed"
echo "policy_mismatch=fail_closed"
echo "forbidden_operation=fail_closed"
echo "redaction_boundary=clean"
echo "new_submit_capability=false"
echo "production_order_mutation_allowed=false"
echo "execution_adapter_call_allowed=false"
echo "dashboard_operation_controls_enabled=false"
echo "negative_selftest=1"
