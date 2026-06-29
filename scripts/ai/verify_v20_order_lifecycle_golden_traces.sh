#!/usr/bin/env bash
set -euo pipefail

# V200-011: production order lifecycle golden trace and fixture coverage.
# This gate is local/offline. It validates the V200-011 JSONL trace, the Rust
# production-order lifecycle trace harness, and the targeted V200 response,
# readback, no-retry, and Dashboard audit fixture tests without venue
# credentials, live broker access, or production network replay.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

python3 scripts/ai/golden_trace_runner.py \
  tests/golden/production_order_lifecycle_schema.jsonl \
  --mode validate-only

TRACE_GLOB=tests/golden/production_order_lifecycle_schema.jsonl \
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
RUN_RUST_ACTUAL_CANCEL_TRACE_REPLAY=0 \
RUN_RUST_PRODUCTION_ORDER_LIFECYCLE_TRACE_REPLAY=1 \
scripts/ai/run_golden_traces.sh

cargo test -p nautilus-cli --test golden_trace_production_order_lifecycle
cargo test -p nautilus-risk --test v20_submit_response_redaction -- --nocapture
cargo test -p nautilus-risk --test v20_submit_readback_reconciliation -- --nocapture
cargo test -p nautilus-risk --test v20_failure_no_retry -- --nocapture
cargo test -p nautilus-cli production_order_lifecycle_audit --lib -- --nocapture

echo "verify_v20_order_lifecycle_golden_traces PASS trace=tests/golden/production_order_lifecycle_schema.jsonl scenarios=pre_submit_blocked_missing_approval,accepted_readback_matched_audit_closed,venue_rejected_failure_no_retry,unknown_response_failure_no_retry,readback_mismatch_failure_no_retry,readback_missing_failure_no_retry no_retry=true dashboard_readonly=true credential_plaintext=false"
