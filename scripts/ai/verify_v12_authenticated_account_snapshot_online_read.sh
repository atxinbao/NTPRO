#!/usr/bin/env bash
set -euo pipefail

# V120-002: authenticated production account snapshot online read proof.
# Default mode is CI-safe and proves the command fails closed without opening
# network. Real production GET /api/v3/account proof is owner-gated and runs
# only when all of these are set:
#
#   NTPRO_V12_MANUAL_ONLINE=1
#   NTPRO_ALLOW_PRODUCTION_AUTHENTICATED_READ=1
#   NTPRO_OWNER_APPROVED_PRODUCTION_READ_ONLY=1
#   NTPRO_CONFIRM_PRODUCTION_ACCOUNT_NO_ORDER_MUTATION=1
#   NTPRO_CONFIRM_NO_SECRET_PERSISTENCE=1
#   BINANCE_PRODUCTION_READONLY_API_KEY=<read-only key>
#   BINANCE_PRODUCTION_READONLY_API_SECRET=<read-only secret>
#
# The online path signs a memory-only GET request and must never persist raw
# account responses, balances, key values, signatures, signed queries, or signed
# URLs.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_V12_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V12_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V12_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi

SNAPSHOT_ROOT="${NTPRO_V12_ACCOUNT_SNAPSHOT_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v12-account-snapshot.XXXXXX")}"
OUTPUT_DIR="$SNAPSHOT_ROOT/command-output"
mkdir -p "$OUTPUT_DIR"

BLOCKED_JSON="$OUTPUT_DIR/blocked-account-snapshot.json"
BLOCKED_STDOUT="$OUTPUT_DIR/blocked.stdout.log"
BLOCKED_STDERR="$OUTPUT_DIR/blocked.stderr.log"
PREFLIGHT_JSON="$OUTPUT_DIR/manual-preflight-account-snapshot.json"
PREFLIGHT_STDOUT="$OUTPUT_DIR/manual-preflight.stdout.log"
PREFLIGHT_STDERR="$OUTPUT_DIR/manual-preflight.stderr.log"
ONLINE_JSON="$OUTPUT_DIR/manual-online-account-snapshot.json"
ONLINE_STDOUT="$OUTPUT_DIR/manual-online.stdout.log"
ONLINE_STDERR="$OUTPUT_DIR/manual-online.stderr.log"

SYNTHETIC_API_KEY="ntpro_v120002_script_synthetic_api_key_value"
SYNTHETIC_API_SECRET="ntpro_v120002_script_synthetic_api_secret_value"

env \
  -u NTPRO_ALLOW_PRODUCTION_AUTHENTICATED_READ \
  -u NTPRO_OWNER_APPROVED_PRODUCTION_READ_ONLY \
  -u NTPRO_CONFIRM_PRODUCTION_ACCOUNT_NO_ORDER_MUTATION \
  -u NTPRO_CONFIRM_NO_SECRET_PERSISTENCE \
  -u NTPRO_V12_MANUAL_ONLINE \
  -u BINANCE_PRODUCTION_READONLY_API_KEY \
  -u BINANCE_PRODUCTION_READONLY_API_SECRET \
  "$NAUTILUS_BIN" live production-account-snapshot-contract \
    --output "$BLOCKED_JSON" \
    >"$BLOCKED_STDOUT" \
    2>"$BLOCKED_STDERR"

if [[ -s "$BLOCKED_STDERR" ]]; then
  echo "v12 account snapshot blocked path wrote stderr" >&2
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

