#!/usr/bin/env bash
set -euo pipefail

# V120-001: production public read-only online probe.
# Default mode is CI-safe and proves the probe fails closed without opening
# network. Real production GET proof is owner-gated and runs only when:
#
#   NTPRO_V12_MANUAL_ONLINE=1
#   NTPRO_ALLOW_PRODUCTION_PUBLIC_READ=1
#   NTPRO_CONFIRM_PRODUCTION_PUBLIC_READ_ONLY=1
#   NTPRO_CONFIRM_NO_PRODUCTION_ORDER_MUTATION=1
#
# The online path uses only unauthenticated GET public endpoints and must never
# read credentials, access account/order endpoints, or enable Dashboard actions.

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

PROBE_ROOT="${NTPRO_V12_PUBLIC_READ_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v12-public-read.XXXXXX")}"
OUTPUT_DIR="$PROBE_ROOT/command-output"
mkdir -p "$OUTPUT_DIR"

BLOCKED_JSON="$OUTPUT_DIR/blocked-public-read-probe.json"
BLOCKED_STDOUT="$OUTPUT_DIR/blocked.stdout.log"
BLOCKED_STDERR="$OUTPUT_DIR/blocked.stderr.log"
PREFLIGHT_JSON="$OUTPUT_DIR/manual-preflight-public-read-probe.json"
PREFLIGHT_STDOUT="$OUTPUT_DIR/manual-preflight.stdout.log"
PREFLIGHT_STDERR="$OUTPUT_DIR/manual-preflight.stderr.log"
ONLINE_JSON="$OUTPUT_DIR/manual-online-public-read-probe.json"
ONLINE_STDOUT="$OUTPUT_DIR/manual-online.stdout.log"
ONLINE_STDERR="$OUTPUT_DIR/manual-online.stderr.log"

env \
  -u NTPRO_ALLOW_PRODUCTION_PUBLIC_READ \
  -u NTPRO_CONFIRM_PRODUCTION_PUBLIC_READ_ONLY \
  -u NTPRO_CONFIRM_NO_PRODUCTION_ORDER_MUTATION \
  -u NTPRO_V12_MANUAL_ONLINE \
  "$NAUTILUS_BIN" live production-public-read-probe \
    --endpoint server-time \
    --output "$BLOCKED_JSON" \
    >"$BLOCKED_STDOUT" \
    2>"$BLOCKED_STDERR"

if [[ -s "$BLOCKED_STDERR" ]]; then
  echo "v12 public read blocked path wrote stderr" >&2
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

require(report["schema_version"] == "ntpro.v110_production_public_read_probe.v1", report)
require(report["status"] == "blocked_missing_gate", report)
require(report["endpoint_class"] == "production_public_read_only", report)
require(report["method"] == "GET", report)
require(report["path"] == "/api/v3/time", report)
require(report["requires_api_key"] is False, report)
require(report["requires_signature"] is False, report)
require(report["endpoint_read_allowed"] is True, report)
require(report["offline_contract_ready"] is False, report)
require(report["read_allowed"] is False, report)
require(report["contract_ready"] is False, report)
require(report["online_read_allowed"] is False, report)
require(report["online_execution_supported"] is False, report)
require(report["network_attempted"] is False, report)
require(report["production_public_online_read_attempted"] is False, report)
require(report["credentials_used"] is False, report)
require(report["account_mutation_attempted"] is False, report)
require(report["production_order_submission_attempted"] is False, report)
require(report["production_order_mutation_attempted"] is False, report)
require(report["dashboard_order_controls_enabled"] is False, report)
require(report["response_shape"] == "binance_server_time_v1", report)
require(report["response_shape_validated"] is False, report)
require(report["error_code"] == "not_attempted", report)
PY

env \
  -u NTPRO_V12_MANUAL_ONLINE \
  NTPRO_ALLOW_PRODUCTION_PUBLIC_READ=1 \
  NTPRO_CONFIRM_PRODUCTION_PUBLIC_READ_ONLY=1 \
  NTPRO_CONFIRM_NO_PRODUCTION_ORDER_MUTATION=1 \
  "$NAUTILUS_BIN" live production-public-read-probe \
    --endpoint server-time \
    --output "$PREFLIGHT_JSON" \
    --manual-online \
    --allow-production-public-read \
    --confirm-read-only \
    --confirm-no-order-mutation \
    >"$PREFLIGHT_STDOUT" \
    2>"$PREFLIGHT_STDERR"

