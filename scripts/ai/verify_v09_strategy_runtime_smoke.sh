#!/usr/bin/env bash
set -euo pipefail

# V090-012: Strategy Runtime Foundation smoke.
# This gate runs a local ntpro-node Strategy Session. It does not connect to
# Binance, does not require secrets, and does not submit exchange orders.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_V09_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V09_NTPRO_NODE_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin ntpro-node
fi

NTPRO_NODE_BIN="${NTPRO_V09_NTPRO_NODE_BIN:-$ROOT_DIR/target/debug/ntpro-node}"
CONFIG="${NTPRO_V09_CONFIG:-$ROOT_DIR/configs/nodes/btc-ema-shadow.toml}"

if [[ ! -x "$NTPRO_NODE_BIN" ]]; then
  echo "missing ntpro-node binary: $NTPRO_NODE_BIN" >&2
  exit 1
fi
if [[ ! -f "$CONFIG" ]]; then
  echo "missing v0.9 strategy runtime config: $CONFIG" >&2
  exit 1
fi

SMOKE_ROOT="${NTPRO_V09_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v09-strategy-runtime.XXXXXX")}"
RUN_ID="${NTPRO_V09_RUN_ID:-v09-strategy-runtime-smoke}"
NODE_DIR="$SMOKE_ROOT/nodes/$RUN_ID"
OUTPUT_DIR="$SMOKE_ROOT/command-output"

mkdir -p "$NODE_DIR" "$OUTPUT_DIR"

assert_output_contains() {
  local file="$1"
  local expected="$2"
  if ! grep -q "$expected" "$file"; then
    echo "missing expected output '$expected' in $file" >&2
    cat "$file" >&2
    exit 1
  fi
}

run_and_capture() {
  local name="$1"
  shift
  echo "+ $*"
  "$@" | tee "$OUTPUT_DIR/$name.txt"
  local status="${PIPESTATUS[0]}"
  return "$status"
}

run_and_capture ntpro_node "$NTPRO_NODE_BIN" \
  --config "$CONFIG" \
  --run-id "$RUN_ID" \
  --output "$NODE_DIR"

assert_output_contains "$OUTPUT_DIR/ntpro_node.txt" "status=ok"
assert_output_contains "$OUTPUT_DIR/ntpro_node.txt" "strategy_id=ema_cross_btcusdt_v1"
assert_output_contains "$OUTPUT_DIR/ntpro_node.txt" "final_state=Stopped"
assert_output_contains "$OUTPUT_DIR/ntpro_node.txt" "external_venue_connection=false"
assert_output_contains "$OUTPUT_DIR/ntpro_node.txt" "real_orders_submitted=false"
assert_output_contains "$OUTPUT_DIR/ntpro_node.txt" "runtime_status=completed"

python3 - "$NODE_DIR" "$RUN_ID" <<'PY'
import json
import sys
from pathlib import Path

node_dir = Path(sys.argv[1])
run_id = sys.argv[2]
strategy_dir = node_dir / "strategy"

expected_files = [
    "status.json",
    "metrics.json",
    "summary.txt",
    "logs/events.log",
    "strategy/session_status.json",
    "strategy/events.jsonl",
    "strategy/market_status.json",
    "strategy/market_events.jsonl",
    "strategy/signal.jsonl",
    "strategy/order_intent.jsonl",
    "strategy/risk_decision.jsonl",
    "strategy/summary.json",
]


def require(condition, message):
    if not condition:
        raise SystemExit(message)


def read_json(relative: str):
    return json.loads((node_dir / relative).read_text())


def read_jsonl(relative: str):
    path = node_dir / relative
    return [
        json.loads(line)
        for line in path.read_text().splitlines()
        if line.strip()
    ]


for relative in expected_files:
    path = node_dir / relative
    require(path.is_file(), f"missing v09 strategy runtime artifact: {path}")

status = read_json("status.json")
metrics = read_json("metrics.json")
session = read_json("strategy/session_status.json")
market = read_json("strategy/market_status.json")
summary = read_json("strategy/summary.json")
events = read_jsonl("strategy/events.jsonl")
market_events = read_jsonl("strategy/market_events.jsonl")
signals = read_jsonl("strategy/signal.jsonl")
order_intents = read_jsonl("strategy/order_intent.jsonl")
risk_decisions = read_jsonl("strategy/risk_decision.jsonl")

