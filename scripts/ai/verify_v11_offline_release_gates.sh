#!/usr/bin/env bash
set -euo pipefail

# V110-009: v0.11 offline release gate bundle.
# This script is safe for local development, PR smoke, and tag release gates.
# It only runs offline account snapshot, read-only Dashboard, and release
# boundary checks. It must not require production credentials, open production
# network connections, or submit/cancel/replace/amend orders.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_V11_LIGHTWEIGHT:-0}" == "1" ]]; then
  echo "== v11 offline release gates: authenticated account snapshot contract unit tests =="
  cargo test -p nautilus-cli production_account_snapshot_contract --lib
else
  if [[ "${NTPRO_V11_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V11_NAUTILUS_BIN:-}" ]]; then
    cargo build -p nautilus-cli --bin nautilus
  fi

  NAUTILUS_BIN="${NTPRO_V11_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
  if [[ ! -x "$NAUTILUS_BIN" ]]; then
    echo "missing nautilus binary: $NAUTILUS_BIN" >&2
    exit 1
  fi

  export NTPRO_V11_SKIP_BUILD=1
  export NTPRO_V11_NAUTILUS_BIN="$NAUTILUS_BIN"

  echo "== v11 offline release gates: authenticated account snapshot contract =="
  scripts/ai/verify_v11_authenticated_account_snapshot_contract.sh
fi

echo "== v11 offline release gates: Dashboard read-only production shadow status =="
cargo test -p nautilus-cli production_shadow --lib

echo "== v11 offline release gates: release boundary markers =="
grep -nE \
  "production_order_submissions_attempted|production_order_mutations_attempted|dashboard_order_controls_enabled|shadow_execution_intent|shadow_portfolio_snapshot|order_lifecycle_state|reconciliation_events" \
  docs/rust-cutover/release/v0_11_0_boundary.md \
  docs/rust-cutover/release/v0_11_0_authenticated_account_snapshot_contract.md \
  docs/rust-cutover/release/v0_11_0_shadow_execution_intent.md \
  docs/rust-cutover/release/v0_11_0_shadow_portfolio_snapshot.md \
  docs/rust-cutover/release/v0_11_1_production_shadow_manifest.md \
  docs/rust-cutover/release/v0_11_0_order_lifecycle_state_model.md \
  docs/rust-cutover/release/v0_11_0_reconciliation_event_model.md >/dev/null

grep -nE \
  "actual_submission=false|production_orders_submitted=0|production_order_mutations_attempted=0|automatic_correction_orders_submitted=0|dashboard_order_controls_enabled=false|full_production_portfolio_parity_claimed=false" \
  docs/rust-cutover/release/v0_11_0_shadow_execution_intent.md \
  docs/rust-cutover/release/v0_11_0_shadow_portfolio_snapshot.md \
  docs/rust-cutover/release/v0_11_1_production_shadow_manifest.md \
  docs/rust-cutover/release/v0_11_0_order_lifecycle_state_model.md \
  docs/rust-cutover/release/v0_11_0_reconciliation_event_model.md >/dev/null

if grep -nE "may claim.*(production trading|real-funds)|supports.*(production trading|real-funds)|(production trading|real-funds trading) ready" \
  docs/rust-cutover/release/v0_11_0_boundary.md \
  docs/rust-cutover/release/v0_11_0_authenticated_account_snapshot_contract.md \
  docs/rust-cutover/release/v0_11_0_shadow_execution_intent.md \
  docs/rust-cutover/release/v0_11_0_shadow_portfolio_snapshot.md \
  docs/rust-cutover/release/v0_11_1_production_shadow_manifest.md \
  docs/rust-cutover/release/v0_11_0_order_lifecycle_state_model.md \
  docs/rust-cutover/release/v0_11_0_reconciliation_event_model.md >/dev/null; then
  echo "v11 release docs contain an enabled production trading claim" >&2
  exit 1
fi

echo "v11_offline_release_gates status=ok mode=${NTPRO_V11_LIGHTWEIGHT:-full} network_attempted=false production_orders_submitted=0 production_order_mutations_attempted=0 dashboard_order_controls=false"
