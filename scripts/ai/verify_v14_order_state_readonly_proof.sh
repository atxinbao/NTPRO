#!/usr/bin/env bash
set -euo pipefail

# V140-001: owner-gated production order-state read-only proof.
# Default mode is CI-safe and proves fail-closed behavior without opening
# network. Real production GET order-state proof is owner-gated and runs only
# when all of these are set:
#
#   NTPRO_V14_MANUAL_ONLINE=1
#   NTPRO_ALLOW_PRODUCTION_ORDER_STATE_READ=1
#   NTPRO_OWNER_APPROVED_PRODUCTION_ORDER_STATE_READ_ONLY=1
#   NTPRO_CONFIRM_PRODUCTION_ORDER_STATE_NO_ORDER_MUTATION=1
#   NTPRO_CONFIRM_NO_SECRET_PERSISTENCE=1
#   NTPRO_CONFIRM_NO_LISTEN_KEY_LIFECYCLE=1
#   NTPRO_CONFIRM_DASHBOARD_ORDER_CONTROLS_DISABLED=1
#   BINANCE_PRODUCTION_READONLY_API_KEY=<read-only key>
#   BINANCE_PRODUCTION_READONLY_API_SECRET=<read-only secret>
#
# The online path signs a memory-only GET request and must never persist raw
# order responses, API key values, signatures, signed queries, or signed URLs.

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

PROOF_ROOT="${NTPRO_V14_ORDER_STATE_PROOF_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v14-order-state-readonly-proof.XXXXXX")}"
OUTPUT_DIR="$PROOF_ROOT/command-output"
mkdir -p "$OUTPUT_DIR"

BLOCKED_JSON="$OUTPUT_DIR/blocked-order-state-proof.json"
BLOCKED_STDOUT="$OUTPUT_DIR/blocked.stdout.log"
BLOCKED_STDERR="$OUTPUT_DIR/blocked.stderr.log"
PREFLIGHT_JSON="$OUTPUT_DIR/manual-preflight-order-state-proof.json"
PREFLIGHT_STDOUT="$OUTPUT_DIR/manual-preflight.stdout.log"
PREFLIGHT_STDERR="$OUTPUT_DIR/manual-preflight.stderr.log"
ONLINE_JSON="$OUTPUT_DIR/manual-online-order-state-proof.json"
ONLINE_STDOUT="$OUTPUT_DIR/manual-online.stdout.log"
ONLINE_STDERR="$OUTPUT_DIR/manual-online.stderr.log"

SYNTHETIC_API_KEY="ntpro_v140001_script_synthetic_api_key_value"
SYNTHETIC_API_SECRET="ntpro_v140001_script_synthetic_api_secret_value"

env \
  -u NTPRO_ALLOW_PRODUCTION_ORDER_STATE_READ \
  -u NTPRO_OWNER_APPROVED_PRODUCTION_ORDER_STATE_READ_ONLY \
  -u NTPRO_CONFIRM_PRODUCTION_ORDER_STATE_NO_ORDER_MUTATION \
  -u NTPRO_CONFIRM_NO_SECRET_PERSISTENCE \
  -u NTPRO_CONFIRM_NO_LISTEN_KEY_LIFECYCLE \
  -u NTPRO_CONFIRM_DASHBOARD_ORDER_CONTROLS_DISABLED \
  -u NTPRO_V14_MANUAL_ONLINE \
  -u BINANCE_PRODUCTION_READONLY_API_KEY \
  -u BINANCE_PRODUCTION_READONLY_API_SECRET \
  "$NAUTILUS_BIN" live production-order-state-read-only-proof \
    --output "$BLOCKED_JSON" \
    >"$BLOCKED_STDOUT" \
    2>"$BLOCKED_STDERR"

if [[ -s "$BLOCKED_STDERR" ]]; then
  echo "v14 order-state blocked path wrote stderr" >&2
  cat "$BLOCKED_STDERR" >&2
  exit 1
fi

python3 - "$BLOCKED_JSON" <<'PY'
import json
import sys
from pathlib import Path