require(status["schema_version"] == "ntpro.node_status.v1", status)
require(status["node_id"] == run_id, status)
require(status["lifecycle_state"] == "stopped", status)
require(status["external_venue_connection"] is False, status)
require(status["real_orders_submitted"] is False, status)
require(metrics["schema_version"] == "ntpro.node_metrics.v1", metrics)
require(metrics["node_id"] == run_id, metrics)
require(metrics["real_orders_submitted"] is False, metrics)
require(metrics["external_venue_connection"] is False, metrics)

require(session["schema_version"] == "ntpro.v09_strategy_session_status.v1", session)
require(session["session_id"] == run_id, session)
require(session["strategy_id"] == "ema_cross_btcusdt_v1", session)
require(session["state"] == "stopped", session)
require(market["schema_version"] == "ntpro.v09_market_stream_status.v1", market)
require(market["state"] == "stopped", market)
require(market["connection"] == "stopped", market)
require(market["source"] == "fixture_bar_stream", market)
require(market["event_count"] > 0, market)
require(summary["schema_version"] == "ntpro.v09_strategy_session_summary.v1", summary)
require(summary["session_id"] == run_id, summary)
require(summary["state"] == "stopped", summary)
require(summary["signal_count"] > 0, summary)
require(summary["intent_count"] == summary["signal_count"], summary)
require(summary["risk_decision_count"] == summary["intent_count"], summary)
require(summary["rejection_count"] == summary["risk_decision_count"], summary)
require(summary["actual_submission_count"] == 0, summary)

require(len(events) >= 5, f"expected lifecycle events, got {len(events)}")
require(len(market_events) == market["event_count"], market_events)
require(len(signals) == summary["signal_count"], signals)
require(len(order_intents) == summary["intent_count"], order_intents)
require(len(risk_decisions) == summary["risk_decision_count"], risk_decisions)

for signal in signals:
    require(signal["schema_version"] == "ntpro.v09_strategy_signal.v1", signal)
    require(signal["session_id"] == run_id, signal)
    require(signal["strategy_id"] == "ema_cross_btcusdt_v1", signal)
    require(signal["symbol"] == "BTCUSDT.BINANCE", signal)

for intent in order_intents:
    require(intent["schema_version"] == "ntpro.v09_order_intent.v1", intent)
    require(intent["session_id"] == run_id, intent)
    require(intent["strategy_id"] == "ema_cross_btcusdt_v1", intent)
    require(intent["symbol"] == "BTCUSDT.BINANCE", intent)
    require(intent["submission_allowed"] is False, intent)
    require(intent["submission_status"] == "blocked_by_v09_strategy_runtime_boundary", intent)
    require("exchange_order_id" not in intent, intent)
    require("venue_order_id" not in intent, intent)

for decision in risk_decisions:
    require(decision["schema_version"] == "ntpro.v09_risk_decision.v1", decision)
    require(decision["session_id"] == run_id, decision)
    require(decision["strategy_id"] == "ema_cross_btcusdt_v1", decision)
    require(decision["decision"] == "rejected", decision)
    require(decision["mode"] == "shadow", decision)
    require(decision["order_submission"] == "disabled", decision)
    require(decision["actual_submission"] is False, decision)
    require("order_submission_disabled" in decision["reasons"], decision)
    require("exchange_order_id" not in decision, decision)
    require("venue_order_id" not in decision, decision)

for path in node_dir.rglob("*"):
    if not path.is_file():
        continue
    text = path.read_text(errors="ignore")
    for forbidden in [
        "api_key",
        "api_secret",
        "private_key",
        "exchange_order_id",
        "venue_order_id",
        "real_orders_submitted=true",
    ]:
        require(forbidden not in text, f"forbidden token {forbidden!r} found in {path}")

print(
    "v09_strategy_runtime_smoke_assertions status=ok "
    f"run_id={run_id} signals={len(signals)} intents={len(order_intents)} "
    f"risk_decisions={len(risk_decisions)} actual_submission_count=0 "
    f"strategy_dir={strategy_dir}"
)
PY

echo "v09_strategy_runtime_smoke status=ok root=$SMOKE_ROOT node_dir=$NODE_DIR"
