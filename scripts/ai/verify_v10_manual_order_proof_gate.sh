#!/usr/bin/env bash
set -euo pipefail

# V100-010: v0.10 manual Binance testnet order proof gate.
# Default mode is CI-safe and proves the real submit/cancel gate remains closed.
# Real online proof is not executed here. If the owner later runs V100-006, this
# script can validate the resulting artifact package by setting:
#
#   NTPRO_V10_MANUAL_ONLINE=1 \
#   NTPRO_ALLOW_BINANCE_TESTNET_ORDER=1 \
#   NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER=1 \
#   NTPRO_CONFIRM_TESTNET_TINY_NOTIONAL=1 \
#   NTPRO_CONFIRM_TESTNET_CANCEL_AFTER_SUBMIT=1 \
#   NTPRO_V10_MANUAL_ORDER_PROOF_DIR=... \
#   scripts/ai/verify_v10_manual_order_proof_gate.sh
#
# The script itself never submits or cancels orders.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

if [[ "${NTPRO_V10_MANUAL_ONLINE:-0}" != "1" ]]; then
  echo "v10_manual_order_proof_gate status=closed manual_online=false network_attempted=false real_orders_submitted=false"
  echo "set NTPRO_V10_MANUAL_ONLINE=1 and NTPRO_V10_MANUAL_ORDER_PROOF_DIR to validate an owner-approved V100-006 artifact package"
  exit 0
fi

required_env=(
  NTPRO_ALLOW_BINANCE_TESTNET_ORDER
  NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER
  NTPRO_CONFIRM_TESTNET_TINY_NOTIONAL
  NTPRO_CONFIRM_TESTNET_CANCEL_AFTER_SUBMIT
  NTPRO_V10_MANUAL_ORDER_PROOF_DIR
)

for name in "${required_env[@]}"; do
  if [[ -z "${!name:-}" ]]; then
    echo "manual v0.10 order proof validation requires $name" >&2
    exit 1
  fi
done

if [[ "${NTPRO_ALLOW_BINANCE_TESTNET_ORDER}" != "1" ]]; then
  echo "manual v0.10 order proof validation requires NTPRO_ALLOW_BINANCE_TESTNET_ORDER=1" >&2
  exit 1
fi
if [[ "${NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER}" != "1" ]]; then
  echo "manual v0.10 order proof validation requires NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER=1" >&2
  exit 1
fi
if [[ "${NTPRO_CONFIRM_TESTNET_TINY_NOTIONAL}" != "1" ]]; then
  echo "manual v0.10 order proof validation requires NTPRO_CONFIRM_TESTNET_TINY_NOTIONAL=1" >&2
  exit 1
fi
if [[ "${NTPRO_CONFIRM_TESTNET_CANCEL_AFTER_SUBMIT}" != "1" ]]; then
  echo "manual v0.10 order proof validation requires NTPRO_CONFIRM_TESTNET_CANCEL_AFTER_SUBMIT=1" >&2
  exit 1
fi

PROOF_ROOT="$NTPRO_V10_MANUAL_ORDER_PROOF_DIR"

python3 - "$PROOF_ROOT" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
if not root.exists():
    raise SystemExit(f"manual proof directory does not exist: {root}")
if (root / "testnet_order_proof").is_dir():
    root = root / "testnet_order_proof"


def read_json(relative: str):
    path = root / relative
    if not path.is_file():
        raise SystemExit(f"missing required manual proof artifact: {path}")
    return json.loads(path.read_text())


def require(condition, message):
    if not condition:
        raise SystemExit(message)


summary = read_json("summary.json")
risk = read_json("risk_preflight.json")
order_test = read_json("order_test.json")
submit_ack = read_json("submit_ack.json")
cancel_ack = read_json("cancel_ack.json")
lifecycle = read_json("lifecycle.json")
reconciliation = read_json("reconciliation.json")

require(risk.get("schema_version") == "ntpro.v100_order_preflight_report.v1", risk)
require(order_test.get("schema_version") == "ntpro.v100_order_test_preflight_report.v1", order_test)
require(submit_ack.get("schema_version") == "ntpro.v100_submit_ack_artifact.v1", submit_ack)
require(cancel_ack.get("schema_version") == "ntpro.v100_cancel_ack_artifact.v1", cancel_ack)
require(lifecycle.get("schema_version") == "ntpro.v100_order_lifecycle_artifact.v1", lifecycle)
require(reconciliation.get("schema_version") == "ntpro.v100_reconciliation_artifact.v1", reconciliation)

require(summary.get("manual_gate_passed") is True, summary)
require(summary.get("testnet_orders_submitted") == 1, summary)
require(summary.get("testnet_orders_canceled") == 1, summary)
require(summary.get("production_orders_submitted") == 0, summary)
require(summary.get("production_orders_canceled") == 0, summary)
require(summary.get("dashboard_order_controls_enabled") is False, summary)
require(summary.get("redaction_passed") is True, summary)
require(summary.get("production_trading") is not True, summary)
require(summary.get("real_funds") is not True, summary)

for label, payload in [
    ("risk_preflight", risk),
    ("order_test", order_test),
    ("submit_ack", submit_ack),
    ("cancel_ack", cancel_ack),
    ("lifecycle", lifecycle),
    ("reconciliation", reconciliation),
]:
    require(payload.get("production_endpoint_allowed") is not True, f"{label} production endpoint allowed")
    require(payload.get("dashboard_order_controls") is not True, f"{label} dashboard controls enabled")
    require(payload.get("secrets_redacted") is not False, f"{label} secrets not redacted")

print(
    "v10_manual_order_proof_gate status=artifact_package_ok "
    "manual_online=true testnet_orders_submitted=1 testnet_orders_canceled=1 "
    "production_orders_submitted=0 dashboard_order_controls=false"
)
PY
