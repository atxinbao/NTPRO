#!/usr/bin/env bash
set -euo pipefail

# V130-002: guarded-live-alpha local shadow preflight session loop.
# This script is CI-safe. It writes local artifacts only, opens no production
# network, and never submits, cancels, replaces, amends, retries, or corrects
# production orders.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

cargo test -p nautilus-cli production_shadow_preflight_session --lib

if [[ "${NTPRO_V13_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V13_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V13_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi

SESSION_ROOT="${NTPRO_V13_SHADOW_PREFLIGHT_SESSION_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v13-shadow-preflight-session.XXXXXX")}"
INPUT_DIR="$SESSION_ROOT/input"
OUTPUT_DIR="$SESSION_ROOT/output"
mkdir -p "$INPUT_DIR" "$OUTPUT_DIR"

ACCOUNT_SNAPSHOT="$INPUT_DIR/production_account_snapshot_redacted.json"
SHADOW_INTENT="$INPUT_DIR/shadow_execution_intent.jsonl"
STRATEGY_STATUS="$INPUT_DIR/strategy_session_status.json"
PORTFOLIO_RUNTIME="$OUTPUT_DIR/shadow_portfolio_runtime.json"
PREFLIGHT_EVENTS="$OUTPUT_DIR/shadow_preflight_session.jsonl"
STOP_FILE="$OUTPUT_DIR/STOP"
STDOUT_LOG="$OUTPUT_DIR/preflight.stdout.log"
STDERR_LOG="$OUTPUT_DIR/preflight.stderr.log"

python3 - "$ACCOUNT_SNAPSHOT" "$SHADOW_INTENT" "$STRATEGY_STATUS" <<'PY'
import json
import sys
from pathlib import Path

account_path = Path(sys.argv[1])
intent_path = Path(sys.argv[2])
status_path = Path(sys.argv[3])

account_path.write_text(json.dumps({
    "schema_version": "ntpro.v120_authenticated_account_snapshot_online_read.v1",
    "status": "online_account_snapshot_ok",
    "response_shape_validated": True,
    "response_shape_summary": {
        "status": "accepted",
        "balance_entry_count": 2,
        "shape_validated": True,
        "raw_account_response_recorded": False,
        "raw_balances_recorded": False,
        "raw_permissions_recorded": False,
    },
    "network_attempted": True,
    "account_read_attempted": True,
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
    "run_id": "v130-shadow",
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

status_path.write_text(json.dumps({
    "schema_version": "ntpro.v09_strategy_session_status.v1",
    "session_id": "v130-shadow-session",
    "strategy_id": "ema_cross_btcusdt_v1",
    "state": "running",
    "reason": "fixture strategy running",
}, indent=2, sort_keys=True) + "\n")
PY

"$NAUTILUS_BIN" live production-shadow-portfolio-runtime \
  --run-id v130-shadow \
  --snapshot-id portfolio-1 \
  --account-snapshot "$ACCOUNT_SNAPSHOT" \
  --shadow-intent "$SHADOW_INTENT" \
  --output "$PORTFOLIO_RUNTIME" \
  >"$OUTPUT_DIR/portfolio.stdout.log" \
  2>"$OUTPUT_DIR/portfolio.stderr.log"

"$NAUTILUS_BIN" live production-shadow-preflight-session \
  --run-id v130-shadow \
  --session-id v130-shadow-session \
  --strategy-id ema_cross_btcusdt_v1 \
  --shadow-portfolio-runtime "$PORTFOLIO_RUNTIME" \
  --strategy-session-status "$STRATEGY_STATUS" \
  --output "$PREFLIGHT_EVENTS" \
  --max-heartbeats 2 \
  --heartbeat-interval-ms 1 \
  --stale-after-ms 60000 \
  --stop-file "$STOP_FILE" \
  >"$STDOUT_LOG" \
  2>"$STDERR_LOG"

if [[ -s "$STDERR_LOG" ]]; then
  echo "v13 shadow preflight session wrote stderr" >&2
  cat "$STDERR_LOG" >&2
  exit 1
fi

python3 - "$PREFLIGHT_EVENTS" "$STDOUT_LOG" <<'PY'
import json
import sys
from pathlib import Path

events = [
    json.loads(line)
    for line in Path(sys.argv[1]).read_text().splitlines()
    if line.strip()
]
stdout = Path(sys.argv[2]).read_text()

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require("session_network_attempted=false" in stdout, stdout)
require("production_order_mutations_attempted=0" in stdout, stdout)
require(len(events) == 4, events)
require(events[0]["schema_version"] == "ntpro.v130_shadow_preflight_session_event.v1", events[0])
require(events[0]["event_type"] == "shadow_preflight_session_started", events[0])
require(events[1]["event_type"] == "shadow_preflight_session_heartbeat", events[1])
require(events[1]["heartbeat_seq"] == 1, events[1])
require(events[2]["event_type"] == "shadow_preflight_session_heartbeat", events[2])
require(events[2]["heartbeat_seq"] == 2, events[2])
require(events[3]["event_type"] == "shadow_preflight_session_stopped", events[3])
require(events[3]["shutdown_reason"] == "max_heartbeats_reached", events[3])
for event in events:
    require(event["session_network_attempted"] is False, event)
    require(event["production_order_submissions_attempted"] == 0, event)
    require(event["production_orders_submitted"] == 0, event)
    require(event["production_order_mutations_attempted"] == 0, event)
    require(event["production_order_state_reads_attempted"] == 0, event)
    require(event["listen_key_lifecycle_attempted"] == 0, event)
    require(event["cancel_replace_amend_attempted"] is False, event)
    require(event["actual_submission_count"] == 0, event)
    require(event["automatic_correction_orders_submitted"] == 0, event)
    require(event["dashboard_order_controls_enabled"] is False, event)
    require(event["real_orders_submitted"] is False, event)
    require(event["values_are_exchange_truth"] is False, event)
PY

echo "v13_shadow_preflight_session status=ok root=$SESSION_ROOT session_network_attempted=false production_orders_submitted=0 production_order_mutations_attempted=0 dashboard_order_controls_enabled=false"
