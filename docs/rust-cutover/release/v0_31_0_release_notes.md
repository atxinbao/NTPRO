# NTPRO Rust-only v0.31.0

Status: RELEASED
Tag: `ntpro-rust-only-v0.31.0`
Release name: `NTPRO Rust-only v0.31.0`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.31.0`
Base release: `ntpro-rust-only-v0.30.1`
Date: 2026-07-14
Executor: Codex

v0.31.0 publishes the Controlled Backend Production Enablement Candidate Foundation.
It proves scoped enablement approval, operator freeze/change-window,
risk/audit/go-no-go gates, canary/rollback/DR boundaries, config/venue
readiness, telemetry/SLO/incident readiness, read-only admin visibility, and
forbidden execution negative gates.

Plain Chinese summary: v0.31.0 是后端生产启用候选的证据和门禁版本。它可以证明
controlled enablement candidate 已具备审批、风控、审计、canary、rollback、DR、
config/venue、telemetry/SLO/incident 和只读 admin 可见性边界；它仍不开放默认
production submit、mutation、adapter send、live exchange、retry scheduler、automatic
remediation 或任何交易控件，也不能被描述为产品级实盘系统。

## Published Closeout

published release status = published_after_gate
published release closeout evidence = docs/rust-cutover/release/v0_31_0_release_closeout_evidence.md
hosted release gate run = 29285960500
hosted release gate result = 96/96 success
publish workflow run = 29290691138
publish workflow result = 1/1 success
published at = 2026-07-13T22:42:06Z
tag peeled commit = 14e582868cb9c18b8b26a1fd50ab21cb56f8f1a1
release body hash semantics = normalized_sha256
release body normalized sha256 = 1fed8bfaa9f73d24d4392ed203cbf12d6c90d0cdabcbc29ec9c87151041aa355
GitHub Release body released-state reconciliation = V311-003 / #1038
generated publication evidence sole proof allowed = false
v0.31.1 closeout patch = required before v0.32.0 execution

## Included Tasks

- `V310-000` - v0.31.0 intake gate and v0.30.1 dependency proof.
- `V310-001` - production enablement boundary and explicit scoped approval.
- `V310-002` - operator approval, freeze, and change-window lifecycle.
- `V310-003` - risk gate, audit gate, and go/no-go control contract.
- `V310-004` - canary enablement, rollback, and DR execution boundary.
- `V310-005` - production config and venue readiness provenance gate.
- `V310-006` - telemetry, SLO, and incident gate for enablement.
- `V310-007` - backend enablement state read model and read-only admin bridge.
- `V310-008` - fail-closed negative tests for forbidden production execution.
- `V310-009` - v31 release gates, strict provenance, and v32 handoff.
- `V310-010` - hosted v31 release gate ARG_MAX corrective blocker.

V310 final release scope issue count = 11
V310 final release scope evidence count = 11
V310 exact milestone issue set = #1006-#1015 plus #1033
V310 registered corrective-scope exception count = 1
V310 registered corrective-scope exception issues = #1033

## Release Gates

v31 release gates = required
v31 strict provenance = required
v31 intake gate = v0.30.1 publication evidence satisfied; explicit scoped approval still required
release surface current guard = required
release publication guard = required
release publish after gate = required
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
remote reconstruction required = true
generated publication evidence sole proof allowed = false
v32 handoff = hard-blocked until v0.31.0 release evidence and explicit scoped approval

```text
scripts/ai/verify_v31_release_gates.sh
scripts/ai/verify_v31_strict_provenance.sh
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
product_grade_trading_terminal_claim = false
product_grade_live_trading_terminal_claim = false
default_production_execution_allowed = false
```

## Next Track

v0.32.0 start gate = blocked until v0.31.0 release evidence and explicit scoped approval

The v32 track does not inherit submit, mutation, adapter send, live exchange
requests, retry scheduler, automatic remediation, Dashboard/Admin/Trader
Terminal trading controls, backend go-live, or product-grade live trading
claims from v0.31.0.
