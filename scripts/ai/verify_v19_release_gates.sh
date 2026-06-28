#!/usr/bin/env bash
set -euo pipefail

# V190-010: v0.19 aggregate release gates.
# 聚合 v0.19 owner-approved single-shot actual cancel 证据链，并保持默认本地/offline。
# 该 gate 不打开生产网络，不允许自动/批量撤单，不增加 Dashboard 写操作控件。

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh
export NTPRO_SOURCE_COMMIT="${NTPRO_SOURCE_COMMIT:-$(git rev-parse HEAD)}"
export NTPRO_SOURCE_RELEASE_TAG="${NTPRO_SOURCE_RELEASE_TAG:-unreleased-v19-local-gate}"

if [[ "${NTPRO_V19_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V19_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V19_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi

export NTPRO_V19_SKIP_BUILD=1
export NTPRO_V19_NAUTILUS_BIN="$NAUTILUS_BIN"

unset NTPRO_ALLOW_PRODUCTION_LIVE_ALPHA_MUTATION
unset NTPRO_ALLOW_PRODUCTION_ORDER_SUBMISSION
unset NTPRO_ALLOW_PRODUCTION_ORDER_MUTATION
unset NTPRO_ALLOW_PRODUCTION_MUTATION_SIGNING_MATERIAL
unset NTPRO_ALLOW_PRODUCTION_MUTATION_HTTP_SEND
unset NTPRO_ALLOW_PRODUCTION_ORDER_STATE_READ
unset NTPRO_ALLOW_DASHBOARD_ORDER_CONTROLS
unset NTPRO_V15_MANUAL_ONLINE
unset BINANCE_API_KEY
unset BINANCE_API_SECRET
unset BINANCE_PRODUCTION_API_KEY
unset BINANCE_PRODUCTION_API_SECRET

GATE_ROOT="${NTPRO_V19_RELEASE_GATE_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v19-release-gates.XXXXXX")}"
LOG_DIR="$GATE_ROOT/logs"
mkdir -p "$LOG_DIR"

run_gate() {
  local name="$1"
  shift
  local log="$LOG_DIR/$name.log"
  echo "== v19 release gates: $name =="
  "$@" 2>&1 | tee "$log"
}

run_gate owner-approval-lifecycle-options \
  cargo test -p nautilus-cli parses_live_production_mutation_actual_cancel_owner_approval_lifecycle_options --lib
run_gate executor-adapter-boundary-options \
  cargo test -p nautilus-cli parses_live_production_mutation_actual_cancel_executor_adapter_boundary_options --lib
run_gate single-shot-options \
  cargo test -p nautilus-cli parses_live_production_mutation_actual_cancel_single_shot_options --lib
run_gate readback-reconciliation-options \
  cargo test -p nautilus-cli parses_live_production_mutation_actual_cancel_readback_reconciliation_options --lib
run_gate failure-evidence-options \
  cargo test -p nautilus-cli parses_live_production_mutation_actual_cancel_failure_evidence_options --lib
run_gate actual-cancel-contract-tests \
  cargo test -p nautilus-cli actual_cancel --lib
run_gate dashboard-actual-cancel-audit-tests \
  cargo test -p nautilus-cli production_actual_cancel_audit --lib

run_gate post-cancel-readback-reconciliation \
  env NTPRO_V19_POST_CANCEL_READBACK_RECONCILIATION_ROOT="$GATE_ROOT/post-cancel-readback-reconciliation" \
  scripts/ai/verify_v19_post_cancel_readback_reconciliation.sh
run_gate actual-cancel-failure-evidence \
  env NTPRO_V19_ACTUAL_CANCEL_FAILURE_EVIDENCE_ROOT="$GATE_ROOT/failure-evidence" \
  scripts/ai/verify_v19_actual_cancel_failure_evidence.sh
run_gate dashboard-actual-cancel-audit-view \
  scripts/ai/verify_v19_dashboard_actual_cancel_audit_view.sh
run_gate actual-cancel-golden-traces \
  scripts/ai/verify_v19_actual_cancel_golden_traces.sh
run_gate golden-trace-release-scope \
  python3 scripts/ai/validate_golden_trace_release_scope.py \
    --manifest docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json \
    --trace-glob 'tests/golden/*.jsonl'

python3 - "$GATE_ROOT" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
violations: list[str] = []
checked = 0

