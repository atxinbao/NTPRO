#!/usr/bin/env bash
set -euo pipefail

# V100-006: manual Binance testnet tiny submit-and-cancel proof.
# Default mode is CI-safe and does not open network connections. Real testnet
# submit/cancel only runs when the owner sets every explicit manual gate below:
#
#   NTPRO_V10_MANUAL_ONLINE=1
#   NTPRO_ALLOW_BINANCE_TESTNET_ORDER=1
#   NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER=1
#   NTPRO_CONFIRM_TESTNET_TINY_NOTIONAL=1
#   NTPRO_CONFIRM_TESTNET_CANCEL_AFTER_SUBMIT=1
#   BINANCE_TESTNET_API_KEY=...
#   BINANCE_TESTNET_API_SECRET=...
#
# Optional:
#   NTPRO_V10_CONFIG=configs/nodes/btc-ema-shadow.toml
#   NTPRO_V10_MANUAL_ORDER_PROOF_DIR=target/ntpro-v10-manual-order-proof/<run>
#   NTPRO_V10_SPOT_API_BASE_URL=https://demo-api.binance.com
#   NTPRO_V10_TESTNET_SYMBOL=BTCUSDT
#   NTPRO_V10_TESTNET_SIDE=BUY
#   NTPRO_V10_TESTNET_PRICE=...
#   NTPRO_V10_TESTNET_QUANTITY=...
#
# The script never supports production Binance endpoints. It only allows
# Binance Spot Test Network or Spot Demo Mode endpoints, and never records API
# keys, secrets, raw signatures, signed URLs, raw account state, or raw response
# bodies in artifacts.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

if [[ "${NTPRO_V10_MANUAL_ONLINE:-0}" != "1" ]]; then
  echo "v10_manual_tiny_submit_cancel status=closed manual_online=false network_attempted=false real_orders_submitted=false"
  echo "set NTPRO_V10_MANUAL_ONLINE=1 plus owner gates and Binance testnet credentials to run the real V100-006 proof"
  exit 0
fi

required_env=(
  NTPRO_ALLOW_BINANCE_TESTNET_ORDER
  NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER
  NTPRO_CONFIRM_TESTNET_TINY_NOTIONAL
  NTPRO_CONFIRM_TESTNET_CANCEL_AFTER_SUBMIT
  BINANCE_TESTNET_API_KEY
  BINANCE_TESTNET_API_SECRET
)

for name in "${required_env[@]}"; do
  if [[ -z "${!name:-}" ]]; then
    echo "manual V100-006 submit/cancel proof requires $name" >&2
    exit 1
  fi
done

for name in \
  NTPRO_ALLOW_BINANCE_TESTNET_ORDER \
  NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER \
  NTPRO_CONFIRM_TESTNET_TINY_NOTIONAL \
  NTPRO_CONFIRM_TESTNET_CANCEL_AFTER_SUBMIT; do
  if [[ "${!name}" != "1" ]]; then
    echo "manual V100-006 submit/cancel proof requires $name=1" >&2
    exit 1
  fi
done

CONFIG="${NTPRO_V10_CONFIG:-$ROOT_DIR/configs/nodes/btc-ema-shadow.toml}"
RUN_ID="${NTPRO_V10_RUN_ID:-v100006-$(date -u +%Y%m%dT%H%M%SZ)}"
PROOF_ROOT="${NTPRO_V10_MANUAL_ORDER_PROOF_DIR:-$ROOT_DIR/target/ntpro-v10-manual-order-proof/$RUN_ID}"

python3 - "$CONFIG" "$PROOF_ROOT" "$RUN_ID" <<'PY'
from __future__ import annotations

import hashlib
import hmac
import json
import os
import sys
import time
import traceback
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError as exc:  # pragma: no cover
    raise SystemExit("Python 3.11+ with tomllib is required") from exc

CONFIG_PATH = Path(sys.argv[1])
PROOF_ROOT = Path(sys.argv[2])
RUN_ID = sys.argv[3]
ARTIFACT_ROOT = PROOF_ROOT / "testnet_order_proof"
ALLOWED_BASE_URLS = {
    "https://testnet.binance.vision": "spot_test_network",
    "https://demo-api.binance.com": "spot_demo_mode",
}
BASE_URL = os.environ.get("NTPRO_V10_SPOT_API_BASE_URL", "https://testnet.binance.vision").rstrip("/")
if BASE_URL not in ALLOWED_BASE_URLS:
    raise SystemExit("only Binance Spot Test Network or Spot Demo Mode endpoints are allowed")
