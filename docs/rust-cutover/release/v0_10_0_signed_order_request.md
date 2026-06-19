# NTPRO v0.10.0 Signed Order Request Preview

Date: 2026-06-19
Executor: Codex
Status: REQUEST PREVIEW CONTRACT - NOT RELEASED

## Summary

V100-004 adds a signed Binance testnet order request preview layer. It can build
request metadata for the three scoped order endpoints, but it does not open a
socket, call Binance, submit an order, or cancel an order.

Plain Chinese summary: 这一步只是把“未来要发给 Binance testnet 的下单请求长什么样”在本地
构造出来，并且把 API key、secret、签名、signed query、signed URL 全部挡在 artifact 外。
它不联网、不下单、不撤单。

## CLI

```bash
cargo run -p nautilus-cli --bin nautilus -- live testnet-order-request-preview \
  --config configs/nodes/btc-ema-shadow.toml \
  --timestamp-ms 1718400000000 \
  --api-key-env BINANCE_TESTNET_API_KEY \
  --api-secret-env BINANCE_TESTNET_API_SECRET \
  --output runs/v100/request-preview.json \
  --allow-testnet-order \
  --confirm-owner-approved-testnet-order \
  --confirm-tiny-notional \
  --confirm-cancel-after-submit
```

The same manual CLI and environment gates from V100-002 are required before the
request preview is built.

## Allowlist

The request preview layer only allows:

- `POST /api/v3/order/test`
- `POST /api/v3/order`
- `DELETE /api/v3/order`

All other method/path combinations fail closed. Production Binance base URLs
also fail closed.

## Secret Handling

The internal request object may hold sensitive values in memory long enough to
derive the signature. The artifact preview never records:

- API key header value;
- API secret;
- signature;
- signed query;
- signed URL;
- request body.

The preview artifact records only the request method, endpoint, redacted query
shape, header name, and boundary booleans.

## Boundary

The request preview does not:

- open network connections;
- call Binance;
- submit orders;
- cancel orders;
- enable Dashboard order controls;
- claim production trading readiness.

Every report keeps:

```text
order_submission_remains_disabled=true
network_attempted=false
real_orders_submitted=false
```
