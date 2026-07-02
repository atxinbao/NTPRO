# NTPRO v0.22.1 Release Closeout Evidence

Date: 2026-07-02
Executor: Codex
Task: `V221-001` / GitHub issue `#705`
Milestone: `v0.22.1`
Status: CLOSEOUT EVIDENCE RECORDED

## Summary

This document records the live GitHub closeout facts for the completed
`v0.21.1` and `v0.22.0` releases before the `v0.22.1` hardening patch starts.
It is a governance ledger only: it does not change runtime behavior, adapter
behavior, release workflow behavior, or public API behavior.

Plain Chinese summary: 本文档把已经完成的 `v0.21.1` 和 `v0.22.0` 发版事实写回
源码仓库。`v0.21.1` / `v0.22.0` 的 GitHub Release、tag、hosted release gate、
issue closeout 和 milestone closeout 都已经成立。`v0.22.1` 的后续任务可以基于这些
证据继续推进 runtime boundary hardening、executable replay 扩展、发布顺序治理和
render smoke。`v0.22.0` 仍只能称为 Trader Terminal Workbench / runtime bridge，
不能称为完整 executable read-model runtime 或产品级实盘交易终端。

## v0.21.1 Publication Closeout

```text
release tag = ntpro-rust-only-v0.21.1
release name = NTPRO Rust-only v0.21.1
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.21.1
GitHub Release draft = false
GitHub Release prerelease = false
published at = 2026-07-01T19:50:54Z
target commitish = main
tag object = af51a0e40c17be4d066f97842eae180245eb3912
peeled tag commit = 016bbb32e6f6a343be1e81bf2ad2e270c11e02b0
peeled tag tree = 636ecbc94d5b97eafd94bec92fd8ef13d820e6a4
hosted release gate = https://github.com/atxinbao/NTPRO/actions/runs/28543669704
hosted release gate created at = 2026-07-01T19:50:48Z
hosted release gate completed at = 2026-07-01T21:19:34Z
hosted release gate conclusion = success
```

## v0.22.0 Publication Closeout

```text
release tag = ntpro-rust-only-v0.22.0
release name = NTPRO Rust-only v0.22.0
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.22.0
GitHub Release draft = false
GitHub Release prerelease = false
published at = 2026-07-02T07:07:55Z
target commitish = main
lightweight tag commit = d9d99854fb0f5d4afdb9c8498cb7d34e9feb2830
lightweight tag tree = 7f167db21b9bc7b40a41ef29e548fedca50ca2ea
hosted release gate = https://github.com/atxinbao/NTPRO/actions/runs/28572064792
hosted release gate created at = 2026-07-02T07:07:57Z
hosted release gate completed at = 2026-07-02T08:36:15Z
hosted release gate conclusion = success
```

## Milestone Closeout

```text
v0.21.1 milestone = #9
v0.21.1 milestone state = closed
v0.21.1 open_issues = 0
v0.21.1 closed_issues = 6

v0.22.0 milestone = #10
v0.22.0 milestone state = closed
v0.22.0 open_issues = 0
v0.22.0 closed_issues = 8

v0.22.1 milestone = #12
v0.22.1 milestone state = open
v0.22.1 open_issues = 6

v0.23.0 milestone = #11
v0.23.0 milestone state = open
v0.23.0 open_issues = 8
```

## Closed Issue Sets

```text
v0.21.1 issues = #677, #678, #679, #680, #681, #682
v0.21.1 all closed = true

v0.22.0 issues = #683, #684, #685, #686, #687, #688, #689, #690
v0.22.0 all closed = true
```

## Current Planning Queue

```text
v0.22.1 issues = #705, #706, #707, #708, #709, #710
v0.22.1 root closeout issue = #705
v0.22.1 dependency rule = #706-#710 are blocked until #705 records closeout evidence

v0.23.0 issues = #711, #712, #713, #714, #715, #716, #717, #718
v0.23.0 dependency rule = hard-blocked by #705-#710 and v0.22.1 release evidence

open pull requests at closeout evidence time = 0
```

## v0.22.0 Boundary Statement

```text
trader_terminal_workbench = true
runtime_bridge = true
read_only_first = true
gated_operation_boundary = true
complete_executable_read_model_runtime = false
product_grade_live_trading_terminal = false
new_submit_capability = false
production_order_mutation_allowed = false
ungated_submit_cancel_replace_amend_flatten = false
multi_account_execution_allowed = false
multi_strategy_execution_allowed = false
multi_venue_execution_allowed = false
```

The `v0.22.0` release is a Trader Terminal Workbench and runtime bridge over
the unified read model. It does not prove full executable read-model runtime
coverage for every read-model path. `v0.22.1` keeps that distinction explicit
and tracks the next hardening work in:

```text
#706 required-false runtime operation boundary hardening
#707 executable read-model replay expansion
#708 gate-before-publish release governance
#709 Workbench artifact/render smoke
#710 v0.22.1 release gates and strict provenance
```

## Evidence Sources

```text
GitHub issue #705 body and comments
GitHub milestone #9 / #10 / #11 / #12 live state
GitHub Release ntpro-rust-only-v0.21.1
GitHub Release ntpro-rust-only-v0.22.0
GitHub Actions run 28543669704
GitHub Actions run 28572064792
docs/rust-cutover/evidence/V211-001.md
docs/rust-cutover/evidence/V211-006.md
docs/rust-cutover/evidence/V220-007.md
docs/rust-cutover/release/v0_21_1_readiness_report.md
docs/rust-cutover/release/v0_22_0_readiness_report.md
```

## Next Step

After this evidence is merged through issue `#705`, proceed to `#706`
`V221-002 required-false runtime operation boundary hardening` on its own
branch and PR.