report = json.loads(Path(sys.argv[1]).read_text())

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require(report["schema_version"] == "ntpro.v140_production_order_state_readonly_proof.v1", report)
require(report["status"] == "blocked_missing_gate", report)
require(report["endpoint"] == "open_orders", report)
require(report["endpoint_class"] == "production_order_state_read_only", report)
require(report["method"] == "GET", report)
require(report["path"] == "/api/v3/openOrders", report)
require(report["requires_api_key"] is True, report)
require(report["requires_signature"] is True, report)
require(report["endpoint_read_allowed"] is True, report)
require(report["offline_contract_ready"] is False, report)
require(report["read_allowed"] is False, report)
require(report["contract_ready"] is False, report)
require(report["online_read_allowed"] is False, report)
require(report["network_attempted"] is False, report)
require(report["order_state_read_attempted"] is False, report)
require(report["production_order_state_reads_attempted"] == 0, report)
require(report["endpoint_shape_validated"] is False, report)
require(report["order_entries_observed"] == 0, report)
require(report["non_empty_order_state_observed"] is False, report)
require(report["order_lifecycle_readiness"] is False, report)
require(report["production_order_submission_attempted"] is False, report)
require(report["production_order_mutation_attempted"] is False, report)
require(report["cancel_replace_amend_attempted"] is False, report)
require(report["listen_key_lifecycle_attempted"] is False, report)
require(report["dashboard_order_controls_enabled"] is False, report)
require(report["automatic_remediation_attempted"] is False, report)
require(report["real_orders_submitted"] is False, report)
require(report["production_trading_enabled"] is False, report)
require(report["secrets_redacted"] is True, report)
PY

env \
  -u NTPRO_V14_MANUAL_ONLINE \
  NTPRO_ALLOW_PRODUCTION_ORDER_STATE_READ=1 \
  NTPRO_OWNER_APPROVED_PRODUCTION_ORDER_STATE_READ_ONLY=1 \
  NTPRO_CONFIRM_PRODUCTION_ORDER_STATE_NO_ORDER_MUTATION=1 \
  NTPRO_CONFIRM_NO_SECRET_PERSISTENCE=1 \
  NTPRO_CONFIRM_NO_LISTEN_KEY_LIFECYCLE=1 \
  NTPRO_CONFIRM_DASHBOARD_ORDER_CONTROLS_DISABLED=1 \
  BINANCE_PRODUCTION_READONLY_API_KEY="$SYNTHETIC_API_KEY" \
  BINANCE_PRODUCTION_READONLY_API_SECRET="$SYNTHETIC_API_SECRET" \
  "$NAUTILUS_BIN" live production-order-state-read-only-proof \
    --output "$PREFLIGHT_JSON" \
    --manual-online \
    --allow-production-order-state-read \
    --confirm-owner-approved-read-only \
    --confirm-no-order-mutation \
    --confirm-no-secret-persistence \
    --confirm-no-listen-key-lifecycle \
    --confirm-dashboard-order-controls-disabled \
    >"$PREFLIGHT_STDOUT" \
    2>"$PREFLIGHT_STDERR"

if [[ -s "$PREFLIGHT_STDERR" ]]; then
  echo "v14 order-state manual preflight wrote stderr" >&2
  cat "$PREFLIGHT_STDERR" >&2
  exit 1
fi

python3 - "$PREFLIGHT_JSON" "$SYNTHETIC_API_KEY" "$SYNTHETIC_API_SECRET" <<'PY'
import json
import sys
from pathlib import Path

report = json.loads(Path(sys.argv[1]).read_text())
synthetic_values = sys.argv[2:]

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require(report["schema_version"] == "ntpro.v140_production_order_state_readonly_proof.v1", report)
require(report["status"] == "blocked_missing_manual_online_gate", report)
require(report["manual_online_requested"] is True, report)
require(report["missing_env_vars"] == ["NTPRO_V14_MANUAL_ONLINE"], report)
require(report["online_execution_supported"] is True, report)
require(report["endpoint_read_allowed"] is True, report)
require(report["offline_contract_ready"] is False, report)
require(report["read_allowed"] is False, report)
require(report["contract_ready"] is False, report)
require(report["online_read_allowed"] is False, report)
require(report["network_attempted"] is False, report)
require(report["order_state_read_attempted"] is False, report)
require(report["production_order_state_reads_attempted"] == 0, report)
require(report["response_shape_validated"] is False, report)
require(report["endpoint_shape_validated"] is False, report)
require(report["order_entries_observed"] == 0, report)
require(report["non_empty_order_state_observed"] is False, report)
require(report["order_lifecycle_readiness"] is False, report)
require(report["error_code"] == "not_attempted", report)
require(report["signature_recorded"] is False, report)
require(report["signed_query_recorded"] is False, report)
require(report["signed_url_recorded"] is False, report)
require(report["production_order_submission_attempted"] is False, report)
require(report["production_order_mutation_attempted"] is False, report)
require(report["listen_key_lifecycle_attempted"] is False, report)
require(report["dashboard_order_controls_enabled"] is False, report)

raw = Path(sys.argv[1]).read_text()
for value in synthetic_values:
    require(value not in raw, "synthetic credential leaked into preflight artifact")
PY

if [[ "${NTPRO_V14_MANUAL_ONLINE:-0}" != "1" ]]; then
  echo "v14_order_state_readonly_proof status=preflight_ok root=$PROOF_ROOT network_attempted=false set NTPRO_V14_MANUAL_ONLINE=1 plus production read credentials/gates to run owner-gated online GET"
  exit 0
fi

