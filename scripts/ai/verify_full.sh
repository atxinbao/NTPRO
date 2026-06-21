#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

FEATURES="${NAUTILUS_RUST_FEATURES:-arrow,ffi,high-precision,streaming,defi}"
CARGO_DOC_JOBS="${VERIFY_FULL_CARGO_DOC_JOBS:-1}"

WORKSPACE_TEST_PACKAGES=(
  nautilus-trader
  nautilus-architect-ax
  nautilus-common
  nautilus-core
  nautilus-indicators
  nautilus-model
  nautilus-serialization
  nautilus-live
  nautilus-data
  nautilus-persistence
  nautilus-persistence-macros
  nautilus-testkit
  nautilus-network
  nautilus-cryptography
  nautilus-portfolio
  nautilus-analysis
  nautilus-trading
  nautilus-execution
  nautilus-plugin
  nautilus-risk
  nautilus-system
  nautilus-sandbox
  nautilus-databento
  nautilus-betfair
  nautilus-backtest
  nautilus-binance
  nautilus-bitmex
  nautilus-blockchain
  nautilus-infrastructure
  nautilus-bybit
  nautilus-coinbase
  nautilus-deribit
  nautilus-dydx
  nautilus-hyperliquid
  nautilus-interactive-brokers
  nautilus-kraken
  nautilus-tardis
  nautilus-okx
  nautilus-polymarket
  nautilus-cli
  nautilus-event-store
)

LIVE_LIB_LOG_GLOBAL_TESTS=(
  node::tests::test_await_engines_connected_returns_shutdown_requested
  node::tests::test_await_engines_connected_returns_stop_requested
  node::tests::test_direct_build_rejects_event_store_config
  node::tests::test_run_event_store_replay_config_failure_aborts_startup
  node::tests::test_run_event_store_replay_consumes_runner_and_stops_before_connections
  node::tests::test_start_event_store_replay_config_failure_aborts_startup
  node::tests::test_start_event_store_replay_skips_live_connections
  node::tests::test_start_stop_request_aborts_startup_without_running
)