false_fields = {
    "actual_cancel_send_allowed",
    "network_readback_endpoint_attempted",
    "readback_execution_attempted",
    "order_state_read_attempted",
    "retry_attempted",
    "replace_attempted",
    "amend_attempted",
    "flatten_attempted",
    "remediation_attempted",
    "compensation_trade_attempted",
    "second_cancel_attempted",
    "automatic_cancel_allowed",
    "automatic_remediation_allowed",
    "bulk_cancel_allowed",
    "cancel_all_allowed",
    "multi_account_cancel_allowed",
    "multi_strategy_cancel_allowed",
    "multi_venue_cancel_allowed",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "production_order_state_reads_allowed",
    "listen_key_lifecycle_allowed",
    "dashboard_order_controls_enabled",
    "dashboard_cancel_controls_enabled",
    "dashboard_execution_allowed",
    "credential_inputs_enabled",
    "api_key_value_recorded",
    "api_secret_value_recorded",
    "api_key_header_value_recorded",
    "signature_recorded",
    "signed_query_recorded",
    "signed_url_recorded",
    "request_body_recorded",
    "raw_request_body_recorded",
    "raw_exchange_response_recorded",
    "raw_readback_body_recorded",
    "response_body_recorded",
    "response_headers_recorded",
    "unrestricted_payload_recorded",
    "account_balances_recorded",
    "fills_recorded",
    "real_orders_submitted",
    "real_funds",
    "production_trading_enabled",
    "platform_production_trading_enabled",
    "production_adapter_called",
    "production_adapter_instantiated",
}
zero_fields = {
    "cancel_requests_sent",
    "production_order_submissions_attempted",
    "production_orders_submitted",
    "production_order_state_reads_attempted",
    "listen_key_lifecycle_attempted",
}
forbidden_tokens = [
    "X-MBX-APIKEY",
    "apiSecret",
    "signature=must_not_persist",
    "signature=",
    "signedQuery=",
    "signedUrl=",
    "raw response must not persist",
    "raw readback must not persist",
    "apiSecret must not persist",
]

single_shot_ok_statuses = {
    "actual_cancel_attempt_recorded",
    "ready_actual_cancel_command_offline_no_send",
}

def visit(path: Path, value, trail: str, schema: str, status: str) -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            child_trail = f"{trail}.{key}" if trail else key
            if key in false_fields and child is True:
                violations.append(f"{path}:{child_trail}=true")
            if key in zero_fields and isinstance(child, int) and child != 0:
                violations.append(f"{path}:{child_trail}={child}")
            if key == "production_order_mutations_attempted" and isinstance(child, int) and child != 0:
                violations.append(f"{path}:{child_trail}={child}")
            visit(path, child, child_trail, schema, status)
    elif isinstance(value, list):
        for index, child in enumerate(value):
            visit(path, child, f"{trail}[{index}]", schema, status)
    elif isinstance(value, str):
        if "forbidden_" in trail and "_markers" in trail:
            return
        for token in forbidden_tokens:
            if token in value:
                violations.append(f"{path}:{trail} contains {token}")

for path in sorted(root.rglob("*.json")):
    if "fixtures" in path.parts:
        continue
    try:
        payload = json.loads(path.read_text())
    except json.JSONDecodeError:
        continue
    schema = payload.get("schema_version")
    if not isinstance(schema, str) or not schema.startswith("ntpro.v190_"):
        continue
    checked += 1
    status = str(payload.get("status", ""))
    visit(path, payload, "", schema, status)
    if schema == "ntpro.v190_actual_cancel_single_shot.v1":
        if status not in single_shot_ok_statuses:
            violations.append(f"{path}:unexpected single-shot status {status}")
        sent = bool(payload.get("request_sent"))
        if sent:
            if payload.get("single_shot_cancel_allowed") is not True:
                violations.append(f"{path}:request_sent without single_shot_cancel_allowed=true")
            if payload.get("owner_approval_ready") is not True:
                violations.append(f"{path}:request_sent without owner_approval_ready=true")
            if payload.get("risk_gate_ready") is not True:
                violations.append(f"{path}:request_sent without risk_gate_ready=true")
            if payload.get("adapter_boundary_ready") is not True:
                violations.append(f"{path}:request_sent without adapter_boundary_ready=true")
            if payload.get("approval_consumed_before_send") is not True:
                violations.append(f"{path}:request_sent without approval_consumed_before_send=true")
            if payload.get("readback_required") is not True:
                violations.append(f"{path}:request_sent without readback_required=true")
            if payload.get("cancel_requests_sent") != 1:
                violations.append(f"{path}:request_sent without cancel_requests_sent=1")

if checked == 0:
    violations.append(f"{root}: no ntpro.v190_* generated artifacts were checked")

if violations:
    print("v19 release gate observed forbidden actual-cancel evidence:", file=sys.stderr)
    for violation in violations:
        print(violation, file=sys.stderr)
    raise SystemExit(1)

print(f"v19_release_gate_artifact_scan checked={checked}")
PY

python3 - <<'PY'
import json
from pathlib import Path

violations: list[str] = []
rows = []
for raw in Path("tests/golden/actual_cancel_schema.jsonl").read_text().splitlines():
    if raw.strip():
        rows.append(json.loads(raw))

