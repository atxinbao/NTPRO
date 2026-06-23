# v0.17 Cancel Recovery Boundary

Date: 2026-06-24
Executor: Codex
Milestone: `v0.17.0`
Task: `V170-005`
Status: SCOPE CONTRACT

## Summary

`v0.17.0` may define the owner-approved cancel recovery boundary needed after
orphan order risk is detected. This is a contract step only. It does not execute
cancel requests and does not introduce automatic remediation.

Plain Chinese summary: 这份文档只定义“以后怎么安全地做撤单恢复”。大白话：如果 v0.17
发现可能有孤儿单，可以准备撤单预览、风控检查、人工确认、脱敏证据这些环节；但 v0.17.0
正式能力里默认不发撤单请求，不自动撤单，不让 Dashboard 出现撤单按钮。

## Product Boundary

Allowed v0.17.0 claim:

```text
cancel recovery boundary = documented
cancel request preview = contract only
cancel risk gate = contract only
manual owner approval = contract only
redacted cancel response = contract only
actual cancel send = deferred
automatic cancel = disabled
Dashboard cancel controls = disabled
```

Not allowed v0.17.0 claim:

```text
owner-approved cancel execution available
automatic orphan cleanup
automatic cancel recovery
Dashboard cancel button
bulk cancel
cancel all open orders
strategy-driven cancel
multi-account cancel recovery
multi-venue cancel recovery
```

## Required Future Flow

If a later version implements actual cancel recovery, it must keep this sequence:

```text
1. orphan detector marks orphan_risk_detected=true
2. system halts risk and blocks new orders
3. cancel request preview is created from known order identifiers only
4. cancel risk gate checks symbol/account/order lineage/scope
5. owner manually approves one cancel candidate
6. signing material is read from env only
7. single cancel request is sent once
8. cancel response is redacted before persistence
9. readback verifies terminal canceled/filled/rejected state
10. incident/audit evidence records the result
```

The flow must stop at any failed gate and must not retry, replace, amend,
flatten, or submit a new order.

## v0.17.0 Included Scope

Included in v0.17.0:

```text
cancel_recovery_scope_defined = true
cancel_request_preview_contract_defined = true
cancel_risk_gate_contract_defined = true
manual_owner_approval_contract_defined = true
cancel_response_redaction_contract_defined = true
actual_cancel_send_allowed = false
automatic_cancel_allowed = false
automatic_remediation_allowed = false
dashboard_cancel_controls_enabled = false
```

## v0.17.0 Explicit Deferral

Actual cancel execution is deferred out of `v0.17.0`.

Recommended follow-up version:

```text
v0.17.1 or v0.18.0
```

Required reason: cancel recovery changes the system from read-only
reconciliation evidence into active order management. It must have its own
owner approval, signing, redaction, readback, incident, and release gates.

## Future Cancel Request Preview Contract

A future preview artifact must be generated before any cancel send is possible.
It must contain only known identifiers from the single lineage:

```text
schema_version = ntpro.v171_or_v180_cancel_request_preview.v1
lineage_scope = single_v16_mutation_candidate
cancel_candidate_source = orphan_order_detector
known_order_id = redacted_or_owner_visible_identifier
known_client_order_id = redacted_or_owner_visible_identifier
symbol = owner_selected_symbol
account_label = owner_selected_account_label
cancel_reason = orphan_risk_detected | local_missing_exchange_seen | stale_restart_review
actual_cancel_send_allowed = false
cancel_attempted = false
retry_attempted = false
remediation_attempted = false
dashboard_cancel_controls_enabled = false
```

The preview must not persist API keys, secrets, signatures, signed query, signed
URL, raw exchange response, response body, or response headers.

## Future Cancel Risk Gate Contract

A future cancel risk gate must block unless all of these are true:

```text
orphan_risk_detected = true
risk_halted = true
new_orders_blocked = true
manual_review_required = true
lineage_scope = single_v16_mutation_candidate
order_identifier_known = true
symbol_matches_lineage = true
account_matches_lineage = true
cancel_request_preview_ready = true
owner_approval_ready = true
dashboard_cancel_controls_enabled = false
```

It must block if any of these are true:

```text
multi_order_cancel_requested = true
cancel_all_requested = true
strategy_driven_cancel_requested = true
retry_requested = true
replace_or_amend_requested = true
flatten_requested = true
dashboard_cancel_requested = true
```

## Manual Owner Approval Contract

Future cancel approval must be a one-time approval scoped to exactly one cancel
candidate:

```text
approval_scope = one_order_cancel_candidate
approval_source = owner_manual_action
approval_reusable = false
approval_expires = required
approval_consumed_before_send = true
approval_consumed_after_send = true
```

No background process, strategy, Dashboard button, or incident handler may
auto-approve a cancel.

## Redaction And Audit Contract

Any future cancel response must be redacted before persistence:

```text
api_key_value_recorded = false
api_secret_value_recorded = false
api_key_header_value_recorded = false
signature_recorded = false
signed_query_recorded = false
signed_url_recorded = false
raw_exchange_response_recorded = false
response_body_recorded = false
response_headers_recorded = false
```

The audit trail must record:

```text
cancel_attempted = true or false
cancel_requests_sent = 0 or 1
retry_attempted = false
replace_attempted = false
amend_attempted = false
flatten_attempted = false
remediation_attempted = false
dashboard_cancel_controls_enabled = false
```

For v0.17.0, these values remain:

```text
cancel_attempted = false
cancel_requests_sent = 0
actual_cancel_send_allowed = false
```

## Required Release Gate Before Actual Cancel

A later version may only send an actual cancel after adding dedicated gates:

```text
cancel request preview fixture gate
cancel risk gate fixture gate
manual owner approval consume gate
single-shot cancel send guard
cancel response redaction gate
post-cancel readback gate
incident/audit closeout gate
hosted release gate evidence
```

## Non-Goals

This document does not implement:

```text
DELETE /api/v3/order
DELETE /api/v3/openOrders
actual cancel send
automatic cancel
automatic remediation
Dashboard cancel button
multi-order cancel
cancel all open orders
multi-account or multi-venue cancel recovery
```
