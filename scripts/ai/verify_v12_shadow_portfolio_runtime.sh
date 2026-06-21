#!/usr/bin/env bash
set -euo pipefail

# V120-004: persistent local shadow portfolio runtime artifacts.
# This script is CI-safe and never opens production network or submits orders.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

cargo test -p nautilus-cli production_shadow_portfolio_runtime --lib
cargo test -p nautilus-cli production_shadow_artifacts_populate_readonly_dashboard_snapshot --lib

if [[ "${NTPRO_V12_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V12_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V12_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi

SNAPSHOT_ROOT="${NTPRO_V12_SHADOW_PORTFOLIO_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v12-shadow-portfolio.XXXXXX")}"
INPUT_DIR="$SNAPSHOT_ROOT/input"
OUTPUT_DIR="$SNAPSHOT_ROOT/output"
mkdir -p "$INPUT_DIR" "$OUTPUT_DIR"

ACCOUNT_SNAPSHOT="$INPUT_DIR/production_account_snapshot_redacted.json"
SHADOW_INTENT="$INPUT_DIR/shadow_execution_intent.jsonl"
RUNTIME_JSON="$OUTPUT_DIR/shadow_portfolio_runtime.json"
COMPAT_JSON="$OUTPUT_DIR/shadow_portfolio_snapshot.json"
STDOUT_LOG="$OUTPUT_DIR/runtime.stdout.log"
STDERR_LOG="$OUTPUT_DIR/runtime.stderr.log"

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
PY

"$NAUTILUS_BIN" live production-shadow-portfolio-runtime \
  --run-id v120-shadow \
  --snapshot-id portfolio-1 \
  --account-snapshot "$ACCOUNT_SNAPSHOT" \
  --shadow-intent "$SHADOW_INTENT" \
  --output "$RUNTIME_JSON" \
  --compat-snapshot-output "$COMPAT_JSON" \
  >"$STDOUT_LOG" \
  2>"$STDERR_LOG"

if [[ -s "$STDERR_LOG" ]]; then
  echo "v12 shadow portfolio runtime wrote stderr" >&2
  cat "$STDERR_LOG" >&2
  exit 1
fi

python3 - "$RUNTIME_JSON" "$COMPAT_JSON" <<'PY'
import json
import sys
from pathlib import Path

runtime = json.loads(Path(sys.argv[1]).read_text())
compat = json.loads(Path(sys.argv[2]).read_text())
raw_runtime = Path(sys.argv[1]).read_text()

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require(runtime["schema_version"] == "ntpro.v120_shadow_portfolio_runtime.v1", runtime)
require(runtime["status"] == "ready_redacted_shadow_portfolio", runtime)
require(runtime["balances"]["status"] == "observed_shape_only", runtime)
require(runtime["balances"]["observed_balance_entry_count"] == 2, runtime)
require(runtime["balances"]["asset_values_recorded"] is False, runtime)
require(runtime["balances"]["free_values_recorded"] is False, runtime)
require(runtime["balances"]["locked_values_recorded"] is False, runtime)
require(runtime["exposure"]["status"] == "derived_from_shadow_intents", runtime)
require(runtime["exposure"]["notional"] == "10", runtime)
require(runtime["notional_preflight"]["status"] == "shadow_decimal_string_evidence_only", runtime)
require(runtime["notional_preflight"]["aggregation"] == "rust_decimal_string_sum", runtime)
require(runtime["notional_preflight"]["decimal_string_sum"] == "10", runtime)
require(runtime["notional_preflight"]["parsed_notional_count"] == 1, runtime)
require(runtime["notional_preflight"]["f64_aggregation_used"] is False, runtime)
require(runtime["notional_preflight"]["live_alpha_money_math_ready"] is False, runtime)
require(runtime["notional_preflight"]["risk_or_execution_grade"] is False, runtime)
require(runtime["pnl"]["status"] == "unavailable", runtime)
require(runtime["risk_summary"]["new_orders_blocked"] is True, runtime)
require(runtime["actual_submission_count"] == 0, runtime)
require(runtime["production_orders_submitted"] == 0, runtime)
require(runtime["production_order_mutations_attempted"] == 0, runtime)
require(runtime["automatic_correction_orders_submitted"] == 0, runtime)
require(runtime["dashboard_order_controls_enabled"] is False, runtime)
require(runtime["full_production_portfolio_parity_claimed"] is False, runtime)
require(runtime["provenance"]["values_are_exchange_truth"] is False, runtime)
for forbidden in [
    '"asset": "BTC"',
    '"free":',
    '"locked":',
    "api_secret",
    "signature=",
    "signed_query",
    "signed_url",
]:
    require(forbidden not in raw_runtime, f"forbidden raw token in runtime: {forbidden}")

require(compat["schema_version"] == "ntpro.v110_shadow_portfolio_snapshot.v1", compat)
require(compat["snapshot_mode"] == "production_readonly_shadow", compat)
require(compat["balances"][0]["asset"] == "redacted", compat)
require(compat["exposure"]["status"] == "derived_from_shadow_intents", compat)
require(compat["pnl"]["status"] == "unavailable", compat)
require(compat["production_orders_submitted"] == 0, compat)
require(compat["dashboard_order_controls_enabled"] is False, compat)
require(compat["full_production_portfolio_parity_claimed"] is False, compat)
PY

echo "v12_shadow_portfolio_runtime status=ok root=$SNAPSHOT_ROOT network_attempted=false production_orders_submitted=0 dashboard_order_controls_enabled=false"