feature_args_for_crate() {
  local supported_csv="$1"
  local selected=()

  IFS=',' read -r -a feature_list <<< "$FEATURES"
  for feature in "${feature_list[@]}"; do
    feature="${feature//[[:space:]]/}"
    case ",$supported_csv," in
      *",$feature,"*) selected+=("$feature") ;;
    esac
  done

  if (( ${#selected[@]} > 0 )); then
    printf '%s\n' "--features"
    (IFS=','; printf '%s\n' "${selected[*]}")
  fi
}

list_cargo_tests() {
  cargo test "$@" -- --list |
    awk '
      /: test$/ {
        sub(/: test$/, "")
        print
        next
      }
      /^[[:alnum:]_][[:alnum:]_:]*:$/ {
        sub(/:$/, "")
        print
      }
    '
}

run_exact_cargo_tests_with_args() {
  local package="$1"
  shift
  local split_index="$1"
  shift
  local cargo_args=("${@:1:split_index}")
  local tests=("${@:$((split_index + 1))}")

  for test_name in "${tests[@]}"; do
    cargo test -p "$package" "${cargo_args[@]}" "$test_name" -- --exact
  done
}

package_is_selected() {
  local needle="$1"
  shift

  local selected
  for selected in "$@"; do
    if [[ "$selected" == "$needle" ]]; then
      return 0
    fi
  done
  return 1
}

set_rust_test_skip_args() {
  RUST_TEST_SKIP_ARGS=(
    --skip logging::logger::tests::serial_tests
    --skip logging::macros::tests::test_colored_logging_macros
    --skip logging::macros::tests::test_default_macro_captures_module_path
    --skip serial_tests
  )

  local test_name
  for test_name in "${LIVE_LIB_LOG_GLOBAL_TESTS[@]}"; do
    RUST_TEST_SKIP_ARGS+=(--skip "$test_name")
  done
}

run_fast_checks() {
  echo "== verify_full: fast checks =="
  scripts/ai/verify_fast.sh
}

run_clippy() {
  echo "== verify_full: clippy =="
  cargo clippy --workspace --lib --tests --features "$FEATURES" -- -D warnings
}

run_rust_workspace_tests() {
  echo "== verify_full: rust tests workspace =="
  set_rust_test_skip_args

  cargo test --workspace --lib --tests --features "$FEATURES" -- \
    "${RUST_TEST_SKIP_ARGS[@]}"
}

run_rust_workspace_partition_tests() {
  local partition="$1"
  shift
  local partition_features="$1"
  shift
  local selected_packages=("$@")
  local exclude_args=()
  local package

  for package in "${WORKSPACE_TEST_PACKAGES[@]}"; do
    if ! package_is_selected "$package" "${selected_packages[@]}"; then
      exclude_args+=(--exclude "$package")
    fi
  done

  set_rust_test_skip_args

  echo "== verify_full: rust tests workspace partition $partition =="
  printf 'packages=%s\n' "${selected_packages[*]}"
  printf 'features=%s\n' "$partition_features"
  cargo test --workspace "${exclude_args[@]}" --lib --tests --features "$partition_features" -- \
    "${RUST_TEST_SKIP_ARGS[@]}"
}

run_rust_workspace_core_tests() {
  run_rust_workspace_partition_tests core \
    arrow,ffi,high-precision,streaming,defi \
    nautilus-trader \
    nautilus-common \
    nautilus-core \
    nautilus-indicators \
    nautilus-model \
    nautilus-serialization \
    nautilus-testkit \
    nautilus-network \
    nautilus-cryptography \
    nautilus-system \
    nautilus-plugin
}

run_rust_workspace_runtime_tests() {
  run_rust_workspace_partition_tests runtime \
    ffi,high-precision,streaming,defi \
    nautilus-live \
    nautilus-data \
    nautilus-persistence \
    nautilus-persistence-macros \
    nautilus-portfolio \
    nautilus-analysis \
    nautilus-trading \
    nautilus-execution \
    nautilus-risk \
    nautilus-sandbox \
    nautilus-backtest \
    nautilus-infrastructure \
    nautilus-event-store \
    nautilus-cli
}

run_rust_workspace_adapters_a_tests() {
  run_rust_workspace_partition_tests adapters-a \
    arrow,high-precision \
    nautilus-architect-ax \
    nautilus-databento \
    nautilus-betfair \
    nautilus-binance \
    nautilus-bitmex \
    nautilus-blockchain \
    nautilus-bybit \
    nautilus-coinbase
}

run_rust_workspace_adapters_b_tests() {
  run_rust_workspace_partition_tests adapters-b \
    arrow,high-precision \
    nautilus-deribit \
    nautilus-dydx \
    nautilus-hyperliquid \
    nautilus-interactive-brokers \
    nautilus-kraken \
    nautilus-tardis \
    nautilus-okx \
    nautilus-polymarket
}

run_common_log_global_tests() {
  echo "== verify_full: nautilus-common log-global tests =="
  common_feature_args=()
  while IFS= read -r arg; do
    common_feature_args+=("$arg")
  done < <(
    feature_args_for_crate "capnp,defi,ffi,high-precision,indicators,live,simulation,tracing-bridge"
  )
  common_lib_args=("${common_feature_args[@]}" --lib)

  logging_tests=()
  while IFS= read -r test_name; do
    logging_tests+=("$test_name")
  done < <(
    list_cargo_tests -p nautilus-common "${common_lib_args[@]}" |
      grep -E '^logging::(logger::tests::serial_tests::|macros::tests::(test_colored_logging_macros|test_default_macro_captures_module_path)$)'
  )

  run_exact_cargo_tests_with_args nautilus-common "${#common_lib_args[@]}" "${common_lib_args[@]}" "${logging_tests[@]}"
}

run_live_log_global_tests() {
  echo "== verify_full: nautilus-live log-global tests =="
  live_feature_args=()
  while IFS= read -r arg; do
    live_feature_args+=("$arg")
  done < <(
    feature_args_for_crate "defi,examples,ffi,ignored,node,plugin,simulation,streaming"
  )
  live_lib_args=("${live_feature_args[@]}" --lib)

  run_exact_cargo_tests_with_args nautilus-live "${#live_lib_args[@]}" "${live_lib_args[@]}" "${LIVE_LIB_LOG_GLOBAL_TESTS[@]}"
}

run_live_node_serial_tests() {
  echo "== verify_full: nautilus-live node serial tests =="
  live_feature_args=()
  while IFS= read -r arg; do
    live_feature_args+=("$arg")
  done < <(
    feature_args_for_crate "defi,examples,ffi,ignored,node,plugin,simulation,streaming"
  )
  live_node_test_args=("${live_feature_args[@]}" --test node)

  live_node_serial_tests=()
  while IFS= read -r test_name; do
    live_node_serial_tests+=("$test_name")
  done < <(
    list_cargo_tests -p nautilus-live "${live_node_test_args[@]}" |
      grep -E '^serial_tests::'
  )

  run_exact_cargo_tests_with_args nautilus-live "${#live_node_test_args[@]}" "${live_node_test_args[@]}" "${live_node_serial_tests[@]}"
}

run_rust_tests() {
  echo "== verify_full: rust tests =="
  if cargo nextest --version >/dev/null 2>&1; then
    cargo nextest run --workspace --lib --tests --features "$FEATURES" --no-fail-fast
    return
  fi

  run_rust_workspace_tests
  run_common_log_global_tests
  run_live_log_global_tests
  run_live_node_serial_tests
}

run_golden_trace_validation() {
  echo "== verify_full: golden trace validation =="
  run_golden_trace_file_validation
  run_golden_trace_harness
  run_golden_market_data_trace
  run_golden_cache_msgbus_trace
  run_golden_backtest_trace
  run_golden_backtest_live_parity_trace
  run_golden_live_sandbox_trace
  run_golden_order_lifecycle_trace
  run_golden_risk_rejection_trace
  run_golden_adapter_payload_trace
  run_golden_live_alpha_reconciliation_trace
}

run_golden_trace_file_validation() {
  echo "== verify_full: golden trace file validation =="
  RUN_RUST_GOLDEN_TRACE_HARNESS=0 \
    RUN_RUST_MARKET_DATA_TRACE_REPLAY=0 \
    RUN_RUST_CACHE_MSGBUS_TRACE_REPLAY=0 \
    RUN_RUST_BACKTEST_TRACE_REPLAY=0 \
    RUN_RUST_BACKTEST_LIVE_PARITY_TRACE_REPLAY=0 \
    RUN_RUST_LIVE_SANDBOX_TRACE_REPLAY=0 \
    RUN_RUST_ORDER_LIFECYCLE_TRACE_REPLAY=0 \
    RUN_RUST_RISK_REJECTION_TRACE_REPLAY=0 \
    RUN_RUST_ADAPTER_PAYLOAD_TRACE_REPLAY=0 \
    RUN_RUST_LIVE_ALPHA_RECONCILIATION_TRACE_REPLAY=0 \
    scripts/ai/run_golden_traces.sh
}

run_golden_trace_harness() {
  echo "== verify_full: golden trace harness =="
  cargo test -p nautilus-testkit --test golden_trace_schema
}

run_golden_market_data_trace() {
  echo "== verify_full: golden trace market data =="
  cargo test -p nautilus-model --test golden_trace_market_data
}

run_golden_cache_msgbus_trace() {
  echo "== verify_full: golden trace cache msgbus =="
  cargo test -p nautilus-common --test golden_trace_cache_msgbus
}

run_golden_backtest_trace() {
  echo "== verify_full: golden trace backtest =="
  cargo test -p nautilus-backtest --test golden_trace_backtest
}

run_golden_backtest_live_parity_trace() {
  echo "== verify_full: golden trace backtest/live parity =="
  cargo test -p nautilus-backtest --test backtest_live_semantic_parity
}

run_golden_live_sandbox_trace() {
  echo "== verify_full: golden trace live sandbox =="
  cargo test -p nautilus-live --test golden_trace_live_sandbox
}

run_golden_order_lifecycle_trace() {
  echo "== verify_full: golden trace order lifecycle =="
  cargo test -p nautilus-execution --test golden_trace_order_lifecycle
}

run_golden_risk_rejection_trace() {
  echo "== verify_full: golden trace risk rejection =="
  cargo test -p nautilus-risk --test golden_trace_risk_rejection
}

run_golden_adapter_payload_trace() {
  echo "== verify_full: golden trace adapter payload =="
  cargo test -p nautilus-okx --test golden_trace_adapter_payload
}

run_golden_live_alpha_reconciliation_trace() {
  echo "== verify_full: golden trace live-alpha reconciliation =="
  cargo test -p nautilus-cli --test golden_trace_live_alpha_reconciliation
}

run_rust_docs() {
  echo "== verify_full: rust docs =="
  echo "== verify_full: cargo doc jobs=$CARGO_DOC_JOBS =="
  cargo doc --workspace --features "$FEATURES" --no-deps --jobs "$CARGO_DOC_JOBS"
}

run_stage() {
  local stage="$1"
  case "$stage" in
    fast)
      run_fast_checks
      ;;
    clippy)
      run_clippy
      ;;
    rust-tests)
      run_rust_tests
      ;;
    rust-tests-workspace)
      run_rust_workspace_tests
      ;;
    rust-tests-workspace-core)
      run_rust_workspace_core_tests
      ;;
    rust-tests-workspace-runtime)
      run_rust_workspace_runtime_tests
      ;;
    rust-tests-workspace-adapters-a)
      run_rust_workspace_adapters_a_tests
      ;;
    rust-tests-workspace-adapters-b)
      run_rust_workspace_adapters_b_tests
      ;;
    rust-tests-common-log-global)
      run_common_log_global_tests
      ;;
    rust-tests-live-log-global)
      run_live_log_global_tests
      ;;
    rust-tests-live-node-serial)
      run_live_node_serial_tests
      ;;
    golden-traces)
      run_golden_trace_validation
      ;;
    golden-traces-files)
      run_golden_trace_file_validation
      ;;
    golden-traces-harness)
      run_golden_trace_harness
      ;;
    golden-traces-market-data)
      run_golden_market_data_trace
      ;;
    golden-traces-cache-msgbus)
      run_golden_cache_msgbus_trace
      ;;
    golden-traces-backtest)
      run_golden_backtest_trace
      ;;
    golden-traces-backtest-live-parity)
      run_golden_backtest_live_parity_trace
      ;;
    golden-traces-live-sandbox)
      run_golden_live_sandbox_trace
      ;;
    golden-traces-order-lifecycle)
      run_golden_order_lifecycle_trace
      ;;
    golden-traces-risk-rejection)
      run_golden_risk_rejection_trace
      ;;
    golden-traces-adapter-payload)
      run_golden_adapter_payload_trace
      ;;
    golden-traces-live-alpha-reconciliation)
      run_golden_live_alpha_reconciliation_trace
      ;;
    rust-docs)
      run_rust_docs
      ;;
    all)
      run_fast_checks
      run_clippy
      run_rust_tests
      run_golden_trace_validation
      run_rust_docs
      ;;
    *)
      echo "unknown verify_full stage: $stage" >&2
      echo "valid stages: all, fast, clippy, rust-tests, rust-tests-workspace, rust-tests-workspace-core, rust-tests-workspace-runtime, rust-tests-workspace-adapters-a, rust-tests-workspace-adapters-b, rust-tests-common-log-global, rust-tests-live-log-global, rust-tests-live-node-serial, golden-traces, golden-traces-files, golden-traces-harness, golden-traces-market-data, golden-traces-cache-msgbus, golden-traces-backtest, golden-traces-backtest-live-parity, golden-traces-live-sandbox, golden-traces-order-lifecycle, golden-traces-risk-rejection, golden-traces-adapter-payload, golden-traces-live-alpha-reconciliation, rust-docs" >&2
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

echo "== verify_full complete =="