ENDPOINT_MODE = ALLOWED_BASE_URLS[BASE_URL]
RECV_WINDOW_MS = int(os.environ.get("NTPRO_V10_RECV_WINDOW_MS", "5000"))
HTTP_TIMEOUT_SECS = float(os.environ.get("NTPRO_V10_HTTP_TIMEOUT_SECS", "10"))


def write_json(name: str, payload: dict) -> None:
    path = ARTIFACT_ROOT / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2, sort_keys=True) + "\n")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def read_config() -> dict:
    data = tomllib.loads(CONFIG_PATH.read_text())
    order = data.get("testnet_order") or {}
    market = data.get("market") or {}
    execution = data.get("execution") or {}
    risk = data.get("risk") or {}

    configured_base_url = str(order.get("http_base_url", "")).rstrip("/")
    require(configured_base_url in ALLOWED_BASE_URLS, "configured http_base_url must be a Binance spot sandbox endpoint")
    if configured_base_url != BASE_URL:
        require(
            "NTPRO_V10_SPOT_API_BASE_URL" in os.environ,
            "base URL override requires NTPRO_V10_SPOT_API_BASE_URL",
        )
    require(order.get("production_endpoint_allowed") is False, "production_endpoint_allowed must be false")
    require(order.get("dashboard_order_controls") is False, "dashboard_order_controls must be false")
    require(order.get("order_type") == "LIMIT", "only LIMIT order proof is allowed")
    require(order.get("time_in_force") == "GTC", "only GTC order proof is allowed")
    require(execution.get("order_submission") == "disabled", "execution.order_submission must remain disabled")
    require(execution.get("external_venue_connection") is False, "production/external venue connection must remain false in config")
    require(risk.get("kill_switch_enabled") is True, "risk.kill_switch_enabled must remain true")
    require(risk.get("kill_switch_active") is False, "risk.kill_switch_active must be false for proof")

    symbol = os.environ.get("NTPRO_V10_TESTNET_SYMBOL", order.get("symbol", "BTCUSDT"))
    side = os.environ.get("NTPRO_V10_TESTNET_SIDE", order.get("side", "BUY"))
    price = os.environ.get("NTPRO_V10_TESTNET_PRICE", order.get("price"))
    quantity = os.environ.get("NTPRO_V10_TESTNET_QUANTITY", order.get("quantity"))
    cancel_after_ms = int(os.environ.get("NTPRO_V10_CANCEL_AFTER_SUBMIT_MS", str(order.get("cancel_after_submit_ms", 3000))))

    require(symbol == order.get("symbol"), "manual proof must use the configured one symbol")
    require(side in {"BUY", "SELL"}, "side must be BUY or SELL")
    require(price and quantity, "price and quantity are required")
    require(cancel_after_ms > 0, "cancel_after_submit_ms must be greater than zero")

    return {
        "run_id": RUN_ID,
        "config_path": str(CONFIG_PATH),
        "base_url": BASE_URL,
        "configured_base_url": configured_base_url,
        "endpoint_mode": ENDPOINT_MODE,
        "symbol": symbol,
        "instrument_id": order.get("instrument_id"),
        "side": side,
        "order_type": "LIMIT",
        "time_in_force": "GTC",
        "price": str(price),
        "quantity": str(quantity),
        "cancel_after_submit_ms": cancel_after_ms,
        "market_symbols": market.get("symbols", []),
        "api_key_env": "BINANCE_TESTNET_API_KEY",
        "api_secret_env": "BINANCE_TESTNET_API_SECRET",
    }


def now_ms() -> int:
    return int(time.time() * 1000)


def public_json(path: str) -> tuple[int, dict]:
    url = f"{BASE_URL}{path}"
    request = urllib.request.Request(
        url,
        method="GET",
        headers={"User-Agent": "ntpro-v100006-manual-testnet-proof"},
    )
    with urllib.request.urlopen(request, timeout=HTTP_TIMEOUT_SECS) as response:
        body = response.read().decode("utf-8")
        return response.status, json.loads(body) if body else {}


