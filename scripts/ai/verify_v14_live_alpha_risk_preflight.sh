#!/usr/bin/env bash
set -euo pipefail

# V140-004: hypothetical live-alpha risk preflight.
# This script is CI-safe. It evaluates only local JSON artifacts and must never
# open network, call an execution adapter, or submit/mutate production orders.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_V14_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V14_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V14_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi

PREFLIGHT_ROOT="${NTPRO_V14_LIVE_ALPHA_RISK_PREFLIGHT_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v14-live-alpha-risk-preflight.XXXXXX")}"
OUTPUT_DIR="$PREFLIGHT_ROOT/command-output"
mkdir -p "$OUTPUT_DIR"

ORDER_GATE="$OUTPUT_DIR/live-alpha-dry-run-order-gate.json"
PASSING_INPUT="$OUTPUT_DIR/risk-input-approved.json"
REJECTED_INPUT="$OUTPUT_DIR/risk-input-rejected.json"
APPROVED_JSON="$OUTPUT_DIR/risk-preflight-approved.json"
REJECTED_JSON="$OUTPUT_DIR/risk-preflight-rejected.json"

"$NAUTILUS_BIN" live production-live-alpha-dry-run-order-gate \
  --run-id v140-risk-order-gate \
  --session-id session-risk \
  --strategy-id ema_cross_btcusdt_v1 \
  --symbol BTCUSDT \
  --side BUY \
  --order-type MARKET \
  --quantity 0.001 \
  --notional 10.00 \
  --output "$ORDER_GATE" \
  --allow-production-live-alpha-dry-run \
  --confirm-owner-approved-dry-run \
  --confirm-no-production-order-submission \
  --confirm-no-production-order-mutation \
  --confirm-no-execution-adapter-call \
  --confirm-no-listen-key-lifecycle \
  --confirm-dashboard-order-controls-disabled \
  --confirm-no-real-funds \
  >/dev/null

cat > "$PASSING_INPUT" <<'JSON'
{
  "schema_version": "ntpro.v140_live_alpha_risk_preflight_input.v1",
  "session": {
    "state": "running"
  },
  "market": {
    "symbol": "BTCUSDT",
    "last_event_at_unix_ms": 1000,
    "now_unix_ms": 1500,
    "max_age_ms": 1000
  },
  "account": {
    "readable": true,
    "account_id": "BINANCE-001"
  },
  "order_state": {
    "readable": true,
    "open_order_count": 0
  },
  "risk": {
    "kill_switch_active": false,
    "allowed_symbols": ["BTCUSDT"]
  },
  "order": {
    "symbol": "BTCUSDT",
    "side": "BUY",
    "order_type": "MARKET",
    "quantity": "0.001",
    "notional": "10.00"
  },
  "limits": {
    "max_order_notional": "25.00",
    "current_position_notional": "50.00",
    "max_position_notional": "100.00",
    "max_open_orders": 5,
    "max_clock_skew_ms": 100,
    "observed_clock_skew_ms": 25
  }
}
JSON

"$NAUTILUS_BIN" live production-live-alpha-risk-preflight \
  --run-id v140-risk-approved \
  --order-gate "$ORDER_GATE" \
  --input "$PASSING_INPUT" \
  --output "$APPROVED_JSON" \
  --confirm-hypothetical-dry-run-only \
  --confirm-no-execution-adapter-call \
  --confirm-no-production-order-submission \
  --confirm-no-production-order-mutation \
  --confirm-dashboard-order-controls-disabled \
  >/dev/null

python3 - "$APPROVED_JSON" <<'PY'
import json
import sys
from pathlib import Path

