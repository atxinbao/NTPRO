#!/usr/bin/env bash
set -euo pipefail

# V100-005: v0.10 Binance testnet POST /api/v3/order/test preflight.
# Safe for local development and CI. It builds a redacted order-test preflight
# report only; it does not open network connections and does not submit orders.

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

PREFLIGHT_ROOT="${NTPRO_V10_ORDER_TEST_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v10-order-test.XXXXXX")}"
OUTPUT_DIR="$PREFLIGHT_ROOT/command-output"
mkdir -p "$OUTPUT_DIR"

SYNTHETIC_API_KEY="ntpro_v100005_synthetic_api_key_value"
SYNTHETIC_API_SECRET="ntpro_v100005_synthetic_api_secret_value"
REPORT="$OUTPUT_DIR/order-test-preflight.json"
PRODUCTION_CONFIG="$PREFLIGHT_ROOT/production-config.toml"

cp "$CONFIG" "$PRODUCTION_CONFIG"
python3 - "$PRODUCTION_CONFIG" <<'PY'
import sys
from pathlib import Path

path = Path(sys.argv[1])
body = path.read_text()
body = body.replace(
    'http_base_url = "https://testnet.binance.vision"',
    'http_base_url = "https://api.binance.com"',
)
path.write_text(body)
PY

BLOCKED_STDOUT="$OUTPUT_DIR/blocked.stdout.log"
BLOCKED_STDERR="$OUTPUT_DIR/blocked.stderr.log"
PASS_STDOUT="$OUTPUT_DIR/pass.stdout.log"
PASS_STDERR="$OUTPUT_DIR/pass.stderr.log"
PRODUCTION_STDERR="$OUTPUT_DIR/production-base.stderr.log"
MISSING_SECRET_STDERR="$OUTPUT_DIR/missing-secret.stderr.log"

set +e
env \
  -u NTPRO_ALLOW_BINANCE_TESTNET_ORDER \
  -u NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER \
  -u NTPRO_CONFIRM_TESTNET_TINY_NOTIONAL \
  -u NTPRO_CONFIRM_TESTNET_CANCEL_AFTER_SUBMIT \
  -u NTPRO_V100005_API_KEY \
  -u NTPRO_V100005_API_SECRET \
  "$NAUTILUS_BIN" live testnet-order-test-preflight \
    --config "$CONFIG" \
    --timestamp-ms 1718400000000 \
    >"$BLOCKED_STDOUT" \
    2>"$BLOCKED_STDERR"
blocked_status=$?
set -e

if [[ "$blocked_status" -eq 0 ]]; then
  echo "v10 order-test preflight expected missing gates to fail closed" >&2
  exit 1
fi
grep -q "testnet order-test preflight blocked" "$BLOCKED_STDERR"
grep -q "request_built=false" "$BLOCKED_STDERR"
grep -q "matching_engine_submission=false" "$BLOCKED_STDERR"
grep -q "order_submission_remains_disabled=true" "$BLOCKED_STDERR"
grep -q "network_attempted=false" "$BLOCKED_STDERR"
grep -q "real_orders_submitted=false" "$BLOCKED_STDERR"

NTPRO_ALLOW_BINANCE_TESTNET_ORDER=1 \
NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER=1 \
NTPRO_CONFIRM_TESTNET_TINY_NOTIONAL=1 \
NTPRO_CONFIRM_TESTNET_CANCEL_AFTER_SUBMIT=1 \
NTPRO_V100005_API_KEY="$SYNTHETIC_API_KEY" \
NTPRO_V100005_API_SECRET="$SYNTHETIC_API_SECRET" \
  "$NAUTILUS_BIN" live testnet-order-test-preflight \
    --config "$CONFIG" \
    --timestamp-ms 1718400000000 \
    --api-key-env NTPRO_V100005_API_KEY \
    --api-secret-env NTPRO_V100005_API_SECRET \
    --output "$REPORT" \
    --allow-testnet-order \
    --confirm-owner-approved-testnet-order \
    --confirm-tiny-notional \
    --confirm-cancel-after-submit \
    >"$PASS_STDOUT" \
    2>"$PASS_STDERR"

