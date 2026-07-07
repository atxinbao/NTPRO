# NTPRO v0.26.0 Release Closeout Evidence

Date: 2026-07-07
Executor: Codex
Task: `V261-001` / GitHub issue `#847`
Milestone: `v0.26.1`
Status: CLOSEOUT EVIDENCE RECORDED

## Summary

This document records the live GitHub closeout facts for the completed
`v0.26.0` release. It is a release governance ledger only: it does not change
runtime behavior, adapter behavior, Dashboard behavior, public API behavior, or
trading semantics.

Plain Chinese summary: 本文档把 `v0.26.0` 已发布后的真实 GitHub 状态写回源码树。
`ntpro-rust-only-v0.26.0` 已公开发布，hosted release gate 已成功，publish
workflow 已成功，GitHub Release body 与源码内 release notes 的哈希一致，`#812-#820`
和 `#837/#839/#841/#843/#845` 全部关闭，`v0.26.0` milestone 已关闭。这个版本仍只是
Product Hardening Foundation；它不是 automatic recovery runtime，不新增 submit，
不允许 production order mutation，不开放 Dashboard 交易控件，也不宣称产品级实盘交易终端。

## Publication Closeout

```text
release tag = ntpro-rust-only-v0.26.0
release name = NTPRO Rust-only v0.26.0
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.26.0
GitHub Release draft = false
GitHub Release prerelease = false
published at = 2026-07-07T05:29:16Z
GitHub Release updated at = 2026-07-07T12:54:42Z
target commitish = main
annotated tag object = 394bb70358766fb18919c888b0075b071ce72d33
annotated tag peeled commit = b09ec3a9f96ac718d6660b345a74cb4b7790f19a
hosted release gate = https://github.com/atxinbao/NTPRO/actions/runs/28853960135
hosted release gate workflow = Rust Cutover Release Gate
hosted release gate status = completed
hosted release gate conclusion = success
hosted release gate head SHA = b09ec3a9f96ac718d6660b345a74cb4b7790f19a
hosted release gate updated at = 2026-07-07T10:16:03Z
publish workflow = https://github.com/atxinbao/NTPRO/actions/runs/28867689146
publish workflow name = Rust Cutover Publish Release
publish workflow status = completed
publish workflow conclusion = success
publish workflow head SHA = a7f5de3086ae1624d9b4870cfda5ce47f5f4dd5c
publish workflow updated at = 2026-07-07T12:54:47Z
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
remote reconstruction required = true
release-publication-evidence/ = generated artifact, not sole proof
```

## Release Body Hash

```text
release body sha256 = ab2ed2be9b10371e4aabea74c7314c1ebae791ffd4e3d129d0f4c208b15a985e
tracked release notes sha256 = ab2ed2be9b10371e4aabea74c7314c1ebae791ffd4e3d129d0f4c208b15a985e
release body matches tracked release notes = true
release body normalized line count = 152
tracked release notes normalized line count = 152
strict release body match required = true
```

## Final Release Scope

```text
#812 V260-000 v0.26.0 intake gate and v0.25.1 dependency proof = CLOSED
#813 V260-001 product hardening boundary contract = CLOSED
#814 V260-002 operator permission model and role boundary evidence = CLOSED
#815 V260-003 operation audit trail and immutable action evidence = CLOSED
#816 V260-004 deployment topology and environment provenance model = CLOSED
#817 V260-005 upgrade rollback and release operation runbook evidence = CLOSED
#818 V260-006 SLO runbook productization and long-run stability evidence = CLOSED
#819 V260-007 Dashboard product hardening read-only admin boundary surface = CLOSED
#820 V260-008 v26 release gates and strict provenance = CLOSED
#837 V260-009 v26 release golden-trace file gate correction = CLOSED
#839 V260-010 v26 release nested historical gate correction = CLOSED
#841 V260-011 release publication guard notes line = CLOSED
#843 V260-012 prepublish publication guard split = CLOSED
#845 V260-013 update existing release body after gate = CLOSED
PR #838 V260-009 merge commit = 70892e473ef0fd63618fd2bb968e8b8fb61cf4f0
PR #840 V260-010 merge commit = eff3e7045e14a5ae9ffba537799fb8b6a7132c00
PR #842 V260-011 merge commit = 7147a5e18a8527730cfb91944eada52eaa9e041c
PR #844 V260-012 merge commit = 959bc488ee430d76a8eb44ea0716f22b232e39d4
PR #846 V260-013 merge commit = b09ec3a9f96ac718d6660b345a74cb4b7790f19a
V260 final release issue set = 14/14 closed
V260 final release PR set = 5/5 corrective release-publication PRs merged
V260 final release scope issue count = 14
V260 final release scope evidence count = 14
corrective release-publication scope changes runtime behavior = false
corrective release-publication scope changes trading behavior = false
```

