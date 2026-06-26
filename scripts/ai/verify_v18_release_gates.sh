#!/usr/bin/env bash
set -euo pipefail

# V180-010: v0.18 aggregate release gates.
# Default execution is local/offline and fail-closed. This gate aggregates the
# v0.18 owner-approved cancel recovery preview evidence chain without opening
# production network access, sending cancel requests, retrying, replacing,
# amending, flattening, auto-remediating, enabling Dashboard controls, or
# persisting raw secrets/responses.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh
export NTPRO_SOURCE_COMMIT="${NTPRO_SOURCE_COMMIT:-$(git rev-parse HEAD)}"
export NTPRO_SOURCE_RELEASE_TAG="${NTPRO_SOURCE_RELEASE_TAG:-unreleased-v18-local-gate}"

if [[ "${NTPRO_V18_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V18_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V18_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi

export NTPRO_V18_SKIP_BUILD=1
export NTPRO_V18_NAUTILUS_BIN="$NAUTILUS_BIN"
export NTPRO_V17_SKIP_BUILD=1
export NTPRO_V17_NAUTILUS_BIN="$NAUTILUS_BIN"
export NTPRO_V16_SKIP_BUILD=1
export NTPRO_V16_NAUTILUS_BIN="$NAUTILUS_BIN"

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

GATE_ROOT="${NTPRO_V18_RELEASE_GATE_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v18-release-gates.XXXXXX")}"
LOG_DIR="$GATE_ROOT/logs"
mkdir -p "$LOG_DIR"

run_gate() {
  local name="$1"
  shift
  local log="$LOG_DIR/$name.log"
  echo "== v18 release gates: $name =="
  "$@" 2>&1 | tee "$log"
}

run_gate cancel-request-preview \
  env NTPRO_V18_CANCEL_REQUEST_PREVIEW_ROOT="$GATE_ROOT/cancel-request-preview" \
  scripts/ai/verify_v18_cancel_request_preview.sh
run_gate cancel-risk-gate \
  env NTPRO_V18_CANCEL_RISK_GATE_ROOT="$GATE_ROOT/cancel-risk-gate" \
  scripts/ai/verify_v18_cancel_risk_gate.sh
run_gate manual-owner-approval-lifecycle \
  env NTPRO_V18_MANUAL_OWNER_APPROVAL_ROOT="$GATE_ROOT/manual-owner-approval" \
  scripts/ai/verify_v18_manual_owner_approval_lifecycle.sh
run_gate cancel-response-redaction \
  env NTPRO_V18_CANCEL_RESPONSE_REDACTION_ROOT="$GATE_ROOT/cancel-response-redaction" \
  scripts/ai/verify_v18_cancel_response_redaction.sh
run_gate post-cancel-readback \
  env NTPRO_V18_POST_CANCEL_READBACK_ROOT="$GATE_ROOT/post-cancel-readback" \
  scripts/ai/verify_v18_post_cancel_readback.sh
run_gate incident-audit-closeout \
  env NTPRO_V18_CANCEL_RECOVERY_INCIDENT_AUDIT_CLOSEOUT_ROOT="$GATE_ROOT/incident-audit-closeout" \
  scripts/ai/verify_v18_cancel_recovery_incident_audit_closeout.sh
run_gate dashboard-cancel-recovery-panel \
  env NTPRO_V18_DASHBOARD_SKIP_CLOSEOUT_CHAIN=1 \
  scripts/ai/verify_v18_dashboard_cancel_recovery_panel.sh

python3 - "$GATE_ROOT" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
violations: list[str] = []
checked = 0

false_fields = {
    "actual_cancel_send_allowed",
    "cancel_attempted",
    "network_attempted",
    "network_cancel_endpoint_attempted",
    "network_readback_endpoint_attempted",
    "retry_attempted",
    "replace_attempted",
    "amend_attempted",
    "flatten_attempted",
    "remediation_attempted",
    "automatic_cancel_allowed",
    "automatic_remediation_allowed",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "production_order_state_reads_allowed",
    "listen_key_lifecycle_allowed",
    "dashboard_order_controls_enabled",
    "dashboard_cancel_controls_enabled",
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
    "readback_execution_attempted",
    "order_state_read_attempted",
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
    "production_order_mutations_attempted",
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

def visit(path: Path, value, trail: str) -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            child_trail = f"{trail}.{key}" if trail else key
            if key in false_fields and child is True:
                violations.append(f"{path}:{child_trail}=true")
            if key in zero_fields and isinstance(child, int) and child != 0:
                violations.append(f"{path}:{child_trail}={child}")
            visit(path, child, child_trail)
    elif isinstance(value, list):
        for index, child in enumerate(value):
            visit(path, child, f"{trail}[{index}]")
    elif isinstance(value, str):
        if "forbidden_" in trail and "_markers" in trail:
            return
        for token in forbidden_tokens:
            if token in value:
                violations.append(f"{path}:{trail} contains {token}")

for path in sorted(root.rglob("*.json")):
    try:
        payload = json.loads(path.read_text())
    except json.JSONDecodeError:
        continue
    schema = payload.get("schema_version")
    if not isinstance(schema, str) or not schema.startswith("ntpro.v180_"):
        continue
    checked += 1
    visit(path, payload, "")

if checked == 0:
    violations.append(f"{root}: no ntpro.v180_* artifacts were checked")

if violations:
    print("v18 release gate observed forbidden cancel recovery evidence:", file=sys.stderr)
    for violation in violations:
        print(violation, file=sys.stderr)
    raise SystemExit(1)

print(f"v18_release_gate_artifact_scan checked={checked}")
PY

required_scope_markers=(
  "actual_cancel_send_allowed = false"
  "cancel_attempted = false"
  "cancel_requests_sent = 0"
  "automatic_cancel_allowed = false"
  "dashboard_cancel_controls_enabled = false"
  "production_order_mutation_allowed = false"
  "network_cancel_endpoint_attempted = false"
)

for marker in "${required_scope_markers[@]}"; do
  if ! grep -F "$marker" docs/rust-cutover/scope/v0_18_0_owner_approved_cancel_recovery_preview.md >/dev/null; then
    echo "missing v0.18 release gate scope marker: $marker" >&2
    exit 1
  fi
done

for evidence in \
  docs/rust-cutover/evidence/V180-001.md \
  docs/rust-cutover/evidence/V180-002.md \
  docs/rust-cutover/evidence/V180-003.md \
  docs/rust-cutover/evidence/V180-004.md \
  docs/rust-cutover/evidence/V180-005.md \
  docs/rust-cutover/evidence/V180-006.md \
  docs/rust-cutover/evidence/V180-007.md \
  docs/rust-cutover/evidence/V180-008.md \
  docs/rust-cutover/evidence/V180-009.md \
  docs/rust-cutover/evidence/V180-010.md; do
  if [[ ! -f "$evidence" ]]; then
    echo "missing v0.18 evidence file: $evidence" >&2
    exit 1
  fi
done

echo "v18_release_gates status=ok root=$GATE_ROOT default_offline=true actual_cancel_send_allowed=false cancel_attempted=false cancel_requests_sent=0 network_cancel_endpoint_attempted=false retry_attempted=false replace_attempted=false amend_attempted=false flatten_attempted=false remediation_attempted=false automatic_cancel_allowed=false automatic_remediation_allowed=false dashboard_order_controls_enabled=false dashboard_cancel_controls_enabled=false production_order_mutation_allowed=false raw_secrets_recorded=false raw_responses_recorded=false production_trading_enabled=false"
