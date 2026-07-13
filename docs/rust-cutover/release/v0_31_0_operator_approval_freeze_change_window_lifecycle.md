# v0.31.0 Operator Approval, Freeze, and Change-Window Lifecycle

Date: 2026-07-14
Executor: Codex
Task: `V310-002` / GitHub issue `#1008`
Milestone: `v0.31.0`

## Goal

Model the operator approval, production freeze, and change-window lifecycle
required before controlled backend enablement can proceed.

Plain Chinese summary: 本文档定义 v0.31.0 controlled backend enablement candidate
所需的 operator approval、production freeze 和 change-window 生命周期。它只记录
启用评估前置状态，不授权 submit、mutation、adapter send、live exchange、
automatic remediation 或交易控件。

## Lifecycle Status

```text
lifecycle_status = operator_approval_change_window_required_no_execution_authority
approval states = missing, draft, approved, expired, revoked, release_mismatch, window_pending, window_active, window_closed
freeze states = none, scheduled, active, lifted, expired
change window evidence required = true
immutable evidence required = true
source provenance required = true
redaction required = true
```

## Required Approval Evidence

```text
approval_id
approver
operator
release_version
release_tag
build_commit
github_issue
environment
venue_scope
account_scope
change_window_id
requested_capability
issued_at
expires_at
revoked_at
revocation_reason
approval_digest
boundary_digest
redaction_profile
source_provenance
```

## Change Window Evidence

```text
change_window_id
environment
venue_scope
account_scope
window_start
window_end
freeze_state
approval_id
release_version
release_tag
build_commit
created_by
approved_by
source_provenance
redaction_profile
```

## Fail-Closed Cases

```text
missing approval -> fail_closed_missing_scoped_approval
expired approval -> fail_closed_expired_approval
revoked approval -> fail_closed_revoked_approval
release mismatch -> fail_closed_release_mismatch
active freeze -> fail_closed_active_production_freeze
outside approved change window -> fail_closed_outside_approved_change_window
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
execution_adapter_call_allowed = false
adapter_send_allowed = false
live_exchange_request_allowed = false
retry_scheduler_enabled = false
automatic_remediation_allowed = false
dashboard_trading_controls_enabled = false
admin_workbench_trading_controls_enabled = false
trader_terminal_order_ticket_enabled = false
backend_go_live_claim = false
product_grade_trading_terminal_claim = false
runtime behavior changed = false
trading behavior changed = false
```
