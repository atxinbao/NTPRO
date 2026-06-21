# NTPRO v0.14.0 Production Order-State Read-Only Boundary

Date: 2026-06-22
Executor: Codex
Milestone: `v0.14.0`
Task: `V140-000`
Status: SCOPE BOUNDARY

## Summary

`v0.14.0` may introduce a narrow, owner-gated production order-state read-only
proof. The boundary allows only explicit production `GET` order-state reads for
diagnostic evidence. It does not authorize production order submission,
production order mutation, real funds, production trading, listenKey lifecycle,
or Dashboard order controls.

Plain Chinese summary: v0.14 可以开始讨论“实盘订单状态只读查询”，但只限 owner
手动开 gate 后用 `GET` 查状态。它不是实盘下单版本，不允许下单、撤单、改单、重试、
纠错、listenKey、Dashboard 下单按钮，也不碰真实资金交易。

## Product Claim

Allowed claim:

```text
capability = production order-state read-only boundary
mode = owner-gated proof only
default execution posture = offline fail-closed
allowed production mutation = none
Dashboard order controls = disabled
real funds trading = not included
```

Not allowed claim:

```text
production order submission
production order cancel
production order replace
production order amend
production order retry
production order correction
production order-test submission
listenKey creation
listenKey keepalive
listenKey close
signed WebSocket user stream runtime
automatic remediation
production trading
Dashboard order controls
```

## Endpoint Allowlist

The v0.14 read-only order-state proof may target Binance Spot production REST
endpoints under `https://api.binance.com/api/v3` only after the owner enables
manual online gates.

| Method | Endpoint | Scope | Required limits |
| --- | --- | --- | --- |
| `GET` | `/api/v3/openOrders` | Read currently open orders for the configured symbol or bounded account proof. | Owner-gated, signed read-only request, redacted artifact, no mutation. |
| `GET` | `/api/v3/order` | Read a single known order state by `symbol` and `orderId` or `origClientOrderId`. | Owner-gated, known identifier only, redacted artifact, no mutation. |

Deferred by default:

| Method | Endpoint | Reason |
| --- | --- | --- |
| `GET` | `/api/v3/allOrders` | Historical order lists can expose broader account history; keep deferred until a later explicit proof contract. |
| `GET` | any user-data stream or WebSocket listen endpoint | listenKey and signed user-stream lifecycle remain out of scope. |

## Forbidden Surfaces

The boundary must reject or keep out of scope:

| Method or action | Surface | Status |
| --- | --- | --- |
| `POST` | `/api/v3/order` | Forbidden. |
| `POST` | `/api/v3/order/test` | Forbidden for production. |
| `DELETE` | `/api/v3/order` | Forbidden. |
| `DELETE` | `/api/v3/openOrders` | Forbidden. |
| `PUT` / `PATCH` | Any order endpoint | Forbidden. |
| submit / cancel / replace / amend | Any production order | Forbidden. |
| retry / correction / auto-remediation | Any production order | Forbidden. |
| `POST` / `PUT` / `DELETE` | `/api/v3/userDataStream` | Forbidden. |
| Dashboard order button | Any production order control | Forbidden. |

## Required Artifact Fields

Any future v0.14 artifact that records this boundary must include these fields:

```text
schema_version = ntpro.v140_order_state_readonly_boundary.v1
owner_gated = true
default_network_required = false
default_execution_posture = offline_fail_closed
allowed_http_methods = GET only
forbidden_http_methods = POST, PUT, PATCH, DELETE
production_order_state_reads_allowed = owner_gated_only
production_order_state_reads_attempted = 0 by default offline
production_order_submission_allowed = false
production_order_mutation_allowed = false
production_order_test_submission_allowed = false
listen_key_lifecycle_allowed = false
signed_user_stream_runtime_allowed = false
dashboard_order_controls_enabled = false
automatic_remediation_allowed = false
real_funds_enabled = false
production_trading_enabled = false
raw_response_persistence_allowed = false
signature_persistence_allowed = false
signed_url_persistence_allowed = false
```

## Default Gate Posture

Default local, PR, CI, and release-gate execution must remain offline and
fail-closed:

```text
network_attempted = false
credentials_required = false
production_order_state_reads_attempted = 0
production_order_submissions_attempted = 0
production_order_mutations_attempted = 0
listen_key_lifecycle_attempted = 0
real_orders_submitted = false
values_are_exchange_truth = false
```

Owner-run online proof, if later implemented by a separate V140 task, must be
explicitly gated and must write only redacted evidence.

## Dashboard Boundary

Dashboard may show order-state read-only proof status after a later
implementation task, but it must not add buttons or forms for:

```text
submit order
cancel order
replace order
amend order
retry order
correct order
create listenKey
enter production credentials
start production trading
```

## Non-Goals

This boundary does not implement the order-state probe. It does not validate an
online production account. It does not query Binance. It does not change
adapter behavior, risk behavior, execution behavior, order lifecycle semantics,
or Dashboard controls.