## Milestone Closeout

```text
v0.26.0 milestone = #18
v0.26.0 milestone title = v0.26.0
v0.26.0 milestone state = closed
v0.26.0 open_issues = 0
v0.26.0 closed_issues = 14
v0.26.0 closed_at = 2026-07-07T08:53:35Z

v0.26.1 milestone = #19
v0.26.1 start rule = patch closeout before v0.27.0 capability work

v0.27.0 milestone = #20
v0.27.0 start rule = blocked until all V261 issues are closed and v0.26.1 release evidence is published
```

## Boundary Statement

```text
v0.26.0 published but runtime capability = Product Hardening Foundation
product_hardening_foundation = true
automatic_recovery_runtime = false
product_grade_live_trading_terminal = false
new_submit_capability = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
execution_adapter_call_allowed = false
adapter_send_allowed = false
live_exchange_request_allowed = false
network_attempted = false
implicit_retry_allowed = false
retry_scheduler_enabled = false
automatic_cancel_allowed = false
automatic_retry_allowed = false
automatic_remediation_allowed = false
automatic_recovery_allowed = false
automatic_operation_action_allowed = false
strategy_driven_production_execution_allowed = false
shared_approval_consumption_allowed = false
cancel_replace_amend_send_allowed = false
flatten_allowed = false
dashboard_operation_controls_enabled = false
dashboard_trading_controls_enabled = false
dashboard_order_controls_enabled = false
dashboard_approval_controls_enabled = false
dashboard_cancel_controls_enabled = false
dashboard_retry_controls_enabled = false
dashboard_submit_controls_enabled = false
dashboard_replace_controls_enabled = false
dashboard_amend_controls_enabled = false
dashboard_flatten_controls_enabled = false
dashboard_remediation_controls_enabled = false
trader_terminal_order_ticket_enabled = false
trader_terminal_live_trading_claim = false
manual_operation_entry_enabled = false
manual_operation_submit_allowed = false
manual_operation_cancel_allowed = false
manual_operation_retry_allowed = false
manual_operation_replace_allowed = false
manual_operation_amend_allowed = false
manual_operation_flatten_allowed = false
product_grade_trading_terminal_claim = false
```

## Reconstruction Commands

```text
gh release view ntpro-rust-only-v0.26.0 --repo atxinbao/NTPRO --json tagName,name,isDraft,isPrerelease,publishedAt,targetCommitish,url
gh run view 28853960135 --repo atxinbao/NTPRO --json status,conclusion,headSha,url,updatedAt,workflowName
gh run view 28867689146 --repo atxinbao/NTPRO --json status,conclusion,headSha,url,updatedAt,workflowName
gh api repos/atxinbao/NTPRO/milestones/18 --jq '{number,title,state,open_issues,closed_issues,closed_at}'
gh issue list --repo atxinbao/NTPRO --milestone v0.26.0 --state closed --limit 30 --json number,title,state
git rev-parse ntpro-rust-only-v0.26.0^{}
git ls-remote --tags origin ntpro-rust-only-v0.26.0
NTPRO_CURRENT_RELEASE_VERSION=v0.26.0 NTPRO_CURRENT_RELEASE_TAG=ntpro-rust-only-v0.26.0 NTPRO_CURRENT_RELEASE_NAME="NTPRO Rust-only v0.26.0" NTPRO_RELEASE_PUBLICATION_STRICT_BODY=1 scripts/ai/check_github_release_published.sh
```

## Evidence Sources

```text
GitHub issue #847 body
GitHub milestone #18 live state
GitHub Release ntpro-rust-only-v0.26.0
GitHub Actions run 28853960135
GitHub Actions run 28867689146
docs/rust-cutover/release/v0_26_0_release_notes.md
docs/rust-cutover/release/v0_26_0_readiness_report.md
docs/rust-cutover/release/v0_26_0_release_manifest.json
docs/rust-cutover/evidence/V260-008.md
docs/rust-cutover/evidence/V260-009.md
docs/rust-cutover/evidence/V260-010.md
docs/rust-cutover/evidence/V260-011.md
docs/rust-cutover/evidence/V260-012.md
docs/rust-cutover/evidence/V260-013.md
```

## Next Step

After this evidence is merged through issue `#847`, proceed to `#848`
`V261-002 V260-009..013 final release scope integration` on its own branch and
PR. No `v0.27.0` implementation starts until all V261 issues are closed and
`v0.26.1` release evidence is published.
