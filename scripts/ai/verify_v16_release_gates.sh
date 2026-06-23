#!/usr/bin/env bash
set -euo pipefail

# V160-012: v0.16 aggregate release gates.
# Default execution is local/offline and fail-closed. This gate aggregates the
# v0.16 minimum owner-approved production order mutation candidate evidence
# without requiring real credentials, opening production network access,
# sending production requests, submitting orders, retrying, canceling, replacing,
# amending, flattening, starting listenKey lifecycle, or enabling Dashboard
# order controls.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_V16_SKIP_BUILD:-0}" != "1" && ( -z "${NTPRO_V16_NAUTILUS_BIN:-}" || -z "${NTPRO_V16_NTPRO_NODE_BIN:-}" ) ]]; then
  cargo build -p nautilus-cli --bin nautilus --bin ntpro-node
fi

NAUTILUS_BIN="${NTPRO_V16_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
NTPRO_NODE_BIN="${NTPRO_V16_NTPRO_NODE_BIN:-$ROOT_DIR/target/debug/ntpro-node}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi
if [[ ! -x "$NTPRO_NODE_BIN" ]]; then
  echo "missing ntpro-node binary: $NTPRO_NODE_BIN" >&2
  exit 1
fi

export NTPRO_V16_SKIP_BUILD=1
export NTPRO_V16_NAUTILUS_BIN="$NAUTILUS_BIN"
export NTPRO_V16_NTPRO_NODE_BIN="$NTPRO_NODE_BIN"
export NTPRO_V151_SKIP_BUILD=1
export NTPRO_V151_NAUTILUS_BIN="$NAUTILUS_BIN"
export NTPRO_V151_NTPRO_NODE_BIN="$NTPRO_NODE_BIN"

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

GATE_ROOT="${NTPRO_V16_RELEASE_GATE_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v16-release-gates.XXXXXX")}"
LOG_DIR="$GATE_ROOT/logs"
mkdir -p "$LOG_DIR"

run_gate() {
  local name="$1"
  shift
  local log="$LOG_DIR/$name.log"
  echo "== v16 release gates: $name =="
  "$@" 2>&1 | tee "$log"
}

run_gate v151-baseline scripts/ai/verify_v151_release_gates.sh
run_gate runtime-gates scripts/ai/verify_v16_runtime_gates.sh
run_gate signing-material-approval scripts/ai/verify_v16_signing_material_approval.sh
run_gate request-builder scripts/ai/verify_v16_request_builder.sh
run_gate guarded-send-path scripts/ai/verify_v16_guarded_send_path.sh
run_gate response-redaction scripts/ai/verify_v16_response_redaction.sh
run_gate order-state-readback scripts/ai/verify_v16_order_state_readback.sh
run_gate kill-switch-around-send scripts/ai/verify_v16_kill_switch_around_send.sh
run_gate audit-trail scripts/ai/verify_v16_mutation_audit_trail.sh
run_gate failure-no-retry scripts/ai/verify_v16_failure_no_retry_semantics.sh
run_gate dashboard-readonly-evidence scripts/ai/verify_v16_dashboard_readonly_evidence.sh

if grep -RE \
  "request_sent=true|network_attempted=true|manual_online_requested=true|production_order_submissions_attempted=[1-9][0-9]*|production_orders_submitted=[1-9][0-9]*|production_order_mutations_attempted=[1-9][0-9]*|production_order_state_reads_attempted=[1-9][0-9]*|listen_key_lifecycle_attempted=[1-9][0-9]*|retry_attempted=true|cancel_attempted=true|replace_attempted=true|amend_attempted=true|correction_attempted=true|flatten_attempted=true|remediation_attempted=true|dashboard_order_controls_enabled=true|real_orders_submitted=true|real_funds=true|production_trading_enabled=true|production_adapter_called=true|production_adapter_instantiated=true|signature_recorded=true|signed_query_recorded=true|signed_url_recorded=true|api_key_value_recorded=true|api_secret_value_recorded=true|raw_exchange_response_recorded=true|response_body_recorded=true|response_headers_recorded=true|request_body_recorded=true|raw_request_body_recorded=true" \
  "$LOG_DIR" >/tmp/ntpro-v16-release-gate-forbidden.txt; then
  echo "v16 release gate observed forbidden default production mutation evidence:" >&2
  cat /tmp/ntpro-v16-release-gate-forbidden.txt >&2
  exit 1
fi

RELEASE_DOCS=(
  docs/rust-cutover/release/v0_16_0_production_mutation_scope.md
  docs/rust-cutover/release/v0_16_0_response_redaction.md
  docs/rust-cutover/release/v0_16_0_audit_trail.md
  docs/rust-cutover/release/v0_16_0_failure_semantics.md
)

required_release_markers=(
  "Minimum Owner-Approved Production Order Mutation Candidate"
  "default execution posture = offline fail-closed"
  "production mutation default = disabled"
  "production order submission default = disabled"
  "maximum production mutation count per run = 1"
  "allowed order type = LIMIT"
  "allowed time in force = GTC"
  "Dashboard order controls"
  "request_sent = false by default/offline"
  "network_attempted = false by default/offline"
  "retry_attempted = false"
  "cancel_attempted = false"
  "replace_attempted = false"
  "amend_attempted = false"
  "flatten_attempted = false"
)

for doc in "${RELEASE_DOCS[@]}"; do
  if [[ ! -f "$doc" ]]; then
    echo "missing v0.16 release document: $doc" >&2
    exit 1
  fi
done

for marker in "${required_release_markers[@]}"; do
  if ! grep -RFi "$marker" "${RELEASE_DOCS[@]}" >/dev/null; then
    echo "missing v0.16 release boundary marker: $marker" >&2
    exit 1
  fi
done

for evidence in \
  docs/rust-cutover/evidence/V160-001.md \
  docs/rust-cutover/evidence/V160-002.md \
  docs/rust-cutover/evidence/V160-003.md \
  docs/rust-cutover/evidence/V160-004.md \
  docs/rust-cutover/evidence/V160-005.md \
  docs/rust-cutover/evidence/V160-006.md \
  docs/rust-cutover/evidence/V160-007.md \
  docs/rust-cutover/evidence/V160-008.md \
  docs/rust-cutover/evidence/V160-009.md \
  docs/rust-cutover/evidence/V160-010.md \
  docs/rust-cutover/evidence/V160-011.md; do
  if [[ ! -f "$evidence" ]]; then
    echo "missing v0.16 evidence file: $evidence" >&2
    exit 1
  fi
done

echo "v16_release_gates status=ok root=$GATE_ROOT default_offline=true request_sent=false network_attempted=false production_order_submissions_attempted=0 production_orders_submitted=0 production_order_mutations_attempted=0 production_order_state_reads_attempted=0 listen_key_lifecycle_attempted=0 retry_attempted=false cancel_attempted=false replace_attempted=false amend_attempted=false correction_attempted=false flatten_attempted=false remediation_attempted=false dashboard_order_controls_enabled=false"
