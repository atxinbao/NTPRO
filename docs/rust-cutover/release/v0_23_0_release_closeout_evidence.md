# NTPRO v0.23.0 Release Closeout Evidence

Date: 2026-07-04
Executor: Codex
Task: `V231-001` / GitHub issue `#737`
Milestone: `v0.23.1`
Status: CLOSEOUT EVIDENCE RECORDED

## Summary

This document records the live GitHub closeout facts for the completed
`v0.23.0` release. It is a release governance ledger only: it does not change
runtime behavior, adapter behavior, Dashboard behavior, public API behavior, or
trading semantics.

Plain Chinese summary: 本文档把 `v0.23.0` 已发布后的真实 GitHub 状态写回源码树。
`ntpro-rust-only-v0.23.0` 已公开发布，hosted release gate 已成功，tag 和
`main` 指向同一个提交，`#711-#718` 全部关闭，`v0.23.0` milestone 已关闭。
这个版本仍只是 multi-node isolation evidence / replay / readonly
observability，不新增 submit，不允许 production order mutation，不开放 Dashboard
操作控件，也不宣称产品级实盘交易终端。

## Publication Closeout

```text
release tag = ntpro-rust-only-v0.23.0
release name = NTPRO Rust-only v0.23.0
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.23.0
GitHub Release draft = false
GitHub Release prerelease = false
published at = 2026-07-03T18:34:39Z
target commitish = main
lightweight tag commit = 783b024621116d50feaf418f12cb95fb95f87575
lightweight tag tree = 1d40d9c962d7500a8b7cbcdf42a47ec30ea8a5f9
origin/main commit = 783b024621116d50feaf418f12cb95fb95f87575
hosted release gate = https://github.com/atxinbao/NTPRO/actions/runs/28673868094
hosted release gate created at = 2026-07-03T16:57:33Z
hosted release gate completed at = 2026-07-03T18:29:30Z
hosted release gate conclusion = success
hosted release gate jobs = 66/66 success
release publication after gate = pass
release publication evidence status = published_after_gate
publication evidence strategy = source_tree_plus_github_remote
publication evidence audit path = docs/rust-cutover/release/v0_23_0_publication_evidence_audit_path.md
local generated publication evidence required in source tree = false
release-publication-evidence/ntpro-rust-only-v0.23.0.json = generated artifact, not sole proof
```

## Corrective Gate History

```text
cancelled tag gate run = 28669329074
cancelled tag gate reason = historical strict provenance stages evaluated the v0.23.0 tag HEAD
corrective PR = #735

cancelled tag gate run = 28670984542
cancelled tag gate reason = historical version snapshot release gates evaluated the v0.23.0 tag HEAD
corrective PR = #736

final successful tag gate run = 28673868094
final successful tag SHA = 783b024621116d50feaf418f12cb95fb95f87575
```

## Issue Closeout

```text
#711 V230-000 = closed
#712 V230-001 = closed
#713 V230-002 = closed
#714 V230-003 = closed
#715 V230-004 = closed
#716 V230-005 = closed
#717 V230-006 = closed
#718 V230-007 = closed
V230 issue set = 8/8 closed
```

## Milestone Closeout

```text
v0.23.0 milestone = #11
v0.23.0 milestone state = closed
v0.23.0 open_issues = 0
v0.23.0 closed_issues = 8

v0.23.1 milestone = #13
v0.23.1 state = open
v0.23.1 open_issues = 6

v0.24.0 milestone = #14
v0.24.0 state = open
v0.24.0 open_issues = 10
v0.24.0 start rule = blocked until all V231 issues are closed and v0.23.1 release evidence is published
```

## Boundary Statement

```text
multi_account_isolation = true
multi_strategy_isolation = true
multi_venue_node_isolation = true
read_only_dashboard_observability = true
evidence_and_replay_release = true
product_grade_live_trading_terminal = false
new_submit_capability = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
dashboard_operation_controls_enabled = false
dashboard_submit_cancel_retry_replace_amend_flatten_controls = false
```

## Evidence Sources

```text
GitHub issue #737 body and comments
GitHub issue #718 closeout comment
GitHub milestone #11 live state
GitHub Release ntpro-rust-only-v0.23.0
GitHub Actions run 28673868094
docs/rust-cutover/release/v0_23_0_publication_evidence_audit_path.md
PR #734
PR #735
PR #736
docs/rust-cutover/evidence/V230-007.md
docs/rust-cutover/release/v0_23_0_readiness_report.md
docs/rust-cutover/release/v0_23_0_release_manifest.json
release-publication-evidence/ntpro-rust-only-v0.23.0.json
```

## Next Step

After this evidence is merged through issue `#737`, proceed to `#738`
`V231-002 remove stale candidate pending and in-progress provenance` on its own
branch and PR.
