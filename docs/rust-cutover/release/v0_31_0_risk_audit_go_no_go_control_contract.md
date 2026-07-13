# v0.31.0 Risk Gate, Audit Gate, and Go/No-Go Control Contract

Date: 2026-07-14
Executor: Codex
Task: `V310-003` / GitHub issue `#1009`
Milestone: `v0.31.0`

## Goal

Create the risk gate, audit gate, and go/no-go control contract for controlled
backend enablement.

Plain Chinese summary: 本文档定义 v0.31.0 controlled backend enablement candidate
的 risk gate、audit gate 和 operator go/no-go 合同。它只决定候选状态，不授权
order submit、mutation、adapter send、live exchange 或任何交易控件。

## Required Inputs

```text
control_status = risk_audit_go_no_go_required_no_execution_authority
risk status required = true
risk freshness required = true
audit readiness required = true
audit freshness required = true
release identity required = true
rollback readiness required = true
operator go/no-go required = true
go alone authorizes execution = false
```

Required release identity fields:

```text
release_version
release_tag
build_commit
workflow_run_id
release_body_hash
```

## Reason Codes

```text
missing risk -> blocked_missing_risk_status
stale risk -> blocked_stale_risk_status
degraded risk -> candidate_degraded_risk_status
missing audit -> blocked_missing_audit_readiness
stale audit -> blocked_stale_audit_readiness
missing release identity -> blocked_missing_release_identity
release mismatch -> blocked_release_identity_mismatch
missing rollback -> blocked_missing_rollback_readiness
stale rollback -> blocked_stale_rollback_readiness
missing go/no-go -> blocked_missing_operator_go_no_go
operator no-go -> blocked_operator_no_go
approved candidate -> approved_candidate_no_execution_authority
```

## Runtime Boundary

```text
go_no_go_authorizes_submit = false
go_no_go_authorizes_mutation = false
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

## Deterministic Decision Cases

```text
missing risk -> blocked_missing_risk_status
stale audit -> blocked_stale_audit_readiness
release mismatch -> blocked_release_identity_mismatch
operator no-go -> blocked_operator_no_go
degraded risk -> candidate_degraded_risk_status
approved candidate -> approved_candidate_no_execution_authority
```
