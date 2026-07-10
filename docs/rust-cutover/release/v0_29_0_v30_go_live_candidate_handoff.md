# v0.29.0 to v0.30.0 Backend Production Go-Live Candidate Handoff

Date: 2026-07-10
Executor: Codex
Task: `V290-010` / GitHub issue `#936`
Milestone: `v0.29.0`
Status: RELEASE GATE READY

## Handoff Contract

```text
v0.30.0 backend production go-live candidate = next track
v0.30.0 default trading controls = false
v0.30.0 backend go-live claim inherited from v0.29.0 = false
v0.30.0 requires new scoped issues before any production enablement = true
v0.30.0 start gate = blocked_until_v290_release_evidence_published
```

v0.29.0 proves backend production readiness evidence. It does not authorize a
backend production deployment, live external request, order mutation, automatic
remediation, or user-facing live trading terminal.

## Required v0.30.0 Inputs

v0.30.0 must define its own GitHub issues before any go-live candidate claim:

```text
backend go-live candidate issue scope = required
production deployment plan = required
runtime enablement boundary = required
operator approval model = required
rollback and DR execution boundary = required
incident escalation and freeze criteria = required
trading controls default disabled = required
```

## Boundary

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
backend_go_live_claim = false
product_grade_trading_terminal_claim = false
```
