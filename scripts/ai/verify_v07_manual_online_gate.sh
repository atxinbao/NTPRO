#!/usr/bin/env bash
set -euo pipefail

# V070-006: manual online gate.
# By default this script proves the manual online path fails closed before any
# socket is created. To run the real read-only Binance testnet HTTP probe, set:
#
#   NTPRO_V07_MANUAL_ONLINE=1 NTPRO_ALLOW_TESTNET_NETWORK=1 scripts/ai/verify_v07_manual_online_gate.sh
#
# The default mode is a CI-safe preflight. The real online mode remains
# read-only and requires a validated Binance server-time response shape before
# it can count as connectivity proof.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_V07_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V07_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V07_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
CONFIG="${NTPRO_V07_CONFIG:-$ROOT_DIR/examples/rust/binance/testnet_dry_run.toml}"

if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi
if [[ ! -f "$CONFIG" ]]; then
  echo "missing Binance testnet config: $CONFIG" >&2
  exit 1
fi

GATE_ROOT="${NTPRO_V07_MANUAL_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v07-manual.XXXXXX")}"
OUTPUT_DIR="$GATE_ROOT/command-output"
WORKFLOW_DIR="$GATE_ROOT/workflows/v070-manual-online"

mkdir -p "$OUTPUT_DIR"

if [[ "${NTPRO_V07_MANUAL_ONLINE:-0}" != "1" ]]; then
  env -u NTPRO_ALLOW_TESTNET_NETWORK "$NAUTILUS_BIN" workflow run \
    --workflow binance-testnet \
    --mode connectivity-probe \
    --allow-testnet-network \
    --config "$CONFIG" \
    --run-id v070-manual-online-blocked \
    --output "$WORKFLOW_DIR" \
    | tee "$OUTPUT_DIR/manual_online_blocked.txt"

  python3 - "$WORKFLOW_DIR" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
probe = json.loads((root / "testnet/connectivity_probe.json").read_text())
http_probe = json.loads((root / "testnet/http_connectivity_probe.json").read_text())
ws_probe = json.loads((root / "testnet/ws_connectivity_probe.json").read_text())
auth_probe = json.loads((root / "testnet/authenticated_readonly_probe.json").read_text())
summary = json.loads((root / "summary.json").read_text())
policy = json.loads((root / "testnet/credential_policy.json").read_text())

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require(probe["network_permission_requested"] is True, probe)
require(probe["env_network_permission"] is False, probe)
require(probe["network_gate_status"] == "blocked", probe)
require(probe["network_gate_reasons"] == ["NTPRO_ALLOW_TESTNET_NETWORK=1 is not set"], probe)
require(probe["network_attempted"] is False, probe)
require(probe["testnet_connection"] is False, probe)
require(http_probe["network_attempted"] is False, http_probe)
require(http_probe["testnet_connection"] is False, http_probe)
require(http_probe["response_shape"] == "binance_server_time_v1", http_probe)
require(http_probe["response_shape_validated"] is False, http_probe)
require(summary["network_attempted"] is False, summary)
require(summary["testnet_connection"] is False, summary)
require(summary["production_venue_connection"] is False, summary)
require(summary["testnet_public_network_connection"] is False, summary)
require(summary["external_network_attempted"] is False, summary)
require(summary["real_orders_submitted"] is False, summary)
require(ws_probe["network_attempted"] is False, ws_probe)
require(ws_probe["websocket_attempted"] is False, ws_probe)
require(ws_probe["subscription_attempted"] is False, ws_probe)
require(ws_probe["real_orders_submitted"] is False, ws_probe)
require(ws_probe["values_recorded"] is False, ws_probe)
require(ws_probe["secrets_redacted"] is True, ws_probe)
require(auth_probe["network_gate_status"] == "blocked", auth_probe)
require(auth_probe["network_attempted"] is False, auth_probe)
require(auth_probe["testnet_connection"] is False, auth_probe)
require(auth_probe["api_key_header_value_recorded"] is False, auth_probe)
require(auth_probe["signature_recorded"] is False, auth_probe)
require(auth_probe["signed_query_recorded"] is False, auth_probe)
require(auth_probe["signed_url_recorded"] is False, auth_probe)
require(auth_probe["raw_response_recorded"] is False, auth_probe)
require(auth_probe["balances_recorded"] is False, auth_probe)
require(auth_probe["uid_recorded"] is False, auth_probe)
require(auth_probe["account_mutation"] is False, auth_probe)
require(auth_probe["order_submission"] == "disabled", auth_probe)
require(auth_probe["real_orders_submitted"] is False, auth_probe)
require(auth_probe["production_venue_connection"] is False, auth_probe)
require(auth_probe["real_funds"] is False, auth_probe)
require(auth_probe["production_trading"] is False, auth_probe)
require(policy["values_recorded"] is False, policy)
require(policy["secrets_redacted"] is True, policy)

