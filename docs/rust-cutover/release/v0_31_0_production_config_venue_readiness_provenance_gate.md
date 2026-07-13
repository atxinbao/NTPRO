# v0.31.0 Production Config and Venue Readiness Provenance Gate

Date: 2026-07-14
Executor: Codex
Task: `V310-005` / GitHub issue `#1011`
Milestone: `v0.31.0`

## Goal

Gate controlled backend enablement on production configuration and venue
readiness provenance.

Plain Chinese summary: 本文档定义 production config、venue readiness、
environment provenance 和 redaction gate。配置和 venue 证据必须 release/tag
一致、来源可追溯、敏感值已脱敏；本任务不授权 adapter send 或 live exchange request。

## Gate

```text
gate_status = production_config_venue_readiness_required_no_adapter_send
production config evidence required = true
venue readiness evidence required = true
environment provenance required = true
source provenance required = true
redaction required = true
sensitive values forbidden = true
config digest required = true
config/release/tag consistency required = true
venue/release/tag consistency required = true
credential material redacted required = true
endpoint class = read_only_or_probe_plan
```

## Fail-Closed Cases

```text
stale config -> fail_closed_stale_config
mismatched venue -> fail_closed_mismatched_venue
missing redaction -> fail_closed_missing_redaction
unproven environment source -> fail_closed_unproven_environment_source
ready candidate -> config_venue_ready_no_adapter_send
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
automatic_recovery_allowed = false
dashboard_trading_controls_enabled = false
admin_workbench_trading_controls_enabled = false
trader_terminal_order_ticket_enabled = false
backend_go_live_claim = false
product_grade_trading_terminal_claim = false
runtime behavior changed = false
trading behavior changed = false
```