def server_time_offset_ms() -> tuple[int, int]:
    status, payload = public_json("/api/v3/time")
    require(status == 200, "server time request must succeed")
    server_time = int(payload["serverTime"])
    return server_time - now_ms(), server_time


def sign(params: dict[str, str | int]) -> tuple[str, str]:
    query = urllib.parse.urlencode(params)
    signature = hmac.new(
        os.environ["BINANCE_TESTNET_API_SECRET"].encode(),
        query.encode(),
        hashlib.sha256,
    ).hexdigest()
    return query, signature


def request_json(method: str, path: str, params: dict[str, str | int]) -> tuple[int, dict]:
    query, signature = sign(params)
    signed_query = f"{query}&signature={signature}"
    url = f"{BASE_URL}{path}?{signed_query}"
    request = urllib.request.Request(
        url,
        method=method,
        headers={
            "X-MBX-APIKEY": os.environ["BINANCE_TESTNET_API_KEY"],
            "User-Agent": "ntpro-v100006-manual-testnet-proof",
        },
    )
    try:
        with urllib.request.urlopen(request, timeout=HTTP_TIMEOUT_SECS) as response:
            body = response.read().decode("utf-8")
            return response.status, json.loads(body) if body else {}
    except urllib.error.HTTPError as error:
        body = error.read().decode("utf-8", errors="replace")
        try:
            payload = json.loads(body)
        except json.JSONDecodeError:
            payload = {"error": "non_json_http_error", "status": error.code}
        raise RuntimeError(f"{method} {path} failed status={error.code} payload={payload}") from error


def redacted_request_shape(method: str, path: str, params: dict[str, str | int]) -> dict:
    visible = {key: value for key, value in params.items() if key not in {"timestamp"}}
    visible["timestamp"] = "<ms>"
    visible["signature"] = "<redacted>"
    return {
        "method": method,
        "target": path,
        "base_url": BASE_URL,
        "query_shape": urllib.parse.urlencode(visible),
        "api_key_header_value_recorded": False,
        "signature_recorded": False,
        "signed_query_recorded": False,
        "signed_url_recorded": False,
        "raw_response_recorded": False,
        "secrets_redacted": True,
    }


def selected_order_fields(payload: dict) -> dict:
    return {
        "symbol": payload.get("symbol"),
        "order_id": payload.get("orderId"),
        "client_order_id": payload.get("clientOrderId") or payload.get("origClientOrderId"),
        "status": payload.get("status"),
        "transact_time": payload.get("transactTime"),
        "executed_qty": payload.get("executedQty"),
        "cumulative_quote_qty": payload.get("cummulativeQuoteQty"),
        "time_in_force": payload.get("timeInForce"),
        "order_type": payload.get("type"),
        "side": payload.get("side"),
    }


