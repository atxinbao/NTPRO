# v0.31.1 Readiness Report

Date: 2026-07-15
Executor: Codex
Milestone: `ntpro-rust-only-v0.31.1`
Status: RELEASE GATE READY

## Summary

v0.31.1 is ready for tag-gate execution after V311-001 through V311-006 are
closed. The release is a governance closeout patch for v0.31.0 and a hard
start-gate for v0.32.0 backend closeout.

Plain Chinese summary: v0.31.1 的范围是发布治理收口和 v32 start gate 硬化，不是
v0.32.0 后端收尾实现。V311-001 到 V311-006 必须全部闭环，hosted release gate 成功后
才能发布 GitHub Release。v0.32.0 必须等待 v0.31.1 发布证据和 scoped approval，不能继承
任何真实交易执行能力。

## Evidence

V311-001 evidence = docs/rust-cutover/evidence/V311-001.md
V311-002 evidence = docs/rust-cutover/evidence/V311-002.md
V311-003 evidence = docs/rust-cutover/evidence/V311-003.md
V311-004 evidence = docs/rust-cutover/evidence/V311-004.md
V311-005 evidence = docs/rust-cutover/evidence/V311-005.md
V311-006 evidence = docs/rust-cutover/evidence/V311-006.md

## Gates

v31.1 release gates = required
v31.1 strict provenance = required
v31 release gates = required
v31 strict provenance = required
v32 start gate = hard-blocked until v0.31.1 release evidence is published and scoped approval exists
release surface current guard = required
release publication guard = required
release publish after gate = required

```text
scripts/ai/verify_v31_1_release_gates.sh
scripts/ai/verify_v31_1_strict_provenance.sh
scripts/ai/verify_v31_1_v32_start_gate.sh
```

## Issue Closeout

#1036 V311-001 = must be closed before v0.31.1 tag gate is accepted
#1037 V311-002 = must be closed before v0.31.1 tag gate is accepted
#1038 V311-003 = must be closed before v0.31.1 tag gate is accepted
#1039 V311-004 = must be closed before v0.31.1 tag gate is accepted
#1040 V311-005 = must be closed before v0.31.1 tag gate is accepted
#1041 V311-006 = must be closed before v0.31.1 tag gate is accepted

## Release Scope

```text
V311 final release scope issue count = 6
V311 final release scope evidence count = 6
V311 exact milestone issue set = #1036-#1041
V311 registered corrective-scope exception count = 0
registered corrective-scope exceptions required = true
unregistered corrective milestone issues fail closed = true
v0.31.0 dependency proof = required
v0.31.0 release evidence = published
v0.32.0 backend closeout start gate = blocked until v0.31.1 release evidence is published and scoped approval exists
strict provenance manifest = target/ntpro-v311/v0_31_1_strict_release_manifest.json
```

## Post-Publication Closeout Target

```text
release tag = ntpro-rust-only-v0.31.1
release name = NTPRO Rust-only v0.31.1
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.31.1
published release status = pending_hosted_gate
hosted release gate required = true
published after hosted gate = required
source-controlled v32 start gate = docs/rust-cutover/release/v0_31_1_v32_start_gate.json
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
generated publication evidence sole proof allowed = false
v0.32.0 start gate requires v0.31.1 release evidence = true
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
