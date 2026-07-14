# v0.32.0 Owner/Operator Approval, Freeze, and Production Change-Window Closeout

Date: 2026-07-15
Executor: Codex
Task: V320-002 / GitHub issue #1044
Milestone: v0.32.0

## Goal

Model the owner/operator approval, production freeze, and production
change-window closeout lifecycle required before later backend production
execution can be considered.

Plain Chinese summary: 本文档定义 v0.32.0 Backend Production Closeout 所需的
owner/operator approval、production freeze 和 production change-window 生命周期。
它只记录后端收尾前置状态，不授权 submit、mutation、adapter send、live exchange、
automatic remediation、frontend completion、backend go-live 或交易控件。

## Lifecycle Status

```text
lifecycle_status = owner_operator_approval_change_window_required_no_execution_authority
depends_on_issue_1042 = closed
depends_on_issue_1043 = closed
approval states = missing, draft, approved, expired, revoked, wrong_owner, wrong_operator, scope_mismatch, release_mismatch, window_pending, window_active, window_closed
freeze states = none, scheduled, active, lifted, expired
change window evidence required = true
immutable evidence required = true
source provenance required = true
redaction required = true
rollback plan reference required = true
risk decision reference required = true
audit evidence reference required = true
telemetry gate reference required = true
```

## Required Approval Evidence

```text
approval_id
owner
operator
reviewer
release_version
release_tag
build_commit
github_issue
environment
venue_scope
account_scope
strategy_scope
change_window_id
requested_capability
risk_decision_ref
audit_evidence_ref
rollback_plan_ref
telemetry_slo_ref
issued_at
expires_at
revoked_at
revocation_reason
approval_digest
boundary_digest
request_digest
redaction_profile
source_provenance
approval_scope_digest
```

## Change Window Evidence

```text
change_window_id
environment
venue_scope
account_scope
strategy_scope
window_start
window_end
freeze_state
approval_id
release_version
release_tag
build_commit
created_by
approved_by
owner
operator
rollback_plan_ref
incident_freeze_ref
source_provenance
redaction_profile
```

## Scope Reuse Boundary

```text
approval scope reuse allowed = false
broader scope approval consumption allowed = false
shared approval consumption allowed = false
cross strategy approval reuse allowed = false
cross venue approval reuse allowed = false
cross account approval reuse allowed = false
cross release approval reuse allowed = false
wrong owner status = fail_closed_wrong_owner
wrong operator status = fail_closed_wrong_operator
scope drift status = fail_closed_scope_reuse_or_drift
```

## Fail-Closed Cases

```text
missing approval -> fail_closed_missing_scoped_approval
expired approval -> fail_closed_expired_approval
revoked approval -> fail_closed_revoked_approval
wrong owner -> fail_closed_wrong_owner
wrong operator -> fail_closed_wrong_operator
release mismatch -> fail_closed_release_mismatch
active freeze -> fail_closed_active_production_freeze
outside approved change window -> fail_closed_outside_approved_change_window
approval scope reuse -> fail_closed_scope_reuse_or_drift
approved active window -> approval_window_valid_execution_still_blocked_by_downstream_gates
```

## Redaction And Provenance

```text
raw_secret_allowed = false
raw_account_identifier_allowed = false
raw_operator_token_allowed = false
chat_or_external_notes_sufficient = false
approval evidence is redacted and provenance-bound = true
```

## Runtime Boundary

```text
new_submit_capability = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
cancel_order_allowed = false
replace_order_allowed = false
amend_order_allowed = false
flatten_position_allowed = false
execution_adapter_call_allowed = false
adapter_send_allowed = false
live_exchange_request_allowed = false
network_attempted = false
retry_scheduler_enabled = false
automatic_remediation_allowed = false
automatic_operation_action_allowed = false
dashboard_operation_controls_enabled = false
dashboard_trading_controls_enabled = false
admin_workbench_operation_controls_enabled = false
admin_workbench_trading_controls_enabled = false
trader_terminal_order_ticket_enabled = false
manual_operation_submit_allowed = false
frontend_completion_claim = false
backend_go_live_claim = false
actual_backend_production_go_live_allowed = false
product_grade_trading_terminal_claim = false
product_grade_live_trading_terminal_claim = false
default_production_execution_allowed = false
unscoped_production_execution_allowed = false
runtime behavior changed = false
trading behavior changed = false
```
