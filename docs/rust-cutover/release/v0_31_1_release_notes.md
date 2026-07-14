# NTPRO Rust-only v0.31.1

Status: RELEASE GATE READY
Tag: `ntpro-rust-only-v0.31.1`
Release name: `NTPRO Rust-only v0.31.1`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.31.1`
Base release: `ntpro-rust-only-v0.31.0`
Date: 2026-07-15
Executor: Codex

v0.31.1 is a release governance closeout patch for the published v0.31.0
Controlled Backend Production Enablement Candidate Foundation line.

Plain Chinese summary: v0.31.1 是 v0.31.0 发布后的治理收口补丁。它收口 v31
post-publication evidence、current release surface、GitHub Release body
reconciliation、corrective scope、publication evidence reconstruction、v31.1
release gate、strict provenance 和 v32 start gate。它不新增 submit，不改变 runtime，
不开放 backend go-live，不开放 Dashboard/Admin/Trader Terminal 交易控件，也不把系统声明为
产品级实盘交易终端。

## Included Tasks

- `V311-001` - v31 post-publication closeout evidence and released manifest.
- `V311-002` - current release surface and guard default binding for v31.
- `V311-003` - v31 publication guard released-state and body-hash reconciliation.
- `V311-004` - v31 corrective scope wording and provenance reconciliation.
- `V311-005` - v31 publication evidence reconstruction and closeout audit path.
- `V311-006` - v31.1 release gates, strict provenance, and v32 start-gate handoff.

V311 final release scope issue count = 6
V311 final release scope evidence count = 6
V311 exact milestone issue set = #1036-#1041
V311 registered corrective-scope exception count = 0

## Release Gates

v31.1 release gates = required
v31.1 strict provenance = required
v31 release gates = required
v31 strict provenance = required
v32 start gate = hard-blocked until v0.31.1 release evidence is published and scoped approval exists
release surface current guard = required
release publication guard = required
release publish after gate = required
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
remote reconstruction required = true
generated publication evidence sole proof allowed = false
v0.32.0 start gate contract = docs/rust-cutover/release/v0_31_1_v32_start_gate.json

```text
scripts/ai/verify_v31_1_release_gates.sh
scripts/ai/verify_v31_1_strict_provenance.sh
scripts/ai/verify_v31_1_v32_start_gate.sh
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

v0.32.0 backend closeout start gate = fail-closed without v0.31.1 publication
evidence and explicit scoped approval.

The `ntpro-rust-only-v0.31.1` GitHub Release must be published after the hosted
release gate for the same tag commit. V32 backend closeout may then record the
dependency proof, but it still does not inherit submit, mutation, adapter send,
live exchange request, retry scheduler, automatic remediation, Dashboard/Admin/
Trader Terminal trading controls, backend go-live, frontend completion, or
product-grade live trading claims.
