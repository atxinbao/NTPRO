#!/usr/bin/env bash
set -euo pipefail

# V150-009: v0.15 aggregate release gates.
# Default execution is local/offline and fail-closed. It validates the guarded
# live-alpha mutation scope and dry-run harness without sending requests,
# submitting/canceling/replacing/amending/retrying/correcting production orders,
# opening network connections, using real funds, or enabling Dashboard order
# controls.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_V15_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V15_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V15_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi

export NTPRO_V15_SKIP_BUILD=1
export NTPRO_V15_NAUTILUS_BIN="$NAUTILUS_BIN"

unset NTPRO_V15_MANUAL_ONLINE
unset NTPRO_ALLOW_PRODUCTION_LIVE_ALPHA_MUTATION
unset NTPRO_ALLOW_PRODUCTION_ORDER_SUBMISSION
unset NTPRO_ALLOW_PRODUCTION_ORDER_MUTATION
unset NTPRO_ALLOW_DASHBOARD_ORDER_CONTROLS
unset BINANCE_API_KEY
unset BINANCE_API_SECRET
unset BINANCE_PRODUCTION_API_KEY
unset BINANCE_PRODUCTION_API_SECRET

GATE_ROOT="${NTPRO_V15_RELEASE_GATE_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v15-release-gates.XXXXXX")}"
LOG_DIR="$GATE_ROOT/logs"
mkdir -p "$LOG_DIR"

run_gate() {
  local name="$1"
  shift
  local log="$LOG_DIR/$name.log"
  echo "== v15 release gates: $name =="
  "$@" 2>&1 | tee "$log"
}

run_gate request-preview scripts/ai/verify_v15_live_order_request_dry_run_builder.sh
run_gate manual-approval scripts/ai/verify_v15_manual_approval_lifecycle.sh
run_gate execution-isolation scripts/ai/verify_v15_execution_adapter_isolation.sh
run_gate kill-switch-runtime scripts/ai/verify_v15_kill_switch_runtime_enforcement.sh
run_gate mutation-dry-run-golden-traces cargo test -p nautilus-cli --test golden_trace_live_alpha_mutation_dry_run
run_gate incident-rollback scripts/ai/verify_v15_incident_rollback_artifact.sh
run_gate dashboard-mutation-preflight cargo test -p nautilus-cli live_alpha_v15_dashboard --lib

if grep -RE \
  "request_sent=true|network_attempted=true|production_orders_submitted=[1-9][0-9]*|production_order_mutations_attempted=[1-9][0-9]*|dashboard_order_controls_enabled=true|production_adapter_called=true|production_adapter_instantiated=true|real_orders_submitted=true|real_funds=true|production_trading_enabled=true" \
  "$LOG_DIR" >/tmp/ntpro-v15-release-gate-forbidden.txt; then
  echo "v15 release gate observed forbidden production mutation evidence:" >&2
  cat /tmp/ntpro-v15-release-gate-forbidden.txt >&2
  exit 1
fi

RELEASE_DOCS=(
  docs/rust-cutover/release/v0_15_0_mutation_scope_decision.md
  docs/rust-cutover/release/v0_15_0_manual_approval_lifecycle.md
  docs/rust-cutover/release/v0_15_0_mutation_dry_run_golden_traces.md
  docs/rust-cutover/release/v0_15_0_incident_rollback_artifact.md
  docs/rust-cutover/release/v0_15_0_dashboard_mutation_preflight_panel.md
)
required_release_markers=(
  "production order request sent = false"
  "production order submission = not included"
  "production order mutation = not included"
  "Dashboard order controls = disabled"
  "real funds trading = not included"
  "production trading"
)

for marker in "${required_release_markers[@]}"; do
  if ! grep -RFi "$marker" "${RELEASE_DOCS[@]}" >/dev/null; then
    echo "missing v15 release boundary marker: $marker" >&2
    exit 1
  fi
done

echo "v15_release_gates status=ok root=$GATE_ROOT request_sent=false network_attempted=false production_orders_submitted=0 production_order_mutations_attempted=0 dashboard_order_controls_enabled=false production_adapter_called=false"
