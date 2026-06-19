# NTPRO v0.10.0 Binance Spot Sandbox Order Proof Boundary

Date: 2026-06-19
Executor: Codex
Status: RELEASED BOUNDARY

## Summary

`v0.10.0` is the Binance spot sandbox order proof release. Its scope is a tiny,
manual-gated Binance spot sandbox order lifecycle proof: risk preflight, signed
sandbox request construction, `/api/v3/order/test`, one tiny LIMIT GTC sandbox
submit, cancel-after-submit, reconciliation, redacted evidence, and read-only
Dashboard display.

Plain Chinese summary: v0.10.0 只证明“在 Binance spot sandbox 上，小额 LIMIT
GTC 订单可以在明确人工批准后提交并马上撤单”。这次正式 proof 使用的是 Binance
Spot Demo Mode `https://demo-api.binance.com`。它不是生产 Binance，不碰真实资金，
不支持生产交易，不提供 Dashboard 下单按钮，也不允许 CI 或默认本地命令自动下单。

## Version Sequence

```text
v0.9.0  = Strategy Runtime Foundation
v0.9.1  = Strategy Runtime semantics and audit hardening
v0.10.0 = Binance Spot Sandbox Order Proof
v0.11.0 = Production Read-Only + Shadow
v0.12.0 = Guarded Live Alpha
```

The current published source release is `ntpro-rust-only-v0.10.0`. This
document records the released v0.10.0 capability boundary and does not expand it
into production Binance, real funds, production trading, or Dashboard order
controls.

## Product Claim

`v0.10.0` may claim only:

- Binance spot sandbox-only order lifecycle proof;
- one explicitly configured symbol;
- tiny-notional LIMIT GTC submit-and-cancel proof;
- fail-closed local and CI defaults;
- explicit manual online opt-in for any sandbox order mutation;
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

## Allowed Spot Sandbox Endpoints

Only the selected Binance spot sandbox base URL may be used by a manual v0.10.0
proof:

| Endpoint mode | Base URL | Purpose |
| --- | --- | --- |
| Spot Test Network | `https://testnet.binance.vision` | Binance spot test network sandbox. |
| Spot Demo Mode | `https://demo-api.binance.com` | Binance Spot Demo Mode sandbox used by the owner-approved v0.10.0 proof. |

For the selected allowlisted base URL, these paths are allowed:

| Method | Endpoint | Purpose |
| --- | --- | --- |
| `POST` | `<spot-sandbox-base-url>/api/v3/order/test` | Validate signed order request shape without creating an order. |
| `POST` | `<spot-sandbox-base-url>/api/v3/order` | Submit one tiny LIMIT GTC sandbox order after all manual gates pass. |
| `DELETE` | `<spot-sandbox-base-url>/api/v3/order` | Cancel the submitted sandbox order immediately after submit ack. |
| `GET` | `<spot-sandbox-base-url>/api/v3/order` | Reconcile the known sandbox order when local state is incomplete. |
| `GET` | `<spot-sandbox-base-url>/api/v3/openOrders` | Detect unexpected open sandbox orders for the configured symbol. |

The v0.10.0 owner-approved proof used Spot Demo Mode:

```text
endpoint_mode = spot_demo_mode
base_url = https://demo-api.binance.com
```

All order-proof endpoint use must stay on one selected allowlisted spot sandbox
base URL. Mixing Spot Test Network and Spot Demo Mode in the same artifact
package is not a PASS.

## Forbidden Surfaces

The following are forbidden in v0.10.0:

- `https://api.binance.com` or any production Binance order endpoint;
- order submission outside the selected Binance spot sandbox endpoint;
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
2. **Execution config gate**: allowlisted spot sandbox base URL, one symbol,
   tiny quantity, tiny notional, cancel timeout, and disabled-by-default
   setting.
3. **Manual intent gate**: command flag and environment variables must both
   opt in before any order mutation path can run.
4. **Risk preflight gate**: session running, fresh market snapshot, account
   readability, inactive kill switch, symbol allowlist, price/quantity/notional
   limits, open-order count, clock skew, and production endpoint rejection.
5. **Signed request gate**: request signing is spot-sandbox endpoint
   allowlisted and artifacts never persist secrets, raw signatures, or signed
   URLs.
6. **`/api/v3/order/test` gate**: signed test request passes before any
   matching-engine order is attempted.
7. **Tiny submit-and-cancel gate**: one tiny LIMIT GTC sandbox order is
   submitted only after explicit owner/manual approval, then canceled
   immediately.
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

The `testnet_orders_*` field names are historical artifact counters for the
v0.10 sandbox proof family. For the published v0.10.0 proof they record the
single Spot Demo Mode submit/cancel event.

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

The `ntpro-rust-only-v0.10.0` tag and formal GitHub Release were published
after the owner-approved Spot Demo Mode proof passed. Future patch work must not
weaken the released boundary: spot sandbox only, no production Binance, no real
funds, no production trading, and no Dashboard order controls.
