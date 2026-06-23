# v0.17.0 Production Reconciliation And Orphan Recovery Scope

Date: 2026-06-24
Executor: Codex
Milestone: `v0.17.0`
Task: `V170-000`
Status: SCOPE DECISION

## Summary

`v0.17.0` may add production reconciliation and orphan-risk evidence for the
single v0.16 owner-approved production mutation candidate lineage. It may compare
local guarded-send/audit evidence with owner-gated read-only exchange readback,
classify mismatches, flag orphan/open-order risk, and force manual review plus
new-order blocking.

Plain Chinese summary: v0.17 不是“自动救单系统”。大白话：它只是在 v0.16 那一笔
owner 批准的小额订单候选之后，帮你把本地记录和交易所只读回查对起来。发现可能有孤儿单、
状态不一致或不知道订单到底怎样，就要求人工复核并阻止继续开新单；默认不自动撤单、不重试、
不改单、不补单。

## Product Claim

Allowed future claim after V170 implementation and gates pass:

```text
capability = Production Reconciliation And Orphan Recovery Evidence
mode = single mutation candidate lineage reconciliation
default execution posture = offline fail-closed
production mutation default = disabled
production order submission default = disabled
production cancel default = disabled
automatic remediation default = disabled
lineage scope = one v0.16 owner-approved tiny LIMIT GTC production order candidate
exchange access = owner-gated read-only order-state readback only
reconciliation output = evidence, classification, manual review, new-order blocking
Dashboard surface = read-only reconciliation/orphan evidence only
```

Not allowed claim:

```text
production trading platform
strategy-driven production execution
multiple production orders
batch production orders
MARKET orders
automatic cancel
automatic replace / amend
automatic retry / correction / flatten
automatic remediation
Dashboard order controls
Dashboard cancel controls
Dashboard credential input
multi-venue reconciliation
multi-account reconciliation
portfolio-grade production PnL accounting
listenKey lifecycle
signed WebSocket user stream runtime
default production network execution
```

## Included Scope

| Area | Included v0.17 scope | Boundary |
| --- | --- | --- |
| Lineage | One local ledger for one v0.16 mutation candidate lineage. | No multi-order ledger or strategy ledger. |
| Venue | One owner-selected production venue candidate. | No multi-venue reconciliation. |
| Account | One owner-selected account label. | No account discovery or multi-account reconciliation. |
| Symbol | One owner-selected symbol from the v0.16 candidate. | No portfolio-wide symbol scan. |
| Exchange readback | Owner-gated read-only order-state readback from known order identifiers. | No default network, no listenKey lifecycle. |
| Local evidence | Guarded-send, response-redaction, order-state readback, audit-trail, and failure semantics artifacts. | No raw response, signed URL, signature, API key, or secret persistence. |
| Classification | Local-vs-exchange outcomes for the one lineage. | No trading decision engine. |
| Orphan detection | Detect open/unknown/mismatch risk and force manual review plus new-order block evidence. | No automatic cancel by default. |
| Restart recovery | Resume ledger/readback evidence after process restart without duplicate submit or retry. | No hidden resend or duplicate mutation. |
| Dashboard | Read-only reconciliation/orphan panel. | No order, cancel, replace, amend, retry, reconnect, or credential controls. |

## Reconciliation Outcomes

The implementation tasks may introduce these outcome classes:

```text
local_only_pending_readback
exchange_confirmed_matching
exchange_open_order_risk
exchange_missing_after_confirmed_submission
exchange_state_mismatch
readback_unavailable
orphan_risk_manual_review_required
terminal_no_action_required
```

Every non-terminal or ambiguous outcome must set:

```text
manual_review_required = true
new_orders_blocked = true
automatic_cancel_attempted = false
retry_attempted = false
remediation_attempted = false
dashboard_order_controls_enabled = false
```

## Required Artifact Fields

Future v0.17 artifacts must keep these fields grepable:

```text
schema_version = ntpro.v170_*
capability = Production Reconciliation And Orphan Recovery Evidence
capability_expansion_from_v16 = reconciliation_evidence_only
lineage_scope = single_v16_mutation_candidate
default_fail_closed = true
owner_gated_readback_required = true
local_ledger_ready = true
exchange_readback_mapped = true
reconciliation_classified = true
orphan_risk_detected = true or false
manual_review_required = true or false
new_orders_blocked = true or false
duplicate_submit_attempted = false
retry_attempted = false
cancel_attempted = false
replace_attempted = false
amend_attempted = false
flatten_attempted = false
remediation_attempted = false
automatic_cancel_allowed = false
automatic_remediation_allowed = false
dashboard_order_controls_enabled = false
dashboard_cancel_controls_enabled = false
listen_key_lifecycle_attempted = false
api_key_value_recorded = false
api_secret_value_recorded = false
signature_recorded = false
signed_query_recorded = false
signed_url_recorded = false
raw_exchange_response_recorded = false
```

## Endpoint Boundary

Allowed only after explicit owner read-only gates:

| Method | Endpoint | v0.17 status |
| --- | --- | --- |
| `GET` | `/api/v3/order` | Read one known order id or client order id for the lineage. |
| `GET` | `/api/v3/openOrders` | Read bounded open-order state for the configured symbol/account proof. |

Forbidden by default:

| Method or action | Surface | Status |
| --- | --- | --- |
| `POST` | `/api/v3/order` | Forbidden outside the existing v0.16 single guarded-send candidate. |
| `DELETE` | `/api/v3/order` | Forbidden; cancel recovery must be a later explicit owner-approved scope. |
| `DELETE` | `/api/v3/openOrders` | Forbidden. |
| `PUT` / `PATCH` | Any order endpoint | Forbidden. |
| retry / resend | Any production order | Forbidden. |
| cancel / replace / amend | Any production order | Forbidden by default. |
| `POST` / `PUT` / `DELETE` | `/api/v3/userDataStream` | Forbidden. |
| Dashboard order or cancel button | Any production order control | Forbidden. |

## Required Gate Sequence

v0.17 production reconciliation work must land in this order:

```text
1. scope decision
2. local production order ledger
3. exchange readback mapper
4. reconciliation classifier
5. orphan order detection
6. owner-approved cancel recovery boundary
7. restart recovery evidence
8. Dashboard reconciliation/orphan panel
9. failure and incident semantics integration
10. v0.17 release gates and readiness notes
```

The cancel recovery boundary is a planning/contract step only. It must not
enable automatic cancel execution by default.

## Non-Goals

This scope decision does not implement a ledger, read exchange state, classify
orders, detect an orphan order, cancel an order, retry a send, create a
Dashboard panel, create a release tag, or change trading semantics.

It also does not permit:

```text
strategy-driven production execution
multi-order production execution
multi-account or multi-venue reconciliation
automatic cancel
automatic retry
automatic remediation
Dashboard order/cancel controls
listenKey lifecycle
production trading platform claim
```
