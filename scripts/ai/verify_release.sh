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

run_v10_offline_release_gates() {
  echo "== verify_release: v0.10 offline release gates =="
  ensure_release_cli_binaries
  NTPRO_V10_SKIP_BUILD=1 \
    NTPRO_V10_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v10_offline_release_gates.sh
}

run_v10_manual_order_proof_preflight() {
  echo "== verify_release: v0.10 manual order proof preflight =="
  ensure_release_cli_binaries
  NTPRO_V10_SKIP_BUILD=1 \
    NTPRO_V10_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v10_manual_order_proof_gate.sh
}

run_v11_offline_release_gates() {
  echo "== verify_release: v0.11 offline release gates =="
  ensure_release_cli_binaries
  NTPRO_V11_SKIP_BUILD=1 \
    NTPRO_V11_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v11_offline_release_gates.sh
}

run_v12_offline_release_gates() {
  echo "== verify_release: v0.12 offline release gates =="
  ensure_release_cli_binaries
  NTPRO_V12_SKIP_BUILD=1 \
    NTPRO_V12_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v12_offline_release_gates.sh
}

run_v12_manual_online_preflight() {
  echo "== verify_release: v0.12 manual online preflight =="
  ensure_release_cli_binaries
  NTPRO_V12_SKIP_BUILD=1 \
    NTPRO_V12_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v12_manual_online_preflight.sh
}

run_v13_no_production_mutation_gate() {
  echo "== verify_release: v0.13 no-production-mutation gate =="
  ensure_release_cli_binaries
  NTPRO_V13_SKIP_BUILD=1 \
    NTPRO_V13_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    NTPRO_V12_SKIP_BUILD=1 \
    NTPRO_V12_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v13_no_production_mutation_gate.sh
}

run_v14_release_gates() {
  echo "== verify_release: v0.14 release gates =="
  ensure_release_cli_binaries
  NTPRO_V14_SKIP_BUILD=1 \
    NTPRO_V14_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    NTPRO_V14_NTPRO_NODE_BIN="$NTPRO_NODE_RELEASE_BIN" \
    scripts/ai/verify_v14_release_gates.sh
}

run_v15_release_gates() {
  echo "== verify_release: v0.15 release gates =="
  ensure_release_cli_binaries
  NTPRO_V15_SKIP_BUILD=1 \
    NTPRO_V15_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v15_release_gates.sh
}

run_v151_release_gates() {
  echo "== verify_release: v0.15.1 release gates =="
  ensure_release_cli_binaries
  NTPRO_V151_SKIP_BUILD=1 \
    NTPRO_V151_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    NTPRO_V151_NTPRO_NODE_BIN="$NTPRO_NODE_RELEASE_BIN" \
    scripts/ai/verify_v151_release_gates.sh
}

run_v16_release_gates() {
  echo "== verify_release: v0.16 release gates =="
  ensure_release_cli_binaries
  NTPRO_V16_SKIP_BUILD=1 \
    NTPRO_V16_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    NTPRO_V16_NTPRO_NODE_BIN="$NTPRO_NODE_RELEASE_BIN" \
    scripts/ai/verify_v16_release_gates.sh
}

run_v17_release_gates() {
  echo "== verify_release: v0.17 release gates =="
  ensure_release_cli_binaries
  NTPRO_V17_SKIP_BUILD=1 \
    NTPRO_V17_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    NTPRO_V16_SKIP_BUILD=1 \
    NTPRO_V16_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v17_release_gates.sh
}

run_v18_release_gates() {
  echo "== verify_release: v0.18 release gates =="
  ensure_release_cli_binaries
  NTPRO_V18_SKIP_BUILD=1 \
    NTPRO_V18_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    NTPRO_V17_SKIP_BUILD=1 \
    NTPRO_V17_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    NTPRO_V16_SKIP_BUILD=1 \
    NTPRO_V16_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v18_release_gates.sh
}

run_v19_release_gates() {
  echo "== verify_release: v0.19 release gates =="
  ensure_release_cli_binaries
  NTPRO_V19_SKIP_BUILD=1 \
    NTPRO_V19_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v19_release_gates.sh
}

run_v20_release_gates() {
  echo "== verify_release: v0.20 release gates =="
  ensure_release_cli_binaries
  NTPRO_V20_SKIP_BUILD=1 \
    NTPRO_V20_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v20_release_gates.sh
}

run_v20_patch_release_gates() {
  echo "== verify_release: v0.20.1 patch release gates =="
  scripts/ai/verify_v20_patch_release_gates.sh
}