if [[ -s "$PASS_STDERR" ]]; then
  echo "v10 order-test preflight wrote stderr on pass path" >&2
  cat "$PASS_STDERR" >&2
  exit 1
fi
grep -q "live.testnet_order_test_preflight status=ready" "$PASS_STDOUT"
grep -q "binance_order_test_acceptance=not_attempted_offline_manual_only" "$PASS_STDOUT"
grep -q "matching_engine_submission=false" "$PASS_STDOUT"
grep -q "network_attempted=false" "$PASS_STDOUT"
grep -q "real_orders_submitted=false" "$PASS_STDOUT"
grep -q '"schema_version": "ntpro.v100_order_test_preflight_report.v1"' "$REPORT"
grep -q '"status": "ready"' "$REPORT"
grep -q '"request_method": "POST"' "$REPORT"
grep -q '"request_target": "/api/v3/order/test"' "$REPORT"
grep -q '"signature_preflight": "created_in_memory_not_recorded"' "$REPORT"
grep -q '"binance_order_test_acceptance": "not_attempted_offline_manual_only"' "$REPORT"
grep -q '"matching_engine_submission": false' "$REPORT"
grep -q '"order_submission_remains_disabled": true' "$REPORT"
grep -q '"network_attempted": false' "$REPORT"
grep -q '"real_orders_submitted": false' "$REPORT"
grep -q '"production_endpoint_allowed": false' "$REPORT"
grep -q '"dashboard_order_controls": false' "$REPORT"
grep -q '"secrets_redacted": true' "$REPORT"
if grep -R -q "$SYNTHETIC_API_KEY\|$SYNTHETIC_API_SECRET" "$OUTPUT_DIR"; then
  echo "v10 order-test preflight leaked a synthetic secret into output artifacts" >&2
  exit 1
fi

set +e
NTPRO_ALLOW_BINANCE_TESTNET_ORDER=1 \
NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER=1 \
NTPRO_CONFIRM_TESTNET_TINY_NOTIONAL=1 \
NTPRO_CONFIRM_TESTNET_CANCEL_AFTER_SUBMIT=1 \
NTPRO_V100005_API_KEY="$SYNTHETIC_API_KEY" \
NTPRO_V100005_API_SECRET="$SYNTHETIC_API_SECRET" \
  "$NAUTILUS_BIN" live testnet-order-test-preflight \
    --config "$PRODUCTION_CONFIG" \
    --timestamp-ms 1718400000000 \
    --api-key-env NTPRO_V100005_API_KEY \
    --api-secret-env NTPRO_V100005_API_SECRET \
    --allow-testnet-order \
    --confirm-owner-approved-testnet-order \
    --confirm-tiny-notional \
    --confirm-cancel-after-submit \
    >/dev/null \
    2>"$PRODUCTION_STDERR"
production_status=$?
set -e
if [[ "$production_status" -eq 0 ]]; then
  echo "v10 order-test preflight expected production base URL to fail closed" >&2
  exit 1
fi
grep -q "testnet_order.http_base_url" "$PRODUCTION_STDERR"

set +e
NTPRO_ALLOW_BINANCE_TESTNET_ORDER=1 \
NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER=1 \
NTPRO_CONFIRM_TESTNET_TINY_NOTIONAL=1 \
NTPRO_CONFIRM_TESTNET_CANCEL_AFTER_SUBMIT=1 \
NTPRO_V100005_API_KEY="$SYNTHETIC_API_KEY" \
  "$NAUTILUS_BIN" live testnet-order-test-preflight \
    --config "$CONFIG" \
    --timestamp-ms 1718400000000 \
    --api-key-env NTPRO_V100005_API_KEY \
    --api-secret-env NTPRO_V100005_API_SECRET \
    --allow-testnet-order \
    --confirm-owner-approved-testnet-order \
    --confirm-tiny-notional \
    --confirm-cancel-after-submit \
    >/dev/null \
    2>"$MISSING_SECRET_STDERR"
missing_secret_status=$?
set -e
if [[ "$missing_secret_status" -eq 0 ]]; then
  echo "v10 order-test preflight expected missing secret to fail closed" >&2
  exit 1
fi
grep -q "requires API secret env value" "$MISSING_SECRET_STDERR"

echo "v10_order_test_preflight status=ok root=$PREFLIGHT_ROOT"
