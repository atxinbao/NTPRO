# NTPRO v0.10.0 Order-Test Preflight

Date: 2026-06-19
Executor: Codex
Status: ORDER-TEST PREFLIGHT CONTRACT - NOT RELEASED

## Summary

V100-005 adds a dedicated offline preflight for Binance testnet
`POST /api/v3/order/test`. It proves that the local CLI can prepare the scoped
request metadata and signature path, but it does not call Binance and does not
claim Binance accepted the request.

Plain Chinese summary: 这一步是 `/api/v3/order/test` 的“发单前检查”。它只在本地生成
order-test 请求报告，不联网、不请求 Binance、不下单、不撤单，也不证明撮合或生命周期。

## CLI

```bash
cargo run -p nautilus-cli --bin nautilus -- live testnet-order-test-preflight \
  --config configs/nodes/btc-ema-shadow.toml \
  --timestamp-ms 1718400000000 \
  --api-key-env BINANCE_TESTNET_API_KEY \
  --api-secret-env BINANCE_TESTNET_API_SECRET \
  --output runs/v100/order-test-preflight.json \
  --allow-testnet-order \
  --confirm-owner-approved-testnet-order \
  --confirm-tiny-notional \
  --confirm-cancel-after-submit
```

The same manual CLI and environment gates from V100-002 are required before the
preflight report is built.

## Report Contract

The report schema is `ntpro.v100_order_test_preflight_report.v1` and records:

- request method: `POST`;
- request target: `/api/v3/order/test`;
- redacted query shape;
- API key header name only;
- signature preflight state;
- Binance order-test acceptance state;
- matching engine submission state;
- no-network and no-order boundary flags.

The report must keep:

```text
binance_order_test_acceptance=not_attempted_offline_manual_only
matching_engine_submission=false
order_submission_remains_disabled=true
network_attempted=false
real_orders_submitted=false
```

## Boundary

The order-test preflight does not:

- open network connections;
- call Binance;
- verify Binance server-side acceptance;
- submit orders;
- cancel orders;
- prove matching engine lifecycle;
- enable Dashboard order controls;
- claim production trading readiness.
