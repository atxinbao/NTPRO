# NTPRO v0.16.0 Production Mutation Scope Contract

Date: 2026-06-23
Executor: Codex
Milestone: `v0.16.0`
Task: `V160-001`
Status: SCOPE DECISION

## Summary

`v0.16.0` may move beyond v0.15 request-preview/dry-run evidence only inside
one minimum owner-approved production mutation candidate. The permitted target
is a single tiny `LIMIT` `GTC` production order submission candidate with
manual owner approval, hard runtime gates, redacted artifacts, no retry, and
post-submit readback evidence.

Plain Chinese summary: v0.16 可以开始做“最小真实生产下单候选”，但边界非常窄。
大白话：只允许老板明确批准的一笔极小 LIMIT GTC 订单候选，默认全部关闭；不能变成
策略自动实盘，不能批量下单，不能市价单，不能撤单、改单、重试、纠错，也不能让
Dashboard 出现下单按钮。

## Product Claim

Allowed future claim after V160 implementation and gates pass:

```text
capability = Minimum Owner-Approved Production Order Mutation Candidate
mode = owner-approved single-order production mutation candidate
default execution posture = offline fail-closed
production mutation default = disabled
production order submission default = disabled
maximum production mutation count per run = 1
allowed order type = LIMIT
allowed time in force = GTC
allowed venue = one owner-selected production venue candidate
allowed account = one owner-selected account label
allowed symbol = one owner-selected symbol
allowed side = one owner-selected side
allowed quantity/notional = tiny owner-capped amount
manual owner approval = required immediately before send
kill switch = enforced immediately before send
request/response artifacts = redacted
post-submit readback = required evidence
```

Not allowed claim:

```text
production trading platform
strategy-driven production execution
multiple production orders
batch production orders
MARKET orders
STOP orders
OCO orders
bracket orders
cancel / replace / amend
retry / correction / flatten
automatic remediation
Dashboard order controls
Dashboard credential input
multi-venue execution
multi-account execution
VWAP / POV / Iceberg execution algorithms
listenKey creation / keepalive / close lifecycle
signed WebSocket user stream runtime
portfolio-grade production PnL accounting
production portfolio parity
default production network execution
```

## Narrow Included Scope

v0.16 may implement only this production mutation envelope:

| Dimension | Included v0.16 scope | Boundary |
| --- | --- | --- |
| Venue | One owner-selected production venue candidate, initially Binance Spot unless a later task narrows it further. | No multi-venue routing. |
| Account | One owner-selected account label. | No multi-account routing or account discovery. |
| Symbol | One owner-selected symbol. | No strategy-selected symbol set. |
| Side | One owner-selected side. | No autonomous side selection. |
| Order type | `LIMIT` only. | No `MARKET`, stop, OCO, bracket, or conditional order. |
| Time in force | `GTC` only. | No IOC, FOK, GTD, post-only, or venue-specific variants unless later scoped. |
| Quantity / notional | Tiny owner-capped amount. | The cap is a safety boundary, not a portfolio sizing engine. |
| Submission count | At most one production order submission attempt per run. | No batch, loop, retry, correction, cancel, replace, or amend. |
| Approval | Explicit owner approval immediately before send. | Approval is single-use and cannot be reused by strategy code. |
| Kill switch | Checked immediately before send. | Active kill switch blocks the send path. |
| Transport | One guarded HTTP send path for the scoped endpoint. | No WebSocket mutation or listenKey lifecycle. |
| Evidence | Redacted request, redacted response summary, readback, audit trail, and no-retry outcome. | No raw credentials, signatures, signed URLs, or raw exchange payloads in artifacts. |
| Dashboard | Read-only evidence display after CLI/runtime artifact creation. | No Dashboard order button or credential entry. |

## Endpoint Boundary

Allowed only after all V160 gates are implemented:

| Method | Endpoint | v0.16 status |
| --- | --- | --- |
| `POST` | `/api/v3/order` | Scope candidate for one owner-approved tiny `LIMIT` `GTC` production order submission. |

Forbidden in v0.16:

| Method or action | Surface | Status |
| --- | --- | --- |
| `POST` | `/api/v3/order/test` | Not the v0.16 production mutation proof path. |
| `DELETE` | `/api/v3/order` | Forbidden. |
| `DELETE` | `/api/v3/openOrders` | Forbidden. |
| `PUT` / `PATCH` | Any order endpoint | Forbidden. |
| cancel / replace / amend | Any production order | Forbidden. |
| retry / correction / flatten | Any production order | Forbidden. |
| `POST` / `PUT` / `DELETE` | `/api/v3/userDataStream` | Forbidden. |
| Dashboard order button | Any production order control | Forbidden. |

## Required Gate Sequence

v0.16 production mutation work must land in this order:

```text
1. scope contract
2. owner-approved runtime gates
3. production signing-material approval artifact
4. single LIMIT GTC request builder
5. guarded production HTTP send path
6. production mutation response redaction contract
7. post-submit order-state readback proof
8. kill switch enforcement around send
9. production mutation audit trail artifact
10. failure-mode and no-retry semantics
11. Dashboard read-only evidence panel
12. v0.16 release gates
13. v0.16 readiness and release notes
```

No task may skip ahead to production request sending before the runtime gates,
approval artifact, request builder, redaction contract, kill switch, and
failure semantics are in place.

## Required Artifact Fields

Every production mutation candidate artifact must make the following fields
grepable:

```text
capability = Minimum Owner-Approved Production Order Mutation Candidate
capability_expansion_from_v15 = true
default_fail_closed = true
owner_approval_required = true
owner_approval_consumed = true
kill_switch_checked_before_send = true
production_order_submission_allowed = owner_approved_single_limit_gtc_only
production_order_submissions_attempted <= 1
production_orders_submitted <= 1
production_order_mutations_attempted <= 1
order_type = LIMIT
time_in_force = GTC
request_redacted = true
response_redacted = true
signature_recorded = false
signed_query_recorded = false
signed_url_recorded = false
api_key_value_recorded = false
api_secret_value_recorded = false
dashboard_order_controls_enabled = false
automatic_remediation_attempted = false
retry_attempted = false
cancel_attempted = false
replace_attempted = false
amend_attempted = false
flatten_attempted = false
listen_key_lifecycle_attempted = false
```

## Credential And Redaction Boundary

Local default, PR, CI, and release-gate execution must remain credential-free
and offline by default:

```text
credentials_required_by_default = false
production_network_required_by_default = false
production_mutation_enabled_by_default = false
api_key_value_recorded = false
api_secret_value_recorded = false
signature_recorded = false
signed_query_recorded = false
signed_url_recorded = false
raw_exchange_response_recorded = false
```

Owner-approved production credentials may be read only inside the explicitly
gated production mutation send task, must never be written to artifacts, and
must not be needed by local or hosted validation.

## Non-Goals

This scope decision does not implement a runtime gate, read credentials, sign a
request, build a production order request, send an HTTP request, submit an
order, query order state, create an audit trail, add Dashboard UI, create a
GitHub Release, or change trading semantics.

It also does not permit:

```text
production trading
strategy-driven production execution
multiple production orders
batch production orders
MARKET orders
cancel / replace / amend
retry / correction / flatten
automatic remediation
Dashboard order controls
multi-venue execution
multi-account execution
listenKey lifecycle
signed WebSocket user stream runtime
real-time portfolio automation
```

