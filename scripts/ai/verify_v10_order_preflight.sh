#!/usr/bin/env bash
set -euo pipefail

# V100-003: v0.10 Binance testnet order risk preflight proof.
# This script is safe for local development and CI. It reads local JSON
# preflight snapshots only; it does not open network connections and does not
# submit Binance testnet orders.

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

PREFLIGHT_ROOT="${NTPRO_V10_PREFLIGHT_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v10-preflight.XXXXXX")}"
OUTPUT_DIR="$PREFLIGHT_ROOT/command-output"
mkdir -p "$OUTPUT_DIR"

PASS_INPUT="$PREFLIGHT_ROOT/pass-input.json"
STALE_INPUT="$PREFLIGHT_ROOT/stale-input.json"
PROD_INPUT="$PREFLIGHT_ROOT/production-endpoint-input.json"
LIMIT_INPUT="$PREFLIGHT_ROOT/limit-input.json"
REPORT="$OUTPUT_DIR/pass-report.json"

cat >"$PASS_INPUT" <<'JSON'
{
  "schema_version": "ntpro.v100_order_preflight_input.v1",
  "session": {"state": "running"},
  "market": {
    "symbol": "BTCUSDT.BINANCE",
    "last_event_at_unix_ms": 1000,
    "now_unix_ms": 1500,
    "max_age_ms": 1000
  },
  "account": {"readable": true, "account_id": "BINANCE_TESTNET-001"},
  "risk": {"kill_switch_active": false, "allowed_symbols": ["BTCUSDT.BINANCE"]},
  "limits": {
    "max_order_notional": "1.00",
    "max_open_orders": 1,
    "open_order_count": 0,
    "max_clock_skew_ms": 100,
    "observed_clock_skew_ms": 25
  },
  "endpoint": {
    "http_base_url": "https://testnet.binance.vision",
    "production_endpoint_allowed": false
  }
}
JSON

python3 - "$PASS_INPUT" "$STALE_INPUT" "$PROD_INPUT" "$LIMIT_INPUT" <<'PY'
import json
import sys
from pathlib import Path

src = Path(sys.argv[1])
base = json.loads(src.read_text())

stale = json.loads(json.dumps(base))
stale["market"]["now_unix_ms"] = 3000
stale["market"]["max_age_ms"] = 100
Path(sys.argv[2]).write_text(json.dumps(stale, indent=2) + "\n")

prod = json.loads(json.dumps(base))
prod["endpoint"]["http_base_url"] = "https://api.binance.com"
prod["endpoint"]["production_endpoint_allowed"] = True
Path(sys.argv[3]).write_text(json.dumps(prod, indent=2) + "\n")

limit = json.loads(json.dumps(base))
limit["limits"]["max_order_notional"] = "0.00000001"
limit["limits"]["open_order_count"] = 1
limit["limits"]["max_clock_skew_ms"] = 10
Path(sys.argv[4]).write_text(json.dumps(limit, indent=2) + "\n")
PY

BLOCKED_STDOUT="$OUTPUT_DIR/blocked.stdout.log"
BLOCKED_STDERR="$OUTPUT_DIR/blocked.stderr.log"
PASS_STDOUT="$OUTPUT_DIR/pass.stdout.log"
PASS_STDERR="$OUTPUT_DIR/pass.stderr.log"
STALE_STDERR="$OUTPUT_DIR/stale.stderr.log"
PROD_STDERR="$OUTPUT_DIR/production-endpoint.stderr.log"
LIMIT_STDERR="$OUTPUT_DIR/limit.stderr.log"

set +e
env \
  -u NTPRO_ALLOW_BINANCE_TESTNET_ORDER \
  -u NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER \
  -u NTPRO_CONFIRM_TESTNET_TINY_NOTIONAL \
  -u NTPRO_CONFIRM_TESTNET_CANCEL_AFTER_SUBMIT \
  "$NAUTILUS_BIN" live testnet-order-preflight \
    --config "$CONFIG" \
    --input "$PASS_INPUT" \
    >"$BLOCKED_STDOUT" \
    2>"$BLOCKED_STDERR"
