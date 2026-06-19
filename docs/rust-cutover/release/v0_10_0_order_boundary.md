# NTPRO v0.10.0 Binance Testnet Order Proof Boundary

Date: 2026-06-19
Executor: Codex
Status: BOUNDARY - NOT RELEASED

## Summary

`v0.10.0` is the Binance Testnet Order Proof milestone. Its scope is a tiny,
manual-gated Binance testnet order lifecycle proof: risk preflight, signed
testnet request construction, `/api/v3/order/test`, one tiny LIMIT GTC
testnet submit, cancel-after-submit, reconciliation, redacted evidence, and
read-only Dashboard display.

Plain Chinese summary: v0.10.0 只证明“在 Binance testnet 上，小额 LIMIT GTC
订单可以在明确人工批准后提交并马上撤单”。它不是生产 Binance，不碰真实资金，不支持生产交易，
不提供 Dashboard 下单按钮，也不允许 CI 或默认本地命令自动下单。

## Version Sequence

```text
v0.9.0  = Strategy Runtime Foundation
v0.9.1  = Strategy Runtime semantics and audit hardening
v0.10.0 = Binance Testnet Order Proof
v0.11.0 = Production Read-Only + Shadow
v0.12.0 = Guarded Live Alpha
```

The current published source release remains `ntpro-rust-only-v0.9.0`. This
document defines the next capability boundary only. It does not create a tag,
publish a GitHub Release, or claim v0.10.0 readiness.

## Product Claim

`v0.10.0` may claim only:

- Binance testnet-only order lifecycle proof;
- one explicitly configured symbol;
- tiny-notional LIMIT GTC submit-and-cancel proof;
- fail-closed local and CI defaults;
- explicit manual online opt-in for any testnet order mutation;
- redacted artifacts that prove request shape, risk decision, submit ack,
  cancel ack, terminal state, and reconciliation outcome;
- read-only Dashboard order-proof display.

`v0.10.0` must not claim:

- Binance production connectivity as a trading surface;
- real funds;
- production trading readiness;
- strategy-driven production execution;
- cancel/replace/amend product workflows beyond the tiny proof cancel;
- Dashboard order controls;
- automatic online order proof in CI;
- production parity.

## Default Execution Posture

Default posture is offline and fail-closed:

```text
order_submission = disabled
manual_online_order_proof = disabled
production_endpoint_allowed = false
dashboard_order_controls = false
```

Any missing gate, missing configuration, stale market data, account-read
failure, kill switch activation, endpoint mismatch, reconciliation mismatch, or
redaction failure must block order submission.

## Allowed Testnet Endpoints

Only these Binance testnet endpoints may be used by a manual v0.10.0 proof:

| Method | Endpoint | Purpose |
| --- | --- | --- |
| `POST` | `https://testnet.binance.vision/api/v3/order/test` | Validate signed order request shape without creating an order. |
| `POST` | `https://testnet.binance.vision/api/v3/order` | Submit one tiny LIMIT GTC testnet order after all manual gates pass. |
| `DELETE` | `https://testnet.binance.vision/api/v3/order` | Cancel the submitted testnet order immediately after submit ack. |
| `GET` | `https://testnet.binance.vision/api/v3/order` | Reconcile the known testnet order when local state is incomplete. |
| `GET` | `https://testnet.binance.vision/api/v3/openOrders` | Detect unexpected open testnet orders for the configured symbol. |

All endpoint use must stay on `https://testnet.binance.vision`.

## Forbidden Surfaces

The following are forbidden in v0.10.0:

- `https://api.binance.com` or any production Binance order endpoint;
- order submission outside Binance testnet;
- order submission without explicit owner/manual gate;
- market order proof;
- multiple-symbol order proof;
- non-tiny notional proof;
- unattended retry that creates another order;
- Dashboard Order, Cancel, Replace, or Amend buttons;
- persisted API key, API secret, raw signature, or signed URL;
- artifact text that claims real funds or production trading.

## Gate Ladder

v0.10.0 work must pass this ladder in order:

1. **Offline config gate**: no network, no credentials, no submit by default.
2. **Execution config gate**: testnet-only base URL, one symbol, tiny quantity,
   tiny notional, cancel timeout, and disabled-by-default setting.
3. **Manual intent gate**: command flag and environment variables must both
   opt in before any order mutation path can run.
4. **Risk preflight gate**: session running, fresh market snapshot, account
   readability, inactive kill switch, symbol allowlist, price/quantity/notional
   limits, open-order count, clock skew, and production endpoint rejection.
5. **Signed request gate**: request signing is testnet endpoint allowlisted and
   artifacts never persist secrets, raw signatures, or signed URLs.
6. **`/api/v3/order/test` gate**: signed test request passes before any
   matching-engine order is attempted.
7. **Tiny submit-and-cancel gate**: one tiny LIMIT GTC testnet order is submitted
   only after explicit owner/manual approval, then canceled immediately.
8. **Reconciliation gate**: submit-without-local-ack, cancel timeout, restart
   with unfinished order, exchange-filled state, and orphan order states must
   risk-halt and block new orders until resolved.
9. **Read-only Dashboard gate**: Dashboard may display proof status and artifact
   paths only; it must not initiate or cancel orders.

## Artifact Boundary

The v0.10.0 order-proof artifact set may include:

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

Required summary counters:

```text
testnet_orders_submitted
testnet_orders_canceled
production_orders_submitted
production_orders_canceled
dashboard_order_controls_enabled
redaction_passed
manual_gate_passed
```

`production_orders_submitted` and `production_orders_canceled` must stay `0`.
`dashboard_order_controls_enabled` must stay `false`.

## Failure Recovery Rules

- If submit ack is missing, reconcile by client order ID before any retry.
- If cancel ack is missing, reconcile by order ID or client order ID before any
  new order path can run.
- If an order is open after timeout, mark `risk_halt=true` and block new orders.
- If an order is filled unexpectedly, mark terminal state and block new orders
  until the artifact records the outcome.
- If an orphan order is detected, stop order submission and require manual
  cleanup evidence.
- If redaction fails, the proof fails even if Binance testnet accepted the
  request.

## Release Rule

The v0.10.0 release may be prepared only after `V100-000` through `V100-011`
are complete. The tiny submit-and-cancel proof remains explicit-owner/manual
gated and must not be simulated as a real online PASS.

Creating an `ntpro-rust-only-v0.10.0` tag or publishing a GitHub Release remains
outside this document and requires explicit owner approval.
