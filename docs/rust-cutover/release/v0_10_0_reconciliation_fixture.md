# v0.10.0 Reconciliation Fixture Contract

Date: 2026-06-19
Executor: Codex
Task: V100-008

## Plain Chinese Summary

这份文档定义 v0.10.0 的离线 reconciliation/orphan-order 证据。它覆盖几种危险状态：
发单后本地没收到 ack、撤单超时、本地还以为订单 open 但交易所已经 filled、重启后还有未完成订单。
这些状态在离线合约里都必须 `risk_halted=true` 且 `new_orders_blocked=true`。

这不是在线 Binance readback，也不是发单或撤单。

## CLI Contract

```bash
nautilus live testnet-reconciliation-fixture \
  --config configs/nodes/btc-ema-shadow.toml \
  --scenario all \
  --output runs/v100/reconciliation-fixture.json
```

Single-scenario evidence is also supported:

```bash
nautilus live testnet-reconciliation-fixture \
  --config configs/nodes/btc-ema-shadow.toml \
  --scenario cancel-timeout \
  --output runs/v100/reconciliation-cancel-timeout.json
```

The output schema is:

```text
ntpro.v100_reconciliation_fixture_report.v1
```

## Required Scenarios

- `submit_without_local_ack`
- `cancel_timeout`
- `local_open_exchange_filled`
- `restart_unfinished_order`

Each scenario must record:

```text
risk_halted=true
new_orders_blocked=true
```

## Offline Boundary

Offline evidence must preserve:

```text
manual_submit_cancel_proof_observed=false
matching_engine_submission=false
order_submission_remains_disabled=true
network_attempted=false
real_orders_submitted=false
production_endpoint_allowed=false
dashboard_order_controls=false
```

Counters remain separated and zero:

```text
testnet_orders_submitted=0
testnet_orders_canceled=0
production_orders_submitted=0
production_orders_canceled=0
```

## Manual Proof Boundary

Real exchange readback after a true Binance testnet tiny submit/cancel remains
manual-gated. This fixture defines fail-safe behavior for inconsistent states;
it does not complete V100-006 and does not authorize production Binance or real
funds.
