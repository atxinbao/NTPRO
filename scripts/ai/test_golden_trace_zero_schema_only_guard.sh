#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/ntpro-par010.XXXXXX")"
manifest="$tmp_dir/release-replay-scope.json"
output="$tmp_dir/output.log"
cleanup() {
  rm -f "$manifest" "$output"
  rmdir "$tmp_dir"
}
trap cleanup EXIT

jq '
  (.cases[] | select(.case_id == "market_data.schema_smoke.001")) |= (
    .status = "schema_only_scoped"
    | .scope_owner = "PAR-010-negative-selftest"
    | .reason = "negative self-test must be rejected by the release runner"
    | .follow_up = "restore executable Rust replay"
    | .release_decision = "schema_only_scope_recorded"
    | del(.evidence_id, .harness, .rust_entrypoint)
  )
' docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json >"$manifest"

if env \
  REQUIRE_GOLDEN_REPLAY=1 \
  GOLDEN_TRACE_RELEASE_SCOPE_MANIFEST="$manifest" \
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
  RUN_RUST_PRODUCTION_ORDER_LIFECYCLE_TRACE_REPLAY=0 \
  RUN_RUST_READ_MODEL_TRACE_REPLAY=0 \
  RUN_RUST_SCHEMA_SMOKE_TRACE_REPLAY=0 \
  scripts/ai/run_golden_traces.sh >"$output" 2>&1
then
  echo "zero-schema-only guard self-test unexpectedly passed" >&2
  exit 1
fi

if ! rg -q 'release replay scope contains 1 schema-only cases; expected 0' "$output"; then
  cat "$output" >&2
  echo "zero-schema-only guard self-test failed for an unexpected reason" >&2
  exit 1
fi

echo "golden_trace_zero_schema_only_guard_selftest=pass cases=1"
