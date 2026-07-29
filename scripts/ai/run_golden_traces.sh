#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

TRACE_GLOB="${TRACE_GLOB:-tests/golden/*.jsonl}"
REQUIRE_GOLDEN_REPLAY="${REQUIRE_GOLDEN_REPLAY:-0}"
RUN_RUST_GOLDEN_TRACE_HARNESS="${RUN_RUST_GOLDEN_TRACE_HARNESS:-1}"
RUN_RUST_MARKET_DATA_TRACE_REPLAY="${RUN_RUST_MARKET_DATA_TRACE_REPLAY:-1}"
RUN_RUST_CACHE_MSGBUS_TRACE_REPLAY="${RUN_RUST_CACHE_MSGBUS_TRACE_REPLAY:-1}"
RUN_RUST_BACKTEST_TRACE_REPLAY="${RUN_RUST_BACKTEST_TRACE_REPLAY:-1}"
RUN_RUST_BACKTEST_LIVE_PARITY_TRACE_REPLAY="${RUN_RUST_BACKTEST_LIVE_PARITY_TRACE_REPLAY:-1}"
RUN_RUST_LIVE_SANDBOX_TRACE_REPLAY="${RUN_RUST_LIVE_SANDBOX_TRACE_REPLAY:-1}"
RUN_RUST_ORDER_LIFECYCLE_TRACE_REPLAY="${RUN_RUST_ORDER_LIFECYCLE_TRACE_REPLAY:-1}"
RUN_RUST_RISK_REJECTION_TRACE_REPLAY="${RUN_RUST_RISK_REJECTION_TRACE_REPLAY:-1}"
RUN_RUST_ADAPTER_PAYLOAD_TRACE_REPLAY="${RUN_RUST_ADAPTER_PAYLOAD_TRACE_REPLAY:-1}"
RUN_RUST_LIVE_ALPHA_RECONCILIATION_TRACE_REPLAY="${RUN_RUST_LIVE_ALPHA_RECONCILIATION_TRACE_REPLAY:-1}"
RUN_RUST_LIVE_ALPHA_MUTATION_DRY_RUN_TRACE_REPLAY="${RUN_RUST_LIVE_ALPHA_MUTATION_DRY_RUN_TRACE_REPLAY:-1}"
RUN_RUST_ACTUAL_CANCEL_TRACE_REPLAY="${RUN_RUST_ACTUAL_CANCEL_TRACE_REPLAY:-1}"
RUN_RUST_PRODUCTION_ORDER_LIFECYCLE_TRACE_REPLAY="${RUN_RUST_PRODUCTION_ORDER_LIFECYCLE_TRACE_REPLAY:-1}"
RUN_RUST_READ_MODEL_TRACE_REPLAY="${RUN_RUST_READ_MODEL_TRACE_REPLAY:-1}"
RUN_RUST_SCHEMA_SMOKE_TRACE_REPLAY="${RUN_RUST_SCHEMA_SMOKE_TRACE_REPLAY:-1}"
REPLAY_COMMAND="${GOLDEN_TRACE_REPLAY_COMMAND:-}"
RELEASE_SCOPE_MANIFEST="${GOLDEN_TRACE_RELEASE_SCOPE_MANIFEST:-docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"

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
  scripts/ai/ntpro_governance.sh golden-trace "$trace" --mode validate-only
  if [ -n "$REPLAY_COMMAND" ]; then
    scripts/ai/ntpro_governance.sh golden-trace "$trace" --mode replay --replay-command "$REPLAY_COMMAND"
  fi
done

if [ "$REQUIRE_GOLDEN_REPLAY" = "1" ] && [ -z "$REPLAY_COMMAND" ]; then
  scripts/ai/ntpro_governance.sh golden-trace-release-scope \
    --manifest "$RELEASE_SCOPE_MANIFEST" \
    --trace-glob "$TRACE_GLOB"
  schema_only_count="$(
    jq '[.cases[] | select(.status == "schema_only_scoped")] | length' \
      "$RELEASE_SCOPE_MANIFEST"
  )"
  if [ "$schema_only_count" -ne 0 ]; then
    echo "release replay scope contains $schema_only_count schema-only cases; expected 0" >&2
    exit 1
  fi
fi

if [ "$RUN_RUST_GOLDEN_TRACE_HARNESS" = "1" ]; then
  cargo test -p nautilus-testkit --test golden_trace_schema
fi

if [ "$RUN_RUST_MARKET_DATA_TRACE_REPLAY" = "1" ]; then
  cargo test -p nautilus-model --test golden_trace_market_data
fi

if [ "$RUN_RUST_CACHE_MSGBUS_TRACE_REPLAY" = "1" ]; then
  cargo test -p nautilus-common --test golden_trace_cache_msgbus
fi

if [ "$RUN_RUST_BACKTEST_TRACE_REPLAY" = "1" ]; then
  cargo test -p nautilus-backtest --test golden_trace_backtest
fi

if [ "$RUN_RUST_BACKTEST_LIVE_PARITY_TRACE_REPLAY" = "1" ]; then
  cargo test -p nautilus-backtest --test backtest_live_semantic_parity
fi

if [ "$RUN_RUST_LIVE_SANDBOX_TRACE_REPLAY" = "1" ]; then
  cargo test -p nautilus-live --test golden_trace_live_sandbox
fi

if [ "$RUN_RUST_ORDER_LIFECYCLE_TRACE_REPLAY" = "1" ]; then
  cargo test -p nautilus-execution --test golden_trace_order_lifecycle
fi

if [ "$RUN_RUST_RISK_REJECTION_TRACE_REPLAY" = "1" ]; then
  cargo test -p nautilus-risk --test golden_trace_risk_rejection
fi

if [ "$RUN_RUST_ADAPTER_PAYLOAD_TRACE_REPLAY" = "1" ]; then
  cargo test -p nautilus-okx --test golden_trace_adapter_payload
fi

if [ "$RUN_RUST_LIVE_ALPHA_RECONCILIATION_TRACE_REPLAY" = "1" ]; then
  cargo test -p nautilus-cli --test golden_trace_live_alpha_reconciliation
fi

if [ "$RUN_RUST_LIVE_ALPHA_MUTATION_DRY_RUN_TRACE_REPLAY" = "1" ]; then
  cargo test -p nautilus-cli --test golden_trace_live_alpha_mutation_dry_run
fi

if [ "$RUN_RUST_ACTUAL_CANCEL_TRACE_REPLAY" = "1" ]; then
  cargo test -p nautilus-cli --test golden_trace_actual_cancel
fi

if [ "$RUN_RUST_PRODUCTION_ORDER_LIFECYCLE_TRACE_REPLAY" = "1" ]; then
  cargo test -p nautilus-cli --test golden_trace_production_order_lifecycle
fi

if [ "$RUN_RUST_READ_MODEL_TRACE_REPLAY" = "1" ]; then
  cargo test -p nautilus-cli --test golden_trace_read_model_projection
fi

if [ "$RUN_RUST_SCHEMA_SMOKE_TRACE_REPLAY" = "1" ]; then
  cargo test -p nautilus-cli --test golden_trace_schema_smoke_runtime
fi
