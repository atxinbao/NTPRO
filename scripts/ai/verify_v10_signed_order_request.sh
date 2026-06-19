#!/usr/bin/env bash
set -euo pipefail

# V100-004: v0.10 Binance testnet signed order request preview proof.
# Safe for local development and CI. It builds request metadata only, writes a
# redacted preview artifact, opens no network connections, and submits no orders.

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

REQUEST_ROOT="${NTPRO_V10_REQUEST_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v10-request.XXXXXX")}"
OUTPUT_DIR="$REQUEST_ROOT/command-output"
mkdir -p "$OUTPUT_DIR"

SYNTHETIC_API_KEY="ntpro_v100004_synthetic_api_key_value"
SYNTHETIC_API_SECRET="ntpro_v100004_synthetic_api_secret_value"
POST_REPORT="$OUTPUT_DIR/post-order-test-preview.json"
CANCEL_REPORT="$OUTPUT_DIR/cancel-order-preview.json"
PRODUCTION_CONFIG="$REQUEST_ROOT/production-config.toml"

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
POST_STDOUT="$OUTPUT_DIR/post.stdout.log"
POST_STDERR="$OUTPUT_DIR/post.stderr.log"
CANCEL_STDOUT="$OUTPUT_DIR/cancel.stdout.log"
CANCEL_STDERR="$OUTPUT_DIR/cancel.stderr.log"
BAD_ENDPOINT_STDERR="$OUTPUT_DIR/bad-endpoint.stderr.log"
PRODUCTION_STDERR="$OUTPUT_DIR/production-base.stderr.log"
MISSING_SECRET_STDERR="$OUTPUT_DIR/missing-secret.stderr.log"

set +e
env \
  -u NTPRO_ALLOW_BINANCE_TESTNET_ORDER \
  -u NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER \
  -u NTPRO_CONFIRM_TESTNET_TINY_NOTIONAL \
  -u NTPRO_CONFIRM_TESTNET_CANCEL_AFTER_SUBMIT \
  -u NTPRO_V100004_API_KEY \
  -u NTPRO_V100004_API_SECRET \
  "$NAUTILUS_BIN" live testnet-order-request-preview \
    --config "$CONFIG" \
    --timestamp-ms 1718400000000 \
    >"$BLOCKED_STDOUT" \
    2>"$BLOCKED_STDERR"
blocked_status=$?
set -e

if [[ "$blocked_status" -eq 0 ]]; then
  echo "v10 signed order request preview expected missing gates to fail closed" >&2
  exit 1
fi
grep -q "testnet signed order request preview blocked" "$BLOCKED_STDERR"
grep -q "request_built=false" "$BLOCKED_STDERR"
grep -q "order_submission_remains_disabled=true" "$BLOCKED_STDERR"
grep -q "network_attempted=false" "$BLOCKED_STDERR"
grep -q "real_orders_submitted=false" "$BLOCKED_STDERR"

NTPRO_ALLOW_BINANCE_TESTNET_ORDER=1 \
NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER=1 \
NTPRO_CONFIRM_TESTNET_TINY_NOTIONAL=1 \
NTPRO_CONFIRM_TESTNET_CANCEL_AFTER_SUBMIT=1 \
NTPRO_V100004_API_KEY="$SYNTHETIC_API_KEY" \
NTPRO_V100004_API_SECRET="$SYNTHETIC_API_SECRET" \
  "$NAUTILUS_BIN" live testnet-order-request-preview \
    --config "$CONFIG" \
    --timestamp-ms 1718400000000 \
    --api-key-env NTPRO_V100004_API_KEY \
    --api-secret-env NTPRO_V100004_API_SECRET \
    --output "$POST_REPORT" \
    --allow-testnet-order \
    --confirm-owner-approved-testnet-order \
    --confirm-tiny-notional \
    --confirm-cancel-after-submit \
    >"$POST_STDOUT" \
    2>"$POST_STDERR"

if [[ -s "$POST_STDERR" ]]; then
  echo "v10 signed order request preview wrote stderr on pass path" >&2
  cat "$POST_STDERR" >&2
  exit 1
fi
grep -q "live.testnet_order_request_preview status=ready" "$POST_STDOUT"
grep -q "network_attempted=false" "$POST_STDOUT"
grep -q "real_orders_submitted=false" "$POST_STDOUT"
grep -q '"schema_version": "ntpro.v100_signed_order_request_preview.v1"' "$POST_REPORT"
grep -q '"order_action": "order_test"' "$POST_REPORT"
grep -q '"request_target": "/api/v3/order/test"' "$POST_REPORT"
grep -q '"query_shape": "symbol=BTCUSDT' "$POST_REPORT"
grep -q 'signature=<redacted>' "$POST_REPORT"
grep -q '"api_key_header_value_recorded": false' "$POST_REPORT"
grep -q '"signature_recorded": false' "$POST_REPORT"
grep -q '"signed_query_recorded": false' "$POST_REPORT"
grep -q '"signed_url_recorded": false' "$POST_REPORT"
grep -q '"order_submission_remains_disabled": true' "$POST_REPORT"
grep -q '"network_attempted": false' "$POST_REPORT"
grep -q '"real_orders_submitted": false' "$POST_REPORT"
grep -q '"production_endpoint_allowed": false' "$POST_REPORT"
grep -q '"dashboard_order_controls": false' "$POST_REPORT"
grep -q '"secrets_redacted": true' "$POST_REPORT"
if grep -R -q "$SYNTHETIC_API_KEY\|$SYNTHETIC_API_SECRET" "$OUTPUT_DIR"; then
  echo "v10 signed order request preview leaked a synthetic secret into output artifacts" >&2
  exit 1
