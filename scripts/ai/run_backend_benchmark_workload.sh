#!/usr/bin/env bash
set -euo pipefail

if (( $# != 5 )); then
  echo "usage: $0 <workload-id> <repository> <commit-sha> <output-json> <target-dir>" >&2
  exit 2
fi

WORKLOAD_ID="$1"
REPOSITORY="$2"
COMMIT_SHA="$3"
OUTPUT_JSON="$4"
TARGET_DIR="$5"
RUST_TOOLCHAIN="${NTPRO_RUST_TOOLCHAIN:-1.95.0}"
if command -v rustup >/dev/null 2>&1; then
  CARGO_BIN="${CARGO:-$(rustup which --toolchain "$RUST_TOOLCHAIN" cargo)}"
  RUSTC_BIN="${RUSTC:-$(rustup which --toolchain "$RUST_TOOLCHAIN" rustc)}"
else
  CARGO_BIN="${CARGO:-$(command -v cargo)}"
  RUSTC_BIN="${RUSTC:-$(command -v rustc)}"
fi
mkdir -p "$(dirname "$OUTPUT_JSON")"
OUTPUT_DIR="$(cd "$(dirname "$OUTPUT_JSON")" && pwd)"

[[ -d "$REPOSITORY" ]] || { echo "benchmark repository missing: $REPOSITORY" >&2; exit 1; }
[[ "$COMMIT_SHA" =~ ^[0-9a-f]{40}$ ]] || { echo "benchmark commit must be a full SHA" >&2; exit 1; }
ACTUAL_SHA="$(git -C "$REPOSITORY" rev-parse HEAD)"
[[ "$ACTUAL_SHA" == "$COMMIT_SHA" ]] || {
  echo "benchmark checkout mismatch: expected $COMMIT_SHA, found $ACTUAL_SHA" >&2
  exit 1
}

case "$WORKLOAD_ID" in
  core_stack_str)
    PACKAGE="nautilus-core"
    BENCH="stack_str"
    FILTER='^StackStr::new \(short\)$'
    ESTIMATE_PATH='StackStr__new (short)/new/estimates.json'
    ;;
  model_price)
    PACKAGE="nautilus-model"
    BENCH="price_criterion"
    FILTER='^Price::new$'
    ESTIMATE_PATH='Price__new/new/estimates.json'
    ;;
  data_engine_ingest)
    PACKAGE="nautilus-data"
    BENCH="engine"
    FILTER='^DataEngine ingest/process_data_trade$'
    ESTIMATE_PATH='DataEngine ingest/process_data_trade/new/estimates.json'
    ;;
  execution_matching_core)
    PACKAGE="nautilus-execution"
    BENCH="matching_core"
    FILTER='^matching_core/get_order/100$'
    ESTIMATE_PATH='matching_core_get_order/100/new/estimates.json'
    ;;
  live_runner_dispatch)
    PACKAGE="nautilus-live"
    BENCH="runner"
    FILTER='^AsyncRunner dispatch/drain_data_events/100$'
    ESTIMATE_PATH='AsyncRunner dispatch/drain_data_events/100/new/estimates.json'
    ;;
  network_rate_limiter)
    PACKAGE="nautilus-network"
    BENCH="ratelimiter"
    FILTER='^ratelimiter/check_key_uncontended/single_key$'
    ESTIMATE_PATH='ratelimiter_check_key_uncontended/single_key/new/estimates.json'
    ;;
  *)
    echo "unknown backend benchmark workload: $WORKLOAD_ID" >&2
    exit 2
    ;;
esac

COMMAND="cargo bench --locked --profile bench-lto -p $PACKAGE --bench $BENCH -- '$FILTER' --warm-up-time 1 --measurement-time 2 --sample-size 20 --noplot"
OBSERVATIONS_FILE="$OUTPUT_DIR/$(basename "$OUTPUT_JSON" .json)-observations.txt"
: >"$OBSERVATIONS_FILE"

for session in 1 2 3; do
  LOG_FILE="$OUTPUT_DIR/$(basename "$OUTPUT_JSON" .json)-run-$session.log"
  (
    cd "$REPOSITORY"
    CARGO_TARGET_DIR="$TARGET_DIR" RUSTC="$RUSTC_BIN" "$CARGO_BIN" bench --locked --profile bench-lto \
      -p "$PACKAGE" --bench "$BENCH" -- "$FILTER" \
      --warm-up-time 1 --measurement-time 2 --sample-size 20 --noplot
  ) 2>&1 | tee "$LOG_FILE"
  ESTIMATE_FILE="$TARGET_DIR/criterion/$ESTIMATE_PATH"
  [[ -f "$ESTIMATE_FILE" ]] || {
    echo "Criterion estimate missing: $ESTIMATE_FILE" >&2
    exit 1
  }
  jq -er '.slope.point_estimate | select(type == "number" and . > 0)' \
    "$ESTIMATE_FILE" >>"$OBSERVATIONS_FILE"
done

OBSERVATIONS="$(jq -s '.' "$OBSERVATIONS_FILE")"
STATS="$(jq -n --argjson values "$OBSERVATIONS" '
  ($values | sort | .[1]) as $median
  | ($values | add / length) as $mean
  | (($values | map((. - $mean) * (. - $mean)) | add / length) | sqrt / $mean * 100) as $cv
  | {median: $median, cv: $cv}
')"
CAPTURED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
if command -v lscpu >/dev/null 2>&1; then
  CPU_MODEL="$(lscpu | awk -F: '/Model name/{sub(/^[[:space:]]+/, "", $2); print $2; exit}')"
else
  CPU_MODEL="$(sysctl -n machdep.cpu.brand_string 2>/dev/null || uname -p)"
fi
RUNNER_IDENTITY="$(jq -n \
  --arg os "$(uname -srv)" \
  --arg architecture "$(uname -m)" \
  --arg cpu "$CPU_MODEL" \
  --arg rustc "$("$RUSTC_BIN" --version)" \
  --arg cargo "$("$CARGO_BIN" --version)" \
  '{os: $os, architecture: $architecture, cpu: $cpu, rustc: $rustc, cargo: $cargo}')"

jq -n \
  --arg schema_version "ntpro.backend_benchmark_result.v1" \
  --arg task_id "BPO-002" \
  --arg workload_id "$WORKLOAD_ID" \
  --arg commit_sha "$COMMIT_SHA" \
  --arg captured_at "$CAPTURED_AT" \
  --arg command "$COMMAND" \
  --argjson runner_identity "$RUNNER_IDENTITY" \
  --argjson observations "$OBSERVATIONS" \
  --argjson median "$(jq '.median' <<<"$STATS")" \
  --argjson cv "$(jq '.cv' <<<"$STATS")" \
  '{
    schema_version: $schema_version,
    task_id: $task_id,
    workload_id: $workload_id,
    commit_sha: $commit_sha,
    captured_at: $captured_at,
    command: $command,
    runner_identity: $runner_identity,
    methodology: {
      profile: "bench-lto",
      warmup_seconds: 1,
      measurement_seconds: 2,
      sample_size: 20,
      session_repetitions: 3,
      estimate: "slope.point_estimate"
    },
    observations_ns: $observations,
    median_ns: $median,
    coefficient_of_variation_pct: $cv
  }' >"$OUTPUT_JSON"

jq empty "$OUTPUT_JSON"
echo "backend_benchmark_capture=pass workload=$WORKLOAD_ID commit=$COMMIT_SHA output=$OUTPUT_JSON"