report = json.loads(Path(sys.argv[1]).read_text())

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require(report["schema_version"] == "ntpro.v140_live_alpha_risk_preflight.v1", report)
require(report["status"] == "approved", report)
require(report["risk_decision"] == "approved", report)
require(report["reasons"] == [], report)
require(report["order_gate_ready"] is True, report)
require(report["projected_position_notional"] == "60", report)
require(report["production_order_submission_allowed"] is False, report)
require(report["production_order_mutation_allowed"] is False, report)
require(report["production_order_submissions_attempted"] == 0, report)
require(report["production_orders_submitted"] == 0, report)
require(report["production_order_mutations_attempted"] == 0, report)
require(report["execution_adapter_called"] is False, report)
require(report["order_endpoint_access_attempted"] is False, report)
require(report["matching_engine_submission"] is False, report)
require(report["network_attempted"] is False, report)
require(report["dashboard_order_controls_enabled"] is False, report)
require(report["real_orders_submitted"] is False, report)
PY

cat > "$REJECTED_INPUT" <<'JSON'
{
  "schema_version": "ntpro.v140_live_alpha_risk_preflight_input.v1",
  "session": {
    "state": "running"
  },
  "market": {
    "symbol": "BTCUSDT",
    "last_event_at_unix_ms": 1000,
    "now_unix_ms": 5000,
    "max_age_ms": 100
  },
  "account": {
    "readable": false,
    "account_id": ""
  },
  "order_state": {
    "readable": false,
    "open_order_count": 5
  },
  "risk": {
    "kill_switch_active": true,
    "allowed_symbols": ["BTCUSDT"]
  },
  "order": {
    "symbol": "BTCUSDT",
    "side": "BUY",
    "order_type": "MARKET",
    "quantity": "0.001",
    "notional": "30.00"
  },
  "limits": {
    "max_order_notional": "25.00",
    "current_position_notional": "90.00",
    "max_position_notional": "100.00",
    "max_open_orders": 5,
    "max_clock_skew_ms": 100,
    "observed_clock_skew_ms": 25
  }
}
JSON

"$NAUTILUS_BIN" live production-live-alpha-risk-preflight \
  --run-id v140-risk-rejected \
  --order-gate "$ORDER_GATE" \
  --input "$REJECTED_INPUT" \
  --output "$REJECTED_JSON" \
  --confirm-hypothetical-dry-run-only \
  --confirm-no-execution-adapter-call \
  --confirm-no-production-order-submission \
  --confirm-no-production-order-mutation \
  --confirm-dashboard-order-controls-disabled \
  >/dev/null

python3 - "$REJECTED_JSON" <<'PY'
import json
import sys
from pathlib import Path

report = json.loads(Path(sys.argv[1]).read_text())
reasons = set(report["reasons"])
required = {
    "market_stale",
    "account_read_failed",
    "order_state_read_failed",
    "kill_switch_active",
    "notional_limit_exceeded",
    "position_limit_exceeded",
    "open_order_limit_exceeded",
}
missing = required - reasons
if missing:
    raise SystemExit(f"missing rejection reasons: {sorted(missing)} report={report}")
for key, expected in {
    "status": "rejected",
    "risk_decision": "rejected",
}.items():
    if report[key] != expected:
        raise SystemExit(report)
for key in [
    "execution_adapter_called",
    "order_endpoint_access_attempted",
    "network_attempted",
    "dashboard_order_controls_enabled",
    "real_orders_submitted",
]:
    if report[key] is not False:
        raise SystemExit(report)
for key in [
    "production_orders_submitted",
    "production_order_mutations_attempted",
    "actual_submission_count",
]:
    if report[key] != 0:
        raise SystemExit(report)
PY

if rg -n \
  '"execution_adapter_called": true|"order_endpoint_access_attempted": true|"dashboard_order_controls_enabled": true|"production_orders_submitted": [1-9]|"production_order_mutations_attempted": [1-9]' \
  "$APPROVED_JSON" "$REJECTED_JSON" >/dev/null; then
  echo "v14 risk preflight artifact contains execution or mutation fields" >&2
  exit 1
fi

echo "v14_live_alpha_risk_preflight status=ok root=$PREFLIGHT_ROOT approved=ok rejected=ok production_orders_submitted=0 production_order_mutations_attempted=0 execution_adapter_called=false order_endpoint_access_attempted=false network_attempted=false dashboard_order_controls_enabled=false"
