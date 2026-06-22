# NTPRO v0.15.0 Production Mutation Scope Decision

Date: 2026-06-22
Executor: Codex
Milestone: `v0.15.0`
Task: `V150-000`
Status: SCOPE DECISION

## Summary

`v0.15.0` may define a guarded production mutation research scope and an
execution dry-run harness. It must not submit production orders or mutate
production orders. The only allowed direction is to build redacted request
preview artifacts and gates that prove production mutation remains disabled.

Plain Chinese summary: v0.15 可以开始把“未来实盘下单请求要怎么构造、怎么隔离、怎么审批”
讲清楚，但只能干跑。它不能真的发请求给交易所，不能真实下单，不能撤单、改单、重试或
纠错，也不能让 Dashboard 出现下单按钮。

## Product Claim

Allowed claim:

```text
capability = Guarded Live Alpha Mutation Scope + Execution Dry-Run Harness
mode = dry-run request preview only
default execution posture = offline fail-closed
production order request sent = false
production order submission = not included
production order mutation = not included
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
production HTTP request execution
production execution adapter call
listenKey creation
listenKey keepalive
listenKey close
signed WebSocket user stream runtime
automatic remediation
real funds
production trading
Dashboard order controls
```

## Narrow Research Envelope

Future v0.15 tasks must stay inside this envelope:

| Dimension | Allowed research scope | Boundary |
| --- | --- | --- |
| Venue | One owner-selected production venue candidate, initially Binance Spot. | Candidate classification only; no request execution. |
| Account | One owner-selected account label. | Account label may identify the preview context; no account mutation. |
| Symbol | One owner-selected symbol. | Symbol is preview metadata only. |
| Order type | One order type, initially `LIMIT`. | No market sweep, no stop order, no OCO, no bracket order. |
| Side | One owner-selected side per preview. | Strategy must not choose side autonomously. |
| Notional | Tiny capped preview notional. | Cap proves request shape only; no funds are used. |
| Approval | Manual owner approval required for preview creation. | Approval permits preview artifact creation only, not submission. |
| Kill switch | Active by default. | Active kill switch blocks dry-run progression. |
| Strategy execution | Disabled. | No autonomous strategy-to-production execution path. |

## Allowed Future Artifacts

Future V150 tasks may create artifacts for:

```text
endpoint classifier decision = forbidden | scope_candidate | owner_approved_manual_only
redacted order request preview
execution adapter isolation proof
kill switch runtime enforcement proof
manual approval lifecycle for preview only
dry-run mutation golden traces
incident rollback and emergency-stop artifact contracts
Dashboard read-only mutation preflight panel
```

Every artifact must preserve:

```text
request_sent = false
network_attempted = false by default
production_order_submission_allowed = false
production_order_mutation_allowed = false
production_order_submissions_attempted = 0
production_order_mutations_attempted = 0
production_orders_submitted = 0
production_order_cancels_attempted = 0
production_order_replaces_attempted = 0
production_order_amends_attempted = 0
production_order_retries_attempted = 0
production_order_corrections_attempted = 0
execution_adapter_called = false
production_adapter_called = false
listen_key_lifecycle_attempted = false
dashboard_order_controls_enabled = false
automatic_remediation_attempted = false
real_funds_enabled = false
production_trading_enabled = false
```

## Endpoint Boundary

v0.15 may classify mutation endpoint candidates, but classification does not
authorize execution.

Allowed as dry-run preview candidates only:

| Method | Endpoint | v0.15 status |
| --- | --- | --- |
| `POST` | `/api/v3/order` | Scope candidate for redacted request preview only; request must not be sent. |
| `POST` | `/api/v3/order/test` | Scope candidate for redacted request preview only; request must not be sent. |

Still forbidden:

| Method or action | Surface | Status |
| --- | --- | --- |
| `DELETE` | `/api/v3/order` | Forbidden. |
| `DELETE` | `/api/v3/openOrders` | Forbidden. |
| `PUT` / `PATCH` | Any order endpoint | Forbidden. |
| cancel / replace / amend | Any production order | Forbidden. |
| retry / correction / flatten | Any production order | Forbidden. |
| `POST` / `PUT` / `DELETE` | `/api/v3/userDataStream` | Forbidden. |
| Dashboard order button | Any production order control | Forbidden. |

## Credential And Redaction Boundary

Default local, PR, CI, and release-gate execution must not require production
credentials:

```text
credentials_required_by_default = false
api_key_value_recorded = false
api_secret_value_recorded = false
signature_recorded = false
signed_query_recorded = false
signed_url_recorded = false
request_body_recorded = redacted_only
raw_exchange_response_recorded = false
```

If a later owner-approved preview task signs request metadata locally, the
signature, signed query, signed URL, API key, API secret, and raw request body
must remain memory-only or redacted in artifacts. Signing metadata still must
not send a production request.

## Non-Goals

This scope decision does not implement endpoint classification, build request
preview artifacts, call adapters, send HTTP requests, submit production orders,
cancel/replace/amend/retry/correct orders, create listenKeys, accept Dashboard
credentials, start production trading, or change trading semantics.
