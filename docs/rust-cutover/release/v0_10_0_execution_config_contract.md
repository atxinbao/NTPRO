# NTPRO v0.10.0 Execution Config Contract

Date: 2026-06-19
Executor: Codex
Status: RELEASED CONFIG CONTRACT

## Summary

V100-001 added the first v0.10.0 Binance spot sandbox order proof configuration
contract. The contract is present in the checked-in strategy-session config, but
it is disabled by default and cannot submit orders. The published v0.10.0 proof
used Binance Spot Demo Mode through an explicit owner-approved environment
override.

Plain Chinese summary: 这份配置合同把 v0.10.0 spot sandbox 下单证明的配置形状固定下来。
配置里可以看到价格、数量、notional、撤单超时和手动 gate 字段，但默认全部失败关闭，
不会下单。正式 proof 使用 `NTPRO_V10_SPOT_API_BASE_URL=https://demo-api.binance.com`
切到 Binance Spot Demo Mode。

## Checked-In Section

The checked-in strategy-session config includes:

```toml
[testnet_order]
enabled = false
mode = "disabled"
manual_gate = "owner-approved-manual"
http_base_url = "https://testnet.binance.vision"
symbol = "BTCUSDT"
instrument_id = "BTCUSDT.BINANCE"
side = "BUY"
order_type = "LIMIT"
time_in_force = "GTC"
price = "1.00"
quantity = "0.00001000"
notional = "0.00001000"
cancel_after_submit_ms = 3000
owner_approval_required = true
manual_env_gate_required = true
production_endpoint_allowed = false
dashboard_order_controls = false
```

## Required Invariants

- `enabled` must be `false` until explicit v0.10 manual gates run.
- `mode` must be `disabled`.
- `manual_gate` must be `owner-approved-manual`.
- `http_base_url` must be an allowlisted spot sandbox base URL:
  `https://testnet.binance.vision` or `https://demo-api.binance.com`.
- `instrument_id` must match the one strategy-session market symbol.
- `symbol` must match the base symbol from `instrument_id`.
- `order_type` must be `LIMIT`.
- `time_in_force` must be `GTC`.
- `price`, `quantity`, and `notional` must be positive decimal strings.
- `cancel_after_submit_ms` must be greater than zero.
- `owner_approval_required` and `manual_env_gate_required` must be true.
- `production_endpoint_allowed` and `dashboard_order_controls` must be false.

The checked-in config may keep `https://testnet.binance.vision` as the default
offline/test-network sandbox value. A manual proof may override the base URL
with:

```bash
NTPRO_V10_SPOT_API_BASE_URL=https://demo-api.binance.com
```

No v0.10 config may use `https://api.binance.com` or any production Binance
order endpoint.

## Behavior Boundary

This config contract does not:

- sign requests;
- call Binance;
- create sandbox orders;
- cancel orders;
- write execution artifacts;
- enable Dashboard order controls;
- change strategy runtime behavior.

The contract only makes v0.10 order-proof tasks fail closed against a concrete
config shape.

## Release Rule

This document is not release notes. v0.10.0 release closure was completed only
after the owner-approved Spot Demo Mode proof passed. Future patch work must
preserve the same boundary: spot sandbox only, no production Binance, no real
funds, no production trading, and no Dashboard order controls.
