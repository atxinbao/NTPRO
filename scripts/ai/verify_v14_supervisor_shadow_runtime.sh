#!/usr/bin/env bash
set -euo pipefail

# V140-002: supervisor-managed long-running shadow runtime evidence.
# This script is CI-safe: it uses local shadow config/artifacts only, opens no
# production network, and never submits, mutates, cancels, replaces, amends,
# retries, or corrects production orders.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

cargo test -p nautilus-cli supervisor_shadow --lib
cargo test -p nautilus-cli refresh_process_state_accepts_graceful_external_stopped_status --lib
cargo test -p nautilus-cli production_shadow_preflight_session --lib

if [[ "${NTPRO_V14_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V14_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus --bin ntpro-node
fi

NAUTILUS_BIN="${NTPRO_V14_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
NTPRO_NODE_BIN="${NTPRO_V14_NTPRO_NODE_BIN:-$ROOT_DIR/target/debug/ntpro-node}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi
if [[ ! -x "$NTPRO_NODE_BIN" ]]; then
  echo "missing ntpro-node binary: $NTPRO_NODE_BIN" >&2
  exit 1
fi

RUN_ROOT="${NTPRO_V14_SHADOW_RUNTIME_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v14-supervisor-shadow-runtime.XXXXXX")}"
REGISTRY="$RUN_ROOT/registry.json"
NODE_ROOT="$RUN_ROOT/nodes/btc-ema-shadow-001"
CONFIG="$ROOT_DIR/configs/nodes/btc-ema-shadow.toml"
COMMAND_OUTPUT="$RUN_ROOT/command-output"
mkdir -p "$COMMAND_OUTPUT"

"$NAUTILUS_BIN" supervisor register \
  --registry "$REGISTRY" \
  --node-id btc-ema-shadow-001 \
  --config "$CONFIG" \
  --artifact-root "$NODE_ROOT" \
  >"$COMMAND_OUTPUT/register.stdout.log" \
  2>"$COMMAND_OUTPUT/register.stderr.log"

"$NAUTILUS_BIN" supervisor start \
  --registry "$REGISTRY" \
  --node-id btc-ema-shadow-001 \
  --ntpro-node-bin "$NTPRO_NODE_BIN" \
  --startup-timeout-ms 5000 \
  --node-max-runtime-ms 60000 \
  --node-heartbeat-interval-ms 50 \
  --node-shutdown-timeout-ms 3000 \
  >"$COMMAND_OUTPUT/start.stdout.log" \
  2>"$COMMAND_OUTPUT/start.stderr.log"

"$NAUTILUS_BIN" supervisor status \
  --registry "$REGISTRY" \
  --node-id btc-ema-shadow-001 \
  >"$COMMAND_OUTPUT/status-running.stdout.log" \
  2>"$COMMAND_OUTPUT/status-running.stderr.log"

"$NAUTILUS_BIN" supervisor shadow-runtime \
  --registry "$REGISTRY" \
  --node-id btc-ema-shadow-001 \
  >"$COMMAND_OUTPUT/shadow-running.stdout.log" \
  2>"$COMMAND_OUTPUT/shadow-running.stderr.log"

python3 - "$REGISTRY" "$NODE_ROOT/status.json" "$NODE_ROOT/metrics.json" "$COMMAND_OUTPUT/shadow-running.stdout.log" <<'PY'
import json
import sys
from pathlib import Path

registry = json.loads(Path(sys.argv[1]).read_text())
status = json.loads(Path(sys.argv[2]).read_text())
metrics = json.loads(Path(sys.argv[3]).read_text())
shadow_stdout = Path(sys.argv[4]).read_text()
node = registry["nodes"]["btc-ema-shadow-001"]

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require(node["process"]["state"] == "running", node)
require(status["lifecycle_state"] == "running", status)
require(status["process_mode"] == "spawned_process", status)
require(status["external_venue_connection"] is False, status)
require(status["real_orders_submitted"] is False, status)
require(metrics["lifecycle_state"] == "running", metrics)
require(metrics["external_venue_connection"] is False, metrics)
require(metrics["real_orders_submitted"] is False, metrics)
require("supervisor.shadow_runtime status=ok" in shadow_stdout, shadow_stdout)
require("strategy_session_state=running" in shadow_stdout, shadow_stdout)
require("production_order_mutations_attempted=0" in shadow_stdout, shadow_stdout)
require("dashboard_order_controls_enabled=false" in shadow_stdout, shadow_stdout)
PY