required = {
    "success": True,
    "approval_missing": False,
    "approval_reused": False,
    "risk_mismatch": False,
    "adapter_unsupported": False,
    "cancel_rejected": True,
    "timeout": True,
    "unknown": True,
    "already_cancelled": True,
    "partial_fill": True,
}
seen: dict[str, bool] = {}
for row in rows:
    payload = row["expected"]["events"][0]["payload"]
    scenario = payload.get("scenario")
    request_sent = payload.get("request_sent")
    seen[scenario] = request_sent
    if payload.get("owner_approval_required") is not True:
        violations.append(f"{scenario}: owner approval not required")
    if payload.get("retry_attempted") is not False:
        violations.append(f"{scenario}: retry attempted")
    if payload.get("second_cancel_attempted") is not False:
        violations.append(f"{scenario}: second cancel attempted")
    if payload.get("remediation_attempted") is not False:
        violations.append(f"{scenario}: remediation attempted")
    if payload.get("dashboard_cancel_controls_enabled") is not False:
        violations.append(f"{scenario}: dashboard cancel controls enabled")
    refs = payload.get("refs") or {}
    for key in ("request_ref", "response_ref", "readback_ref", "audit_ref", "provenance_ref"):
        if not refs.get(key):
            violations.append(f"{scenario}: missing {key}")

for scenario, request_sent in required.items():
    if scenario not in seen:
        violations.append(f"{scenario}: missing golden trace case")
    elif seen[scenario] is not request_sent:
        violations.append(f"{scenario}: request_sent={seen[scenario]} expected {request_sent}")

if violations:
    print("v19 actual cancel golden trace release scan failed:", file=sys.stderr)
    for violation in violations:
        print(violation, file=sys.stderr)
    raise SystemExit(1)

print(f"v19_actual_cancel_golden_trace_release_scan checked={len(rows)}")
PY

required_docs=(
  docs/rust-cutover/release/v0_19_0_actual_cancel_safety_contract.md
  docs/rust-cutover/release/v0_19_0_owner_approval_execution_lifecycle.md
  docs/rust-cutover/release/v0_19_0_cancel_executor_adapter_boundary.md
  docs/rust-cutover/release/v0_19_0_single_shot_cancel_command.md
  docs/rust-cutover/release/v0_19_0_post_cancel_readback_reconciliation.md
  docs/rust-cutover/release/v0_19_0_actual_cancel_failure_evidence.md
  docs/rust-cutover/release/v0_19_0_dashboard_actual_cancel_audit_view.md
  docs/rust-cutover/release/v0_19_0_actual_cancel_golden_trace_fixtures.md
  docs/rust-cutover/release/v0_19_0_release_notes.md
  docs/rust-cutover/release/v0_19_0_readiness_report.md
  docs/rust-cutover/evidence/V190-001.md
  docs/rust-cutover/evidence/V190-002.md
  docs/rust-cutover/evidence/V190-003.md
  docs/rust-cutover/evidence/V190-004.md
  docs/rust-cutover/evidence/V190-005.md
  docs/rust-cutover/evidence/V190-006.md
  docs/rust-cutover/evidence/V190-007.md
  docs/rust-cutover/evidence/V190-008.md
  docs/rust-cutover/evidence/V190-009.md
  docs/rust-cutover/evidence/V190-010.md
)

for doc in "${required_docs[@]}"; do
  if [[ ! -f "$doc" ]]; then
    echo "missing v0.19 release gate document: $doc" >&2
    exit 1
  fi
done

required_markers=(
  "owner-approved single-shot actual cancel"
  "actual cancel only owner-approved single-shot"
  "automatic cancel = not included"
  "bulk cancel = not included"
  "Dashboard cancel button = not included"
  "missing readback = release-blocking"
  "missing approval provenance = release-blocking"
  "v0.20 enters owner-approved production order lifecycle"
  "production order submit lifecycle = not included"
)

for marker in "${required_markers[@]}"; do
  if ! grep -F "$marker" docs/rust-cutover/release/v0_19_0_release_notes.md docs/rust-cutover/release/v0_19_0_readiness_report.md >/dev/null; then
    echo "missing v0.19 release boundary marker: $marker" >&2
    exit 1
  fi
done

if ! grep -F "v19-release-gates" docs/rust-cutover/evidence/V190-010.md >/dev/null; then
  echo "missing V190-010 v19-release-gates evidence marker" >&2
  exit 1
fi

echo "v19_release_gates status=ok root=$GATE_ROOT owner_approved_single_shot_only=true approval_provenance_required=true risk_gate_required=true adapter_boundary_required=true readback_required=true golden_traces_checked=true automatic_cancel_allowed=false bulk_cancel_allowed=false dashboard_cancel_controls_enabled=false retry_attempted=false second_cancel_attempted=false remediation_attempted=false production_order_submit_lifecycle=false v20_owner_approved_production_order_lifecycle_boundary=true"