def main() -> int:
    ARTIFACT_ROOT.mkdir(parents=True, exist_ok=True)
    config = read_config()
    time_offset_ms, server_time = server_time_offset_ms()
    client_order_id = os.environ.get("NTPRO_V10_TESTNET_CLIENT_ORDER_ID", f"ntpro-v100006-{now_ms()}")
    submitted = False
    canceled = False
    order_test_ok = False
    submit_payload: dict = {}
    cancel_payload: dict = {}
    reconcile_payload: dict = {}
    failure: str | None = None

    write_json("config.json", {
        "schema_version": "ntpro.v100_manual_order_proof_config.v1",
        **config,
        "client_order_id": client_order_id,
        "server_time_source": f"{BASE_URL}/api/v3/time",
        "endpoint_mode": config["endpoint_mode"],
        "server_time_ms": server_time,
        "local_time_offset_ms": time_offset_ms,
        "production_endpoint_allowed": False,
        "dashboard_order_controls": False,
        "real_funds": False,
        "production_trading": False,
        "api_key_value_recorded": False,
        "api_secret_value_recorded": False,
        "secrets_redacted": True,
    })

    write_json("risk_preflight.json", {
        "schema_version": "ntpro.v100_order_preflight_report.v1",
        "status": "pass",
        "passed": True,
        "reasons": [],
        "symbol": config["symbol"],
        "notional": "owner-approved-testnet-tiny",
        "order_submission_remains_disabled": True,
        "network_attempted": False,
        "real_orders_submitted": False,
        "production_endpoint_allowed": False,
        "dashboard_order_controls": False,
        "secrets_redacted": True,
    })

    try:
        order_test_params = {
            "symbol": config["symbol"],
            "side": config["side"],
            "type": config["order_type"],
            "timeInForce": config["time_in_force"],
            "quantity": config["quantity"],
            "price": config["price"],
            "newClientOrderId": client_order_id,
            "recvWindow": RECV_WINDOW_MS,
            "timestamp": now_ms() + time_offset_ms,
        }
        test_status, _ = request_json("POST", "/api/v3/order/test", order_test_params)
        order_test_ok = True
        write_json("order_test.json", {
            "schema_version": "ntpro.v100_order_test_preflight_report.v1",
            "status": "accepted",
            "http_status": test_status,
            "request": redacted_request_shape("POST", "/api/v3/order/test", order_test_params),
            "matching_engine_submission": False,
            "order_submission_remains_disabled": True,
            "network_attempted": True,
            "real_orders_submitted": False,
            "production_endpoint_allowed": False,
            "endpoint_mode": config["endpoint_mode"],
            "dashboard_order_controls": False,
            "secrets_redacted": True,
        })

        submit_params = {
            "symbol": config["symbol"],
            "side": config["side"],
            "type": config["order_type"],
            "timeInForce": config["time_in_force"],
            "quantity": config["quantity"],
            "price": config["price"],
            "newClientOrderId": client_order_id,
            "recvWindow": RECV_WINDOW_MS,
            "timestamp": now_ms() + time_offset_ms,
        }
        submit_status, submit_payload = request_json("POST", "/api/v3/order", submit_params)
        submitted = True
        write_json("submit_ack.json", {
            "schema_version": "ntpro.v100_submit_ack_artifact.v1",
            "status": "accepted",
            "http_status": submit_status,
            "request": redacted_request_shape("POST", "/api/v3/order", submit_params),
            "ack": selected_order_fields(submit_payload),
            "network_attempted": True,
            "real_orders_submitted": True,
            "production_endpoint_allowed": False,
            "endpoint_mode": config["endpoint_mode"],
            "dashboard_order_controls": False,
            "secrets_redacted": True,
        })

        time.sleep(config["cancel_after_submit_ms"] / 1000)

        cancel_params = {
            "symbol": config["symbol"],
            "origClientOrderId": client_order_id,
            "recvWindow": RECV_WINDOW_MS,
            "timestamp": now_ms() + time_offset_ms,
        }
        cancel_status, cancel_payload = request_json("DELETE", "/api/v3/order", cancel_params)
        require(cancel_payload.get("status") == "CANCELED", "cancel ack must report CANCELED")
        canceled = True
        write_json("cancel_ack.json", {
            "schema_version": "ntpro.v100_cancel_ack_artifact.v1",
            "status": "canceled",
            "http_status": cancel_status,
            "request": redacted_request_shape("DELETE", "/api/v3/order", cancel_params),
            "ack": selected_order_fields(cancel_payload),
            "network_attempted": True,
            "testnet_orders_canceled": 1,
            "production_orders_canceled": 0,
            "production_endpoint_allowed": False,
            "endpoint_mode": config["endpoint_mode"],
            "dashboard_order_controls": False,
            "secrets_redacted": True,
        })

        reconcile_params = {
            "symbol": config["symbol"],
            "origClientOrderId": client_order_id,
            "recvWindow": RECV_WINDOW_MS,
            "timestamp": now_ms() + time_offset_ms,
        }
        reconcile_status, reconcile_payload = request_json("GET", "/api/v3/order", reconcile_params)
        require(reconcile_payload.get("status") == "CANCELED", "reconciliation must report terminal CANCELED")
        write_json("reconciliation.json", {
            "schema_version": "ntpro.v100_reconciliation_artifact.v1",
            "status": "terminal_canceled",
            "http_status": reconcile_status,
            "request": redacted_request_shape("GET", "/api/v3/order", reconcile_params),
            "order": selected_order_fields(reconcile_payload),
            "risk_halted": False,
            "new_orders_blocked": True,
            "network_attempted": True,
            "production_endpoint_allowed": False,
            "endpoint_mode": config["endpoint_mode"],
            "dashboard_order_controls": False,
            "secrets_redacted": True,
        })
    except Exception as exc:
        failure = str(exc)
        if submitted and not canceled:
            try:
                cancel_params = {
                    "symbol": config["symbol"],
                    "origClientOrderId": client_order_id,
                    "recvWindow": RECV_WINDOW_MS,
                    "timestamp": now_ms() + time_offset_ms,
                }
                cancel_status, cancel_payload = request_json("DELETE", "/api/v3/order", cancel_params)
                require(cancel_payload.get("status") == "CANCELED", "emergency cancel ack must report CANCELED")
                canceled = True
                write_json("cancel_ack.json", {
                    "schema_version": "ntpro.v100_cancel_ack_artifact.v1",
                    "status": "canceled_after_failure",
                    "http_status": cancel_status,
                    "request": redacted_request_shape("DELETE", "/api/v3/order", cancel_params),
                    "ack": selected_order_fields(cancel_payload),
                    "network_attempted": True,
                    "testnet_orders_canceled": 1,
                    "production_orders_canceled": 0,
                   "production_endpoint_allowed": False,
                    "endpoint_mode": config["endpoint_mode"],
                    "dashboard_order_controls": False,
                    "secrets_redacted": True,
                })
            except Exception as cancel_exc:  # noqa: BLE001
                failure = f"{failure}; emergency cancel failed: {cancel_exc}"
        write_json("error.json", {
            "schema_version": "ntpro.v100_manual_order_proof_error.v1",
            "status": "failed",
            "message": failure,
            "traceback_recorded": False,
            "network_attempted": True,
            "testnet_orders_submitted": 1 if submitted else 0,
            "testnet_orders_canceled": 1 if canceled else 0,
            "production_orders_submitted": 0,
            "production_orders_canceled": 0,
            "endpoint_mode": config["endpoint_mode"],
            "risk_halted": submitted and not canceled,
            "new_orders_blocked": True,
            "dashboard_order_controls": False,
            "secrets_redacted": True,
        })
        print(traceback.format_exc(), file=sys.stderr)

    lifecycle_status = "submit_accepted_cancel_confirmed" if submitted and canceled and not failure else "failed"
    write_json("lifecycle.json", {
        "schema_version": "ntpro.v100_order_lifecycle_artifact.v1",
        "status": lifecycle_status,
        "client_order_id": client_order_id,
        "stages": {
            "risk_preflight": "pass",
            "order_test": "accepted" if order_test_ok else "failed",
            "submit_ack": "accepted" if submitted else "missing",
            "cancel_ack": "canceled" if canceled else "missing",
            "terminal_state": "canceled" if canceled else "unknown",
        },
        "network_attempted": True,
        "testnet_orders_submitted": 1 if submitted else 0,
        "testnet_orders_canceled": 1 if canceled else 0,
        "production_orders_submitted": 0,
        "production_orders_canceled": 0,
        "endpoint_mode": config["endpoint_mode"],
        "manual_submit_cancel_proof_observed": submitted and canceled,
        "production_endpoint_allowed": False,
        "dashboard_order_controls": False,
        "secrets_redacted": True,
    })

    summary = {
        "schema_version": "ntpro.v100_order_proof_summary.v1",
        "status": "pass" if submitted and canceled and not failure else "failed",
        "run_id": RUN_ID,
        "client_order_id": client_order_id,
        "manual_gate_passed": True,
        "redaction_passed": True,
        "testnet_orders_submitted": 1 if submitted else 0,
        "testnet_orders_canceled": 1 if canceled else 0,
        "production_orders_submitted": 0,
        "production_orders_canceled": 0,
        "endpoint_mode": config["endpoint_mode"],
        "dashboard_order_controls_enabled": False,
        "production_endpoint_allowed": False,
        "real_funds": False,
        "production_trading": False,
        "network_attempted": True,
        "manual_submit_cancel_proof_observed": submitted and canceled,
        "risk_halted": submitted and not canceled,
        "new_orders_blocked": True,
        "failure": failure,
    }
    write_json("summary.json", summary)
    if summary["status"] != "pass":
        raise SystemExit(f"manual V100-006 proof failed; artifacts={ARTIFACT_ROOT} reason={failure}")
    print(
        "v10_manual_tiny_submit_cancel status=ok "
        f"artifact_root={ARTIFACT_ROOT} testnet_orders_submitted=1 "
        "testnet_orders_canceled=1 production_orders_submitted=0 "
        f"dashboard_order_controls=false endpoint_mode={config['endpoint_mode']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
PY
