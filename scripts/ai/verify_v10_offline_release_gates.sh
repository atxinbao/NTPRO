#!/usr/bin/env bash
set -euo pipefail

# V100-010: v0.10 offline release gate bundle.
# This script is safe for local development, PR smoke, and tag release gates.
# It only runs offline fail-closed, schema, redaction, and read-only Dashboard
# checks. It must not open Binance network connections or submit/cancel orders.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_V10_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V10_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V10_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"

if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi

export NTPRO_V10_SKIP_BUILD=1
export NTPRO_V10_NAUTILUS_BIN="$NAUTILUS_BIN"

echo "== v10 offline release gates: fail-closed order gate =="
scripts/ai/verify_v10_offline_fail_closed.sh

echo "== v10 offline release gates: risk preflight =="
scripts/ai/verify_v10_order_preflight.sh

echo "== v10 offline release gates: signed request redaction =="
scripts/ai/verify_v10_signed_order_request.sh

echo "== v10 offline release gates: order-test preflight =="
scripts/ai/verify_v10_order_test_preflight.sh

echo "== v10 offline release gates: execution artifact contract =="
scripts/ai/verify_v10_execution_artifact_contract.sh

echo "== v10 offline release gates: reconciliation/orphan fixture =="
scripts/ai/verify_v10_reconciliation_fixture.sh

echo "== v10 offline release gates: Dashboard read-only proof =="
cargo test -p nautilus-cli testnet_order_proof_artifacts_populate_dashboard_read_only_fields --lib

if grep -E -n "NTPRO_V10_MANUAL_ONLINE=1|testnet_orders_submitted=0|production_orders_submitted=0|dashboard_order_controls=false|manual_submit_cancel_proof_observed=false" \
  scripts/ai/verify_v10_manual_order_proof_gate.sh \
  scripts/ai/verify_v10_execution_artifact_contract.sh \
  scripts/ai/verify_v10_reconciliation_fixture.sh \
  docs/rust-cutover/release/v0_10_0_order_boundary.md >/dev/null; then
  echo "v10 offline release gate boundary markers present"
else
  echo "v10 offline release gate boundary markers missing" >&2
  exit 1
fi

echo "v10_offline_release_gates status=ok network_attempted=false real_orders_submitted=false production_orders_submitted=0 dashboard_order_controls=false"
