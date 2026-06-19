#!/usr/bin/env bash
set -euo pipefail

# V100-007: v0.10 Binance testnet execution artifact contract.
# Safe for local development and CI. It writes a redacted artifact contract only;
# it does not open network connections and does not submit or cancel orders.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_V10_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V10_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V10_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
CONFIG="${NTPRO_V10_CONFIG:-$ROOT_DIR/configs/nodes/btc-ema-shadow.toml}"

if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi
if [[ ! -f "$CONFIG" ]]; then
  echo "missing v0.10 strategy config: $CONFIG" >&2
  exit 1
fi

ARTIFACT_ROOT="${NTPRO_V10_EXECUTION_ARTIFACT_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v10-execution-artifact.XXXXXX")}"
OUTPUT_DIR="$ARTIFACT_ROOT/command-output"
mkdir -p "$OUTPUT_DIR"

SYNTHETIC_API_KEY="ntpro_v100007_synthetic_api_key_value"
SYNTHETIC_API_SECRET="ntpro_v100007_synthetic_api_secret_value"
REPORT="$OUTPUT_DIR/execution-artifact-contract.json"

BLOCKED_STDOUT="$OUTPUT_DIR/blocked.stdout.log"
BLOCKED_STDERR="$OUTPUT_DIR/blocked.stderr.log"
PASS_STDOUT="$OUTPUT_DIR/pass.stdout.log"
PASS_STDERR="$OUTPUT_DIR/pass.stderr.log"
MISSING_SECRET_STDERR="$OUTPUT_DIR/missing-secret.stderr.log"

set +e
env \
  -u NTPRO_ALLOW_BINANCE_TESTNET_ORDER \
  -u NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER \
  -u NTPRO_CONFIRM_TESTNET_TINY_NOTIONAL \
  -u NTPRO_CONFIRM_TESTNET_CANCEL_AFTER_SUBMIT \
  -u NTPRO_V100007_API_KEY \
  -u NTPRO_V100007_API_SECRET \
  "$NAUTILUS_BIN" live testnet-execution-artifact-contract \
    --config "$CONFIG" \
    --timestamp-ms 1718400000000 \
    >"$BLOCKED_STDOUT" \
    2>"$BLOCKED_STDERR"
blocked_status=$?
set -e

if [[ "$blocked_status" -eq 0 ]]; then
  echo "v10 execution artifact contract expected missing gates to fail closed" >&2
  exit 1
fi
grep -q "testnet execution artifact contract blocked" "$BLOCKED_STDERR"
grep -q "artifact_built=false" "$BLOCKED_STDERR"
grep -q "matching_engine_submission=false" "$BLOCKED_STDERR"
grep -q "order_submission_remains_disabled=true" "$BLOCKED_STDERR"
grep -q "network_attempted=false" "$BLOCKED_STDERR"
grep -q "real_orders_submitted=false" "$BLOCKED_STDERR"

NTPRO_ALLOW_BINANCE_TESTNET_ORDER=1 \
NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER=1 \
NTPRO_CONFIRM_TESTNET_TINY_NOTIONAL=1 \
NTPRO_CONFIRM_TESTNET_CANCEL_AFTER_SUBMIT=1 \
NTPRO_V100007_API_KEY="$SYNTHETIC_API_KEY" \
NTPRO_V100007_API_SECRET="$SYNTHETIC_API_SECRET" \
  "$NAUTILUS_BIN" live testnet-execution-artifact-contract \
    --config "$CONFIG" \
    --timestamp-ms 1718400000000 \
    --api-key-env NTPRO_V100007_API_KEY \
    --api-secret-env NTPRO_V100007_API_SECRET \
    --orig-client-order-id ntpro-v100007-cancel-only \
    --output "$REPORT" \
    --allow-testnet-order \
    --confirm-owner-approved-testnet-order \
    --confirm-tiny-notional \
    --confirm-cancel-after-submit \
    >"$PASS_STDOUT" \
    2>"$PASS_STDERR"

if [[ -s "$PASS_STDERR" ]]; then
  echo "v10 execution artifact contract wrote stderr on pass path" >&2
  cat "$PASS_STDERR" >&2
  exit 1
fi
grep -q "live.testnet_execution_artifact_contract status=ready" "$PASS_STDOUT"
grep -q "manual_submit_cancel_proof_observed=false" "$PASS_STDOUT"
grep -q "testnet_orders_submitted=0" "$PASS_STDOUT"
grep -q "production_orders_submitted=0" "$PASS_STDOUT"
grep -q "network_attempted=false" "$PASS_STDOUT"
grep -q "real_orders_submitted=false" "$PASS_STDOUT"
grep -q '"schema_version": "ntpro.v100_execution_artifact_contract.v1"' "$REPORT"
grep -q '"artifact_family": "binance-testnet-order-lifecycle-proof"' "$REPORT"
grep -q '"name": "request.json"' "$REPORT"
grep -q '"name": "order_test.json"' "$REPORT"
grep -q '"name": "submit_ack.json"' "$REPORT"
grep -q '"name": "cancel_ack.json"' "$REPORT"
grep -q '"name": "lifecycle.json"' "$REPORT"
grep -q '"name": "reconciliation.json"' "$REPORT"
grep -q '"testnet_orders_submitted": 0' "$REPORT"
grep -q '"testnet_orders_canceled": 0' "$REPORT"
grep -q '"production_orders_submitted": 0' "$REPORT"
grep -q '"production_orders_canceled": 0' "$REPORT"
grep -q '"manual_submit_cancel_proof_observed": false' "$REPORT"
grep -q '"matching_engine_submission": false' "$REPORT"
grep -q '"order_submission_remains_disabled": true' "$REPORT"
grep -q '"network_attempted": false' "$REPORT"
grep -q '"real_orders_submitted": false' "$REPORT"
grep -q '"production_endpoint_allowed": false' "$REPORT"
grep -q '"dashboard_order_controls": false' "$REPORT"
grep -q '"secrets_redacted": true' "$REPORT"
if grep -R -q "$SYNTHETIC_API_KEY\|$SYNTHETIC_API_SECRET" "$OUTPUT_DIR"; then
  echo "v10 execution artifact contract leaked a synthetic secret into output artifacts" >&2
  exit 1
fi

set +e
NTPRO_ALLOW_BINANCE_TESTNET_ORDER=1 \
NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER=1 \
NTPRO_CONFIRM_TESTNET_TINY_NOTIONAL=1 \
NTPRO_CONFIRM_TESTNET_CANCEL_AFTER_SUBMIT=1 \
NTPRO_V100007_API_KEY="$SYNTHETIC_API_KEY" \
  "$NAUTILUS_BIN" live testnet-execution-artifact-contract \
    --config "$CONFIG" \
    --timestamp-ms 1718400000000 \
    --api-key-env NTPRO_V100007_API_KEY \
    --api-secret-env NTPRO_V100007_API_SECRET \
    --allow-testnet-order \
    --confirm-owner-approved-testnet-order \
    --confirm-tiny-notional \
    --confirm-cancel-after-submit \
    >/dev/null \
    2>"$MISSING_SECRET_STDERR"
missing_secret_status=$?
set -e
if [[ "$missing_secret_status" -eq 0 ]]; then
  echo "v10 execution artifact contract expected missing secret to fail closed" >&2
  exit 1
fi
grep -q "requires API secret env value" "$MISSING_SECRET_STDERR"

echo "v10_execution_artifact_contract status=ok root=$ARTIFACT_ROOT"
