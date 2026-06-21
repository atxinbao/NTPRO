#!/usr/bin/env bash
set -euo pipefail

# V130-007: v0.13 Guarded Live Alpha Preflight no-production-mutation gate.
# This script is CI-safe. It keeps default execution offline/fail-closed and
# proves that v0.13 preflight evidence does not submit, cancel, replace, amend,
# retry, correct, reconnect, or otherwise mutate production orders.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_V13_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V13_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V13_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi

export NTPRO_V13_SKIP_BUILD=1
export NTPRO_V13_NAUTILUS_BIN="$NAUTILUS_BIN"
export NTPRO_V12_SKIP_BUILD=1
export NTPRO_V12_NAUTILUS_BIN="$NAUTILUS_BIN"
unset NTPRO_V12_MANUAL_ONLINE
unset NTPRO_V13_OWNER_RUN_ONLINE_READONLY_PROOF
unset NTPRO_V13_OWNER_ACCEPTS_PRODUCTION_READONLY_RISK

echo "== v13 no-production-mutation: shadow preflight session =="
scripts/ai/verify_v13_shadow_preflight_session.sh

echo "== v13 no-production-mutation: online read-only proof pack offline preflight =="
scripts/ai/verify_v13_online_readonly_proof_pack.sh

echo "== v13 no-production-mutation: kill-switch approval artifact =="
scripts/ai/verify_v13_kill_switch_approval_artifact.sh

echo "== v13 no-production-mutation: Dashboard control boundary =="
scripts/ai/verify_v13_dashboard_control_boundary.sh

echo "== v13 no-production-mutation: Decimal amount boundary =="
scripts/ai/verify_v13_decimal_amount_boundary.sh

echo "== v13 no-production-mutation: release boundary markers =="
grep -nE \
  "production_order_submission_allowed=false|production_order_mutation_allowed=false|dashboard_order_controls_enabled=false|live_alpha_money_math_ready=false|risk_or_execution_grade=false|production_reconnect_allowed=false|listen_key_lifecycle_allowed=false" \
  docs/rust-cutover/release/v0_13_0_scope_decision.md \
  docs/rust-cutover/release/v0_13_0_shadow_session_preflight.md \
  docs/rust-cutover/release/v0_13_0_online_readonly_proof_pack.md \
  docs/rust-cutover/release/v0_13_0_kill_switch_approval_artifact.md \
  docs/rust-cutover/release/v0_13_0_dashboard_control_boundary.md \
  docs/rust-cutover/release/v0_13_0_decimal_amount_boundary.md >/dev/null

echo "== v13 no-production-mutation: forbidden claims =="
if grep -nE \
  "production_order_submission_allowed=true|production_order_mutation_allowed=true|dashboard_order_controls_enabled=true|production_reconnect_allowed=true|listen_key_lifecycle_allowed=true|live_alpha_money_math_ready=true|risk_or_execution_grade=true|production trading ready|real funds ready|Dashboard order controls = enabled" \
  docs/rust-cutover/release/v0_13_0_scope_decision.md \
  docs/rust-cutover/release/v0_13_0_shadow_session_preflight.md \
  docs/rust-cutover/release/v0_13_0_online_readonly_proof_pack.md \
  docs/rust-cutover/release/v0_13_0_kill_switch_approval_artifact.md \
  docs/rust-cutover/release/v0_13_0_dashboard_control_boundary.md \
  docs/rust-cutover/release/v0_13_0_decimal_amount_boundary.md \
  docs/rust-cutover/evidence/V130-002.md \
  docs/rust-cutover/evidence/V130-003.md \
  docs/rust-cutover/evidence/V130-004.md \
  docs/rust-cutover/evidence/V130-005.md \
  docs/rust-cutover/evidence/V130-006.md >/dev/null; then
  echo "v13 release docs/evidence contain an enabled production mutation or money-math claim" >&2
  exit 1
fi

echo "v13_no_production_mutation_gate status=ok network_default_offline=true production_orders_submitted=0 production_order_mutations_attempted=0 dashboard_order_controls_enabled=false"
