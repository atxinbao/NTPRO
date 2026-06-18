#!/usr/bin/env bash
set -euo pipefail

# V090-012: Shadow-mode no-order gate.
# This gate proves v0.9 strategy order intents remain local audit records and
# never become exchange order submissions.

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

GATE_ROOT="${NTPRO_V09_NO_ORDER_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v09-no-order.XXXXXX")}"
RUN_ID="${NTPRO_V09_NO_ORDER_RUN_ID:-v09-shadow-no-order-gate}"
NODE_DIR="$GATE_ROOT/nodes/$RUN_ID"
OUTPUT_DIR="$GATE_ROOT/command-output"

mkdir -p "$NODE_DIR" "$OUTPUT_DIR"

"$NTPRO_NODE_BIN" \
  --config "$CONFIG" \
  --run-id "$RUN_ID" \
  --output "$NODE_DIR" \
  >"$OUTPUT_DIR/ntpro-node.stdout.log" \
  2>"$OUTPUT_DIR/ntpro-node.stderr.log"

python3 - "$NODE_DIR" "$RUN_ID" "$OUTPUT_DIR" <<'PY'
import json
import sys
from pathlib import Path

node_dir = Path(sys.argv[1])
run_id = sys.argv[2]
output_dir = Path(sys.argv[3])


def require(condition, message):
    if not condition:
        raise SystemExit(message)


def read_json(relative: str):
    return json.loads((node_dir / relative).read_text())


def read_jsonl(relative: str):
    return [
        json.loads(line)
        for line in (node_dir / relative).read_text().splitlines()
        if line.strip()
    ]


summary = read_json("strategy/summary.json")
order_intents = read_jsonl("strategy/order_intent.jsonl")
risk_decisions = read_jsonl("strategy/risk_decision.jsonl")
status = read_json("status.json")
metrics = read_json("metrics.json")
stdout = (output_dir / "ntpro-node.stdout.log").read_text()
stderr = (output_dir / "ntpro-node.stderr.log").read_text()

require(run_id in stdout, stdout)
require(stderr.strip() == "", stderr)
require(summary["signal_count"] > 0, summary)
require(summary["intent_count"] > 0, summary)
require(summary["intent_count"] == len(order_intents), summary)
require(summary["risk_decision_count"] == len(risk_decisions), summary)
require(summary["actual_submission_count"] == 0, summary)
require(status["external_venue_connection"] is False, status)
require(status["real_orders_submitted"] is False, status)
require(metrics["external_venue_connection"] is False, metrics)
require(metrics["real_orders_submitted"] is False, metrics)

intent_ids = {intent["intent_id"] for intent in order_intents}
decision_intent_ids = {decision["intent_id"] for decision in risk_decisions}
require(intent_ids == decision_intent_ids, (intent_ids, decision_intent_ids))

for intent in order_intents:
    require(intent["submission_allowed"] is False, intent)
    require(intent["submission_status"] == "blocked_by_v09_strategy_runtime_boundary", intent)
    require(intent["order_type"] == "market", intent)
    require("exchange_order_id" not in intent, intent)
    require("venue_order_id" not in intent, intent)
    require("client_order_id" not in intent, intent)
    require("account_id" not in intent, intent)

for decision in risk_decisions:
    require(decision["decision"] == "rejected", decision)
    require(decision["mode"] == "shadow", decision)
    require(decision["order_submission"] == "disabled", decision)
    require(decision["actual_submission"] is False, decision)
    require(decision["account_state"] == "missing", decision)
    require("order_submission_disabled" in decision["reasons"], decision)
    require("exchange_order_id" not in decision, decision)
    require("venue_order_id" not in decision, decision)
    require("client_order_id" not in decision, decision)
    require("account_id" not in decision, decision)

for path in [*node_dir.rglob("*"), *output_dir.rglob("*")]:
    if not path.is_file():
        continue
    text = path.read_text(errors="ignore")
    for forbidden in [
        "exchange_order_id",
        "venue_order_id",
        "client_order_id",
        "account_id",
        "api_key",
        "api_secret",
        "real_orders_submitted=true",
        "external_venue_connection=true",
    ]:
        require(forbidden not in text, f"forbidden token {forbidden!r} found in {path}")

print(
    "v09_shadow_mode_no_order_gate_assertions status=ok "
    f"run_id={run_id} order_intents={len(order_intents)} "
    f"risk_decisions={len(risk_decisions)} actual_submission_count=0"
)
PY

echo "v09_shadow_mode_no_order_gate status=ok root=$GATE_ROOT node_dir=$NODE_DIR"
