#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

CONTRACT_PATH="${NTPRO_V24_ORDER_SLICING_CONTRACT:-docs/rust-cutover/release/v0_24_0_order_slicing_preview.md}"
TASK_PATH="${NTPRO_V24_ORDER_SLICING_TASK:-docs/rust-cutover/tasks/V240-004.md}"
EVIDENCE_PATH="${NTPRO_V24_ORDER_SLICING_EVIDENCE:-docs/rust-cutover/evidence/V240-004.md}"
TRACE_PATH="${NTPRO_V24_ORDER_SLICING_TRACE:-tests/golden/v240_order_slicing_preview.jsonl}"
REPLAY_SCOPE_PATH="${NTPRO_V24_ORDER_SLICING_REPLAY_SCOPE:-docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"

fail() {
  echo "v24 order slicing preview failed: $*" >&2
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
  "schema_version = ntpro.v240_order_slicing_preview.v1" \
  "contract_id = ntpro.v240_order_slicing_preview_foundation.v1" \
  "contract_status = preview_evidence_only_no_child_order_submission" \
  "start_gate_dependency = scripts/ai/verify_release.sh v24-rate-limit-throttle-gate" \
  "golden_trace = tests/golden/v240_order_slicing_preview.jsonl" \
  "parent_intent_id_required = true" \
  "execution_policy_id_required = true" \
  "slicing_policy_id_required = true" \
  "account_key_required = true" \
  "strategy_key_required = true" \
  "venue_node_key_required = true" \
  "isolation_scope_key_required = true" \
  "max_child_size_required = true" \
  "min_child_interval_ms_required = true" \
  "quantity_precision_required = true" \
  "price_precision_required = true" \
  "rounding_mode_required = true" \
  "risk_policy_refs_required = true" \
  "preview_ready = deterministic child-order preview plan produced" \
  "blocked_invalid_size = parent quantity or max child size is invalid" \
  "blocked_precision_mismatch = parent quantity, child quantity, price, or notional precision is invalid" \
  "blocked_scope_mismatch = slicing policy scope differs from parent intent scope" \
  "blocked_missing_policy = slicing policy or risk policy refs missing" \
  "blocked_forbidden_order_combo = market/limit combination is forbidden by policy" \
  "child_plan_preview_only = true" \
  "child_quantity_sum_equals_parent = true" \
  "child_quantity_lte_max_child_size = true" \
  "min_interval_enforced = true" \
  "notional_totals_required = true" \
  "rounding_evidence_required = true" \
  "dashboard_readonly_evidence = true" \
  "network_attempted = false" \
  "execution_adapter_call_allowed = false" \
  "production_order_mutation_allowed = false" \
  "new_submit_capability = false" \
  "child_order_submission_allowed = false" \
  "child_order_scheduler_enabled = false" \
  "dashboard_operation_controls_enabled = false" \
  "signed_request_present = false"; do
  require_contains "$CONTRACT_PATH" "$marker"
done

for marker in \
  "new_submit_capability = true" \
  "production_order_mutation_allowed = true" \
  "execution_adapter_call_allowed = true" \
  "child_order_submission_allowed = true" \
  "child_order_scheduler_enabled = true" \
  "dashboard_operation_controls_enabled = true" \
  "signed_request_present = true" \
  "blocked_invalid_size = allow" \
  "blocked_precision_mismatch = allow" \
  "blocked_scope_mismatch = allow" \
  "blocked_missing_policy = allow" \
  "blocked_forbidden_order_combo = allow"; do
  if contains "$CONTRACT_PATH" "$marker"; then
    fail "forbidden marker in $CONTRACT_PATH: $marker"
  fi
done

for marker in \
  "Task: \`V240-004\` / GitHub issue \`#747\`" \
  "tests/golden/v240_order_slicing_preview.jsonl" \
  "scripts/ai/verify_release.sh v24-order-slicing-preview"; do
  require_contains "$TASK_PATH" "$marker"
  require_contains "$EVIDENCE_PATH" "$marker"
done

python3 scripts/ai/golden_trace_runner.py "$TRACE_PATH" --mode validate-only
python3 scripts/ai/validate_golden_trace_release_scope.py \
  --manifest "$REPLAY_SCOPE_PATH" \
  --trace-glob 'tests/golden/*.jsonl'

python3 - "$TRACE_PATH" "$REPLAY_SCOPE_PATH" <<'PY'
from decimal import Decimal, InvalidOperation
import json
import sys
from pathlib import Path

trace_path = Path(sys.argv[1])
scope_path = Path(sys.argv[2])

required = {
    "execution.v240_order_slicing.valid_plan.001": ("preview_ready", "v240_order_slicing_preview_ready"),
    "execution.v240_order_slicing.invalid_size.001": ("blocked_invalid_size", "v240_order_slicing_invalid_size"),
    "execution.v240_order_slicing.precision_mismatch.001": ("blocked_precision_mismatch", "v240_order_slicing_precision_mismatch"),
    "execution.v240_order_slicing.scope_mismatch.001": ("blocked_scope_mismatch", "v240_order_slicing_scope_mismatch"),
    "execution.v240_order_slicing.policy_missing.001": ("blocked_missing_policy", "v240_order_slicing_missing_policy"),
    "execution.v240_order_slicing.forbidden_market_limit_combo.001": ("blocked_forbidden_order_combo", "v240_order_slicing_forbidden_order_combo"),
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


def decimal_value(value, case_id: str, key: str) -> Decimal:
    try:
        return Decimal(str(value))
    except InvalidOperation:
        fail(f"{case_id}: invalid decimal {key}={value!r}")


def scale_ok(value: str, precision: int) -> bool:
    decimal = Decimal(str(value))
    exponent = decimal.as_tuple().exponent
    return abs(exponent) <= precision if exponent < 0 else True


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
    if input_events[0].get("event_type") != "execution.order_slicing.request":
        fail(f"{case_id}: input event_type mismatch")
    if expected_events[0].get("event_type") != "execution.order_slicing.decision":
        fail(f"{case_id}: expected event_type mismatch")

    payload = input_events[0].get("payload") or {}
    expected = expected_events[0].get("payload") or {}
    status, code = required[case_id]
    if expected.get("status") != status or expected.get("code") != code:
        fail(f"{case_id}: status/code mismatch")

    for key in ("parent_intent_id", "execution_policy_id", "account_key", "strategy_key", "venue_node_key", "isolation_scope_key", "policy_scope_key"):
        if not payload.get(key):
            fail(f"{case_id}: missing {key}")
    for key in ("parent_quantity", "max_child_size", "min_child_interval_ms", "quantity_precision", "price_precision", "rounding_mode"):
        if key not in payload:
            fail(f"{case_id}: missing {key}")
    if payload.get("submit_requested") is not False or payload.get("network_requested") is not False:
        fail(f"{case_id}: preview request must not request submit or network")

    if expected.get("dashboard_readonly_evidence") is not True:
        fail(f"{case_id}: dashboard_readonly_evidence must be true")
    if expected.get("child_plan_preview_only") is not True:
        fail(f"{case_id}: child_plan_preview_only must be true")
    for key in (
        "network_attempted",
        "execution_adapter_call_allowed",
        "live_exchange_request_allowed",
        "production_order_mutation_allowed",
        "new_submit_capability",
        "child_order_submission_allowed",
        "child_order_scheduler_enabled",
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

    child_plan = expected.get("child_plan") or []
    if status == "preview_ready":
        if not child_plan:
            fail(f"{case_id}: preview_ready must include child_plan")
        parent_quantity = decimal_value(payload.get("parent_quantity"), case_id, "parent_quantity")
        max_child_size = decimal_value(payload.get("max_child_size"), case_id, "max_child_size")
        limit_price = decimal_value(payload.get("limit_price"), case_id, "limit_price")
        min_interval_ms = int(payload["min_child_interval_ms"])
        quantity_precision = int(payload["quantity_precision"])
        price_precision = int(payload["price_precision"])
        if parent_quantity <= 0 or max_child_size <= 0 or min_interval_ms <= 0:
            fail(f"{case_id}: valid plan must have positive quantity, max child size, and interval")
        if not scale_ok(payload["parent_quantity"], quantity_precision):
            fail(f"{case_id}: parent quantity exceeds precision")
        if not scale_ok(payload["limit_price"], price_precision):
            fail(f"{case_id}: limit price exceeds precision")

        total_quantity = Decimal("0")
        total_notional = Decimal("0")
        previous_offset = None
        for child in child_plan:
            if child.get("preview_only") is not True:
                fail(f"{case_id}: child must be preview_only")
            if child.get("signed_request_present") is not False:
                fail(f"{case_id}: child signed_request_present must be false")
            child_quantity = decimal_value(child.get("quantity"), case_id, "child.quantity")
            child_notional = decimal_value(child.get("notional"), case_id, "child.notional")
            if child_quantity <= 0 or child_quantity > max_child_size:
                fail(f"{case_id}: child quantity outside max_child_size")
            if not scale_ok(child["quantity"], quantity_precision):
                fail(f"{case_id}: child quantity exceeds precision")
            expected_notional = child_quantity * limit_price
            if child_notional != expected_notional:
                fail(f"{case_id}: child notional mismatch")
            total_quantity += child_quantity
            total_notional += child_notional
            offset = int(child.get("offset_ms"))
            if previous_offset is not None and offset - previous_offset < min_interval_ms:
                fail(f"{case_id}: min interval not enforced")
            previous_offset = offset
        if total_quantity != parent_quantity:
            fail(f"{case_id}: child quantity sum must equal parent quantity")
        if decimal_value(expected.get("child_quantity_sum"), case_id, "child_quantity_sum") != parent_quantity:
            fail(f"{case_id}: child_quantity_sum mismatch")
        if decimal_value(expected.get("notional_total"), case_id, "notional_total") != total_notional:
            fail(f"{case_id}: notional_total mismatch")
        rounding = expected.get("rounding_evidence") or {}
        if rounding.get("mode") != payload.get("rounding_mode") or rounding.get("remainder_quantity") is None:
            fail(f"{case_id}: rounding evidence incomplete")
        if not expected.get("risk_policy_refs"):
            fail(f"{case_id}: risk policy refs required")
    else:
        if child_plan:
            fail(f"{case_id}: blocked cases must not emit child_plan")

    if code == "v240_order_slicing_invalid_size":
        if decimal_value(payload.get("parent_quantity"), case_id, "parent_quantity") > 0 and decimal_value(payload.get("max_child_size"), case_id, "max_child_size") > 0:
            fail(f"{case_id}: invalid size case must contain invalid quantity or max child size")
    if code == "v240_order_slicing_precision_mismatch":
        if scale_ok(payload["parent_quantity"], int(payload["quantity_precision"])):
            fail(f"{case_id}: precision mismatch case must exceed quantity precision")
    if code == "v240_order_slicing_scope_mismatch":
        if payload.get("isolation_scope_key") == payload.get("policy_scope_key"):
            fail(f"{case_id}: scope mismatch case must differ")
    if code == "v240_order_slicing_missing_policy":
        if payload.get("slicing_policy_present") is not False or payload.get("risk_policy_refs"):
            fail(f"{case_id}: missing policy case must omit policy data")
    if code == "v240_order_slicing_forbidden_order_combo":
        if not (payload.get("parent_order_type") == "market" and payload.get("child_order_type") == "limit" and payload.get("limit_price")):
            fail(f"{case_id}: forbidden combo case must mix market parent and limit child")

missing = sorted(set(required) - seen)
if missing:
    fail("missing cases: " + ", ".join(missing))

scope = json.loads(scope_path.read_text(encoding="utf-8"))
scope_cases = {case.get("case_id"): case for case in scope.get("cases", []) if isinstance(case, dict)}
for case_id in required:
    entry = scope_cases.get(case_id)
    if entry is None:
        fail(f"{case_id}: missing replay scope entry")
    if entry.get("status") != "schema_only_scoped":
        fail(f"{case_id}: V240-004 scope must be schema_only_scoped")
    if entry.get("release_decision") != "schema_only_scope_recorded":
        fail(f"{case_id}: release_decision mismatch")
    if "harness" in entry or "rust_entrypoint" in entry:
        fail(f"{case_id}: schema-only scope must not claim executable replay fields")

print("v24 order slicing preview trace ok: 6 cases, deterministic preview-only boundary clean")
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
    if row["case_id"] == "execution.v240_order_slicing.valid_plan.001":
        row["expected"]["events"][0]["payload"]["child_plan"][0]["quantity"] = "3.000"
        row["expected"]["events"][0]["payload"]["child_plan"][0]["notional"] = "90000.00"
    rows.append(row)
path.write_text("\n".join(json.dumps(row, separators=(",", ":")) for row in rows) + "\n", encoding="utf-8")
PY

if python3 - "$tmp_trace" 2>/dev/null <<'PY'
from decimal import Decimal
import json
import sys
from pathlib import Path

for line in Path(sys.argv[1]).read_text(encoding="utf-8").splitlines():
    if not line.strip():
        continue
    row = json.loads(line)
    if row.get("case_id") == "execution.v240_order_slicing.valid_plan.001":
        payload = row["input"]["events"][0]["payload"]
        expected = row["expected"]["events"][0]["payload"]
        max_child_size = Decimal(str(payload["max_child_size"]))
        for child in expected["child_plan"]:
            if Decimal(str(child["quantity"])) > max_child_size:
                raise SystemExit("child quantity cannot exceed max_child_size")
PY
then
  rm -f "$tmp_trace"
  fail "negative selftest failed: child quantity above max_child_size was accepted"
else
  rm -f "$tmp_trace"
fi

scripts/ai/verify_release.sh v24-rate-limit-throttle-gate

echo "v24_order_slicing_preview=pass"
echo "contract_id=ntpro.v240_order_slicing_preview_foundation.v1"
echo "golden_trace_cases=6"
echo "valid_plan=preview_ready"
echo "invalid_size=blocked_invalid_size"
echo "precision_mismatch=blocked_precision_mismatch"
echo "scope_mismatch=blocked_scope_mismatch"
echo "policy_missing=blocked_missing_policy"
echo "forbidden_market_limit_combo=blocked_forbidden_order_combo"
echo "dashboard_readonly_evidence=true"
echo "network_attempted=false"
echo "execution_adapter_call_allowed=false"
echo "production_order_mutation_allowed=false"
echo "new_submit_capability=false"
echo "child_order_submission_allowed=false"
echo "child_order_scheduler_enabled=false"
echo "dashboard_operation_controls_enabled=false"
echo "signed_request_present=false"
echo "negative_selftest=1"