for required in \
  NTPRO_ALLOW_PRODUCTION_ORDER_STATE_READ \
  NTPRO_OWNER_APPROVED_PRODUCTION_ORDER_STATE_READ_ONLY \
  NTPRO_CONFIRM_PRODUCTION_ORDER_STATE_NO_ORDER_MUTATION \
  NTPRO_CONFIRM_NO_SECRET_PERSISTENCE \
  NTPRO_CONFIRM_NO_LISTEN_KEY_LIFECYCLE \
  NTPRO_CONFIRM_DASHBOARD_ORDER_CONTROLS_DISABLED \
  BINANCE_PRODUCTION_READONLY_API_KEY \
  BINANCE_PRODUCTION_READONLY_API_SECRET
do
  if [[ -z "${!required:-}" ]]; then
    echo "manual online production order-state proof requires $required" >&2
    exit 1
  fi
done

"$NAUTILUS_BIN" live production-order-state-read-only-proof \
  --output "$ONLINE_JSON" \
  --manual-online \
  --allow-production-order-state-read \
  --confirm-owner-approved-read-only \
  --confirm-no-order-mutation \
  --confirm-no-secret-persistence \
  --confirm-no-listen-key-lifecycle \
  --confirm-dashboard-order-controls-disabled \
  >"$ONLINE_STDOUT" \
  2>"$ONLINE_STDERR"

if [[ -s "$ONLINE_STDERR" ]]; then
  echo "v14 order-state manual online wrote stderr" >&2
  cat "$ONLINE_STDERR" >&2
  exit 1
fi

python3 - "$ONLINE_JSON" "$PROOF_ROOT" <<'PY'
import json
import os
import sys
from pathlib import Path

report_path = Path(sys.argv[1])
proof_root = Path(sys.argv[2])
report = json.loads(report_path.read_text())

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require(report["schema_version"] == "ntpro.v140_production_order_state_readonly_proof.v1", report)
require(report["manual_online_requested"] is True, report)
require(report["endpoint_read_allowed"] is True, report)
require(report["online_execution_supported"] is True, report)
require(report["online_read_allowed"] is True, report)
require(report["network_attempted"] is True, report)
require(report["order_state_read_attempted"] is True, report)
require(report["production_order_state_reads_attempted"] == 1, report)
require(report["production_order_submission_attempted"] is False, report)
require(report["production_order_mutation_attempted"] is False, report)
require(report["cancel_replace_amend_attempted"] is False, report)
require(report["listen_key_lifecycle_attempted"] is False, report)
require(report["dashboard_order_controls_enabled"] is False, report)
require(report["automatic_remediation_attempted"] is False, report)
require(report["real_orders_submitted"] is False, report)
require(report["production_trading_enabled"] is False, report)
require(report["api_key_value_recorded"] is False, report)
require(report["api_secret_value_recorded"] is False, report)
require(report["signature_recorded"] is False, report)
require(report["signed_query_recorded"] is False, report)
require(report["signed_url_recorded"] is False, report)
require(report["secrets_redacted"] is True, report)

stable_errors = {
    "http_status_not_success",
    "timeout",
    "connect_error",
    "decode_error",
    "request_error",
    "body_error",
    "unknown_http_error",
    "response_shape_invalid",
}

if report["status"] == "online_order_state_read_ok":
    require(report["error_code"] == "none", report)
    require(report["response_shape_validated"] is True, report)
    require(report["endpoint_shape_validated"] is True, report)
    require(isinstance(report["order_entries_observed"], int), report)
    require(report["order_entries_observed"] >= 0, report)
    if report["order_entries_observed"] == 0:
        require(report["non_empty_order_state_observed"] is False, report)
        require(report["order_lifecycle_readiness"] is False, report)
    else:
        require(report["non_empty_order_state_observed"] is True, report)
        require(report["order_lifecycle_readiness"] is True, report)
elif report["status"] == "online_order_state_read_failed":
    require(report["error_code"] in stable_errors, report)
    require(report["response_shape_validated"] is False, report)
    require(report["endpoint_shape_validated"] is False, report)
    require(report["order_entries_observed"] == 0, report)
    require(report["non_empty_order_state_observed"] is False, report)
    require(report["order_lifecycle_readiness"] is False, report)
else:
    raise SystemExit(report)

secret_values = [
    os.environ.get("BINANCE_PRODUCTION_READONLY_API_KEY", ""),
    os.environ.get("BINANCE_PRODUCTION_READONLY_API_SECRET", ""),
]
secret_values = [value for value in secret_values if value]
for path in proof_root.rglob("*"):
    if not path.is_file():
        continue
    text = path.read_text(errors="ignore")
    for secret in secret_values:
        require(secret not in text, f"secret value leaked into {path}")
PY

echo "v14_order_state_readonly_proof status=manual_online_classified root=$PROOF_ROOT network_attempted=true production_order_submission_attempted=false production_order_mutation_attempted=false"
