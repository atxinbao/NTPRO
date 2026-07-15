#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

CONTRACT_PATH="${NTPRO_V24_READBACK_AUDIT_CONTRACT:-docs/rust-cutover/release/v0_24_0_readback_audit_evidence.md}"
TASK_PATH="${NTPRO_V24_READBACK_AUDIT_TASK:-docs/rust-cutover/tasks/V240-007.md}"
EVIDENCE_PATH="${NTPRO_V24_READBACK_AUDIT_EVIDENCE:-docs/rust-cutover/evidence/V240-007.md}"
TRACE_PATH="${NTPRO_V24_READBACK_AUDIT_TRACE:-tests/golden/v240_readback_audit_evidence.jsonl}"
REPLAY_SCOPE_PATH="${NTPRO_V24_READBACK_AUDIT_REPLAY_SCOPE:-docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"

fail() {
  echo "v24 readback audit evidence failed: $*" >&2
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
  "schema_version = ntpro.v240_order_control_readback_audit.v1" \
  "contract_id = ntpro.v240_order_control_readback_audit_evidence.v1" \
  "contract_status = preview_readback_audit_only_no_exchange_truth" \
  "start_gate_dependency = scripts/ai/verify_release.sh v24-retry-policy-ledger" \
  "golden_trace = tests/golden/v240_readback_audit_evidence.jsonl" \
  "preview_input_ref_required = true" \
  "decision_output_ref_required = true" \
  "policy_ref_required = true" \
  "risk_ref_required = true" \
  "readback_ref_required = true" \
  "audit_ref_required = true" \
  "provenance_ref_required = true" \
  "source_commit_current_required = true" \
  "dashboard_redacted_ref_required = true" \
  "ready_preview = readback and audit evidence complete" \
  "blocked = scope mismatch blocks preview closeout" \
  "degraded_unavailable = readback source unavailable, redacted audit retained, no exchange truth claim" \
  "fail_closed = missing readback, missing audit, missing provenance, stale source, or redaction breach" \
  "dashboard_readonly_evidence = true" \
  "redacted_audit_only = true" \
  "dashboard_can_consume_redacted_audit = true" \
  "exchange_truth_claimed = false" \
  "network_attempted = false" \
  "execution_adapter_call_allowed = false" \
  "production_order_mutation_allowed = false" \
  "new_submit_capability = false" \
  "real_order_state_read_expanded = false" \
  "dashboard_operation_controls_enabled = false" \
  "signed_request_present = false" \
  "secret_material_present = false" \
  "raw_readback_body_present = false"; do
  require_contains "$CONTRACT_PATH" "$marker"
done

for marker in \
  "new_submit_capability = true" \
  "production_order_mutation_allowed = true" \
  "execution_adapter_call_allowed = true" \
  "real_order_state_read_expanded = true" \
  "dashboard_operation_controls_enabled = true" \
  "signed_request_present = true" \
  "secret_material_present = true" \
  "raw_readback_body_present = true" \
  "exchange_truth_claimed = true" \
  "missing readback = allow" \
  "missing audit = allow" \
  "missing provenance = allow" \
  "redaction breach = allow"; do
  if contains "$CONTRACT_PATH" "$marker"; then
    fail "forbidden marker in $CONTRACT_PATH: $marker"
  fi
done

for marker in \
  "Task: \`V240-007\` / GitHub issue \`#750\`" \
  "tests/golden/v240_readback_audit_evidence.jsonl" \
  "scripts/ai/verify_release.sh v24-readback-audit-evidence"; do
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
    "execution.v240_readback_audit.ready_preview.001": ("ready_preview", "v240_readback_audit_ready", "ready_preview"),
    "execution.v240_readback_audit.missing_readback.001": ("fail_closed", "v240_readback_missing", "fail_closed"),
    "execution.v240_readback_audit.missing_audit.001": ("fail_closed", "v240_audit_missing", "fail_closed"),
    "execution.v240_readback_audit.missing_provenance.001": ("fail_closed", "v240_provenance_missing", "fail_closed"),
    "execution.v240_readback_audit.stale_source.001": ("fail_closed", "v240_source_stale", "fail_closed"),
    "execution.v240_readback_audit.redaction_breach.001": ("fail_closed", "v240_redaction_breach", "fail_closed"),
    "execution.v240_readback_audit.cross_scope_mismatch.001": ("blocked", "v240_scope_mismatch", "blocked"),
    "execution.v240_readback_audit.degraded_unavailable.001": ("degraded_unavailable", "v240_readback_degraded_unavailable", "degraded_unavailable"),
}

