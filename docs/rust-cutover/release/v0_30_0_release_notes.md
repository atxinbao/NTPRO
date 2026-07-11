# NTPRO Rust-only v0.30.0

Status: RELEASE GATE READY
Tag: `ntpro-rust-only-v0.30.0`
Release name: `NTPRO Rust-only v0.30.0`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.30.0`
Base release: `ntpro-rust-only-v0.29.1`
Date: 2026-07-11
Executor: Codex

v0.30.0 publishes the Backend Production Go-Live Candidate Foundation. It is a
source-controlled release package for candidate review, not actual backend
production go-live and not product-grade live trading enablement.

Plain Chinese summary: v0.30.0 发布的是 Backend Production Go-Live Candidate
Foundation。它把 deployment plan、runtime enablement boundary、operator approval
freeze、canary preflight、rollback/DR、config/venue readiness、telemetry/SLO gate、
audit retention、go/no-go runbook、release gates 和 v31 handoff 全部收口为可重建证据。
它不开放默认生产 submit，不允许生产订单 mutation，不调用 adapter send 或 live
exchange request，不启用 automatic remediation/retry，也不提供 Dashboard/Admin/
Trader Terminal 交易控件。v31 生产启用轨必须等待本 release 证据和显式 scoped
approval。

## Included Tasks

- `V300-000` - v0.30.0 intake gate and v0.29.1 dependency proof.
- `V300-001` - backend go-live candidate boundary contract.
- `V300-002` - production deployment plan and environment readiness.
- `V300-003` - runtime enablement boundary and controlled feature flags.
- `V300-004` - operator approval freeze and change-window lifecycle.
- `V300-005` - canary execution preflight and no-default-execution gate.
- `V300-006` - rollback and disaster recovery execution boundary.
- `V300-007` - production config provenance and venue connectivity readiness.
- `V300-008` - telemetry SLO gate and incident freeze integration.
- `V300-009` - audit retention and evidence export readiness.
- `V300-010` - go/no-go runbook and live readiness decision record.
- `V300-011` - v30 release gates and v31 production enablement handoff.

V300 final release scope issue count = 12
V300 final release scope evidence count = 12
V300 exact milestone issue set = #969-#980
V300 registered corrective-scope exception count = 0

## Release Gates

v30 release gates = required
v30 strict provenance = required
release surface current guard = required
release publication guard = required
release publish after gate = required
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
remote reconstruction required = true
generated publication evidence sole proof allowed = false
post-publication closeout evidence path = docs/rust-cutover/release/v0_30_0_release_closeout_evidence.md

```text
scripts/ai/verify_v30_release_gates.sh
scripts/ai/verify_v30_strict_provenance.sh
scripts/ai/check_release_surface_current.sh
scripts/ai/check_github_release_published.sh
scripts/ai/publish_ntpro_release_after_gate.sh
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
ambiguous_backend_go_live_claim = false
actual_backend_production_go_live_allowed = false
product_grade_trading_terminal_claim = false
product_grade_live_trading_terminal_claim = false
default_production_execution_allowed = false
adapter_send_attempted = false
live_exchange_request_attempted = false
automatic_remediation_attempted = false
go_no_go_record_enables_execution = false
decision_record_backend_go_live_allowed = false
```

## Next Track

v31 production enablement track = hard-blocked until v0.30.0 release gate passes, all V300 issues are closed, the `ntpro-rust-only-v0.30.0` GitHub Release is published after the hosted release gate, and an explicit scoped production enablement issue records owner/operator approval, risk gate, audit gate, release gate, rollback readiness, telemetry/SLO gate, and no-default-trading boundary.

v31 does not inherit submit, mutation, adapter send, live exchange request,
retry scheduler, automatic remediation, Dashboard/Admin/Trader Terminal
trading controls, product-grade live trading claim, or actual backend
production go-live from v0.30.0.