run_v21_read_model_contract() {
  echo "== verify_release: v0.21 read model contract =="
  scripts/ai/verify_v21_read_model_contract.sh
}

run_v21_account_snapshot_read_model() {
  echo "== verify_release: v0.21 account snapshot read model =="
  scripts/ai/verify_v21_account_snapshot_read_model.sh
}

run_v21_position_read_model() {
  echo "== verify_release: v0.21 position read model =="
  scripts/ai/verify_v21_position_read_model.sh
}

run_v21_order_lifecycle_read_model() {
  echo "== verify_release: v0.21 order lifecycle read model =="
  scripts/ai/verify_v21_order_lifecycle_read_model.sh
}

run_v21_fill_execution_read_model() {
  echo "== verify_release: v0.21 fill/execution read model =="
  scripts/ai/verify_v21_fill_execution_read_model.sh
}

run_v21_risk_state_projection() {
  echo "== verify_release: v0.21 risk state projection =="
  scripts/ai/verify_v21_risk_state_projection.sh
}

run_v21_trader_terminal_readonly_dashboard() {
  echo "== verify_release: v0.21 Trader Terminal read-only dashboard foundation =="
  scripts/ai/verify_v21_trader_terminal_readonly_dashboard.sh
}

run_v21_release_gates() {
  echo "== verify_release: v0.21 release gates =="
  scripts/ai/verify_v21_release_gates.sh
}

run_v211_health_status_semantics() {
  echo "== verify_release: v0.21.1 health status semantics =="
  scripts/ai/verify_v211_health_status_semantics.sh
}

run_v211_read_model_projection_replay() {
  echo "== verify_release: v0.21.1 read-model projection replay =="
  scripts/ai/verify_v21_1_read_model_projection_replay.sh
}

run_v211_read_model_schema_boundary() {
  echo "== verify_release: v0.21.1 read-model JSON schema boundary =="
  scripts/ai/verify_v21_1_read_model_schema_boundary.sh
}

run_v211_trader_terminal_read_model_bridge() {
  echo "== verify_release: v0.21.1 Trader Terminal read-model runtime bridge =="
  scripts/ai/verify_v21_1_trader_terminal_read_model_bridge.sh
}

run_v211_release_gates() {
  echo "== verify_release: v0.21.1 release gates =="
  scripts/ai/verify_v21_1_release_gates.sh
}

run_v211_strict_provenance() {
  echo "== verify_release: v0.21.1 strict release provenance =="
  scripts/ai/verify_v21_1_strict_provenance.sh
}

run_v22_runtime_boundary_tests() {
  echo "== verify_release: v0.22 runtime degradation and boundary tests =="
  scripts/ai/verify_v22_runtime_boundary_tests.sh
}

run_v22_release_gates() {
  echo "== verify_release: v0.22 release gates =="
  scripts/ai/verify_v22_release_gates.sh
}

run_v22_strict_provenance() {
  echo "== verify_release: v0.22 strict release provenance =="
  scripts/ai/verify_release_strict.sh v22
}

run_v221_release_gates() {
  echo "== verify_release: v0.22.1 release gates =="
  scripts/ai/verify_v22_1_release_gates.sh
}

run_v221_strict_provenance() {
  echo "== verify_release: v0.22.1 strict release provenance =="
  scripts/ai/verify_v22_1_strict_provenance.sh
}

run_v23_release_gates() {
  echo "== verify_release: v0.23 release gates =="
  scripts/ai/verify_v23_release_gates.sh
}

run_v23_strict_provenance() {
  echo "== verify_release: v0.23 strict release provenance =="
  scripts/ai/verify_v23_strict_provenance.sh
}

run_v231_release_closeout_evidence() {
  echo "== verify_release: v0.23.1 release closeout evidence =="
  scripts/ai/verify_v23_1_release_closeout_evidence.sh
}

run_v231_stale_provenance_cleanup() {
  echo "== verify_release: v0.23.1 stale provenance cleanup =="
  scripts/ai/verify_v23_1_stale_provenance_cleanup.sh
}

run_v231_gate_phase_split() {
  echo "== verify_release: v0.23.1 gate phase split =="
  scripts/ai/verify_v23_1_gate_phase_split.sh
}

run_v231_evidence_replay_only_boundary() {
  echo "== verify_release: v0.23.1 evidence replay only boundary =="
  scripts/ai/verify_v23_1_evidence_replay_only_boundary.sh
}