print(
    "v07_manual_online_gate status=blocked_missing_env "
    "network_permission_requested=true network_attempted=false "
    "set NTPRO_V07_MANUAL_ONLINE=1 and NTPRO_ALLOW_TESTNET_NETWORK=1 to run real read-only probe"
)
PY
  echo "v07_manual_online_gate status=blocked_missing_env root=$GATE_ROOT"
  exit 0
fi

if [[ "${NTPRO_ALLOW_TESTNET_NETWORK:-0}" != "1" ]]; then
  echo "manual online probe requires NTPRO_ALLOW_TESTNET_NETWORK=1" >&2
  exit 1
fi

"$NAUTILUS_BIN" workflow run \
  --workflow binance-testnet \
  --mode connectivity-probe \
  --allow-testnet-network \
  --config "$CONFIG" \
  --run-id v070-manual-online \
  --output "$WORKFLOW_DIR" \
  | tee "$OUTPUT_DIR/manual_online_probe.txt"

python3 - "$WORKFLOW_DIR" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
manifest = json.loads((root / "manifest.json").read_text())
summary = json.loads((root / "summary.json").read_text())
probe = json.loads((root / "testnet/connectivity_probe.json").read_text())
http_probe = json.loads((root / "testnet/http_connectivity_probe.json").read_text())
ws_probe = json.loads((root / "testnet/ws_connectivity_probe.json").read_text())
auth_probe = json.loads((root / "testnet/authenticated_readonly_probe.json").read_text())
policy = json.loads((root / "testnet/credential_policy.json").read_text())
lifecycle = json.loads((root / "orders/testnet_dry_run_lifecycle.json").read_text())
reconciliation = json.loads((root / "orders/reconciliation.json").read_text())


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

require(manifest["run_id"] == "v070-manual-online", manifest)
require(manifest["runtime_status"] in {"http_read_only_probe_ok", "http_read_only_probe_failed"}, manifest)
require(summary["requested_mode"] == "connectivity-probe", summary)
require(summary["network_permission_requested"] is True, summary)
require(summary["network_attempted"] is True, summary)
require(summary["production_venue_connection"] is False, summary)
require(summary["real_orders_submitted"] is False, summary)
require(probe["network_permission_requested"] is True, probe)
require(probe["env_network_permission"] is True, probe)
require(probe["network_gate_status"] == "allowed", probe)
require(probe["network_gate_reasons"] == [], probe)
require(probe["network_attempted"] is True, probe)
require(probe["error_code"] in stable_errors, probe)
require(http_probe["schema_version"] == "ntpro.v07_binance_testnet_http_probe.v1", http_probe)
require(http_probe["endpoint_kind"] == "http_read_only", http_probe)
require(http_probe["request_method"] == "GET", http_probe)
require(http_probe["request_target"] == "/api/v3/time", http_probe)
require(http_probe["network_attempted"] is True, http_probe)
require(http_probe["error_code"] in stable_errors, http_probe)

