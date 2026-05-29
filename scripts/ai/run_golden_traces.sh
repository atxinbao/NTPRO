#!/usr/bin/env bash
set -euo pipefail

TRACE_GLOB="${TRACE_GLOB:-tests/golden/*.jsonl}"
REQUIRE_GOLDEN_REPLAY="${REQUIRE_GOLDEN_REPLAY:-0}"
RUN_RUST_GOLDEN_TRACE_HARNESS="${RUN_RUST_GOLDEN_TRACE_HARNESS:-1}"
RUN_RUST_CACHE_MSGBUS_TRACE_REPLAY="${RUN_RUST_CACHE_MSGBUS_TRACE_REPLAY:-1}"
RUN_RUST_BACKTEST_TRACE_REPLAY="${RUN_RUST_BACKTEST_TRACE_REPLAY:-1}"
RUN_RUST_LIVE_SANDBOX_TRACE_REPLAY="${RUN_RUST_LIVE_SANDBOX_TRACE_REPLAY:-1}"
RUN_RUST_ADAPTER_PAYLOAD_TRACE_REPLAY="${RUN_RUST_ADAPTER_PAYLOAD_TRACE_REPLAY:-1}"
REPLAY_COMMAND="${GOLDEN_TRACE_REPLAY_COMMAND:-}"
PYTHON_BIN="${PYTHON_BIN:-}"

if [ -z "$PYTHON_BIN" ]; then
  if command -v python3 >/dev/null 2>&1; then
    PYTHON_BIN=python3
  elif command -v python >/dev/null 2>&1; then
    PYTHON_BIN=python
  else
    echo "python3 or python is required to run golden trace validation" >&2
    exit 127
  fi
fi

shopt -s nullglob
traces=( $TRACE_GLOB )
if [ "${#traces[@]}" -eq 0 ]; then
  echo "no golden trace files found for TRACE_GLOB=$TRACE_GLOB" >&2
  if [ "$REQUIRE_GOLDEN_REPLAY" = "1" ]; then
    exit 1
  fi
  exit 0
fi

for trace in "${traces[@]}"; do
  "$PYTHON_BIN" scripts/ai/golden_trace_runner.py "$trace" --mode validate-only
  if [ -n "$REPLAY_COMMAND" ]; then
    "$PYTHON_BIN" scripts/ai/golden_trace_runner.py "$trace" --mode replay --replay-command "$REPLAY_COMMAND"
  elif [ "$REQUIRE_GOLDEN_REPLAY" = "1" ]; then
    echo "GOLDEN_TRACE_REPLAY_COMMAND is required for final release replay gate" >&2
    exit 1
  fi
done

if [ "$RUN_RUST_GOLDEN_TRACE_HARNESS" = "1" ]; then
  cargo test -p nautilus-testkit --test golden_trace_schema
fi

if [ "$RUN_RUST_CACHE_MSGBUS_TRACE_REPLAY" = "1" ]; then
  cargo test -p nautilus-common --test golden_trace_cache_msgbus
fi

if [ "$RUN_RUST_BACKTEST_TRACE_REPLAY" = "1" ]; then
  cargo test -p nautilus-backtest --test golden_trace_backtest
fi

if [ "$RUN_RUST_LIVE_SANDBOX_TRACE_REPLAY" = "1" ]; then
  cargo test -p nautilus-live --test golden_trace_live_sandbox
fi

if [ "$RUN_RUST_ADAPTER_PAYLOAD_TRACE_REPLAY" = "1" ]; then
  cargo test -p nautilus-okx --test golden_trace_adapter_payload
fi
