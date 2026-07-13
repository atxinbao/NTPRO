# v0.31.0 Canary, Rollback, and DR Execution Boundary

Date: 2026-07-14
Executor: Codex
Task: `V310-004` / GitHub issue `#1010`
Milestone: `v0.31.0`

## Goal

Define the canary enablement, rollback, and disaster-recovery execution boundary
for v0.31.0 controlled backend enablement.

Plain Chinese summary: 本文档定义 canary scope、blast-radius limit、rollback
checkpoints 和 DR readiness evidence。它不启用 automatic remediation、automatic
recovery、submit、mutation、adapter send、live exchange 或交易控件。

## Boundary

```text
boundary_status = canary_rollback_dr_required_no_automatic_recovery
canary scope required = true
blast radius limit required = true
allowed canary scope = single_release_single_environment_single_venue_candidate
canary bypasses rollback = false
canary bypasses DR = false
rollback checkpoints required = true
DR readiness required = true
rollback source provenance required = true
DR source provenance required = true
rollback release-bound required = true
DR release-bound required = true
```

## Fail-Closed Cases

```text
missing rollback path -> fail_closed_missing_rollback_path
stale DR evidence -> fail_closed_stale_dr_evidence
widened canary scope -> fail_closed_widened_canary_scope
ready candidate -> canary_rollback_dr_ready_no_automatic_recovery
```

## Runtime Boundary

```text
automatic_remediation_allowed = false
automatic_recovery_allowed = false
retry_scheduler_enabled = false
new_submit_capability = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
execution_adapter_call_allowed = false
adapter_send_allowed = false
live_exchange_request_allowed = false
dashboard_trading_controls_enabled = false
admin_workbench_trading_controls_enabled = false
trader_terminal_order_ticket_enabled = false
backend_go_live_claim = false
product_grade_trading_terminal_claim = false
runtime behavior changed = false
trading behavior changed = false
```
