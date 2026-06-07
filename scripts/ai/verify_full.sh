#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

FEATURES="${NAUTILUS_RUST_FEATURES:-arrow,ffi,high-precision,streaming,defi}"
CARGO_DOC_JOBS="${VERIFY_FULL_CARGO_DOC_JOBS:-1}"

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

echo "== verify_full: fast checks =="
scripts/ai/verify_fast.sh

echo "== verify_full: clippy =="
cargo clippy --workspace --lib --tests --features "$FEATURES" -- -D warnings

echo "== verify_full: rust tests =="
if cargo nextest --version >/dev/null 2>&1; then
  cargo nextest run --workspace --lib --tests --features "$FEATURES" --no-fail-fast
else
  live_lib_log_global_tests=(
    node::tests::test_await_engines_connected_returns_shutdown_requested
    node::tests::test_await_engines_connected_returns_stop_requested
    node::tests::test_direct_build_rejects_event_store_config
    node::tests::test_run_event_store_replay_config_failure_aborts_startup
    node::tests::test_run_event_store_replay_consumes_runner_and_stops_before_connections
    node::tests::test_start_event_store_replay_config_failure_aborts_startup
    node::tests::test_start_event_store_replay_skips_live_connections
    node::tests::test_start_stop_request_aborts_startup_without_running
  )

  rust_test_skip_args=(
    --skip logging::logger::tests::serial_tests
    --skip logging::macros::tests::test_colored_logging_macros
    --skip logging::macros::tests::test_default_macro_captures_module_path
    --skip serial_tests
  )

  for test_name in "${live_lib_log_global_tests[@]}"; do
    rust_test_skip_args+=(--skip "$test_name")
  done

  cargo test --workspace --lib --tests --features "$FEATURES" -- \
    "${rust_test_skip_args[@]}"

  common_feature_args=()
  while IFS= read -r arg; do
    common_feature_args+=("$arg")
  done < <(
    feature_args_for_crate "capnp,defi,ffi,high-precision,indicators,live,simulation,tracing-bridge"
  )

  live_feature_args=()
  while IFS= read -r arg; do
    live_feature_args+=("$arg")
  done < <(
    feature_args_for_crate "defi,examples,ffi,ignored,node,plugin,simulation,streaming"
  )
  common_lib_args=("${common_feature_args[@]}" --lib)
  live_lib_args=("${live_feature_args[@]}" --lib)
  live_node_test_args=("${live_feature_args[@]}" --test node)

  echo "== verify_full: nautilus-common log-global tests =="
  logging_tests=()
  while IFS= read -r test_name; do
    logging_tests+=("$test_name")
  done < <(
    list_cargo_tests -p nautilus-common "${common_lib_args[@]}" |
      grep -E '^logging::(logger::tests::serial_tests::|macros::tests::(test_colored_logging_macros|test_default_macro_captures_module_path)$)'
  )

  run_exact_cargo_tests_with_args nautilus-common "${#common_lib_args[@]}" "${common_lib_args[@]}" "${logging_tests[@]}"

  echo "== verify_full: nautilus-live log-global tests =="
  run_exact_cargo_tests_with_args nautilus-live "${#live_lib_args[@]}" "${live_lib_args[@]}" "${live_lib_log_global_tests[@]}"

  live_node_serial_tests=()
  while IFS= read -r test_name; do
    live_node_serial_tests+=("$test_name")
  done < <(
    list_cargo_tests -p nautilus-live "${live_node_test_args[@]}" |
      grep -E '^serial_tests::'
  )

  run_exact_cargo_tests_with_args nautilus-live "${#live_node_test_args[@]}" "${live_node_test_args[@]}" "${live_node_serial_tests[@]}"
fi

echo "== verify_full: golden trace validation =="
scripts/ai/run_golden_traces.sh

echo "== verify_full: rust docs =="
echo "== verify_full: cargo doc jobs=$CARGO_DOC_JOBS =="
cargo doc --workspace --features "$FEATURES" --no-deps --jobs "$CARGO_DOC_JOBS"

echo "== verify_full complete =="