forbidden_keys = {
    "api_key",
    "api_secret",
    "raw_credential",
    "signature",
    "signed_payload",
    "signed_query",
    "signed_url",
    "raw_exchange_response",
    "raw_request_body",
    "raw_readback_body",
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
observed_states = set()
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
    if input_events[0].get("event_type") != "execution.readback_audit.request":
        fail(f"{case_id}: input event_type mismatch")
    if expected_events[0].get("event_type") != "execution.readback_audit.decision":
        fail(f"{case_id}: expected event_type mismatch")

    payload = input_events[0].get("payload") or {}
    expected = expected_events[0].get("payload") or {}
    status, code, closeout_state = required[case_id]
    observed_states.add(closeout_state)
    if expected.get("status") != status or expected.get("code") != code:
        fail(f"{case_id}: status/code mismatch")
    if expected.get("audit_closeout_state") != closeout_state:
        fail(f"{case_id}: audit closeout state mismatch")
    if expected.get("readback_audit_digest") != payload.get("readback_audit_digest"):
        fail(f"{case_id}: digest mismatch")

    for key in (
        "readback_audit_digest",
        "preview_input_ref",
        "decision_output_ref",
        "policy_ref",
        "risk_ref",
        "dashboard_redacted_ref",
        "source_commit",
        "expected_source_commit",
        "account_key",
        "strategy_key",
        "venue_node_key",
        "isolation_scope_key",
        "preview_scope_key",
        "decision_scope_key",
        "policy_scope_key",
        "risk_scope_key",
        "readback_scope_key",
        "audit_scope_key",
    ):
        if not payload.get(key):
            fail(f"{case_id}: missing {key}")

    if case_id != "execution.v240_readback_audit.missing_readback.001" and not payload.get("readback_ref"):
        fail(f"{case_id}: readback_ref required")
    if case_id != "execution.v240_readback_audit.missing_audit.001":
        for key in ("audit_ref", "audit_closeout_ref"):
            if not payload.get(key):
                fail(f"{case_id}: missing {key}")
    if case_id != "execution.v240_readback_audit.missing_provenance.001" and not payload.get("provenance_ref"):
        fail(f"{case_id}: provenance_ref required")

    if payload.get("submit_requested") is not False or payload.get("network_requested") is not False or payload.get("adapter_call_requested") is not False:
        fail(f"{case_id}: request must not request submit, network, or adapter")

    for trail, key, value in walk(row):
        if key in forbidden_keys:
            fail(f"{case_id}: forbidden key {trail}")
        if isinstance(value, str) and any(fragment in value for fragment in forbidden_fragments):
            fail(f"{case_id}: forbidden secret/request fragment at {trail}")

    if expected.get("dashboard_readonly_evidence") is not True or expected.get("redacted_audit_only") is not True:
        fail(f"{case_id}: expected read-only redacted audit evidence")
    for key in (
        "exchange_truth_claimed",
        "network_attempted",
        "execution_adapter_call_allowed",
        "live_exchange_request_allowed",
        "production_order_mutation_allowed",
        "new_submit_capability",
        "real_order_state_read_expanded",
        "dashboard_operation_controls_enabled",
        "signed_request_present",
        "secret_material_present",
        "raw_readback_body_present",
    ):
        if expected.get(key) is not False:
            fail(f"{case_id}: {key} must be false")

    scope_values = [
        payload.get("preview_scope_key"),
        payload.get("decision_scope_key"),
        payload.get("policy_scope_key"),
        payload.get("risk_scope_key"),
        payload.get("readback_scope_key"),
        payload.get("audit_scope_key"),
        payload.get("isolation_scope_key"),
    ]
    scopes_all_match = len(set(scope_values)) == 1
    if case_id == "execution.v240_readback_audit.cross_scope_mismatch.001":
        if payload.get("scopes_match") is not False or scopes_all_match:
            fail(f"{case_id}: cross-scope mismatch case must differ")
        if expected.get("scope_mismatch") is not True or expected.get("preview_ready") is not False:
            fail(f"{case_id}: scope mismatch must be blocked")
    else:
        if payload.get("scopes_match") is not True or not scopes_all_match:
            fail(f"{case_id}: scopes must match")

    if case_id == "execution.v240_readback_audit.ready_preview.001":
        if expected.get("preview_ready") is not True:
            fail(f"{case_id}: ready preview must be true")
        for key in ("readback_artifact_present", "audit_artifact_present", "provenance_present", "source_current", "redaction_clean"):
            if payload.get(key) is not True:
                fail(f"{case_id}: {key} must be true")
        if payload.get("readback_unavailable") is not False:
            fail(f"{case_id}: ready preview cannot be unavailable")
    if case_id == "execution.v240_readback_audit.missing_readback.001":
        if payload.get("readback_artifact_present") is not False or payload.get("readback_ref"):
            fail(f"{case_id}: missing readback case must omit readback")
        if expected.get("missing_readback") is not True or expected.get("preview_ready") is not False:
            fail(f"{case_id}: missing readback must fail closed")
    if case_id == "execution.v240_readback_audit.missing_audit.001":
        if payload.get("audit_artifact_present") is not False or payload.get("audit_ref"):
            fail(f"{case_id}: missing audit case must omit audit")
        if expected.get("missing_audit") is not True or expected.get("dashboard_can_consume_redacted_audit") is not False:
            fail(f"{case_id}: missing audit must block dashboard consumption")
    if case_id == "execution.v240_readback_audit.missing_provenance.001":
        if payload.get("provenance_present") is not False or payload.get("provenance_ref"):
            fail(f"{case_id}: missing provenance case must omit provenance")
        if expected.get("missing_provenance") is not True:
            fail(f"{case_id}: missing provenance must fail closed")
    if case_id == "execution.v240_readback_audit.stale_source.001":
        if payload.get("source_current") is not False or payload.get("source_commit") == payload.get("expected_source_commit"):
            fail(f"{case_id}: stale source case must differ")
        if expected.get("stale_source") is not True:
            fail(f"{case_id}: stale source must fail closed")
    if case_id == "execution.v240_readback_audit.redaction_breach.001":
        if payload.get("redaction_clean") is not False or payload.get("redaction_breach_detected") is not True:
            fail(f"{case_id}: redaction breach marker required")
        if expected.get("redaction_breach") is not True or expected.get("dashboard_can_consume_redacted_audit") is not False:
            fail(f"{case_id}: redaction breach must fail closed")
    if case_id == "execution.v240_readback_audit.degraded_unavailable.001":
        if payload.get("readback_unavailable") is not True or payload.get("readback_artifact_present") is not True:
            fail(f"{case_id}: degraded unavailable must retain an unavailable readback artifact")
        if expected.get("readback_unavailable") is not True or expected.get("preview_ready") is not False:
            fail(f"{case_id}: degraded unavailable must not be ready")

missing = sorted(set(required) - seen)
if missing:
    fail("missing cases: " + ", ".join(missing))
if observed_states != {"ready_preview", "blocked", "degraded_unavailable", "fail_closed"}:
    fail(f"audit closeout state coverage mismatch: {sorted(observed_states)}")

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
        fail(f"{case_id}: unsupported V240-007 replay scope status {status!r}")

print("v24 readback audit evidence trace ok: 8 cases, redacted no-exchange-truth boundary clean")
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
    if row["case_id"] == "execution.v240_readback_audit.redaction_breach.001":
        expected = row["expected"]["events"][0]["payload"]
        expected["status"] = "ready_preview"
        expected["code"] = "v240_readback_audit_ready"
        expected["audit_closeout_state"] = "ready_preview"
        expected["preview_ready"] = True
        expected["dashboard_can_consume_redacted_audit"] = True
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
    payload = row["input"]["events"][0]["payload"]
    expected = row["expected"]["events"][0]["payload"]
    if payload.get("redaction_clean") is False:
        if expected.get("status") != "fail_closed" or expected.get("code") != "v240_redaction_breach":
            raise SystemExit("redaction breach must fail closed")
PY
then
  rm -f "$tmp_trace"
  fail "negative selftest failed: redaction breach was accepted as ready"
else
  rm -f "$tmp_trace"
fi

scripts/ai/verify_release.sh v24-retry-policy-ledger

echo "v24_readback_audit_evidence=pass"
echo "contract_id=ntpro.v240_order_control_readback_audit_evidence.v1"
echo "golden_trace_cases=8"
echo "ready_preview=ready_preview"
echo "missing_readback=fail_closed"
echo "missing_audit=fail_closed"
echo "missing_provenance=fail_closed"
echo "stale_source=fail_closed"
echo "redaction_breach=fail_closed"
echo "cross_scope_mismatch=blocked"
echo "degraded_unavailable=degraded_unavailable"
echo "dashboard_readonly_evidence=true"
echo "redacted_audit_only=true"
echo "exchange_truth_claimed=false"
echo "network_attempted=false"
echo "execution_adapter_call_allowed=false"
echo "production_order_mutation_allowed=false"
echo "new_submit_capability=false"
echo "real_order_state_read_expanded=false"
echo "dashboard_operation_controls_enabled=false"
echo "signed_request_present=false"
echo "secret_material_present=false"
echo "raw_readback_body_present=false"
echo "negative_selftest=1"
