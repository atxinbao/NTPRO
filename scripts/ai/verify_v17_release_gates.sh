#!/usr/bin/env bash
set -euo pipefail

# V170-009: v0.17 aggregate release gates.
# Default execution is local/offline and fail-closed. This gate aggregates the
# v0.17 production reconciliation and orphan recovery evidence chain without
# opening production network access, sending production requests, submitting
# orders, retrying, canceling, replacing, amending, flattening, starting
# listenKey lifecycle, enabling Dashboard controls, or using real credentials.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_V17_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V17_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V17_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi

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

GATE_ROOT="${NTPRO_V17_RELEASE_GATE_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v17-release-gates.XXXXXX")}"
LOG_DIR="$GATE_ROOT/logs"
mkdir -p "$LOG_DIR"

run_gate() {
  local name="$1"
  shift
  local log="$LOG_DIR/$name.log"
  echo "== v17 release gates: $name =="
  "$@" 2>&1 | tee "$log"
}

run_gate local-order-ledger scripts/ai/verify_v17_local_order_ledger.sh
run_gate exchange-readback-mapper scripts/ai/verify_v17_exchange_readback_mapper.sh
run_gate reconciliation-classifier scripts/ai/verify_v17_reconciliation_classifier.sh
run_gate orphan-order-detection scripts/ai/verify_v17_orphan_order_detection.sh
run_gate restart-recovery scripts/ai/verify_v17_restart_recovery.sh
run_gate dashboard-reconciliation-panel scripts/ai/verify_v17_dashboard_reconciliation_panel.sh
run_gate failure-incident-integration scripts/ai/verify_v17_failure_incident_integration.sh

if grep -RE \
  "request_sent=true|network_attempted=true|manual_online_requested=true|production_order_submissions_attempted=[1-9][0-9]*|production_orders_submitted=[1-9][0-9]*|production_order_mutations_attempted=[1-9][0-9]*|production_order_state_reads_attempted=[1-9][0-9]*|listen_key_lifecycle_attempted=[1-9][0-9]*|duplicate_submit_attempted=true|retry_attempted=true|cancel_attempted=true|replace_attempted=true|amend_attempted=true|correction_attempted=true|flatten_attempted=true|remediation_attempted=true|automatic_cancel_allowed=true|automatic_remediation_allowed=true|actual_cancel_send_allowed=true|dashboard_order_controls_enabled=true|dashboard_cancel_controls_enabled=true|credential_inputs_enabled=true|real_orders_submitted=true|real_funds=true|production_trading_enabled=true|production_adapter_called=true|production_adapter_instantiated=true|signature_recorded=true|signed_query_recorded=true|signed_url_recorded=true|api_key_value_recorded=true|api_secret_value_recorded=true|api_key_header_value_recorded=true|raw_exchange_response_recorded=true|response_body_recorded=true|response_headers_recorded=true|request_body_recorded=true|raw_request_body_recorded=true" \
  "$LOG_DIR" >/tmp/ntpro-v17-release-gate-forbidden.txt; then
  echo "v17 release gate observed forbidden default production mutation evidence:" >&2
  cat /tmp/ntpro-v17-release-gate-forbidden.txt >&2
  exit 1
fi

RELEASE_DOCS=(
  docs/rust-cutover/release/v0_17_0_readiness_report.md
  docs/rust-cutover/release/v0_17_0_release_notes.md
  docs/rust-cutover/scope/v0_17_cancel_recovery_boundary.md
)

required_release_markers=(
  "Production Reconciliation And Orphan Recovery Evidence"
  "capability_expansion_from_v16 = reconciliation_evidence_only"
  "lineage_scope = single_v16_mutation_candidate"
  "default execution posture = offline fail-closed"
  "network readback execution = not included"
  "production order submission = not included"
  "production order mutation = not included"
  "actual cancel send = deferred"
  "automatic cancel = disabled"
  "Dashboard order controls = disabled"
  "Dashboard cancel controls = disabled"
  "retry_attempted = false"
  "cancel_attempted = false"
  "remediation_attempted = false"
)

for doc in "${RELEASE_DOCS[@]}"; do
  if [[ ! -f "$doc" ]]; then
    echo "missing v0.17 release document: $doc" >&2
    exit 1
  fi
done

for marker in "${required_release_markers[@]}"; do
  if ! grep -RFi "$marker" "${RELEASE_DOCS[@]}" >/dev/null; then
    echo "missing v0.17 release boundary marker: $marker" >&2
    exit 1
  fi
done

for evidence in \
  docs/rust-cutover/evidence/V170-000.md \
  docs/rust-cutover/evidence/V170-001.md \
  docs/rust-cutover/evidence/V170-002.md \
  docs/rust-cutover/evidence/V170-003.md \
  docs/rust-cutover/evidence/V170-004.md \
  docs/rust-cutover/evidence/V170-005.md \
  docs/rust-cutover/evidence/V170-006.md \
  docs/rust-cutover/evidence/V170-007.md \
  docs/rust-cutover/evidence/V170-008.md \
  docs/rust-cutover/evidence/V170-009.md; do
  if [[ ! -f "$evidence" ]]; then
    echo "missing v0.17 evidence file: $evidence" >&2
    exit 1
  fi
done

echo "v17_release_gates status=ok root=$GATE_ROOT default_offline=true capability_expansion_from_v16=reconciliation_evidence_only request_sent=false network_attempted=false production_order_submissions_attempted=0 production_orders_submitted=0 production_order_mutations_attempted=0 production_order_state_reads_attempted=0 listen_key_lifecycle_attempted=0 duplicate_submit_attempted=false retry_attempted=false cancel_attempted=false replace_attempted=false amend_attempted=false correction_attempted=false flatten_attempted=false remediation_attempted=false dashboard_order_controls_enabled=false dashboard_cancel_controls_enabled=false actual_cancel_send_allowed=false"