if http_probe["error_code"] != "none":
    print(
        "v07_manual_online_gate status=classified_failure "
        "connectivity_proof=false "
        f"error_code={http_probe['error_code']} "
        f"response_shape_validated={str(http_probe.get('response_shape_validated')).lower()}",
        file=sys.stderr,
    )
    raise SystemExit({
        "reason": "stable_failure_is_classification_only_not_connectivity_proof",
        "http_probe": {
            "error_code": http_probe["error_code"],
            "network_attempted": http_probe["network_attempted"],
            "testnet_connection": http_probe.get("testnet_connection"),
            "response_shape": http_probe.get("response_shape"),
            "response_shape_validated": http_probe.get("response_shape_validated"),
        },
    })

require(http_probe["testnet_connection"] is True, http_probe)
require(http_probe["response_status_code"] is not None, http_probe)
require(http_probe["response_shape"] == "binance_server_time_v1", http_probe)
require(http_probe["response_shape_validated"] is True, http_probe)
require(probe["testnet_connection"] is True, probe)
require(probe["error_code"] == "none", probe)
require(probe["http_status"] is not None, probe)
require(probe["response_shape"] == "binance_server_time_v1", probe)
require(probe["response_shape_validated"] is True, probe)
require(summary["testnet_connection"] == probe["testnet_connection"], summary)
require(summary["testnet_public_network_connection"] is True, summary)
require(summary["external_network_attempted"] is True, summary)
require(policy["values_recorded"] is False, policy)
require(policy["api_key_value_recorded"] is False, policy)
require(policy["api_secret_value_recorded"] is False, policy)
require(policy["secrets_redacted"] is True, policy)
require(ws_probe["websocket_probe_gate"] == "manual-online-only", ws_probe)
require(ws_probe["network_attempted"] is False, ws_probe)
require(ws_probe["subscription_attempted"] is False, ws_probe)
require(ws_probe["message_count"] == 0, ws_probe)
require(ws_probe["real_orders_submitted"] is False, ws_probe)
require(ws_probe["values_recorded"] is False, ws_probe)
require(ws_probe["secrets_redacted"] is True, ws_probe)
require(auth_probe["schema_version"] == "ntpro.v08_binance_testnet_authenticated_readonly_probe.v1", auth_probe)
require(auth_probe["endpoint_kind"] == "authenticated_http_read_only", auth_probe)
require(auth_probe["request_method"] == "GET", auth_probe)
require(auth_probe["request_target"] == "/api/v3/account", auth_probe)
require(auth_probe["api_key_header_value_recorded"] is False, auth_probe)
require(auth_probe["signature_recorded"] is False, auth_probe)
require(auth_probe["signed_query_recorded"] is False, auth_probe)
require(auth_probe["signed_url_recorded"] is False, auth_probe)
require(auth_probe["raw_response_recorded"] is False, auth_probe)
require(auth_probe["balances_recorded"] is False, auth_probe)
require(auth_probe["uid_recorded"] is False, auth_probe)
require(auth_probe["account_mutation"] is False, auth_probe)
require(auth_probe["order_submission"] == "disabled", auth_probe)
require(auth_probe["real_orders_submitted"] is False, auth_probe)
require(auth_probe["production_venue_connection"] is False, auth_probe)
require(auth_probe["real_funds"] is False, auth_probe)
require(auth_probe["production_trading"] is False, auth_probe)
require(lifecycle["real_orders_submitted"] is False, lifecycle)
require(reconciliation["real_orders_submitted"] is False, reconciliation)

print(
    "v07_manual_online_gate status=ok "
    f"network_attempted={str(probe['network_attempted']).lower()} "
    f"testnet_connection={str(probe['testnet_connection']).lower()} "
    f"error_code={probe['error_code']} "
    "real_orders_submitted=false values_recorded=false secrets_redacted=true"
)
PY

echo "v07_manual_online_gate status=ok root=$GATE_ROOT"
