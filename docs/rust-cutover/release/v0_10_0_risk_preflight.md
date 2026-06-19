# NTPRO v0.10.0 Order Risk Preflight

Date: 2026-06-19
Executor: Codex
Status: PREFLIGHT CONTRACT - NOT RELEASED

## Summary

V100-003 adds an offline risk preflight for the future v0.10 Binance testnet
order proof. The preflight consumes a local JSON snapshot and checks whether a
future order path would be allowed to proceed to request construction.

Plain Chinese summary: 这一步加的是“下单前体检”。它只读本地 JSON，不联网、不签名、不下单。
如果 session 没运行、行情过期、账户不可读、kill switch 打开、symbol 不在白名单、notional
或 open order 或 clock skew 超限，都会失败关闭。

## CLI

```bash
cargo run -p nautilus-cli --bin nautilus -- live testnet-order-preflight \
  --config configs/nodes/btc-ema-shadow.toml \
  --input runs/v100/preflight-input.json \
  --output runs/v100/preflight-report.json \
  --allow-testnet-order \
  --confirm-owner-approved-testnet-order \
  --confirm-tiny-notional \
  --confirm-cancel-after-submit
```

The same manual CLI and environment gates from V100-002 are required before
preflight evaluation.

## Input Contract

The input schema is `ntpro.v100_order_preflight_input.v1` and includes:

- session state;
- market symbol, latest event timestamp, current timestamp, and max age;
- account readability and account id;
- kill switch state and allowed symbols;
- max order notional, max open orders, open order count, max clock skew, and
  observed clock skew;
- HTTP base URL and production endpoint flag.

## Checks

The preflight fails closed for:

- missing manual CLI flags or env vars;
- non-running session;
- stale or future-dated market data;
- unreadable account state;
- active kill switch;
- symbol not allowlisted;
- notional above max order notional;
- open order count at or above max open orders;
- observed clock skew above max clock skew;
- non-testnet endpoint;
- any production endpoint permission.

## Boundary

The preflight does not:

- sign requests;
- open network connections;
- call Binance;
- submit orders;
- cancel orders;
- enable Dashboard order controls.

Every report keeps:

```text
order_submission_remains_disabled=true
network_attempted=false
real_orders_submitted=false
```