require(report["schema_version"] == "ntpro.v110_authenticated_account_snapshot_contract.v1", report)
require(report["status"] == "blocked_missing_gate", report)
require(report["endpoint_class"] == "production_authenticated_read_only", report)
require(report["method"] == "GET", report)
require(report["path"] == "/api/v3/account", report)
require(report["requires_api_key"] is True, report)
require(report["requires_signature"] is True, report)
require(report["endpoint_read_allowed"] is True, report)
require(report["offline_contract_ready"] is False, report)
require(report["read_allowed"] is False, report)
require(report["contract_ready"] is False, report)
require(report["online_read_allowed"] is False, report)
require(report["online_execution_supported"] is False, report)
require(report["network_attempted"] is False, report)
require(report["account_read_attempted"] is False, report)
require(report["response_shape"] == "binance_account_snapshot_v1", report)
require(report["response_shape_validated"] is False, report)
require(report["error_code"] == "not_attempted", report)
require(report["api_key_value_recorded"] is False, report)
require(report["api_secret_value_recorded"] is False, report)
require(report["signature_recorded"] is False, report)
require(report["signed_query_recorded"] is False, report)
require(report["signed_url_recorded"] is False, report)
require(report["account_mutation_attempted"] is False, report)
require(report["order_endpoint_access_attempted"] is False, report)
require(report["production_order_submission_attempted"] is False, report)
require(report["production_order_mutation_attempted"] is False, report)
require(report["dashboard_order_controls_enabled"] is False, report)
require(report["secrets_redacted"] is True, report)
PY

env \
  -u NTPRO_V12_MANUAL_ONLINE \
  NTPRO_ALLOW_PRODUCTION_AUTHENTICATED_READ=1 \
  NTPRO_OWNER_APPROVED_PRODUCTION_READ_ONLY=1 \
  NTPRO_CONFIRM_PRODUCTION_ACCOUNT_NO_ORDER_MUTATION=1 \
  NTPRO_CONFIRM_NO_SECRET_PERSISTENCE=1 \
  BINANCE_PRODUCTION_READONLY_API_KEY="$SYNTHETIC_API_KEY" \
  BINANCE_PRODUCTION_READONLY_API_SECRET="$SYNTHETIC_API_SECRET" \
  "$NAUTILUS_BIN" live production-account-snapshot-contract \
    --output "$PREFLIGHT_JSON" \
    --manual-online \
    --allow-production-authenticated-read \
    --confirm-owner-approved-read-only \
    --confirm-no-order-mutation \
    --confirm-no-secret-persistence \
    >"$PREFLIGHT_STDOUT" \
    2>"$PREFLIGHT_STDERR"

if [[ -s "$PREFLIGHT_STDERR" ]]; then
  echo "v12 account snapshot manual preflight wrote stderr" >&2
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

require(report["schema_version"] == "ntpro.v120_authenticated_account_snapshot_online_read.v1", report)
require(report["status"] == "blocked_missing_manual_online_gate", report)
require(report["manual_online_requested"] is True, report)
require(report["missing_env_vars"] == ["NTPRO_V12_MANUAL_ONLINE"], report)
require(report["online_execution_supported"] is True, report)
require(report["endpoint_read_allowed"] is True, report)
require(report["offline_contract_ready"] is False, report)
require(report["read_allowed"] is False, report)
require(report["contract_ready"] is False, report)
require(report["online_read_allowed"] is False, report)
require(report["network_attempted"] is False, report)
require(report["account_read_attempted"] is False, report)
require(report["response_shape_validated"] is False, report)
require(report["error_code"] == "not_attempted", report)
require(report["signature_recorded"] is False, report)
require(report["signed_query_recorded"] is False, report)
require(report["signed_url_recorded"] is False, report)
require(report["production_order_mutation_attempted"] is False, report)
require(report["dashboard_order_controls_enabled"] is False, report)

raw = Path(sys.argv[1]).read_text()
for value in synthetic_values:
    require(value not in raw, "synthetic credential leaked into preflight artifact")
PY

if [[ "${NTPRO_V12_MANUAL_ONLINE:-0}" != "1" ]]; then
  echo "v12_authenticated_account_snapshot_online_read status=preflight_ok root=$SNAPSHOT_ROOT network_attempted=false set NTPRO_V12_MANUAL_ONLINE=1 plus production read credentials/gates to run owner-gated online GET"
  exit 0
fi

for required in \
  NTPRO_ALLOW_PRODUCTION_AUTHENTICATED_READ \
  NTPRO_OWNER_APPROVED_PRODUCTION_READ_ONLY \
  NTPRO_CONFIRM_PRODUCTION_ACCOUNT_NO_ORDER_MUTATION \
  NTPRO_CONFIRM_NO_SECRET_PERSISTENCE \
  BINANCE_PRODUCTION_READONLY_API_KEY \
  BINANCE_PRODUCTION_READONLY_API_SECRET
