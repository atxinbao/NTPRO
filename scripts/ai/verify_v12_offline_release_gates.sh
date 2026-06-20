#!/usr/bin/env bash
set -euo pipefail

# V120-008: v0.12 offline release gate bundle.
# This script is CI-safe. It keeps production network access disabled by
# default and only validates local artifacts, fail-closed preflights, and
# read-only/shadow invariants.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_V12_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V12_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V12_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi

export NTPRO_V12_SKIP_BUILD=1
export NTPRO_V12_NAUTILUS_BIN="$NAUTILUS_BIN"
unset NTPRO_V12_MANUAL_ONLINE

echo "== v12 offline release gates: public production read-only probe =="
scripts/ai/verify_v12_public_online_read_probe.sh

echo "== v12 offline release gates: authenticated account snapshot preflight =="
scripts/ai/verify_v12_authenticated_account_snapshot_online_read.sh

echo "== v12 offline release gates: response-shape validation =="
scripts/ai/verify_v12_response_shape.sh

echo "== v12 offline release gates: shadow portfolio runtime =="
scripts/ai/verify_v12_shadow_portfolio_runtime.sh

echo "== v12 offline release gates: persistent shadow strategy session =="
scripts/ai/verify_v12_persistent_shadow_strategy_session.sh

echo "== v12 offline release gates: read-only reconciliation =="
scripts/ai/verify_v12_production_readonly_reconciliation.sh

echo "== v12 offline release gates: Dashboard production shadow read-only panel =="
cargo test -p nautilus-cli production_shadow_v12_artifacts_populate_dashboard_readonly_panel --lib

echo "== v12 offline release gates: release boundary markers =="
grep -nE \
  "production_order_submissions_attempted|production_order_mutations_attempted|production_order_state_reads_attempted|listen_key_lifecycle_attempted|dashboard_order_controls_enabled|real_orders_submitted|values_are_exchange_truth" \
  docs/rust-cutover/release/v0_12_0_boundary.md \
  docs/rust-cutover/release/v0_12_0_release_gates.md \
  docs/rust-cutover/release/v0_12_0_response_shape.md \
  docs/rust-cutover/release/v0_12_0_shadow_portfolio_runtime.md \
  docs/rust-cutover/release/v0_12_0_persistent_shadow_strategy_session.md \
  docs/rust-cutover/release/v0_12_0_production_readonly_reconciliation.md \
  docs/rust-cutover/release/v0_12_0_dashboard_production_shadow_readonly_panel.md >/dev/null

if grep -nE "may claim.*(production trading|real-funds)|supports.*(production trading|real-funds)|(production trading|real-funds trading) ready|Dashboard order controls = enabled" \
  docs/rust-cutover/release/v0_12_0_boundary.md \
  docs/rust-cutover/release/v0_12_0_release_gates.md \
  docs/rust-cutover/release/v0_12_0_response_shape.md \
  docs/rust-cutover/release/v0_12_0_shadow_portfolio_runtime.md \
  docs/rust-cutover/release/v0_12_0_persistent_shadow_strategy_session.md \
  docs/rust-cutover/release/v0_12_0_production_readonly_reconciliation.md \
  docs/rust-cutover/release/v0_12_0_dashboard_production_shadow_readonly_panel.md >/dev/null; then
  echo "v12 release docs contain an enabled production trading or Dashboard order-control claim" >&2
  exit 1
fi

echo "v12_offline_release_gates status=ok network_attempted=false production_orders_submitted=0 production_order_mutations_attempted=0 production_order_state_reads_attempted=0 dashboard_order_controls_enabled=false"
