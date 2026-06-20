#!/usr/bin/env bash
set -euo pipefail

# V120-006: local production read-only reconciliation classifications.
# This script is CI-safe and never opens production network, reads order state, or submits orders.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

cargo test -p nautilus-cli production_readonly_reconciliation --lib
cargo test -p nautilus-cli production_shadow_strategy_session --lib

if [[ "${NTPRO_V12_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V12_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V12_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi

RECON_ROOT="${NTPRO_V12_RECONCILIATION_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v12-readonly-reconciliation.XXXXXX")}"
INPUT_DIR="$RECON_ROOT/input"
OUTPUT_DIR="$RECON_ROOT/output"
mkdir -p "$INPUT_DIR" "$OUTPUT_DIR"

ACCOUNT_SNAPSHOT="$INPUT_DIR/production_account_snapshot_redacted.json"
SHADOW_INTENT="$INPUT_DIR/shadow_execution_intent.jsonl"
STRATEGY_STATUS="$INPUT_DIR/strategy_session_status.json"
PORTFOLIO_RUNTIME="$OUTPUT_DIR/shadow_portfolio_runtime.json"
SESSION_EVENTS="$OUTPUT_DIR/shadow_strategy_session.jsonl"
RECON_EVENTS="$OUTPUT_DIR/reconciliation_events.jsonl"
MUTATING_PORTFOLIO="$INPUT_DIR/mutating_shadow_portfolio_runtime.json"
MUTATION_RECON_EVENTS="$OUTPUT_DIR/reconciliation_mutation_events.jsonl"

python3 - "$ACCOUNT_SNAPSHOT" "$SHADOW_INTENT" "$STRATEGY_STATUS" "$MUTATING_PORTFOLIO" <<'PY'
import json
import sys
from pathlib import Path

account_path = Path(sys.argv[1])
intent_path = Path(sys.argv[2])
status_path = Path(sys.argv[3])
mutating_portfolio_path = Path(sys.argv[4])

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

mutating_portfolio_path.write_text(json.dumps({
    "schema_version": "ntpro.v120_shadow_portfolio_runtime.v1",
    "status": "ready_redacted_shadow_portfolio",
    "production_orders_submitted": 1,
    "production_order_mutations_attempted": 0,
    "automatic_correction_orders_submitted": 0,
    "actual_submission_count": 0,
    "dashboard_order_controls_enabled": False,
    "real_orders_submitted": False,
    "provenance": {
        "values_are_exchange_truth": False,
    },
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
  --heartbeat-count 1 \
  >"$OUTPUT_DIR/session.stdout.log" \
  2>"$OUTPUT_DIR/session.stderr.log"

"$NAUTILUS_BIN" live production-readonly-reconciliation \
  --run-id v120-shadow \
  --account-snapshot "$ACCOUNT_SNAPSHOT" \
  --shadow-portfolio-runtime "$PORTFOLIO_RUNTIME" \
  --shadow-strategy-session "$SESSION_EVENTS" \
  --shadow-intent "$SHADOW_INTENT" \
  --output "$RECON_EVENTS" \
  >"$OUTPUT_DIR/reconciliation.stdout.log" \
  2>"$OUTPUT_DIR/reconciliation.stderr.log"

"$NAUTILUS_BIN" live production-readonly-reconciliation \
  --run-id v120-shadow \
  --account-snapshot "$ACCOUNT_SNAPSHOT" \
  --shadow-portfolio-runtime "$MUTATING_PORTFOLIO" \
  --output "$MUTATION_RECON_EVENTS" \
  >"$OUTPUT_DIR/reconciliation-mutation.stdout.log" \
  2>"$OUTPUT_DIR/reconciliation-mutation.stderr.log"

python3 - "$RECON_EVENTS" "$MUTATION_RECON_EVENTS" <<'PY'
import json
import sys
from pathlib import Path

ok_events = [json.loads(line) for line in Path(sys.argv[1]).read_text().splitlines() if line.strip()]
mutation_events = [json.loads(line) for line in Path(sys.argv[2]).read_text().splitlines() if line.strip()]

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require(len(ok_events) == 1, ok_events)
ok = ok_events[0]
require(ok["schema_version"] == "ntpro.v120_readonly_reconciliation_event.v1", ok)
require(ok["classification"] == "ok", ok)
require(ok["severity"] == "info", ok)
require(ok["recommended_action"] == "record_only", ok)
require(ok["risk_halted"] is False, ok)
require(ok["production_order_submissions_attempted"] == 0, ok)
require(ok["production_orders_submitted"] == 0, ok)
require(ok["production_order_mutations_attempted"] == 0, ok)
require(ok["production_order_state_reads_attempted"] == 0, ok)
require(ok["listen_key_lifecycle_attempted"] == 0, ok)
require(ok["automatic_correction_orders_submitted"] == 0, ok)
require(ok["dashboard_order_controls_enabled"] is False, ok)
require(ok["real_orders_submitted"] is False, ok)
require(ok["values_are_exchange_truth"] is False, ok)

require(len(mutation_events) == 1, mutation_events)
mutation = mutation_events[0]
require(mutation["classification"] == "production_mutation_forbidden", mutation)
require(mutation["severity"] == "halt", mutation)
require(mutation["recommended_action"] == "halt_shadow_flow", mutation)
require(mutation["production_orders_submitted"] == 0, mutation)
require(mutation["production_order_mutations_attempted"] == 0, mutation)
require(mutation["automatic_correction_orders_submitted"] == 0, mutation)
require(mutation["dashboard_order_controls_enabled"] is False, mutation)
PY

if rg -n "submit_correction_order|cancel_production_order|replace_production_order|amend_production_order|retry_production_order|auto_flatten_position" "$RECON_EVENTS" "$MUTATION_RECON_EVENTS"; then
  echo "forbidden production mutation action found in reconciliation events" >&2
  exit 1
fi

echo "v12_production_readonly_reconciliation status=ok root=$RECON_ROOT production_orders_submitted=0 production_order_mutations_attempted=0 production_order_state_reads_attempted=0 dashboard_order_controls_enabled=false"
