# v0.18.0 Cancel Recovery Artifact Contracts

Date: 2026-06-26
Executor: Codex
Milestone: `v0.18.0`
Task: `V180-002`
Status: CONTRACT

## Summary

This document defines the v0.18.0 cancel recovery preview artifact contracts.
The contracts are versioned JSON evidence surfaces only. They do not permit an
actual cancel send, runtime network behavior, or Dashboard cancel controls.

Plain Chinese summary: 这份文档只定义 JSON 证据应该长什么样。大白话：撤单预览、风控门禁、
人工审批、脱敏、回查、审计这些 artifact 都必须证明“还没有真正撤单”，并且不能出现 Dashboard
撤单按钮。

## Shared Boundary Fields

Every v0.18.0 cancel recovery artifact must include these fields:

```text
capability = Owner-Approved Cancel Recovery Preview
capability_expansion = preview_gate_approval_only
lineage_scope = single_v16_mutation_candidate
order_lineage_id = required
orphan_risk_detected = true or false
risk_halted = true
new_orders_blocked = true
manual_review_required = true
actual_cancel_send_allowed = false
cancel_attempted = false
cancel_requests_sent = 0
retry_attempted = false
remediation_attempted = false
automatic_cancel_allowed = false
automatic_remediation_allowed = false
dashboard_cancel_controls_enabled = false
network_cancel_endpoint_attempted = false
production_order_mutation_allowed = false
```

The artifacts align with v0.17 orphan-risk evidence by carrying the same
`order_lineage_id`, `lineage_scope`, `orphan_risk_detected`, `risk_halted`,
`manual_review_required`, and `new_orders_blocked` fields.

## Artifact Set

| Artifact | Schema version | Purpose |
| --- | --- | --- |
| Cancel request preview | `ntpro.v180_cancel_request_preview.v1` | Owner-visible preview of one cancel candidate. |
| Cancel risk gate | `ntpro.v180_cancel_risk_gate.v1` | Fail-closed gate proving the candidate is in scope. |
| Manual owner approval lifecycle | `ntpro.v180_manual_owner_approval_lifecycle.v1` | Approval state machine evidence without send. |
| Cancel response redaction | `ntpro.v180_cancel_response_redaction.v1` | Redaction contract for future response evidence. |
| Post-cancel readback | `ntpro.v180_post_cancel_readback.v1` | Readback contract for future terminal-state proof. |
| Incident and audit closeout | `ntpro.v180_cancel_recovery_incident_audit_closeout.v1` | Closeout contract tying evidence together. |

## Cancel Request Preview

```text
schema_version = ntpro.v180_cancel_request_preview.v1
artifact_type = cancel_request_preview
cancel_candidate_source = production_mutation_orphan_order_detector
known_order_id = redacted_or_owner_visible_identifier
known_client_order_id = redacted_or_owner_visible_identifier
symbol = owner_selected_symbol
account_label = owner_selected_account_label
cancel_reason = orphan_risk_detected | local_missing_exchange_seen | stale_restart_review
candidate_count = 1
multi_order_cancel_requested = false
cancel_all_requested = false
bulk_cancel_requested = false
strategy_driven_cancel_requested = false
actual_cancel_send_allowed = false
cancel_attempted = false
retry_attempted = false
remediation_attempted = false
dashboard_cancel_controls_enabled = false
```

## Cancel Risk Gate

```text
schema_version = ntpro.v180_cancel_risk_gate.v1
artifact_type = cancel_risk_gate
cancel_request_preview_ready = true
orphan_risk_detected = true
risk_halted = true
new_orders_blocked = true
manual_review_required = true
lineage_scope = single_v16_mutation_candidate
order_identifier_known = true
symbol_matches_lineage = true
account_matches_lineage = true
owner_approval_required = true
actual_cancel_send_allowed = false
cancel_attempted = false
automatic_cancel_allowed = false
dashboard_cancel_controls_enabled = false
```

The gate must fail closed if any of these are true:

```text
multi_order_cancel_requested = true
cancel_all_requested = true
bulk_cancel_requested = true
strategy_driven_cancel_requested = true
retry_requested = true
replace_or_amend_requested = true
flatten_requested = true
dashboard_cancel_requested = true
```

## Manual Owner Approval Lifecycle

```text
schema_version = ntpro.v180_manual_owner_approval_lifecycle.v1
artifact_type = manual_owner_approval_lifecycle
approval_scope = one_order_cancel_candidate
approval_source = owner_manual_action
approval_reusable = false
approval_expires = required
approval_consumed_before_send = false
approval_consumed_after_send = false
owner_approval_required = true
owner_approval_lifecycle_recorded = true
actual_cancel_send_allowed = false
cancel_attempted = false
automatic_cancel_allowed = false
dashboard_cancel_controls_enabled = false
```

## Cancel Response Redaction

```text
schema_version = ntpro.v180_cancel_response_redaction.v1
artifact_type = cancel_response_redaction
redaction_contract_ready = true
api_key_value_recorded = false
api_secret_value_recorded = false
api_key_header_value_recorded = false
signature_recorded = false
signed_query_recorded = false
signed_url_recorded = false
raw_exchange_response_recorded = false
response_body_recorded = false
response_headers_recorded = false
actual_cancel_send_allowed = false
cancel_attempted = false
cancel_requests_sent = 0
dashboard_cancel_controls_enabled = false
```

## Post-Cancel Readback

```text
schema_version = ntpro.v180_post_cancel_readback.v1
artifact_type = post_cancel_readback
post_cancel_readback_contract_ready = true
readback_source = future_owner_gated_read_only_order_state
terminal_state_required = true
terminal_state_observed = false
network_cancel_endpoint_attempted = false
actual_cancel_send_allowed = false
cancel_attempted = false
cancel_requests_sent = 0
automatic_cancel_allowed = false
dashboard_cancel_controls_enabled = false
```

## Incident And Audit Closeout

```text
schema_version = ntpro.v180_cancel_recovery_incident_audit_closeout.v1
artifact_type = cancel_recovery_incident_audit_closeout
incident_closeout_ready = true
audit_trail_ready = true
cancel_request_preview_path = required
cancel_risk_gate_path = required
manual_owner_approval_lifecycle_path = required
cancel_response_redaction_path = required
post_cancel_readback_path = required
actual_cancel_send_allowed = false
cancel_attempted = false
cancel_requests_sent = 0
retry_attempted = false
remediation_attempted = false
automatic_cancel_allowed = false
dashboard_cancel_controls_enabled = false
```

## Non-Goals

These contracts do not implement:

```text
DELETE /api/v3/order
DELETE /api/v3/openOrders
actual cancel send
runtime network behavior
Dashboard cancel controls
automatic cancel
automatic remediation
strategy-driven cancel
bulk cancel
multi-account cancel recovery
multi-venue cancel recovery
```
