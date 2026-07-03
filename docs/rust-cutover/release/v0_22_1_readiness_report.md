# NTPRO v0.22.1 Readiness Report

Date: 2026-07-03
Executor: Codex
Milestone: `ntpro-rust-only-v0.22.1`
Status: RELEASED

## Summary

v0.22.1 is ready as a Trader Terminal Workbench hardening patch after V221-001
through V221-006 complete, the `ntpro-rust-only-v0.22.1` tag is created from
`main`, the hosted `Rust Cutover Release Gate` succeeds, and the GitHub Release
is published through the gate-before-publish entrypoint.

Plain Chinese summary: v0.22.1 的发布条件是：V221 全部任务完成、tag 从 `main`
创建、hosted release gate 成功、再通过 gate-before-publish 入口公开 GitHub Release。
它不开放真实交易操作，不启动 v0.23.0，也不把 Workbench 夸大成产品级实盘终端。

## Required Evidence

```text
V221-001 evidence = docs/rust-cutover/evidence/V221-001.md
V221-002 evidence = docs/rust-cutover/evidence/V221-002.md
V221-003 evidence = docs/rust-cutover/evidence/V221-003.md
V221-004 evidence = docs/rust-cutover/evidence/V221-004.md
V221-005 evidence = docs/rust-cutover/evidence/V221-005.md
V221-006 evidence = docs/rust-cutover/evidence/V221-006.md
```

## Patch Inputs

```text
release closeout ledger = docs/rust-cutover/release/v0_22_1_release_closeout_evidence.md
required-false runtime boundary = docs/rust-cutover/release/v0_22_1_required_false_runtime_boundary.md
read-model executable replay expansion = docs/rust-cutover/release/v0_22_1_read_model_executable_replay.md
gate-before-publish governance = docs/rust-cutover/release/v0_22_1_gate_before_publish.md
workbench render fixture = tests/golden/v221/workbench_render_snapshot.json
golden trace release scope = docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json
```

## Gates

```text
v22.1 release gates = required
v22.1 strict provenance = required
v22 required-false runtime boundary = required
v21.1 read-model projection replay = required with V221-003 promoted rows
workbench render smoke = required
release publish after gate = required
release publication before hosted gate success = forbidden
```

## Issue Closeout

```text
#705 V221-001 = required closed before release
#706 V221-002 = required closed before release
#707 V221-003 = required closed before release
#708 V221-004 = required closed before release
#709 V221-005 = required closed before release
#710 V221-006 = stays open until tag, hosted gate, public release, and publication evidence are recorded
```

## Boundary

```text
trader_terminal_workbench = true
read_only_first = true
runtime_bridge = true
hardening_patch_only = true
complete_executable_read_model_runtime = false
product_grade_trading_terminal_claim = false
new_submit_capability = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
manual_operation_entry_enabled = false
manual_operation_submit_allowed = false
manual_operation_cancel_allowed = false
manual_operation_retry_allowed = false
manual_operation_replace_allowed = false
manual_operation_amend_allowed = false
manual_operation_flatten_allowed = false
automatic_operation_action_allowed = false
automatic_cancel_allowed = false
automatic_order_remediation_allowed = false
dashboard_order_controls_enabled = false
dashboard_approval_controls_enabled = false
dashboard_cancel_controls_enabled = false
dashboard_retry_controls_enabled = false
dashboard_fill_controls_enabled = false
dashboard_risk_controls_enabled = false
dashboard_submit_controls_enabled = false
dashboard_replace_controls_enabled = false
dashboard_amend_controls_enabled = false
dashboard_flatten_controls_enabled = false
trader_terminal_order_ticket_enabled = false
```

## v0.23.0 Dependency Boundary

The `v0.23.0` GitHub issues are already published as `#711-#718`, but they
remain hard-blocked. No `v0.23.0` implementation starts until all V221 issues
are closed and this `v0.22.1` release evidence is published.
