# v0.32.0 Readiness Report

Date: 2026-07-15
Executor: Codex
Milestone: `ntpro-rust-only-v0.32.0`
Status: RELEASED

## Summary

v0.32.0 is the backend production closeout version. It is ready for release
publication only after V320-000 through V320-009 are closed and the hosted
release gate succeeds for tag `ntpro-rust-only-v0.32.0`.

Plain Chinese summary: v0.32.0 是后端收尾版本，只证明 backend closeout readiness。
发布顺序必须是 tag 触发 hosted release gate、gate 成功、再发布 GitHub Release。它不
开放前端完成、产品级实盘终端、默认 submit/mutation、adapter send、live exchange、
retry scheduler、automatic remediation 或交易控件。

## Evidence

V320-000 evidence = docs/rust-cutover/evidence/V320-000.md
V320-001 evidence = docs/rust-cutover/evidence/V320-001.md
V320-002 evidence = docs/rust-cutover/evidence/V320-002.md
V320-003 evidence = docs/rust-cutover/evidence/V320-003.md
V320-004 evidence = docs/rust-cutover/evidence/V320-004.md
V320-005 evidence = docs/rust-cutover/evidence/V320-005.md
V320-006 evidence = docs/rust-cutover/evidence/V320-006.md
V320-007 evidence = docs/rust-cutover/evidence/V320-007.md
V320-008 evidence = docs/rust-cutover/evidence/V320-008.md
V320-009 evidence = docs/rust-cutover/evidence/V320-009.md

## Gates

v32 release gates = required
v32 strict provenance = required
release surface current guard = required
release publication guard = required
release publish after gate = required
publish after hosted gate success = required

```text
scripts/ai/verify_v32_release_gates.sh
scripts/ai/verify_v32_strict_provenance.sh
scripts/ai/publish_ntpro_release_after_gate.sh
```

## Issue Closeout

#1042 V320-000 = must be closed before v0.32.0 tag gate is accepted
#1043 V320-001 = must be closed before v0.32.0 tag gate is accepted
#1044 V320-002 = must be closed before v0.32.0 tag gate is accepted
#1045 V320-003 = must be closed before v0.32.0 tag gate is accepted
#1046 V320-004 = must be closed before v0.32.0 tag gate is accepted
#1047 V320-005 = must be closed before v0.32.0 tag gate is accepted
#1048 V320-006 = must be closed before v0.32.0 tag gate is accepted
#1049 V320-007 = must be closed before v0.32.0 tag gate is accepted
#1050 V320-008 = must be closed before v0.32.0 tag gate is accepted
#1051 V320-009 = must be closed before v0.32.0 tag gate is accepted

## Release Scope

```text
V320 final release scope issue count = 10
V320 final release scope evidence count = 10
V320 exact milestone issue set = #1042-#1051
V320 registered corrective-scope exception count = 0
registered corrective-scope exceptions required = true
unregistered corrective milestone issues fail closed = true
v0.31.1 dependency proof = required
v0.31.1 release evidence = published
strict provenance manifest = target/ntpro-v320/v0_32_0_strict_release_manifest.json
```

## Post-Publication Closeout Target

```text
release tag = ntpro-rust-only-v0.32.0
release name = NTPRO Rust-only v0.32.0
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.32.0
hosted release gate required = true
published after hosted gate = required
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
generated publication evidence sole proof allowed = false
remote reconstruction required = true
v0.33.0 inheritance = separately scoped only
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
