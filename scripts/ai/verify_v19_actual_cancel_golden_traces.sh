#!/usr/bin/env bash
set -euo pipefail

# V190-009: actual-cancel golden trace and fixture coverage.
# This gate is local/offline. It validates the V190-009 JSONL trace, the Rust
# actual-cancel trace harness, and the existing targeted actual-cancel fixture
# commands without real venue credentials or live broker access.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

export NTPRO_SOURCE_COMMIT="${NTPRO_SOURCE_COMMIT:-$(git rev-parse HEAD)}"
export NTPRO_SOURCE_RELEASE_TAG="${NTPRO_SOURCE_RELEASE_TAG:-unreleased-v19-local-gate}"

if [[ "${NTPRO_V19_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V19_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V19_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi

scripts/ai/ntpro_governance.sh golden-trace tests/golden/actual_cancel_schema.jsonl --mode validate-only

TRACE_GLOB=tests/golden/actual_cancel_schema.jsonl \
REQUIRE_GOLDEN_REPLAY=0 \
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
RUN_RUST_LIVE_ALPHA_MUTATION_DRY_RUN_TRACE_REPLAY=0 \
RUN_RUST_ACTUAL_CANCEL_TRACE_REPLAY=1 \
scripts/ai/run_golden_traces.sh

cargo test -p nautilus-cli --test golden_trace_actual_cancel
cargo test -p nautilus-cli actual_cancel --lib

NTPRO_V19_SKIP_BUILD=1 \
NTPRO_V19_NAUTILUS_BIN="$NAUTILUS_BIN" \
scripts/ai/verify_v19_post_cancel_readback_reconciliation.sh >/dev/null

NTPRO_V19_SKIP_BUILD=1 \
NTPRO_V19_NAUTILUS_BIN="$NAUTILUS_BIN" \
scripts/ai/verify_v19_actual_cancel_failure_evidence.sh >/dev/null

echo "verify_v19_actual_cancel_golden_traces PASS trace=tests/golden/actual_cancel_schema.jsonl scenarios=success,approval_missing,approval_reused,risk_mismatch,adapter_unsupported,cancel_rejected,timeout,unknown,already_cancelled,partial_fill network_dependency=false live_broker=false"
