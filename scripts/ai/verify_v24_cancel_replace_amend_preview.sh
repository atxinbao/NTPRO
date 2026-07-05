#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

CONTRACT_PATH="${NTPRO_V24_CANCEL_REPLACE_AMEND_CONTRACT:-docs/rust-cutover/release/v0_24_0_cancel_replace_amend_preview.md}"
TASK_PATH="${NTPRO_V24_CANCEL_REPLACE_AMEND_TASK:-docs/rust-cutover/tasks/V240-005.md}"
EVIDENCE_PATH="${NTPRO_V24_CANCEL_REPLACE_AMEND_EVIDENCE:-docs/rust-cutover/evidence/V240-005.md}"
TRACE_PATH="${NTPRO_V24_CANCEL_REPLACE_AMEND_TRACE:-tests/golden/v240_cancel_replace_amend_preview.jsonl}"
REPLAY_SCOPE_PATH="${NTPRO_V24_CANCEL_REPLACE_AMEND_REPLAY_SCOPE:-docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"

fail() {
  echo "v24 cancel replace amend preview failed: $*" >&2
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
  "schema_version = ntpro.v240_cancel_replace_amend_preview.v1" \
  "contract_id = ntpro.v240_cancel_replace_amend_preview_contract.v1" \
  "contract_status = preview_evidence_only_no_cancel_replace_amend_send" \
  "start_gate_dependency = scripts/ai/verify_release.sh v24-order-slicing-preview" \
  "golden_trace = tests/golden/v240_cancel_replace_amend_preview.jsonl" \
  "cancel_intent_schema_version = ntpro.v240_cancel_intent.v1" \
  "replace_intent_schema_version = ntpro.v240_replace_intent.v1" \
  "amend_intent_schema_version = ntpro.v240_amend_intent.v1" \
  "original_order_lineage_required = true" \
  "owner_approval_id_required = true" \
  "policy_approval_id_required = true" \
  "risk_gate_id_required = true" \
  "audit_gate_id_required = true" \
  "field_change_audit_required = true" \
  "cancel_preview_ready = cancel preview plan produced" \
  "replace_preview_ready = replace preview plan produced with audited field changes" \
  "amend_preview_ready = amend preview plan produced with audited field changes" \
  "blocked_missing_lineage = original order lineage missing" \
  "blocked_scope_mismatch = original order lineage scope differs from intent scope" \
  "blocked_expired_approval = owner or policy approval expired" \
  "blocked_forbidden_operation = forbidden replace, amend, or flatten requested" \
  "dashboard_readonly_evidence = true" \
  "network_attempted = false" \
  "execution_adapter_call_allowed = false" \
  "production_order_mutation_allowed = false" \
  "new_submit_capability = false" \
  "cancel_replace_amend_send_allowed = false" \
  "flatten_allowed = false" \
  "dashboard_operation_controls_enabled = false" \
  "signed_request_present = false"; do
  require_contains "$CONTRACT_PATH" "$marker"
done

for marker in \
  "new_submit_capability = true" \
  "production_order_mutation_allowed = true" \
  "execution_adapter_call_allowed = true" \
  "cancel_replace_amend_send_allowed = true" \
  "flatten_allowed = true" \
  "dashboard_operation_controls_enabled = true" \
  "signed_request_present = true" \
  "blocked_missing_lineage = allow" \
  "blocked_scope_mismatch = allow" \
  "blocked_expired_approval = allow" \
  "blocked_forbidden_operation = allow"; do
  if contains "$CONTRACT_PATH" "$marker"; then
    fail "forbidden marker in $CONTRACT_PATH: $marker"
  fi
done

for marker in \
  "Task: \`V240-005\` / GitHub issue \`#748\`" \
  "tests/golden/v240_cancel_replace_amend_preview.jsonl" \
  "scripts/ai/verify_release.sh v24-cancel-replace-amend-preview"; do
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
    "execution.v240_cancel_replace_amend.cancel_preview.001": ("cancel_preview_ready", "v240_cancel_preview_ready", "cancel"),
    "execution.v240_cancel_replace_amend.replace_preview.001": ("replace_preview_ready", "v240_replace_preview_ready", "replace"),
    "execution.v240_cancel_replace_amend.amend_preview.001": ("amend_preview_ready", "v240_amend_preview_ready", "amend"),
    "execution.v240_cancel_replace_amend.missing_lineage.001": ("blocked_missing_lineage", "v240_cancel_replace_amend_missing_lineage", "cancel"),
    "execution.v240_cancel_replace_amend.scope_mismatch.001": ("blocked_scope_mismatch", "v240_cancel_replace_amend_scope_mismatch", "replace"),
    "execution.v240_cancel_replace_amend.expired_approval.001": ("blocked_expired_approval", "v240_cancel_replace_amend_expired_approval", "amend"),
    "execution.v240_cancel_replace_amend.forbidden_operation.001": ("blocked_forbidden_operation", "v240_cancel_replace_amend_forbidden_operation", "flatten"),
}
expected_schema = {
    "cancel": "ntpro.v240_cancel_intent.v1",
    "replace": "ntpro.v240_replace_intent.v1",
    "amend": "ntpro.v240_amend_intent.v1",
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
    "exchange_order_id",
    "adapter_request_body",
    "production_route_handle",
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
    "exchangeOrderId",
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
    if input_events[0].get("event_type") != "execution.cancel_replace_amend.request":
        fail(f"{case_id}: input event_type mismatch")
    if expected_events[0].get("event_type") != "execution.cancel_replace_amend.decision":
        fail(f"{case_id}: expected event_type mismatch")

    payload = input_events[0].get("payload") or {}
    expected = expected_events[0].get("payload") or {}
    status, code, operation = required[case_id]
    if payload.get("operation") != operation or expected.get("operation") != operation:
        fail(f"{case_id}: operation mismatch")
    if expected.get("status") != status or expected.get("code") != code:
        fail(f"{case_id}: status/code mismatch")
    if operation in expected_schema and payload.get("intent_schema_version") != expected_schema[operation]:
        fail(f"{case_id}: intent schema mismatch")

    for key in ("operation_intent_id", "account_key", "strategy_key", "venue_node_key", "isolation_scope_key", "owner_approval_id", "policy_approval_id", "risk_gate_id", "audit_gate_id", "approval_expires_at_ns", "evaluated_at_ns"):
        if not payload.get(key):
            fail(f"{case_id}: missing {key}")
    if payload.get("submit_requested") is not False or payload.get("network_requested") is not False:
        fail(f"{case_id}: preview request must not request submit or network")

    if expected.get("dashboard_readonly_evidence") is not True or expected.get("no_send_preview") is not True:
        fail(f"{case_id}: expected read-only no-send evidence")
    for key in (
        "network_attempted",
        "execution_adapter_call_allowed",
        "live_exchange_request_allowed",
        "production_order_mutation_allowed",
        "new_submit_capability",
        "cancel_replace_amend_send_allowed",
        "flatten_allowed",
        "dashboard_operation_controls_enabled",
        "signed_request_present",
    ):
        if expected.get(key) is not False:
            fail(f"{case_id}: {key} must be false")

    for trail, key, value in walk(row):
        if key in forbidden_keys:
            fail(f"{case_id}: forbidden key {trail}")
        if isinstance(value, str) and any(fragment in value for fragment in forbidden_fragments):
            fail(f"{case_id}: forbidden secret/request fragment at {trail}")

    lineage = payload.get("original_order_lineage")
    if status != "blocked_missing_lineage":
        if not isinstance(lineage, dict):
            fail(f"{case_id}: lineage required")
        for key in ("original_order_id", "source_intent_id", "account_key", "strategy_key", "venue_node_key", "isolation_scope_key"):
            if not lineage.get(key):
                fail(f"{case_id}: missing lineage {key}")
    if status in {"cancel_preview_ready", "replace_preview_ready", "amend_preview_ready"}:
        if lineage.get("isolation_scope_key") != payload.get("isolation_scope_key"):
            fail(f"{case_id}: ready preview lineage scope mismatch")
        if int(payload["approval_expires_at_ns"]) <= int(payload["evaluated_at_ns"]):
            fail(f"{case_id}: ready preview approval must be fresh")
    if status == "replace_preview_ready":
        audit = expected.get("field_change_audit") or []
        input_changes = payload.get("field_changes") or []
        if not audit or len(audit) != len(input_changes):
            fail(f"{case_id}: replace field change audit incomplete")
        for entry in audit:
            for key in ("field", "from", "to", "reason", "audit_gate_id"):
                if not entry.get(key):
                    fail(f"{case_id}: replace audit entry missing {key}")
    if status == "amend_preview_ready":
        audit = expected.get("field_change_audit") or []
        if not audit:
            fail(f"{case_id}: amend field change audit required")
        for entry in audit:
            if not entry.get("audit_gate_id"):
                fail(f"{case_id}: amend audit gate required")
    if status == "blocked_missing_lineage" and lineage:
        fail(f"{case_id}: missing lineage case must omit lineage")
    if status == "blocked_scope_mismatch":
        if not lineage or lineage.get("isolation_scope_key") == payload.get("isolation_scope_key"):
            fail(f"{case_id}: scope mismatch case must differ")
        if expected.get("field_change_audit"):
            fail(f"{case_id}: blocked scope mismatch must not emit field audit")
    if status == "blocked_expired_approval":
        if int(payload["approval_expires_at_ns"]) > int(payload["evaluated_at_ns"]):
            fail(f"{case_id}: expired approval case must be stale")
    if status == "blocked_forbidden_operation":
        if payload.get("forbidden_operation_requested") is not True or operation != "flatten":
            fail(f"{case_id}: forbidden operation case must request flatten")

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
        fail(f"{case_id}: unsupported V240-005 replay scope status {status!r}")

print("v24 cancel replace amend preview trace ok: 7 cases, no-send boundary clean")
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
    if row["case_id"] == "execution.v240_cancel_replace_amend.cancel_preview.001":
        row["expected"]["events"][0]["payload"]["signed_request_present"] = True
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
    expected = row["expected"]["events"][0]["payload"]
    if expected.get("signed_request_present") is not False:
        raise SystemExit("signed_request_present must be false")
PY
then
  rm -f "$tmp_trace"
  fail "negative selftest failed: signed request marker was accepted"
else
  rm -f "$tmp_trace"
fi

scripts/ai/verify_release.sh v24-order-slicing-preview

echo "v24_cancel_replace_amend_preview=pass"
echo "contract_id=ntpro.v240_cancel_replace_amend_preview_contract.v1"
echo "golden_trace_cases=7"
echo "cancel_preview=ready"
echo "replace_preview=ready"
echo "amend_preview=ready"
echo "missing_lineage=blocked_missing_lineage"
echo "scope_mismatch=blocked_scope_mismatch"
echo "expired_approval=blocked_expired_approval"
echo "forbidden_operation=blocked_forbidden_operation"
echo "dashboard_readonly_evidence=true"
echo "network_attempted=false"
echo "execution_adapter_call_allowed=false"
echo "production_order_mutation_allowed=false"
echo "new_submit_capability=false"
echo "cancel_replace_amend_send_allowed=false"
echo "flatten_allowed=false"
echo "dashboard_operation_controls_enabled=false"
echo "signed_request_present=false"
echo "negative_selftest=1"
