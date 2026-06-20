#!/usr/bin/env bash
set -euo pipefail

# V120-005: persistent local shadow strategy session artifacts.
# This script is CI-safe and never opens production network or submits orders.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

cargo test -p nautilus-cli production_shadow_strategy_session --lib
cargo test -p nautilus-cli production_shadow_portfolio_runtime --lib

if [[ "${NTPRO_V12_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V12_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V12_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi

SESSION_ROOT="${NTPRO_V12_SHADOW_STRATEGY_SESSION_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v12-shadow-strategy-session.XXXXXX")}"
INPUT_DIR="$SESSION_ROOT/input"
OUTPUT_DIR="$SESSION_ROOT/output"
mkdir -p "$INPUT_DIR" "$OUTPUT_DIR"

ACCOUNT_SNAPSHOT="$INPUT_DIR/production_account_snapshot_redacted.json"
SHADOW_INTENT="$INPUT_DIR/shadow_execution_intent.jsonl"
STRATEGY_STATUS="$INPUT_DIR/strategy_session_status.json"
PORTFOLIO_RUNTIME="$OUTPUT_DIR/shadow_portfolio_runtime.json"
SESSION_EVENTS="$OUTPUT_DIR/shadow_strategy_session.jsonl"
STDOUT_LOG="$OUTPUT_DIR/session.stdout.log"
STDERR_LOG="$OUTPUT_DIR/session.stderr.log"

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
    "run_id": "v120-shadow",
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
    "session_id": "v120-shadow-session",
    "strategy_id": "ema_cross_btcusdt_v1",
    "state": "running",
    "reason": "fixture strategy running",
}, indent=2, sort_keys=True) + "\n")
PY

"$NAUTILUS_BIN" live production-shadow-portfolio-runtime \
  --run-id v120-shadow \
  --snapshot-id portfolio-1 \
  --account-snapshot "$ACCOUNT_SNAPSHOT" \
  --shadow-intent "$SHADOW_INTENT" \
  --output "$PORTFOLIO_RUNTIME" \
  >"$OUTPUT_DIR/portfolio.stdout.log" \
  2>"$OUTPUT_DIR/portfolio.stderr.log"

"$NAUTILUS_BIN" live production-shadow-strategy-session \
  --run-id v120-shadow \
  --session-id v120-shadow-session \
  --strategy-id ema_cross_btcusdt_v1 \
  --shadow-portfolio-runtime "$PORTFOLIO_RUNTIME" \
  --strategy-session-status "$STRATEGY_STATUS" \
  --output "$SESSION_EVENTS" \
  --heartbeat-count 2 \
  --stop-after-heartbeats \
  >"$STDOUT_LOG" \
  2>"$STDERR_LOG"

if [[ -s "$STDERR_LOG" ]]; then
  echo "v12 shadow strategy session wrote stderr" >&2
  cat "$STDERR_LOG" >&2
  exit 1
fi

python3 - "$SESSION_EVENTS" <<'PY'
import json
import sys
from pathlib import Path

events = [
    json.loads(line)
    for line in Path(sys.argv[1]).read_text().splitlines()
    if line.strip()
]

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require(len(events) == 4, events)
require(events[0]["schema_version"] == "ntpro.v120_shadow_strategy_session_event.v1", events[0])
require(events[0]["event_type"] == "shadow_strategy_session_started", events[0])
require(events[0]["state"] == "running", events[0])
require("artifact_gap" not in events[0], events[0])
require(events[1]["event_type"] == "shadow_strategy_session_heartbeat", events[1])
require(events[1]["heartbeat_seq"] == 1, events[1])
require(events[2]["event_type"] == "shadow_strategy_session_heartbeat", events[2])
require(events[2]["heartbeat_seq"] == 2, events[2])
require(events[3]["event_type"] == "shadow_strategy_session_stopped", events[3])
require(events[3]["state"] == "stopped", events[3])
for event in events:
    require(event["production_order_submissions_attempted"] == 0, event)
    require(event["production_orders_submitted"] == 0, event)
    require(event["production_order_mutations_attempted"] == 0, event)
    require(event["production_order_state_reads_attempted"] == 0, event)
    require(event["listen_key_lifecycle_attempted"] == 0, event)
    require(event["actual_submission_count"] == 0, event)
    require(event["automatic_correction_orders_submitted"] == 0, event)
    require(event["dashboard_order_controls_enabled"] is False, event)
    require(event["real_orders_submitted"] is False, event)
    require(event["values_are_exchange_truth"] is False, event)
    require(event["shadow_portfolio_runtime_ref"]["values_are_exchange_truth"] is False, event)
PY

echo "v12_persistent_shadow_strategy_session status=ok root=$SESSION_ROOT production_orders_submitted=0 production_order_mutations_attempted=0 dashboard_order_controls_enabled=false"