do
  if [[ -z "${!required:-}" ]]; then
    echo "manual online production account snapshot requires $required" >&2
    exit 1
  fi
done

"$NAUTILUS_BIN" live production-account-snapshot-contract \
  --output "$ONLINE_JSON" \
  --manual-online \
  --allow-production-authenticated-read \
  --confirm-owner-approved-read-only \
  --confirm-no-order-mutation \
  --confirm-no-secret-persistence \
  >"$ONLINE_STDOUT" \
  2>"$ONLINE_STDERR"

if [[ -s "$ONLINE_STDERR" ]]; then
  echo "v12 account snapshot manual online wrote stderr" >&2
  cat "$ONLINE_STDERR" >&2
  exit 1
fi

python3 - "$ONLINE_JSON" <<'PY'
import json
import sys
from pathlib import Path

report = json.loads(Path(sys.argv[1]).read_text())

def require(condition, message):
    if not condition:
        raise SystemExit(message)

stable_errors = {
    "none",
    "timeout",
    "connect_error",
    "decode_error",
    "request_error",
    "body_error",
    "unknown_http_error",
    "http_client_build_failed",
    "http_probe_thread_panicked",
    "http_status_not_success",
    "response_shape_invalid",
    "signed_request_builder_failed",
}

require(report["schema_version"] == "ntpro.v120_authenticated_account_snapshot_online_read.v1", report)
require(report["manual_online_requested"] is True, report)
require(report["method"] == "GET", report)
require(report["path"] == "/api/v3/account", report)
require(report["requires_api_key"] is True, report)
require(report["requires_signature"] is True, report)
require(report["endpoint_read_allowed"] is True, report)
require(report["offline_contract_ready"] is False, report)
require(report["read_allowed"] is False, report)
require(report["contract_ready"] is False, report)
require(report["online_read_allowed"] is True, report)
require(report["network_attempted"] is True, report)
require(report["account_read_attempted"] is True, report)
require(report["account_mutation_attempted"] is False, report)
require(report["order_endpoint_access_attempted"] is False, report)
require(report["production_order_submission_attempted"] is False, report)
require(report["production_order_mutation_attempted"] is False, report)
require(report["dashboard_order_controls_enabled"] is False, report)
require(report["signature_recorded"] is False, report)
require(report["signed_query_recorded"] is False, report)
require(report["signed_url_recorded"] is False, report)
require(report["error_code"] in stable_errors, report)

if report["error_code"] != "none":
    print(
        "v12_authenticated_account_snapshot_online_read status=classified_failure "
        "account_snapshot_proof=false "
        f"error_code={report['error_code']} "
        f"response_shape_validated={str(report.get('response_shape_validated')).lower()}",
        file=sys.stderr,
    )
    raise SystemExit({
        "reason": "stable_failure_is_classification_only_not_account_snapshot_proof",
        "report": {
            "status": report.get("status"),
            "error_code": report["error_code"],
            "network_attempted": report["network_attempted"],
            "account_read_attempted": report["account_read_attempted"],
            "response_shape": report.get("response_shape"),
            "response_shape_validated": report.get("response_shape_validated"),
        },
    })

require(report["status"] == "online_account_snapshot_ok", report)
require(report["response_status_code"] is not None, report)
require(report["response_shape"] == "binance_account_snapshot_v1", report)
require(report["response_shape_validated"] is True, report)
require(report["latency_ms"] is not None, report)

print(
    "v12_authenticated_account_snapshot_online_read status=ok "
    f"network_attempted={str(report['network_attempted']).lower()} "
    f"account_read_attempted={str(report['account_read_attempted']).lower()} "
    f"response_shape_validated={str(report['response_shape_validated']).lower()} "
    f"error_code={report['error_code']} "
    "production_order_mutation_attempted=false dashboard_order_controls_enabled=false"
)
PY

echo "v12_authenticated_account_snapshot_online_read status=ok root=$SNAPSHOT_ROOT"
