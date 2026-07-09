# NTPRO v0.27.1 Release Closeout Evidence

Date: 2026-07-09
Executor: Codex
Task: `V281-003` / GitHub issue `#921`
Milestone: `v0.28.1`
Status: CLOSEOUT EVIDENCE RECORDED

## Summary

This document records the live GitHub closeout facts for the completed
`v0.27.1` release. It is a release governance ledger only: it does not change
runtime behavior, adapter behavior, Dashboard behavior, Admin Workbench
behavior, Trader Terminal behavior, public API behavior, or trading semantics.

Plain Chinese summary: 本文档把 `v0.27.1` 已发布后的真实 GitHub 状态写回源码树。
`ntpro-rust-only-v0.27.1` 已公开发布，hosted release gate 已成功，GitHub Release
body 与源码内 release notes 的哈希一致，`#887-#892` 全部关闭，`v0.27.1`
milestone 已关闭。这个版本仍只是发布治理和证据硬化补丁，不新增 submit，不允许
production order mutation，不开放 Dashboard/Admin/Trader Terminal 交易控件，也不访问
live exchange。

## Publication Closeout

```text
release tag = ntpro-rust-only-v0.27.1
release name = NTPRO Rust-only v0.27.1
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.27.1
GitHub Release id = 350940808
GitHub Release node id = RE_kwDOSox1D84U6u6I
GitHub Release draft = false
GitHub Release prerelease = false
GitHub Release created at = 2026-07-08T11:50:26Z
published at = 2026-07-08T13:18:35Z
GitHub Release updated at = 2026-07-08T13:18:35Z
target commitish = main
GitHub Release author login = atxinbao
GitHub Release author id = 254527493
GitHub Release author type = User
annotated tag object = ab379be6725243ea1b8a9ffd9631409842361344
annotated tag peeled commit = 0fdc11dc983bbfb9fe124a3f171a58fb1e7ccf19
hosted release gate = https://github.com/atxinbao/NTPRO/actions/runs/28940442369
hosted release gate workflow = Rust Cutover Release Gate
hosted release gate status = completed
hosted release gate conclusion = success
hosted release gate head SHA = 0fdc11dc983bbfb9fe124a3f171a58fb1e7ccf19
hosted release gate created at = 2026-07-08T11:52:05Z
hosted release gate completed at = 2026-07-08T13:17:36Z
hosted release gate jobs = 82/82 success
release publication after gate = pass
release publication evidence status = published_after_gate
publication evidence strategy = source_tree_plus_github_remote
source-controlled closeout evidence = docs/rust-cutover/release/v0_27_1_release_closeout_evidence.md
local generated publication evidence required in source tree = false
remote reconstruction required = true
release-publication-evidence/ntpro-rust-only-v0.27.1.json = generated artifact, not sole proof
```

## Release Body Hash

```text
release body sha256 = 74bbc4d42d8e6f70d93a63fa4b42ae684cba4ccf0ce7c06e60490d4ad3a0f3f0
tracked release notes sha256 = 74bbc4d42d8e6f70d93a63fa4b42ae684cba4ccf0ce7c06e60490d4ad3a0f3f0
release body matches tracked release notes = true
release body normalized line count = 73
tracked release notes normalized line count = 73
strict release body match required = true
```

## Final Release Scope

```text
#887 V271-001 v0.27.1 release base and dependency proof = CLOSED
#888 V271-002 v0.27.1 release governance hardening = CLOSED
#889 V271-003 v0.27.1 strict release provenance = CLOSED
#890 V271-004 v0.27.1 release surface current guard = CLOSED
#891 V271-005 v0.27.1 post-publication guard = CLOSED
#892 V271-006 v27.1 release gates strict provenance and v28 hard-block = CLOSED
V271 final release issue set = 6/6 closed
V271 final release scope issue count = 6
V271 final release scope evidence count = 6
corrective release-publication scope changes runtime behavior = false
corrective release-publication scope changes trading behavior = false
```

## Milestone Closeout

```text
v0.27.1 milestone = #21
v0.27.1 milestone title = v0.27.1
v0.27.1 milestone state = closed
v0.27.1 open_issues = 0
v0.27.1 closed_issues = 6
v0.27.1 closed_at = 2026-07-08T11:48:57Z
v0.27.1 milestone URL = https://github.com/atxinbao/NTPRO/milestone/21
```

## Boundary Statement

```text
v0.27.1 published but runtime capability = release governance and evidence hardening patch
patch_hardening_only = true
release_governance_hardening = true
product_grade_live_trading_terminal = false
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

## Reconstruction Commands

```text
gh release view ntpro-rust-only-v0.27.1 --repo atxinbao/NTPRO --json tagName,name,isDraft,isPrerelease,publishedAt,targetCommitish
gh run view 28940442369 --repo atxinbao/NTPRO --json status,conclusion,headSha,url,createdAt,updatedAt,workflowName,jobs
gh api repos/atxinbao/NTPRO/milestones/21 --jq '{number,title,state,open_issues,closed_issues,closed_at}'
gh issue list --repo atxinbao/NTPRO --milestone v0.27.1 --state closed --limit 30 --json number,title,state
git ls-remote --tags origin refs/tags/ntpro-rust-only-v0.27.1 refs/tags/ntpro-rust-only-v0.27.1^{}
NTPRO_CURRENT_RELEASE_VERSION=v0.27.1 NTPRO_CURRENT_RELEASE_TAG=ntpro-rust-only-v0.27.1 NTPRO_CURRENT_RELEASE_NAME="NTPRO Rust-only v0.27.1" NTPRO_RELEASE_PUBLICATION_STRICT_BODY=1 scripts/ai/check_github_release_published.sh
```

## Evidence Sources

```text
GitHub issue #921 body
GitHub milestone #21 live state
GitHub Release ntpro-rust-only-v0.27.1
GitHub Actions run 28940442369
docs/rust-cutover/release/v0_27_1_release_notes.md
docs/rust-cutover/release/v0_27_1_readiness_report.md
docs/rust-cutover/release/v0_27_1_release_manifest.json
docs/rust-cutover/evidence/V271-006.md
docs/rust-cutover/evidence/V280-000.md
docs/rust-cutover/release/v0_28_0_intake_gate.md
release-publication-evidence/ntpro-rust-only-v0.27.1.json
```

## Next Step

After this evidence is merged through issue `#921`, proceed to `#922`
`V281-004 release body hash normalization portability` on its own branch and
PR. No `v0.29.0` implementation starts until all V281 issues are closed and
`v0.28.1` release evidence is published.