"$NAUTILUS_BIN" supervisor stop \
  --registry "$REGISTRY" \
  --node-id btc-ema-shadow-001 \
  --stop-timeout-ms 5000 \
  >"$COMMAND_OUTPUT/stop.stdout.log" \
  2>"$COMMAND_OUTPUT/stop.stderr.log"

python3 - "$REGISTRY" "$NODE_ROOT/status.json" "$NODE_ROOT/metrics.json" <<'PY'
import json
import sys
from pathlib import Path

registry = json.loads(Path(sys.argv[1]).read_text())
status = json.loads(Path(sys.argv[2]).read_text())
metrics = json.loads(Path(sys.argv[3]).read_text())
node = registry["nodes"]["btc-ema-shadow-001"]

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require(node["process"]["state"] == "stopped", node)
require(status["lifecycle_state"] == "stopped", status)
require(metrics["lifecycle_state"] == "stopped", metrics)
require(metrics["stops_total"] == 1, metrics)
require(status["real_orders_submitted"] is False, status)
PY

"$NAUTILUS_BIN" supervisor start \
  --registry "$REGISTRY" \
  --node-id btc-ema-shadow-001 \
  --ntpro-node-bin "$NTPRO_NODE_BIN" \
  --startup-timeout-ms 5000 \
  --node-max-runtime-ms 60000 \
  --node-heartbeat-interval-ms 50 \
  --node-shutdown-timeout-ms 3000 \
  >"$COMMAND_OUTPUT/restart.stdout.log" \
  2>"$COMMAND_OUTPUT/restart.stderr.log"

PID="$(python3 - "$REGISTRY" <<'PY'
import json
import sys
from pathlib import Path

registry = json.loads(Path(sys.argv[1]).read_text())
print(registry["nodes"]["btc-ema-shadow-001"]["process"]["pid"]["value"])
PY
)"

kill -TERM "$PID"
for _ in $(seq 1 50); do
  if ! kill -0 "$PID" 2>/dev/null; then
    break
  fi
  sleep 0.1
done
if kill -0 "$PID" 2>/dev/null; then
  echo "shadow runtime process $PID did not exit after SIGTERM" >&2
  exit 1
fi

"$NAUTILUS_BIN" supervisor status \
  --registry "$REGISTRY" \
  --node-id btc-ema-shadow-001 \
  >"$COMMAND_OUTPUT/status-after-sigterm.stdout.log" \
  2>"$COMMAND_OUTPUT/status-after-sigterm.stderr.log"

python3 - "$REGISTRY" "$NODE_ROOT/status.json" "$COMMAND_OUTPUT/status-after-sigterm.stdout.log" <<'PY'
import json
import sys
from pathlib import Path

registry = json.loads(Path(sys.argv[1]).read_text())
status = json.loads(Path(sys.argv[2]).read_text())
stdout = Path(sys.argv[3]).read_text()
node = registry["nodes"]["btc-ema-shadow-001"]

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require(node["process"]["state"] == "stopped", node)
require(node["process"]["pid"]["availability"] == "not_configured", node)
require(status["lifecycle_state"] == "stopped", status)
require("lifecycle_state=stopped" in stdout, stdout)
require(status["external_venue_connection"] is False, status)
require(status["real_orders_submitted"] is False, status)
PY

SHADOW_V14_DIR="$NODE_ROOT/v0_14"
INPUT_DIR="$RUN_ROOT/input"
mkdir -p "$SHADOW_V14_DIR" "$INPUT_DIR"
ACCOUNT_SNAPSHOT="$INPUT_DIR/production_account_snapshot_redacted.json"
SHADOW_INTENT="$INPUT_DIR/shadow_execution_intent.jsonl"
PORTFOLIO_RUNTIME="$SHADOW_V14_DIR/shadow_portfolio_runtime.json"
PREFLIGHT_EVENTS="$SHADOW_V14_DIR/shadow_preflight_session.jsonl"
PREFLIGHT_STOP_FILE="$SHADOW_V14_DIR/STOP"

python3 - "$ACCOUNT_SNAPSHOT" "$SHADOW_INTENT" <<'PY'
import json
import sys
from pathlib import Path