if [[ -s "$PREFLIGHT_STDERR" ]]; then
  echo "v12 public read manual preflight wrote stderr" >&2
  cat "$PREFLIGHT_STDERR" >&2
  exit 1
fi

python3 - "$PREFLIGHT_JSON" <<'PY'
import json
import sys
from pathlib import Path

report = json.loads(Path(sys.argv[1]).read_text())

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require(report["schema_version"] == "ntpro.v120_production_public_online_read_probe.v1", report)
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
require(report["production_public_online_read_attempted"] is False, report)
require(report["credentials_used"] is False, report)
require(report["account_mutation_attempted"] is False, report)
require(report["production_order_submission_attempted"] is False, report)
require(report["production_order_mutation_attempted"] is False, report)
require(report["dashboard_order_controls_enabled"] is False, report)
require(report["error_code"] == "not_attempted", report)
PY

if [[ "${NTPRO_V12_MANUAL_ONLINE:-0}" != "1" ]]; then
  echo "v12_public_online_read_probe status=preflight_ok root=$PROBE_ROOT network_attempted=false set NTPRO_V12_MANUAL_ONLINE=1 plus production read env gates to run owner-gated online GET"
  exit 0
fi

for required in \
  NTPRO_ALLOW_PRODUCTION_PUBLIC_READ \
  NTPRO_CONFIRM_PRODUCTION_PUBLIC_READ_ONLY \
  NTPRO_CONFIRM_NO_PRODUCTION_ORDER_MUTATION
do
  if [[ "${!required:-0}" != "1" ]]; then
    echo "manual online production public read requires $required=1" >&2
    exit 1
  fi
done

"$NAUTILUS_BIN" live production-public-read-probe \
  --endpoint server-time \
  --output "$ONLINE_JSON" \
  --manual-online \
  --allow-production-public-read \
  --confirm-read-only \
  --confirm-no-order-mutation \
  >"$ONLINE_STDOUT" \
  2>"$ONLINE_STDERR"

if [[ -s "$ONLINE_STDERR" ]]; then
  echo "v12 public read manual online wrote stderr" >&2
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
}

require(report["schema_version"] == "ntpro.v120_production_public_online_read_probe.v1", report)
require(report["manual_online_requested"] is True, report)
require(report["method"] == "GET", report)
require(report["path"] == "/api/v3/time", report)
require(report["requires_api_key"] is False, report)
require(report["requires_signature"] is False, report)
require(report["endpoint_read_allowed"] is True, report)
require(report["offline_contract_ready"] is False, report)
require(report["read_allowed"] is False, report)
require(report["contract_ready"] is False, report)
require(report["online_read_allowed"] is True, report)
require(report["network_attempted"] is True, report)
require(report["production_public_online_read_attempted"] is True, report)
require(report["credentials_used"] is False, report)
require(report["account_mutation_attempted"] is False, report)
require(report["production_order_submission_attempted"] is False, report)
require(report["production_order_mutation_attempted"] is False, report)
require(report["dashboard_order_controls_enabled"] is False, report)
require(report["error_code"] in stable_errors, report)

if report["error_code"] != "none":
    print(
        "v12_public_online_read_probe status=classified_failure "
        "connectivity_proof=false "
        f"error_code={report['error_code']} "
        f"response_shape_validated={str(report.get('response_shape_validated')).lower()}",
        file=sys.stderr,
    )
    raise SystemExit({
        "reason": "stable_failure_is_classification_only_not_connectivity_proof",
        "report": {
            "status": report.get("status"),
            "error_code": report["error_code"],
            "network_attempted": report["network_attempted"],
            "response_shape": report.get("response_shape"),
            "response_shape_validated": report.get("response_shape_validated"),
        },
    })

require(report["status"] == "online_read_probe_ok", report)
require(report["response_status_code"] is not None, report)
require(report["response_shape"] == "binance_server_time_v1", report)
require(report["response_shape_validated"] is True, report)
require(report["latency_ms"] is not None, report)

print(
    "v12_public_online_read_probe status=ok "
    f"network_attempted={str(report['network_attempted']).lower()} "
    f"response_shape_validated={str(report['response_shape_validated']).lower()} "
    f"error_code={report['error_code']} "
    "credentials_used=false production_order_mutation_attempted=false dashboard_order_controls_enabled=false"
)
PY

echo "v12_public_online_read_probe status=ok root=$PROBE_ROOT"
