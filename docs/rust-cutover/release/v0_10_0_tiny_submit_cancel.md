# v0.10.0 Manual Tiny Submit-And-Cancel Proof

Date: 2026-06-19
Executor: Codex
Task: V100-006
Status: PROOF EXECUTED - SPOT DEMO MODE PASS

## Plain Chinese Summary

这份文档说明 V100-006 怎么人工执行，以及 v0.10.0 正式 proof 使用的 endpoint
边界。默认情况下脚本不会联网、不会下单。只有 owner 明确设置所有 gate，并提供
Binance spot sandbox API key/secret 后，脚本才会在 allowlisted spot sandbox
endpoint 上执行一次小额 LIMIT GTC submit，然后按配置立刻 cancel，并写出脱敏证据包。

v0.10.0 正式 proof 使用 Binance Spot Demo Mode：

```text
NTPRO_V10_SPOT_API_BASE_URL=https://demo-api.binance.com
```

这不是生产 Binance，不是真实资金，不是生产交易，也不会给 Dashboard 增加下单按钮。

## Command

Default closed mode:

```bash
scripts/ai/verify_v10_manual_tiny_submit_cancel.sh
```

Real owner-approved manual proof:

```bash
NTPRO_V10_MANUAL_ONLINE=1 \
NTPRO_ALLOW_BINANCE_TESTNET_ORDER=1 \
NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER=1 \
NTPRO_CONFIRM_TESTNET_TINY_NOTIONAL=1 \
NTPRO_CONFIRM_TESTNET_CANCEL_AFTER_SUBMIT=1 \
BINANCE_TESTNET_API_KEY=... \
BINANCE_TESTNET_API_SECRET=... \
scripts/ai/verify_v10_manual_tiny_submit_cancel.sh
```

Spot Demo Mode 使用官方 Demo Mode base URL：

```bash
NTPRO_V10_SPOT_API_BASE_URL=https://demo-api.binance.com
```

Spot Test Network remains an allowlisted sandbox base URL:

```bash
NTPRO_V10_SPOT_API_BASE_URL=https://testnet.binance.vision
```

Optional overrides:

```bash
NTPRO_V10_CONFIG=configs/nodes/btc-ema-shadow.toml
NTPRO_V10_MANUAL_ORDER_PROOF_DIR=target/ntpro-v10-manual-order-proof/<run-id>
NTPRO_V10_TESTNET_PRICE=...
NTPRO_V10_TESTNET_QUANTITY=...
```

`NTPRO_V10_TESTNET_SYMBOL` must match the configured one-symbol scope. The
selected `NTPRO_V10_SPOT_API_BASE_URL` must be one of the allowlisted spot
sandbox endpoints:

```text
https://demo-api.binance.com
https://testnet.binance.vision
```

Production `https://api.binance.com` is forbidden for v0.10.0.

## Artifacts

The script writes:

```text
testnet_order_proof/config.json
testnet_order_proof/risk_preflight.json
testnet_order_proof/order_test.json
testnet_order_proof/submit_ack.json
testnet_order_proof/cancel_ack.json
testnet_order_proof/reconciliation.json
testnet_order_proof/lifecycle.json
testnet_order_proof/summary.json
```

The artifacts record selected order fields and boundary counters only. They do
not record:

- API key value;
- API secret value;
- raw signature;
- signed URL;
- signed query;
- raw response body;
- account balances;
- production Binance endpoint data.

The runner reads Binance spot sandbox server time from `/api/v3/time` on the
selected allowlisted endpoint and applies that offset to signed requests. This
prevents local machine clock drift from causing `-1021` timestamp failures.

The published v0.10.0 proof used `endpoint_mode=spot_demo_mode` with
`https://demo-api.binance.com`.

## Required PASS Boundary

```text
manual_gate_passed=true
testnet_orders_submitted=1
testnet_orders_canceled=1
production_orders_submitted=0
production_orders_canceled=0
dashboard_order_controls_enabled=false
redaction_passed=true
real_funds=false
production_trading=false
```

The `testnet_orders_*` counter names are retained for v0.10 artifact
compatibility. For the published v0.10.0 proof they refer to the single Spot
Demo Mode sandbox submit/cancel event.

If submit succeeds but cancel fails, the script writes failure artifacts,
sets `risk_halted=true`, sets `new_orders_blocked=true`, and exits non-zero.

## Release Boundary

V100-006 completed when the real owner-approved Spot Demo Mode command passed
and the artifact package was validated for the published v0.10.0 release.

If Binance returns `-2015 Invalid API-key, IP, or permissions for action`, the
provided key is not accepted for the selected spot sandbox endpoint from the
current machine. Create or update the matching Spot Test Network or Spot Demo
Mode key, enable the required trading permission, and make sure any IP allowlist
includes the current outbound IP.
