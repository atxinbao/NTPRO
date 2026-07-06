# NTPRO v0.25.0 Release Closeout Evidence

Date: 2026-07-06
Executor: Codex
Task: `V251-001` / GitHub issue `#806`
Milestone: `v0.25.1`
Status: CLOSEOUT EVIDENCE RECORDED

## Summary

This document records the live GitHub closeout facts for the completed
`v0.25.0` release. It is a release governance ledger only: it does not change
runtime behavior, adapter behavior, Dashboard behavior, public API behavior, or
trading semantics.

Plain Chinese summary: 本文档把 `v0.25.0` 已发布后的真实 GitHub 状态写回源码树。
`ntpro-rust-only-v0.25.0` 已公开发布，hosted release gate 已成功，publish
workflow 已成功，`#777-#785` 全部关闭，`v0.25.0` milestone 已关闭，`#804` /
PR `#805` 的 corrective release scope 已包含在最终 tag commit。这个版本仍只是
monitoring / incident / disaster-recovery foundation；它不是 automatic recovery
runtime，不新增 submit，不允许 production order mutation，不开放 Dashboard 交易控件，
也不宣称产品级实盘交易终端。

## Publication Closeout

```text
release tag = ntpro-rust-only-v0.25.0
release name = NTPRO Rust-only v0.25.0
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.25.0
GitHub Release draft = false
GitHub Release prerelease = false
published at = 2026-07-06T04:02:02Z
target commitish = main
lightweight tag commit = eedcdab1d3ca85d6f51b368b5f36208a7b591026
lightweight tag tree = c9f908d502aa83c80869bdee37c705f718ae2ced
origin/main post-release release source = eedcdab1d3ca85d6f51b368b5f36208a7b591026
current origin/main exact match required = false
current origin/main rule = eedcdab1d3ca85d6f51b368b5f36208a7b591026 must remain an ancestor after V251 closeout commits
tag is ancestor of origin/main = true
hosted release gate = https://github.com/atxinbao/NTPRO/actions/runs/28764231552
hosted release gate completed at = 2026-07-06T04:00:17Z
hosted release gate conclusion = success
hosted release gate jobs = 74/74 success
publish workflow = https://github.com/atxinbao/NTPRO/actions/runs/28766874471
publish workflow completed at = 2026-07-06T04:02:07Z
publish workflow conclusion = success
release publication after gate = pass
release publication evidence status = already_published_after_gate
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
remote reconstruction required = true
release-publication-evidence/ntpro-rust-only-v0.25.0.json = generated artifact, not sole proof
```

## Corrective Release Scope

```text
corrective release scope issue = #804 closed
corrective release scope issue closed at = 2026-07-06T02:36:13Z
corrective release scope PR = #805 merged into v0.25.0 tag commit
corrective release scope PR merged at = 2026-07-06T02:36:12Z
corrective release scope merge commit = eedcdab1d3ca85d6f51b368b5f36208a7b591026
corrective release scope included in release tag = true
```

## Issue Closeout

```text
#777 V250-000 = closed
#778 V250-001 = closed
#779 V250-002 = closed
#780 V250-003 = closed
#781 V250-004 = closed
#782 V250-005 = closed
#783 V250-006 = closed
#784 V250-007 = closed
#785 V250-008 = closed
V250 issue set = 9/9 closed
```

## Milestone Closeout

```text
v0.25.0 milestone = #16
v0.25.0 milestone state = closed
v0.25.0 open_issues = 0
v0.25.0 closed_issues = 9

v0.25.1 milestone = #17
v0.25.1 state = open
v0.25.1 start rule = patch closeout before v0.26.0 capability work

v0.26.0 milestone = #18
v0.26.0 state = open
v0.26.0 start rule = blocked until all V251 issues are closed and v0.25.1 release evidence is published
```

## Boundary Statement

```text
v0.25.0 published but runtime capability = monitoring/incident/disaster-recovery foundation
monitoring_incident_dr_foundation = true
automatic_recovery_runtime = false
product_grade_live_trading_terminal = false
new_submit_capability = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
execution_adapter_call_allowed = false
live_exchange_request_allowed = false
retry_scheduler_enabled = false
automatic_remediation_allowed = false
automatic_operation_action_allowed = false
dashboard_operation_controls_enabled = false
dashboard_trading_controls_enabled = false
trader_terminal_order_ticket_enabled = false
manual_operation_submit_allowed = false
```

## Evidence Sources

```text
GitHub issue #806 body
GitHub issue #804 closeout state
GitHub PR #805 merge state
GitHub milestone #16 live state
GitHub Release ntpro-rust-only-v0.25.0
GitHub Actions run 28764231552
GitHub Actions run 28766874471
docs/rust-cutover/evidence/V250-008.md
docs/rust-cutover/release/v0_25_0_readiness_report.md
docs/rust-cutover/release/v0_25_0_release_manifest.json
release-publication-evidence/ntpro-rust-only-v0.25.0.json
```

## Next Step

After this evidence is merged through issue `#806`, proceed to `#807`
`V251-002 V250-009 corrective release scope integration` on its own branch and
PR. No V260 implementation starts until all V251 issues are closed and
v0.25.1 release evidence is published.