blocked_status=$?
set -e

if [[ "$blocked_status" -eq 0 ]]; then
  echo "v10 order preflight expected missing gates to fail closed" >&2
  exit 1
fi
grep -q "testnet order preflight blocked" "$BLOCKED_STDERR"
grep -q "preflight_evaluated=false" "$BLOCKED_STDERR"
grep -q "network_attempted=false" "$BLOCKED_STDERR"
grep -q "real_orders_submitted=false" "$BLOCKED_STDERR"

NTPRO_ALLOW_BINANCE_TESTNET_ORDER=1 \
NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER=1 \
NTPRO_CONFIRM_TESTNET_TINY_NOTIONAL=1 \
NTPRO_CONFIRM_TESTNET_CANCEL_AFTER_SUBMIT=1 \
  "$NAUTILUS_BIN" live testnet-order-preflight \
    --config "$CONFIG" \
    --input "$PASS_INPUT" \
    --output "$REPORT" \
    --allow-testnet-order \
    --confirm-owner-approved-testnet-order \
    --confirm-tiny-notional \
    --confirm-cancel-after-submit \
    >"$PASS_STDOUT" \
    2>"$PASS_STDERR"

if [[ -s "$PASS_STDERR" ]]; then
  echo "v10 order preflight pass path wrote stderr" >&2
  cat "$PASS_STDERR" >&2
  exit 1
fi
grep -q "live.testnet_order_preflight status=pass" "$PASS_STDOUT"
grep -q "network_attempted=false" "$PASS_STDOUT"
grep -q "real_orders_submitted=false" "$PASS_STDOUT"
grep -q '"status": "pass"' "$REPORT"
grep -q '"order_submission_remains_disabled": true' "$REPORT"
grep -q '"network_attempted": false' "$REPORT"
grep -q '"real_orders_submitted": false' "$REPORT"

run_failure_case() {
  local case_name="$1"
  local input_path="$2"
  local stderr_path="$3"
  set +e
  NTPRO_ALLOW_BINANCE_TESTNET_ORDER=1 \
  NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER=1 \
  NTPRO_CONFIRM_TESTNET_TINY_NOTIONAL=1 \
  NTPRO_CONFIRM_TESTNET_CANCEL_AFTER_SUBMIT=1 \
    "$NAUTILUS_BIN" live testnet-order-preflight \
      --config "$CONFIG" \
      --input "$input_path" \
      --allow-testnet-order \
      --confirm-owner-approved-testnet-order \
      --confirm-tiny-notional \
      --confirm-cancel-after-submit \
      >/dev/null \
      2>"$stderr_path"
  status=$?
  set -e
  if [[ "$status" -eq 0 ]]; then
    echo "v10 order preflight expected $case_name to fail closed" >&2
    exit 1
  fi
  grep -q "testnet order preflight failed" "$stderr_path"
  grep -q "network_attempted=false" "$stderr_path"
  grep -q "real_orders_submitted=false" "$stderr_path"
}

run_failure_case "stale" "$STALE_INPUT" "$STALE_STDERR"
run_failure_case "production-endpoint" "$PROD_INPUT" "$PROD_STDERR"
run_failure_case "limit" "$LIMIT_INPUT" "$LIMIT_STDERR"

grep -q "market_stale" "$STALE_STDERR"
grep -q "endpoint_not_testnet" "$PROD_STDERR"
grep -q "production_endpoint_allowed" "$PROD_STDERR"
grep -q "notional_limit_exceeded" "$LIMIT_STDERR"
grep -q "open_order_limit_exceeded" "$LIMIT_STDERR"
grep -q "clock_skew_limit_exceeded" "$LIMIT_STDERR"

echo "v10_order_preflight status=ok root=$PREFLIGHT_ROOT"
