#!/usr/bin/env bash
set -euo pipefail

TRACE_GLOB="${TRACE_GLOB:-tests/golden/*.jsonl}"
REQUIRE_GOLDEN_REPLAY="${REQUIRE_GOLDEN_REPLAY:-0}"
REPLAY_COMMAND="${GOLDEN_TRACE_REPLAY_COMMAND:-}"

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
  python scripts/ai/golden_trace_runner.py "$trace" --mode validate-only
  if [ -n "$REPLAY_COMMAND" ]; then
    python scripts/ai/golden_trace_runner.py "$trace" --mode replay --replay-command "$REPLAY_COMMAND"
  elif [ "$REQUIRE_GOLDEN_REPLAY" = "1" ]; then
    echo "GOLDEN_TRACE_REPLAY_COMMAND is required for final release replay gate" >&2
    exit 1
  fi
done