account_path = Path(sys.argv[1])
intent_path = Path(sys.argv[2])
account_path.write_text(json.dumps({
    "schema_version": "ntpro.v120_authenticated_account_snapshot_online_read.v1",
    "status": "online_account_snapshot_ok",
    "response_shape_validated": True,
    "response_shape_summary": {
        "status": "accepted",
        "balance_entry_count": 1,
        "shape_validated": True,
        "raw_account_response_recorded": False,
        "raw_balances_recorded": False,
        "raw_permissions_recorded": False,
    },
    "network_attempted": False,
    "account_read_attempted": False,
    "api_key_value_recorded": False,
    "api_secret_value_recorded": False,
    "signature_recorded": False,
    "signed_query_recorded": False,
    "signed_url_recorded": False,
    "production_order_submission_attempted": False,
    "production_order_mutation_attempted": False,
    "dashboard_order_controls_enabled": False,
    "secrets_redacted": True,
}, indent=2, sort_keys=True) + "\n")
intent_path.write_text(json.dumps({
    "schema_version": "ntpro.v110_shadow_execution_intent.v1",
    "run_id": "v140-shadow-runtime",
    "intent_id": "intent-1",
    "strategy_id": "ema_cross_btcusdt_v1",
    "symbol": "BTCUSDT.BINANCE",
    "venue": "BINANCE",
    "side": "buy",
    "order_type": "market",
    "quantity": "0.001",
    "notional": "10.00",
    "mode": "production_shadow",
    "submission_allowed": False,
    "actual_submission": False,
    "submission_status": "blocked_by_v110_shadow_execution_boundary",
    "execution_adapter_called": False,
    "order_endpoint_access_attempted": False,
    "production_order_mutation_attempted": False,
    "dashboard_order_controls_enabled": False,
}, sort_keys=True) + "\n")
PY

"$NAUTILUS_BIN" live production-shadow-portfolio-runtime \
  --run-id v140-shadow-runtime \
  --snapshot-id portfolio-1 \
  --account-snapshot "$ACCOUNT_SNAPSHOT" \
  --shadow-intent "$SHADOW_INTENT" \
  --output "$PORTFOLIO_RUNTIME" \
  >"$COMMAND_OUTPUT/portfolio.stdout.log" \
  2>"$COMMAND_OUTPUT/portfolio.stderr.log"

sleep 0.01
"$NAUTILUS_BIN" live production-shadow-preflight-session \
  --run-id v140-shadow-runtime \
  --session-id v140-shadow-runtime-session \
  --strategy-id ema_cross_btcusdt_v1 \
  --shadow-portfolio-runtime "$PORTFOLIO_RUNTIME" \
  --output "$PREFLIGHT_EVENTS" \
  --max-heartbeats 2 \
  --heartbeat-interval-ms 1 \
  --stale-after-ms 1 \
  --stop-file "$PREFLIGHT_STOP_FILE" \
  >"$COMMAND_OUTPUT/preflight-stale.stdout.log" \
  2>"$COMMAND_OUTPUT/preflight-stale.stderr.log"

"$NAUTILUS_BIN" supervisor shadow-runtime \
  --registry "$REGISTRY" \
  --node-id btc-ema-shadow-001 \
  >"$COMMAND_OUTPUT/shadow-stale.stdout.log" \
  2>"$COMMAND_OUTPUT/shadow-stale.stderr.log"

python3 - "$PREFLIGHT_EVENTS" "$COMMAND_OUTPUT/shadow-stale.stdout.log" <<'PY'
import json
import sys
from pathlib import Path

events = [json.loads(line) for line in Path(sys.argv[1]).read_text().splitlines() if line.strip()]
stdout = Path(sys.argv[2]).read_text()

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require(events[-1]["state"] == "stale_data_halted", events[-1])
require(events[-1]["shutdown_reason"] == "stale_shadow_portfolio_runtime", events[-1])
for event in events:
    require(event["session_network_attempted"] is False, event)
    require(event["production_order_submissions_attempted"] == 0, event)
    require(event["production_orders_submitted"] == 0, event)
    require(event["production_order_mutations_attempted"] == 0, event)
    require(event["production_order_state_reads_attempted"] == 0, event)
    require(event["listen_key_lifecycle_attempted"] == 0, event)
    require(event["dashboard_order_controls_enabled"] is False, event)
    require(event["real_orders_submitted"] is False, event)
require("preflight_status=available" in stdout, stdout)
require("stale_data_halted=true" in stdout, stdout)
require("production_order_mutations_attempted=0" in stdout, stdout)
require("dashboard_order_controls_enabled=false" in stdout, stdout)
PY

echo "v14_supervisor_shadow_runtime status=ok root=$RUN_ROOT start_stop=ok restart=ok sigterm=ok stale_data_halt=ok production_orders_submitted=0 production_order_mutations_attempted=0 dashboard_order_controls_enabled=false"
