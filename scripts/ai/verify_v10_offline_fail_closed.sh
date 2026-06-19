#!/usr/bin/env bash
set -euo pipefail

# V100-002: v0.10 Binance testnet order gate fail-closed proof.
# This script is safe for local development and CI. It does not open network
# connections and does not submit Binance testnet orders.

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

GATE_ROOT="${NTPRO_V10_OFFLINE_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v10-offline-gate.XXXXXX")}"
OUTPUT_DIR="$GATE_ROOT/command-output"
mkdir -p "$OUTPUT_DIR"

BLOCKED_STDOUT="$OUTPUT_DIR/blocked.stdout.log"
BLOCKED_STDERR="$OUTPUT_DIR/blocked.stderr.log"
READY_STDOUT="$OUTPUT_DIR/ready.stdout.log"
READY_STDERR="$OUTPUT_DIR/ready.stderr.log"

set +e
env \
  -u NTPRO_ALLOW_BINANCE_TESTNET_ORDER \
  -u NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER \
  -u NTPRO_CONFIRM_TESTNET_TINY_NOTIONAL \
  -u NTPRO_CONFIRM_TESTNET_CANCEL_AFTER_SUBMIT \
  "$NAUTILUS_BIN" live testnet-order-gate \
    --config "$CONFIG" \
    >"$BLOCKED_STDOUT" \
    2>"$BLOCKED_STDERR"
blocked_status=$?
set -e

if [[ "$blocked_status" -eq 0 ]]; then
  echo "v10 offline gate expected missing gates to fail closed" >&2
  cat "$BLOCKED_STDOUT" >&2
  cat "$BLOCKED_STDERR" >&2
  exit 1
fi

grep -q "testnet order gate blocked" "$BLOCKED_STDERR"
grep -q "missing_cli_flags=--allow-testnet-order" "$BLOCKED_STDERR"
grep -q "NTPRO_ALLOW_BINANCE_TESTNET_ORDER" "$BLOCKED_STDERR"
grep -q "NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER" "$BLOCKED_STDERR"
grep -q "NTPRO_CONFIRM_TESTNET_TINY_NOTIONAL" "$BLOCKED_STDERR"
grep -q "NTPRO_CONFIRM_TESTNET_CANCEL_AFTER_SUBMIT" "$BLOCKED_STDERR"
grep -q "order_submission_remains_disabled=true" "$BLOCKED_STDERR"
grep -q "network_attempted=false" "$BLOCKED_STDERR"
grep -q "real_orders_submitted=false" "$BLOCKED_STDERR"

NTPRO_ALLOW_BINANCE_TESTNET_ORDER=1 \
NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER=1 \
NTPRO_CONFIRM_TESTNET_TINY_NOTIONAL=1 \
NTPRO_CONFIRM_TESTNET_CANCEL_AFTER_SUBMIT=1 \
  "$NAUTILUS_BIN" live testnet-order-gate \
    --config "$CONFIG" \
    --allow-testnet-order \
    --confirm-owner-approved-testnet-order \
    --confirm-tiny-notional \
    --confirm-cancel-after-submit \
    >"$READY_STDOUT" \
    2>"$READY_STDERR"

if [[ -s "$READY_STDERR" ]]; then
  echo "v10 offline gate ready path wrote stderr" >&2
  cat "$READY_STDERR" >&2
  exit 1
fi

grep -q "live.testnet_order_gate status=ready" "$READY_STDOUT"
grep -q "manual_gate_ready=true" "$READY_STDOUT"
grep -q "order_submission_remains_disabled=true" "$READY_STDOUT"
grep -q "network_attempted=false" "$READY_STDOUT"
grep -q "real_orders_submitted=false" "$READY_STDOUT"
grep -q "production_endpoint_allowed=false" "$READY_STDOUT"
grep -q "dashboard_order_controls=false" "$READY_STDOUT"

echo "v10_offline_fail_closed status=ok root=$GATE_ROOT"
