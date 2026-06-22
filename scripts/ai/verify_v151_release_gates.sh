#!/usr/bin/env bash
set -euo pipefail

# V151-007: v0.15.1 patch/hardening release gates.
# This aggregate gate preserves the v0.15 guarded live-alpha dry-run boundary.
# It must not enable production order submission, production order mutation,
# production HTTP execution, production execution adapters, real funds, or
# Dashboard order controls.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_V151_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V151_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus --bin ntpro-node
fi

NAUTILUS_BIN="${NTPRO_V151_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
NTPRO_NODE_BIN="${NTPRO_V151_NTPRO_NODE_BIN:-$ROOT_DIR/target/debug/ntpro-node}"
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
export NTPRO_V15_SKIP_BUILD=1
export NTPRO_V15_NAUTILUS_BIN="$NAUTILUS_BIN"

unset NTPRO_V15_MANUAL_ONLINE
unset NTPRO_ALLOW_PRODUCTION_LIVE_ALPHA_MUTATION
unset NTPRO_ALLOW_PRODUCTION_ORDER_SUBMISSION
unset NTPRO_ALLOW_PRODUCTION_ORDER_MUTATION
unset NTPRO_ALLOW_DASHBOARD_ORDER_CONTROLS
unset NTPRO_ALLOW_PRODUCTION_MUTATION_SIGNING_MATERIAL
unset NTPRO_OWNER_APPROVED_MUTATION_SIGNING_DRY_RUN
unset BINANCE_API_KEY
unset BINANCE_API_SECRET
unset BINANCE_PRODUCTION_API_KEY
unset BINANCE_PRODUCTION_API_SECRET

GATE_ROOT="${NTPRO_V151_RELEASE_GATE_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v151-release-gates.XXXXXX")}"
LOG_DIR="$GATE_ROOT/logs"
mkdir -p "$LOG_DIR"

run_gate() {
  local name="$1"
  shift
  local log="$LOG_DIR/$name.log"
  echo "== v151 release gates: $name =="
  "$@" 2>&1 | tee "$log"
}

run_gate endpoint-classifier cargo test -p nautilus-cli endpoint_classifier --lib
run_gate dry-run-order-gate cargo test -p nautilus-cli production_live_alpha_dry_run_order_gate --lib
run_gate request-preview cargo test -p nautilus-cli production_live_alpha_order_request_preview --lib
run_gate execution-boundary cargo test -p nautilus-cli production_live_alpha_execution_dry_run --lib
run_gate v14-limit-alignment scripts/ai/verify_v14_release_gates.sh
run_gate v15-aggregate scripts/ai/verify_v15_release_gates.sh

if grep -RE \
  "request_sent=true|network_attempted=true|production_orders_submitted=[1-9][0-9]*|production_order_mutations_attempted=[1-9][0-9]*|dashboard_order_controls_enabled=true|production_adapter_called=true|production_adapter_instantiated=true|production_adapter_route_allowed=true|production_adapter_instantiation_allowed=true|real_orders_submitted=true|real_funds=true|production_trading_enabled=true" \
  "$LOG_DIR" >/tmp/ntpro-v151-release-gate-forbidden.txt; then
  echo "v151 release gate observed forbidden production mutation evidence:" >&2
  cat /tmp/ntpro-v151-release-gate-forbidden.txt >&2
  exit 1
fi

RELEASE_DOCS=(
  docs/rust-cutover/release/v0_15_1_readiness_report.md
  docs/rust-cutover/release/v0_15_1_release_notes.md
  docs/rust-cutover/release/v0_15_1_execution_dry_run_adapter_boundary.md
)
required_release_markers=(
  "patch/hardening"
  "capability expansion = false"
  "production order submission = not included"
  "production order mutation = not included"
  "production HTTP request execution = not included"
  "production execution adapter implementation = not included"
  "Dashboard order controls = not included"
  "StrategyIntent -> RiskDecision -> ExecutionCommand -> DryRunExecutionAdapter"
  "production_adapter_route_allowed = false"
  "production_adapter_instantiation_allowed = false"
)

for doc in "${RELEASE_DOCS[@]}"; do
  if [[ ! -f "$doc" ]]; then
    echo "missing v0.15.1 release document: $doc" >&2
    exit 1
  fi
done

for marker in "${required_release_markers[@]}"; do
  if ! grep -RFi "$marker" "${RELEASE_DOCS[@]}" >/dev/null; then
    echo "missing v0.15.1 release boundary marker: $marker" >&2
    exit 1
  fi
done

for evidence in \
  docs/rust-cutover/evidence/V151-001.md \
  docs/rust-cutover/evidence/V151-002.md \
  docs/rust-cutover/evidence/V151-003.md \
  docs/rust-cutover/evidence/V151-004.md \
  docs/rust-cutover/evidence/V151-005.md \
  docs/rust-cutover/evidence/V151-006.md \
  docs/rust-cutover/evidence/V151-007.md; do
  if [[ ! -f "$evidence" ]]; then
    echo "missing v0.15.1 evidence file: $evidence" >&2
    exit 1
  fi
done

echo "v151_release_gates status=ok root=$GATE_ROOT capability_expansion=false request_sent=false network_attempted=false production_orders_submitted=0 production_order_mutations_attempted=0 dashboard_order_controls_enabled=false production_adapter_called=false production_adapter_route_allowed=false"
