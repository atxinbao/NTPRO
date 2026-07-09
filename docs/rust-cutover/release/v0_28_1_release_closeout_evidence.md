# NTPRO v0.28.1 Release Closeout Evidence

Date: 2026-07-09
Executor: Codex
Task: `V281-008` / GitHub issue `#944`
Milestone: `v0.28.1`
Status: PENDING PUBLICATION

## Summary

This source-controlled closeout ledger is the required post-publication target
for `ntpro-rust-only-v0.28.1`. Before publication it records the required facts
that must be written back after the GitHub Release is public. It is not a
generated artifact and must not be replaced by `release-publication-evidence/*`
as the sole proof.

Plain Chinese summary: 本文件是 v0.28.1 发布后的源码证据落点。发布前它记录必须回写的
字段；发布后需要写入真实 tag、hosted gate、GitHub Release、release body hash 和 milestone
事实。未跟踪的 generated evidence 不能单独作为 v0.29.0 intake 依据。

## Required Post-Publication Facts

```text
release tag = ntpro-rust-only-v0.28.1
release name = NTPRO Rust-only v0.28.1
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.28.1
publication status = pending_publication
published after hosted gate = pending
hosted release gate run id = pending
hosted release gate conclusion = pending
hosted release gate jobs = pending
release body matches tracked release notes = pending
source-controlled closeout evidence = docs/rust-cutover/release/v0_28_1_release_closeout_evidence.md
release-publication-evidence/ntpro-rust-only-v0.28.1.json = generated artifact, not sole proof
generated publication evidence sole proof allowed = false
v0.29.0 intake requires this source-controlled closeout evidence = true
```

## Boundary

```text
new_submit_capability = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
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
product_grade_trading_terminal_claim = false
```
