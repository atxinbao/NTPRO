# NTPRO v0.10.0 Multi-Layer Order Gate

Date: 2026-06-19
Executor: Codex
Status: GATE CONTRACT - NOT RELEASED

## Summary

V100-002 adds a local multi-layer gate for the future v0.10 Binance testnet
order proof. The gate is intentionally separate from request signing and order
submission.

Plain Chinese summary: 这一步只加门禁，不加下单。默认没有任何真实 Binance 调用；
只有 CLI flags 和环境变量都齐了，命令才会显示 `status=ready`，但仍然不会联网或下单。

## CLI Gate

The local gate command is:

```bash
cargo run -p nautilus-cli --bin nautilus -- live testnet-order-gate \
  --config configs/nodes/btc-ema-shadow.toml \
  --allow-testnet-order \
  --confirm-owner-approved-testnet-order \
  --confirm-tiny-notional \
  --confirm-cancel-after-submit
```

Required CLI flags:

```text
--allow-testnet-order
--confirm-owner-approved-testnet-order
--confirm-tiny-notional
--confirm-cancel-after-submit
```

## Environment Gate

Required env vars:

```text
NTPRO_ALLOW_BINANCE_TESTNET_ORDER=1
NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER=1
NTPRO_CONFIRM_TESTNET_TINY_NOTIONAL=1
NTPRO_CONFIRM_TESTNET_CANCEL_AFTER_SUBMIT=1
```

Any missing env var blocks the gate.

## Fail-Closed Output

Missing gates produce a failing command with:

```text
testnet order gate blocked
missing_cli_flags=...
missing_env_vars=...
order_submission_remains_disabled=true
network_attempted=false
real_orders_submitted=false
```

All gates present produce:

```text
live.testnet_order_gate status=ready
manual_gate_ready=true
order_submission_remains_disabled=true
network_attempted=false
real_orders_submitted=false
production_endpoint_allowed=false
dashboard_order_controls=false
```

## Behavior Boundary

The gate does not:

- sign requests;
- open network connections;
- call `/api/v3/order/test`;
- submit orders;
- cancel orders;
- write order lifecycle artifacts;
- enable Dashboard order controls.

It only blocks or marks the local gate as ready for later V100 tasks.

## Offline Verification

`scripts/ai/verify_v10_offline_fail_closed.sh` proves:

- missing gates fail;
- gate errors report missing CLI flags and env vars;
- all gates present return ready;
- both paths keep `network_attempted=false`;
- both paths keep `real_orders_submitted=false`.