fi

NTPRO_ALLOW_BINANCE_TESTNET_ORDER=1 \
NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER=1 \
NTPRO_CONFIRM_TESTNET_TINY_NOTIONAL=1 \
NTPRO_CONFIRM_TESTNET_CANCEL_AFTER_SUBMIT=1 \
NTPRO_V100004_API_KEY="$SYNTHETIC_API_KEY" \
NTPRO_V100004_API_SECRET="$SYNTHETIC_API_SECRET" \
  "$NAUTILUS_BIN" live testnet-order-request-preview \
    --config "$CONFIG" \
    --method DELETE \
    --endpoint-path /api/v3/order \
    --timestamp-ms 1718400000000 \
    --api-key-env NTPRO_V100004_API_KEY \
    --api-secret-env NTPRO_V100004_API_SECRET \
    --orig-client-order-id ntpro-cancel-001 \
    --output "$CANCEL_REPORT" \
    --allow-testnet-order \
    --confirm-owner-approved-testnet-order \
    --confirm-tiny-notional \
    --confirm-cancel-after-submit \
    >"$CANCEL_STDOUT" \
    2>"$CANCEL_STDERR"

if [[ -s "$CANCEL_STDERR" ]]; then
  echo "v10 signed order request cancel preview wrote stderr on pass path" >&2
  cat "$CANCEL_STDERR" >&2
  exit 1
fi
grep -q '"order_action": "cancel"' "$CANCEL_REPORT"
grep -q '"request_method": "DELETE"' "$CANCEL_REPORT"
grep -q '"request_target": "/api/v3/order"' "$CANCEL_REPORT"
grep -q 'origClientOrderId=ntpro-cancel-001' "$CANCEL_REPORT"
grep -q '"network_attempted": false' "$CANCEL_REPORT"
grep -q '"real_orders_submitted": false' "$CANCEL_REPORT"

run_expected_failure() {
  local name="$1"
  local stderr_path="$2"
  shift 2
  set +e
  NTPRO_ALLOW_BINANCE_TESTNET_ORDER=1 \
  NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER=1 \
  NTPRO_CONFIRM_TESTNET_TINY_NOTIONAL=1 \
  NTPRO_CONFIRM_TESTNET_CANCEL_AFTER_SUBMIT=1 \
  NTPRO_V100004_API_KEY="$SYNTHETIC_API_KEY" \
  NTPRO_V100004_API_SECRET="$SYNTHETIC_API_SECRET" \
    "$NAUTILUS_BIN" live testnet-order-request-preview "$@" \
      --timestamp-ms 1718400000000 \
      --api-key-env NTPRO_V100004_API_KEY \
      --api-secret-env NTPRO_V100004_API_SECRET \
      --allow-testnet-order \
      --confirm-owner-approved-testnet-order \
      --confirm-tiny-notional \
      --confirm-cancel-after-submit \
      >/dev/null \
      2>"$stderr_path"
  status=$?
  set -e
  if [[ "$status" -eq 0 ]]; then
    echo "v10 signed order request preview expected $name to fail closed" >&2
    exit 1
  fi
}

run_expected_failure "bad endpoint" "$BAD_ENDPOINT_STDERR" \
  --config "$CONFIG" \
  --endpoint-path /api/v3/account
grep -q "signed order request allowlist only includes" "$BAD_ENDPOINT_STDERR"

run_expected_failure "production base" "$PRODUCTION_STDERR" \
  --config "$PRODUCTION_CONFIG"
grep -q "testnet_order.http_base_url" "$PRODUCTION_STDERR"

set +e
NTPRO_ALLOW_BINANCE_TESTNET_ORDER=1 \
NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER=1 \
NTPRO_CONFIRM_TESTNET_TINY_NOTIONAL=1 \
NTPRO_CONFIRM_TESTNET_CANCEL_AFTER_SUBMIT=1 \
NTPRO_V100004_API_KEY="$SYNTHETIC_API_KEY" \
  "$NAUTILUS_BIN" live testnet-order-request-preview \
    --config "$CONFIG" \
    --timestamp-ms 1718400000000 \
    --api-key-env NTPRO_V100004_API_KEY \
    --api-secret-env NTPRO_V100004_API_SECRET \
    --allow-testnet-order \
    --confirm-owner-approved-testnet-order \
    --confirm-tiny-notional \
    --confirm-cancel-after-submit \
    >/dev/null \
    2>"$MISSING_SECRET_STDERR"
missing_secret_status=$?
set -e
if [[ "$missing_secret_status" -eq 0 ]]; then
  echo "v10 signed order request preview expected missing secret to fail closed" >&2
  exit 1
fi
grep -q "requires API secret env value" "$MISSING_SECRET_STDERR"

echo "v10_signed_order_request status=ok root=$REQUEST_ROOT"
