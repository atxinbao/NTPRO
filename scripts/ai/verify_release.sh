#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

FEATURES="${NAUTILUS_RUST_FEATURES:-arrow,ffi,high-precision,streaming,defi}"
export REQUIRE_GOLDEN_REPLAY="${REQUIRE_GOLDEN_REPLAY:-1}"

NAUTILUS_RELEASE_BIN="$ROOT/target/release/nautilus"
NTPRO_NODE_RELEASE_BIN="$ROOT/target/release/ntpro-node"

require_help_contains() {
  local file="$1"
  shift
  local pattern
  for pattern in "$@"; do
    if ! grep -Eiq -- "$pattern" "$file"; then
      echo "help output missing required pattern '$pattern' in $file" >&2
      cat "$file" >&2
      exit 1
    fi
  done
}

run_full_checks() {
  echo "== verify_release: full checks =="
  scripts/ai/verify_full.sh
}

release_cli_feature_args() {
  local selected=()
  local feature

  IFS=',' read -r -a feature_list <<< "$FEATURES"
  for feature in "${feature_list[@]}"; do
    feature="${feature//[[:space:]]/}"
    if [[ "$feature" == "defi" ]]; then
      selected+=("$feature")
    fi
  done

  if (( ${#selected[@]} > 0 )); then
    printf '%s\n' "--features"
    (IFS=','; printf '%s\n' "${selected[*]}")
  fi
}

ensure_release_cli_binaries() {
  echo "== verify_release: release CLI binary build =="
  local cli_feature_args=()
  while IFS= read -r arg; do
    cli_feature_args+=("$arg")
  done < <(release_cli_feature_args)

  cargo build -p nautilus-cli --release --bin nautilus --bin ntpro-node "${cli_feature_args[@]}"

  if [[ ! -x "$NAUTILUS_RELEASE_BIN" ]]; then
    echo "missing release nautilus binary: $NAUTILUS_RELEASE_BIN" >&2
    exit 1
  fi
  if [[ ! -x "$NTPRO_NODE_RELEASE_BIN" ]]; then
    echo "missing release ntpro-node binary: $NTPRO_NODE_RELEASE_BIN" >&2
    exit 1
  fi
}

run_release_build_product_surface() {
  ensure_release_cli_binaries

  echo "== verify_release: Rust CLI product surface =="
  if ! cargo metadata --no-deps --format-version=1 | grep -q '"name":"nautilus-cli"'; then
    echo "nautilus-cli package is missing" >&2
    exit 1
  fi

  "$NAUTILUS_RELEASE_BIN" --help >/tmp/nautilus_cli_help.txt
  require_help_contains /tmp/nautilus_cli_help.txt \
    'backtest' \
    'live' \
    'sandbox' \
    'data' \
    'supervisor'

  "$NAUTILUS_RELEASE_BIN" supervisor --help >/tmp/nautilus_supervisor_help.txt
  require_help_contains /tmp/nautilus_supervisor_help.txt \
    'register' \
    'list' \
    'start' \
    'stop' \
    'pause' \
    'resume' \
    'reconnect-data' \
    'reconnect-execution' \
    'status' \
    'logs' \
    'metrics' \
    'sandbox-only'

  "$NTPRO_NODE_RELEASE_BIN" --help >/tmp/ntpro_node_help.txt
  require_help_contains /tmp/ntpro_node_help.txt \
    'config' \
    'run-id' \
    'output' \
    'stop-file' \
    'sandbox-only'
}

run_rust_only_gates() {
  echo "== verify_release: Rust-only runtime check =="
  scripts/ai/check_rust_only_runtime.sh

  echo "== verify_release: final Cython removed check =="
  scripts/ai/check_cython_removed.sh
}

run_v02_supervisor_smoke() {
  echo "== verify_release: v0.2 two-node supervisor smoke =="
  if [[ "${NTPRO_V02_009_SKIP_BUILD:-0}" == "1" ]]; then
    echo "release gate must not set NTPRO_V02_009_SKIP_BUILD=1" >&2
    exit 1
  fi
  NTPRO_RELEASE_GATE=1 NTPRO_V02_009_SKIP_BUILD=0 scripts/ai/v02_two_node_supervisor_smoke.sh
}

run_v03_supervisor_control_smoke() {
  echo "== verify_release: v0.3 supervisor control smoke =="
  ensure_release_cli_binaries
  NTPRO_V03_CONTROL_SKIP_BUILD=1 \
    NTPRO_V03_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    NTPRO_V03_NODE_BIN="$NTPRO_NODE_RELEASE_BIN" \
    scripts/ai/v03_supervisor_control_smoke.sh
}

run_v03_dashboard_smoke() {
  echo "== verify_release: v0.3 dashboard control smoke =="
  ensure_release_cli_binaries
  NTPRO_V03_010_SKIP_BUILD=1 \
    NTPRO_V03_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    NTPRO_V03_NODE_BIN="$NTPRO_NODE_RELEASE_BIN" \
    scripts/ai/v03_dashboard_smoke.sh
}

run_v05_workflow_artifacts_smoke() {
  echo "== verify_release: v0.5 workflow artifact smoke =="
  ensure_release_cli_binaries
  NTPRO_V05_WORKFLOW_SKIP_BUILD=1 \
    NTPRO_V05_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v05_workflow_artifacts.sh
}

run_v06_binance_testnet_dry_run_smoke() {
  echo "== verify_release: v0.6 Binance testnet dry-run smoke =="
  ensure_release_cli_binaries
  NTPRO_V06_SKIP_BUILD=1 \
    NTPRO_V06_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v06_binance_testnet_dry_run.sh
}

run_v07_default_offline_gate() {
  echo "== verify_release: v0.7 default offline gate =="
  ensure_release_cli_binaries
  NTPRO_V07_SKIP_BUILD=1 \
    NTPRO_V07_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v07_default_offline_gate.sh
}

run_v07_manual_online_preflight() {
  echo "== verify_release: v0.7 manual online preflight =="
  ensure_release_cli_binaries
  NTPRO_V07_SKIP_BUILD=1 \
    NTPRO_V07_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v07_manual_online_gate.sh
}

run_v08_default_offline_gate() {
  echo "== verify_release: v0.8 default offline synthetic secret leak gate =="
  ensure_release_cli_binaries
  NTPRO_V08_SKIP_BUILD=1 \
    NTPRO_V08_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v08_default_offline_gate.sh
}

run_v08_authenticated_readonly_preflight() {
  echo "== verify_release: v0.8 authenticated read-only preflight =="
  ensure_release_cli_binaries
  NTPRO_V08_SKIP_BUILD=1 \
    NTPRO_V08_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v08_authenticated_readonly_gate.sh
}

run_v09_strategy_runtime_smoke() {
  echo "== verify_release: v0.9 strategy runtime smoke =="
  ensure_release_cli_binaries
  NTPRO_V09_SKIP_BUILD=1 \
    NTPRO_V09_NTPRO_NODE_BIN="$NTPRO_NODE_RELEASE_BIN" \
    scripts/ai/verify_v09_strategy_runtime_smoke.sh
}

run_v09_shadow_mode_no_order_gate() {
  echo "== verify_release: v0.9 shadow-mode no-order gate =="
  ensure_release_cli_binaries
  NTPRO_V09_SKIP_BUILD=1 \
    NTPRO_V09_NTPRO_NODE_BIN="$NTPRO_NODE_RELEASE_BIN" \
    scripts/ai/verify_v09_shadow_mode_no_order_gate.sh
}

run_v091_strategy_supervisor_dashboard_integration() {
  echo "== verify_release: v0.9.1 strategy supervisor/dashboard integration =="
  ensure_release_cli_binaries
  NTPRO_V091_SKIP_BUILD=1 \
    NTPRO_V091_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    NTPRO_V091_NTPRO_NODE_BIN="$NTPRO_NODE_RELEASE_BIN" \
    scripts/ai/verify_v091_strategy_supervisor_dashboard_integration.sh
}

run_stage() {
  local stage="$1"
  case "$stage" in
    all)
      run_full_checks
      run_release_build_product_surface
      run_rust_only_gates
      run_v02_supervisor_smoke
      run_v03_supervisor_control_smoke
      run_v03_dashboard_smoke
      run_v05_workflow_artifacts_smoke
      run_v06_binance_testnet_dry_run_smoke
      run_v07_default_offline_gate
      run_v07_manual_online_preflight
      run_v08_default_offline_gate
      run_v08_authenticated_readonly_preflight
      run_v09_strategy_runtime_smoke
      run_v09_shadow_mode_no_order_gate
      run_v091_strategy_supervisor_dashboard_integration
      ;;
    full)
      run_full_checks
      ;;
    release-build-product-surface)
      run_release_build_product_surface
      ;;
    rust-only-gates)
      run_rust_only_gates
      ;;
    v02-supervisor-smoke)
      run_v02_supervisor_smoke
      ;;
    v03-supervisor-control-smoke)
      run_v03_supervisor_control_smoke
      ;;
    v03-dashboard-smoke)
      run_v03_dashboard_smoke
      ;;
    v05-workflow-artifacts-smoke)
      run_v05_workflow_artifacts_smoke
      ;;
    v06-binance-testnet-dry-run-smoke)
      run_v06_binance_testnet_dry_run_smoke
      ;;
    v07-default-offline-gate)
      run_v07_default_offline_gate
      ;;
    v07-manual-online-preflight)
      run_v07_manual_online_preflight
      ;;
    v08-default-offline-gate)
      run_v08_default_offline_gate
      ;;
    v08-authenticated-readonly-preflight)
      run_v08_authenticated_readonly_preflight
      ;;
    v09-strategy-runtime-smoke)
      run_v09_strategy_runtime_smoke
      ;;
    v09-shadow-mode-no-order-gate)
      run_v09_shadow_mode_no_order_gate
      ;;
    v091-strategy-supervisor-dashboard-integration)
      run_v091_strategy_supervisor_dashboard_integration
      ;;
    *)
      echo "unknown verify_release stage: $stage" >&2
      echo "valid stages: all, full, release-build-product-surface, rust-only-gates, v02-supervisor-smoke, v03-supervisor-control-smoke, v03-dashboard-smoke, v05-workflow-artifacts-smoke, v06-binance-testnet-dry-run-smoke, v07-default-offline-gate, v07-manual-online-preflight, v08-default-offline-gate, v08-authenticated-readonly-preflight, v09-strategy-runtime-smoke, v09-shadow-mode-no-order-gate, v091-strategy-supervisor-dashboard-integration" >&2
      exit 2
      ;;
  esac
}

if (( $# == 0 )); then
  run_stage all
else
  for stage in "$@"; do
    run_stage "$stage"
  done
fi

echo "== verify_release complete =="