run_v231_publication_evidence_audit_path() {
  echo "== verify_release: v0.23.1 publication evidence audit path =="
  scripts/ai/verify_v23_1_publication_evidence_audit_path.sh
}

run_v171_release_hardening() {
  echo "== verify_release: v0.17.1 release hardening =="
  scripts/ai/verify_v171_release_hardening.sh
}

run_v18_strict_provenance() {
  echo "== verify_release: v0.18 strict binary provenance =="
  scripts/ai/verify_release_strict.sh v18
}

run_v19_strict_provenance() {
  echo "== verify_release: v0.19 strict release provenance =="
  scripts/ai/verify_release_strict.sh v19
}

run_v20_strict_provenance() {
  echo "== verify_release: v0.20 strict release provenance =="
  scripts/ai/verify_release_strict.sh v20
}

run_v21_strict_provenance() {
  echo "== verify_release: v0.21 strict release provenance =="
  scripts/ai/verify_release_strict.sh v21
}

run_release_surface_current_guard() {
  echo "== verify_release: release surface current guard =="
  scripts/ai/check_release_surface_current.sh
}

run_release_publication_guard() {
  echo "== verify_release: release publication guard =="
  scripts/ai/check_github_release_published.sh
}

run_release_publish_after_gate() {
  echo "== verify_release: release publish after gate =="
  scripts/ai/verify_release_publish_after_gate.sh
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
      run_v10_offline_release_gates
      run_v10_manual_order_proof_preflight
      run_v11_offline_release_gates
      run_v12_offline_release_gates
      run_v12_manual_online_preflight
      run_v13_no_production_mutation_gate
      run_v14_release_gates
      run_v15_release_gates
      run_v151_release_gates
      run_v16_release_gates
      run_v17_release_gates
      run_v18_release_gates
      run_v171_release_hardening
      run_v19_release_gates
      run_v19_strict_provenance
      run_v20_release_gates
      run_v20_strict_provenance
      run_v20_patch_release_gates
      run_v21_read_model_contract
      run_v21_account_snapshot_read_model
      run_v21_position_read_model
      run_v21_order_lifecycle_read_model
      run_v21_fill_execution_read_model
      run_v21_risk_state_projection
      run_v21_trader_terminal_readonly_dashboard
      run_v21_release_gates
      run_v21_strict_provenance
      run_v211_health_status_semantics
      run_v211_read_model_projection_replay
      run_v211_read_model_schema_boundary
      run_v211_trader_terminal_read_model_bridge
      run_v211_release_gates
      run_v211_strict_provenance
      run_v22_runtime_boundary_tests
      run_v22_release_gates
      run_v22_strict_provenance
      run_v221_release_gates
      run_v221_strict_provenance
      run_v23_release_gates
      run_v23_strict_provenance
      run_v231_release_closeout_evidence
      run_v231_stale_provenance_cleanup
      run_v231_gate_phase_split
      run_v231_evidence_replay_only_boundary
      run_v231_publication_evidence_audit_path
      run_release_surface_current_guard
      run_release_publication_guard
      run_release_publish_after_gate
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
    v10-offline-release-gates)
      run_v10_offline_release_gates
      ;;
    v10-manual-order-proof-preflight)
      run_v10_manual_order_proof_preflight
      ;;
    v11-offline-release-gates)
      run_v11_offline_release_gates
      ;;
    v12-offline-release-gates)
      run_v12_offline_release_gates
      ;;
    v12-manual-online-preflight)
      run_v12_manual_online_preflight
      ;;
    v13-no-production-mutation-gate)
      run_v13_no_production_mutation_gate
      ;;
    v14-release-gates)
      run_v14_release_gates
      ;;
    v15-release-gates)
      run_v15_release_gates
      ;;
    v151-release-gates)
      run_v151_release_gates
      ;;
    v16-release-gates)
      run_v16_release_gates
      ;;
    v17-release-gates)
      run_v17_release_gates
      ;;
    v18-release-gates)
      run_v18_release_gates
      ;;
    v19-release-gates)
      run_v19_release_gates
      ;;
    v20-release-gates)
      run_v20_release_gates
      ;;
    v20.1-release-gates)
      run_v20_patch_release_gates
      ;;
    v21-read-model-contract)
      run_v21_read_model_contract
      ;;
    v21-account-snapshot-read-model)
      run_v21_account_snapshot_read_model
      ;;
    v21-position-read-model)
      run_v21_position_read_model
      ;;
    v21-order-lifecycle-read-model)
      run_v21_order_lifecycle_read_model
      ;;
    v21-fill-execution-read-model)
      run_v21_fill_execution_read_model
      ;;
    v21-risk-state-projection)
      run_v21_risk_state_projection
      ;;
    v21-trader-terminal-readonly-dashboard)
      run_v21_trader_terminal_readonly_dashboard
      ;;
    v21-release-gates)
      run_v21_release_gates
      ;;
    v21-strict-provenance)
      run_v21_strict_provenance
      ;;
    v21.1-health-status-semantics)
      run_v211_health_status_semantics
      ;;
    v21.1-read-model-projection-replay)
      run_v211_read_model_projection_replay
      ;;
    v21.1-read-model-schema-boundary)
      run_v211_read_model_schema_boundary
      ;;
    v21.1-trader-terminal-read-model-bridge)
      run_v211_trader_terminal_read_model_bridge
      ;;
    v21.1-release-gates)
      run_v211_release_gates
      ;;
    v21.1-strict-provenance)
      run_v211_strict_provenance
      ;;
    v22-runtime-boundary-tests)
      run_v22_runtime_boundary_tests
      ;;
    v22-release-gates)
      run_v22_release_gates
      ;;
    v22-strict-provenance)
      run_v22_strict_provenance
      ;;
    v22.1-release-gates)
      run_v221_release_gates
      ;;
    v22.1-strict-provenance)
      run_v221_strict_provenance
      ;;
    v23-release-gates)
      run_v23_release_gates
      ;;
    v23-strict-provenance)
      run_v23_strict_provenance
      ;;
    v23.1-release-closeout-evidence)
      run_v231_release_closeout_evidence
      ;;
    v23.1-stale-provenance-cleanup)
      run_v231_stale_provenance_cleanup
      ;;
    v23.1-gate-phase-split)
      run_v231_gate_phase_split
      ;;
    v23.1-evidence-replay-only-boundary)
      run_v231_evidence_replay_only_boundary
      ;;
    v23.1-publication-evidence-audit-path)
      run_v231_publication_evidence_audit_path
      ;;
    v171-release-hardening)
      run_v171_release_hardening
      ;;
    v18-strict-provenance)
      run_v18_strict_provenance
      ;;
    v19-strict-provenance)
      run_v19_strict_provenance
      ;;
    v20-strict-provenance)
      run_v20_strict_provenance
      ;;
    release-surface-current-guard)
      run_release_surface_current_guard
      ;;
    release-publication-guard)
      run_release_publication_guard
      ;;
    release-publish-after-gate)
      run_release_publish_after_gate
      ;;
    *)
      echo "unknown verify_release stage: $stage" >&2
      echo "valid stages: all, full, release-build-product-surface, rust-only-gates, v02-supervisor-smoke, v03-supervisor-control-smoke, v03-dashboard-smoke, v05-workflow-artifacts-smoke, v06-binance-testnet-dry-run-smoke, v07-default-offline-gate, v07-manual-online-preflight, v08-default-offline-gate, v08-authenticated-readonly-preflight, v09-strategy-runtime-smoke, v09-shadow-mode-no-order-gate, v091-strategy-supervisor-dashboard-integration, v10-offline-release-gates, v10-manual-order-proof-preflight, v11-offline-release-gates, v12-offline-release-gates, v12-manual-online-preflight, v13-no-production-mutation-gate, v14-release-gates, v15-release-gates, v151-release-gates, v16-release-gates, v17-release-gates, v18-release-gates, v19-release-gates, v20-release-gates, v20.1-release-gates, v21-read-model-contract, v21-account-snapshot-read-model, v21-position-read-model, v21-order-lifecycle-read-model, v21-fill-execution-read-model, v21-risk-state-projection, v21-trader-terminal-readonly-dashboard, v21-release-gates, v21-strict-provenance, v21.1-health-status-semantics, v21.1-read-model-projection-replay, v21.1-read-model-schema-boundary, v21.1-trader-terminal-read-model-bridge, v21.1-release-gates, v21.1-strict-provenance, v22-runtime-boundary-tests, v22-release-gates, v22-strict-provenance, v22.1-release-gates, v22.1-strict-provenance, v23-release-gates, v23-strict-provenance, v23.1-release-closeout-evidence, v23.1-stale-provenance-cleanup, v23.1-gate-phase-split, v23.1-evidence-replay-only-boundary, v23.1-publication-evidence-audit-path, v171-release-hardening, v18-strict-provenance, v19-strict-provenance, v20-strict-provenance, release-surface-current-guard, release-publication-guard, release-publish-after-gate" >&2
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
