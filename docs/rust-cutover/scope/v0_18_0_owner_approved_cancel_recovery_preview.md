# v0.18.0 Owner-Approved Cancel Recovery Preview Scope

Date: 2026-06-26
Executor: Codex
Milestone: `v0.18.0`
Task: `V180-001`
Status: SCOPE DECISION

## Summary

`v0.18.0` may prepare owner-approved cancel recovery preview, risk gate, and
manual approval lifecycle artifacts after v0.17.1 closeout. It must not send an
actual cancel request and must not expose automatic cancel or Dashboard cancel
controls.

Plain Chinese summary: v0.18.0 先做“撤单恢复预案”，不真正撤单。大白话：可以生成撤单预览、
做风控检查、记录人工批准生命周期，但不能调用交易所撤单接口，不能自动撤单，也不能在 Dashboard
上放撤单按钮。

## Version Entry Condition

```text
blocked_by = V171-008
blocked_by_issue = #538
blocked_by_status_required = closed_completed
v0_18_work_allowed_after = v0.17.1 readiness closeout merged
```

This scope decision is executable only because V171-008 has been completed and
closed. v0.18.0 work must remain blocked if v0.17.1 closeout is reopened,
reverted, or superseded.

## Allowed v0.18.0 Claim

```text
capability = Owner-Approved Cancel Recovery Preview
cancel_request_preview = allowed
cancel_risk_gate = allowed
manual_owner_approval_lifecycle = allowed
incident_and_audit_closeout_contract = allowed
post_cancel_readback_contract = allowed
actual_cancel_send_allowed = false
cancel_attempted = false
automatic_cancel_allowed = false
dashboard_cancel_controls_enabled = false
capability_expansion = preview_gate_approval_only
```

## Not Allowed

```text
actual cancel send
automatic cancel
Dashboard cancel button
strategy-driven cancel
cancel all open orders
bulk cancel
multi-account cancel recovery
multi-venue cancel recovery
automatic remediation
retry / replace / amend / flatten
raw secret or signed payload persistence
```

## Required v0.18.0 Sequence

v0.18.0 work must land in this order:

```text
1. scope decision
2. cancel recovery artifact contracts
3. cancel request preview artifact
4. cancel risk gate
5. manual owner approval lifecycle
6. cancel response redaction contract
7. post-cancel readback contract
8. incident and audit closeout contract
9. read-only Dashboard cancel recovery panel
10. v0.18 release gates
11. v0.18 readiness report and release notes
```

The sequence remains preview/gate/approval evidence only. Any actual cancel send
requires a later explicit scope decision and release gate.

## Required Artifact Fields

Every v0.18 artifact must keep these boundary fields grepable:

```text
schema_version = ntpro.v180_*
capability = Owner-Approved Cancel Recovery Preview
capability_expansion = preview_gate_approval_only
lineage_scope = single_v16_mutation_candidate
owner_approval_required = true
owner_approval_lifecycle_recorded = true or false
actual_cancel_send_allowed = false
cancel_attempted = false
cancel_requests_sent = 0
automatic_cancel_allowed = false
automatic_remediation_allowed = false
dashboard_cancel_controls_enabled = false
strategy_driven_cancel_requested = false
cancel_all_requested = false
bulk_cancel_requested = false
multi_account_cancel_requested = false
multi_venue_cancel_requested = false
retry_attempted = false
replace_attempted = false
amend_attempted = false
flatten_attempted = false
api_key_value_recorded = false
api_secret_value_recorded = false
signature_recorded = false
signed_query_recorded = false
signed_url_recorded = false
raw_exchange_response_recorded = false
```

## Dashboard Boundary

The Dashboard may display read-only cancel recovery preview state:

```text
cancel_preview_ready = true or false
cancel_risk_gate_ready = true or false
manual_owner_approval_ready = true or false
post_cancel_readback_ready = true or false
incident_closeout_ready = true or false
dashboard_cancel_controls_enabled = false
```

The Dashboard must not render a cancel button, fetch action, credential input,
or any `data-dashboard-action` that can cancel, retry, replace, amend, flatten,
or remediate an order.

## Release Gate Requirements

v0.18.0 release gates must prove:

```text
actual_cancel_send_allowed = false
cancel_attempted = false
cancel_requests_sent = 0
automatic_cancel_allowed = false
dashboard_cancel_controls_enabled = false
production_order_mutation_allowed = false
network_cancel_endpoint_attempted = false
```

## Non-Goals

This scope decision does not implement:

```text
DELETE /api/v3/order
DELETE /api/v3/openOrders
actual cancel send
automatic cancel
Dashboard cancel button
strategy-driven cancel
cancel all open orders
bulk cancel
multi-account cancel recovery
multi-venue cancel recovery
automatic remediation
```
