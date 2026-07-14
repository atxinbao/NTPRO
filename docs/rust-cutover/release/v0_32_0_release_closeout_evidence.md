# v0.32.0 Release Closeout Evidence

Date: 2026-07-15
Executor: Codex
Release: `ntpro-rust-only-v0.32.0`
Status: SOURCE-CONTROLLED CLOSEOUT CONTRACT

## Summary

This closeout evidence records the source-controlled contract for publishing
v0.32.0 after hosted release gate success. The generated workflow publication
artifact is not the sole proof; live GitHub tag, release, workflow, milestone,
and source-tree evidence must be reconstructable.

Plain Chinese summary: 本文件记录 v0.32.0 发布收口证据的 source-controlled contract。
真实发布时间、hosted gate run id、release body hash 和 publication artifact 由 GitHub
live state 重建；本文件保证 release 证据不是只靠本地生成文件。

## Required Reconstruction

```text
release tag = ntpro-rust-only-v0.32.0
release name = NTPRO Rust-only v0.32.0
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.32.0
hosted release gate required = true
published after hosted gate = required
release body must match tracked notes = docs/rust-cutover/release/v0_32_0_release_notes.md
release manifest = docs/rust-cutover/release/v0_32_0_release_manifest.json
release closeout evidence = docs/rust-cutover/release/v0_32_0_release_closeout_evidence.md
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
remote reconstruction required = true
generated publication evidence sole proof allowed = false
milestone v0.32.0 must close after release publication = true
```

## Boundary

```text
backend closeout version only = true
frontend_completion_claim = false
product_grade_live_trading_terminal_claim = false
new_submit_capability = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
adapter_send_allowed = false
live_exchange_request_allowed = false
retry_scheduler_enabled = false
automatic_remediation_allowed = false
dashboard_trading_controls_enabled = false
admin_workbench_trading_controls_enabled = false
trader_terminal_order_ticket_enabled = false
backend_go_live_claim = false
v0.33.0 inheritance = separately scoped only
```
