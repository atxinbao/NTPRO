# NTPRO v0.10.0 Execution Config Contract

Date: 2026-06-19
Executor: Codex
Status: CONFIG CONTRACT - NOT RELEASED

## Summary

V100-001 adds the first v0.10.0 Binance testnet order proof configuration
contract. The contract is present in the checked-in strategy-session config, but
it is disabled by default and cannot submit orders.

Plain Chinese summary: 这一步只是把 v0.10.0 未来要用的 testnet 下单配置形状固定下来。
配置里可以看到价格、数量、notional、撤单超时和手动 gate 字段，但默认全部失败关闭，
不会下单。

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
- `http_base_url` must be `https://testnet.binance.vision`.
- `instrument_id` must match the one strategy-session market symbol.
- `symbol` must match the base symbol from `instrument_id`.
- `order_type` must be `LIMIT`.
- `time_in_force` must be `GTC`.
- `price`, `quantity`, and `notional` must be positive decimal strings.
- `cancel_after_submit_ms` must be greater than zero.
- `owner_approval_required` and `manual_env_gate_required` must be true.
- `production_endpoint_allowed` and `dashboard_order_controls` must be false.

## Behavior Boundary

This config contract does not:

- sign requests;
- call Binance;
- create testnet orders;
- cancel orders;
- write execution artifacts;
- enable Dashboard order controls;
- change strategy runtime behavior.

The contract only makes future V100 tasks fail closed against a concrete config
shape.

## Release Rule

This document is not release notes. v0.10.0 release closure still requires all
V100 tasks, including manual online proof where explicitly owner-approved.
