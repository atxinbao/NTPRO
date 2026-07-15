#!/usr/bin/env bash
set -euo pipefail

# V140-007: v0.14 release gate aggregation.
# This gate is CI-safe by default. It proves that v0.14 remains offline
# fail-closed unless the owner explicitly enables manual online read-only
# order-state proof gates. It never submits, cancels, replaces, amends, retries,
# corrects, or auto-remediates production orders.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_V14_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V14_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus --bin ntpro-node
fi

NAUTILUS_BIN="${NTPRO_V14_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
NTPRO_NODE_BIN="${NTPRO_V14_NTPRO_NODE_BIN:-$ROOT_DIR/target/debug/ntpro-node}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi
if [[ ! -x "$NTPRO_NODE_BIN" ]]; then
  echo "missing ntpro-node binary: $NTPRO_NODE_BIN" >&2
  exit 1
fi

export NTPRO_V14_SKIP_BUILD=1
export NTPRO_V14_NAUTILUS_BIN="$NAUTILUS_BIN"
export NTPRO_V14_NTPRO_NODE_BIN="$NTPRO_NODE_BIN"

unset NTPRO_V14_MANUAL_ONLINE
unset NTPRO_ALLOW_PRODUCTION_ORDER_STATE_READ
unset NTPRO_OWNER_APPROVED_PRODUCTION_ORDER_STATE_READ_ONLY
unset NTPRO_CONFIRM_PRODUCTION_ORDER_STATE_NO_ORDER_MUTATION
unset NTPRO_CONFIRM_NO_SECRET_PERSISTENCE
unset NTPRO_CONFIRM_NO_LISTEN_KEY_LIFECYCLE
unset NTPRO_CONFIRM_DASHBOARD_ORDER_CONTROLS_DISABLED
unset BINANCE_PRODUCTION_READONLY_API_KEY
unset BINANCE_PRODUCTION_READONLY_API_SECRET

echo "== v14 release gates: order-state read-only proof =="
scripts/ai/verify_v14_order_state_readonly_proof.sh

echo "== v14 release gates: supervisor shadow runtime =="
scripts/ai/verify_v14_supervisor_shadow_runtime.sh

echo "== v14 release gates: live-alpha dry-run order gate =="
scripts/ai/verify_v14_live_alpha_dry_run_order_gate.sh

echo "== v14 release gates: live-alpha risk preflight =="
scripts/ai/verify_v14_live_alpha_risk_preflight.sh

echo "== v14 release gates: live-alpha reconciliation golden traces =="
scripts/ai/ntpro_governance.sh golden-trace \
  tests/golden/live_alpha_reconciliation_schema.jsonl \
  --mode validate-only
cargo test -p nautilus-cli --test golden_trace_live_alpha_reconciliation

echo "== v14 release gates: release boundary markers =="
grep -nE \
  "default_execution_posture = offline_fail_closed|production_order_state_reads_allowed = owner_gated_only|production_order_state_reads_attempted = 0|production_order_submission_allowed = false|production_order_mutation_allowed = false|listen_key_lifecycle_allowed = false|dashboard_order_controls_enabled = false|real_funds_enabled = false|production_trading_enabled = false" \
  docs/rust-cutover/release/v0_14_0_order_state_readonly_boundary.md >/dev/null

grep -nE \
  "production_order_submission_allowed=false|production_order_mutation_allowed=false|production_order_state_reads_allowed=false|production_orders_submitted=0|production_order_mutations_attempted=0|dashboard_order_controls_enabled=false|execution_adapter_called=false|order_endpoint_access_attempted=false|network_attempted=false|real_orders_submitted=false|values_are_exchange_truth=false" \
  docs/rust-cutover/evidence/V140-003.md \
  docs/rust-cutover/evidence/V140-004.md \
  docs/rust-cutover/evidence/V140-005.md \
  docs/rust-cutover/evidence/V140-006.md >/dev/null

echo "== v14 release gates: forbidden claims =="
if grep -nE \
  "production_order_submission_allowed\\s*=\\s*true|production_order_mutation_allowed\\s*=\\s*true|dashboard_order_controls_enabled\\s*=\\s*true|listen_key_lifecycle_allowed\\s*=\\s*true|real_funds_enabled\\s*=\\s*true|production_trading_enabled\\s*=\\s*true|production trading ready|real funds ready|Dashboard order controls = enabled" \
  docs/rust-cutover/release/v0_14_0_order_state_readonly_boundary.md \
  docs/rust-cutover/evidence/V140-*.md \
  docs/rust-cutover/tasks/V140-*.md >/dev/null; then
  echo "v14 release docs/evidence contain an enabled production mutation or trading claim" >&2
  exit 1
fi

echo "v14_release_gates status=ok network_default_offline=true production_orders_submitted=0 production_order_mutations_attempted=0 dashboard_order_controls_enabled=false"
