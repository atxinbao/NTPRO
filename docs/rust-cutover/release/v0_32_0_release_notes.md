# NTPRO Rust-only v0.32.0

Status: RELEASED
Tag: `ntpro-rust-only-v0.32.0`
Release name: `NTPRO Rust-only v0.32.0`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.32.0`
Base release: `ntpro-rust-only-v0.31.1`
Date: 2026-07-15
Executor: Codex

v0.32.0 publishes the Backend Production Closeout version. It closes the
backend readiness, approval, risk/audit/go-no-go, config/venue, canary,
rollback/DR, telemetry/SLO/incident, read-only admin bridge, fail-closed
negative-test, release-gate, and strict-provenance package for the backend
line only.

Plain Chinese summary: v0.32.0 是后端收尾版本。它证明后端 closeout 所需的审批、
change window、risk/audit/go-no-go、config/venue、rollback/DR、telemetry/SLO/incident、
只读 admin bridge、负例 fail-closed、release gate 和 strict provenance 都已进入发布证据。
它不是前端完成，不是产品级实盘交易终端，不开放默认 submit/mutation/adapter send/live
exchange/retry/remediation，也不新增 Dashboard/Admin/Trader Terminal 交易控件。

## Included Tasks

- `V320-000` - v32 intake gate and v31.1 dependency proof.
- `V320-001` - backend closeout boundary and scoped authorization.
- `V320-002` - owner/operator approval, freeze, and change-window closeout.
- `V320-003` - risk, audit, and go/no-go closeout.
- `V320-004` - config, venue, credential, and environment provenance.
- `V320-005` - canary, rollback, and disaster recovery closeout.
- `V320-006` - telemetry, SLO, alerting, and incident closeout.
- `V320-007` - backend enablement read model and read-only admin bridge.
- `V320-008` - fail-closed negative tests for unscoped controls.
- `V320-009` - release gates, strict provenance, and publication.

V320 final release scope issue count = 10
V320 final release scope evidence count = 10
V320 exact milestone issue set = #1042-#1051
V320 registered corrective-scope exception count = 0

## Release Gates

v32 release gates = required
v32 strict provenance = required
release surface current guard = required
release publication guard = required
release publish after gate = required
publish after hosted gate success = required
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
remote reconstruction required = true
generated publication evidence sole proof allowed = false

```text
scripts/ai/verify_v32_release_gates.sh
scripts/ai/verify_v32_strict_provenance.sh
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
automatic_recovery_allowed = false
dashboard_operation_controls_enabled = false
dashboard_trading_controls_enabled = false
admin_workbench_operation_controls_enabled = false
admin_workbench_trading_controls_enabled = false
trader_terminal_order_ticket_enabled = false
manual_operation_submit_allowed = false
backend_go_live_claim = false
actual_backend_production_go_live_allowed = false
frontend_completion_claim = false
product_grade_trading_terminal_claim = false
product_grade_live_trading_terminal_claim = false
default_production_execution_allowed = false
```

## Next Track

v0.33.0 inheritance = separately scoped only

The `ntpro-rust-only-v0.32.0` GitHub Release must be published after the hosted
release gate for the same tag commit. Any v0.33.0 production expansion must be
separately scoped and must not inherit backend go-live, frontend completion,
product-grade live trading terminal readiness, submit, mutation, adapter send,
live exchange request, retry scheduler, automatic remediation, Dashboard/Admin/
Trader Terminal trading controls, or order-ticket enablement from v0.32.0.
