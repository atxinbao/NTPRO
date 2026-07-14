# v0.31.0 Release Closeout Evidence

Date: 2026-07-15
Executor: Codex
Release: `ntpro-rust-only-v0.31.0`
Status: CLOSEOUT EVIDENCE RECORDED

## Summary

This source-controlled closeout ledger records the live GitHub publication facts
for the completed `v0.31.0` release. It is not a generated artifact and must
not be replaced by `release-publication-evidence/*` as the sole proof.

Plain Chinese summary: 本文档把 `v0.31.0` 已发布后的真实 GitHub 状态写回源码树。
`ntpro-rust-only-v0.31.0` 已公开发布，hosted release gate 已成功，publish workflow
已成功，`#1006-#1015 plus #1033` 全部关闭，`v0.31.0` milestone 已关闭。v0.32.0 仍必须
等待 v0.31.1 全部任务关闭并发布 `ntpro-rust-only-v0.31.1` 证据后才能作为后端收尾版本继续。

## Closeout Target

```text
release tag = ntpro-rust-only-v0.31.0
release name = NTPRO Rust-only v0.31.0
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.31.0
source-controlled closeout evidence = docs/rust-cutover/release/v0_31_0_release_closeout_evidence.md
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
generated publication evidence sole proof allowed = false
release gate before publication required = true
publication after hosted gate required = true
same tag commit hosted gate required = true
v0.32.0 backend closeout start rule = hard-blocked until v0.31.1 release evidence is published
```

## Publication Closeout

```text
GitHub Release id = 353477788
GitHub Release node id = RE_kwDOSox1D84VEaSc
GitHub Release draft = false
GitHub Release prerelease = false
GitHub Release created at = 2026-07-13T21:20:19Z
GitHub Release published at = 2026-07-13T22:42:06Z
target commitish = main
GitHub Release author login = github-actions[bot]
annotated tag object = 8c0d71f6e6ef2a890daf1e07299c658fa187a262
annotated tag peeled commit = 14e582868cb9c18b8b26a1fd50ab21cb56f8f1a1
release tag tree = 7ace21b252c8a14b66eae9642baa2fd4ad3b895a
origin/main release source = 14e582868cb9c18b8b26a1fd50ab21cb56f8f1a1
remote tag object = 8c0d71f6e6ef2a890daf1e07299c658fa187a262
remote tag peeled commit = 14e582868cb9c18b8b26a1fd50ab21cb56f8f1a1
hosted release gate = https://github.com/atxinbao/NTPRO/actions/runs/29285960500
hosted release gate workflow = Rust Cutover Release Gate
hosted release gate status = completed
hosted release gate conclusion = success
hosted release gate head SHA = 14e582868cb9c18b8b26a1fd50ab21cb56f8f1a1
hosted release gate created at = 2026-07-13T21:20:25Z
hosted release gate completed at = 2026-07-13T22:41:01Z
hosted release gate jobs = 96/96 success
publish workflow = https://github.com/atxinbao/NTPRO/actions/runs/29290691138
publish workflow name = Rust Cutover Publish Release
publish workflow status = completed
publish workflow conclusion = success
publish workflow head SHA = 14e582868cb9c18b8b26a1fd50ab21cb56f8f1a1
publish workflow created at = 2026-07-13T22:41:28Z
publish workflow completed at = 2026-07-13T22:42:11Z
publish workflow jobs = 1/1 success
release publication after gate = pass
release body hash semantics = normalized_sha256
publication status = published_after_gate
published after hosted gate = true
release_gate_run_id = 29285960500
publish_workflow_run_id = 29290691138
published_at is public publication proof = true
published_at >= release_gate_completed_at = true
source_tree_plus_github_remote reconstruction accepted = true
generated-evidence-only proof accepted = false
```

## Release Body Hash

```text
release body hash semantics = normalized_sha256
release body normalization = line_rstrip_and_outer_strip
release body normalized sha256 = 1fed8bfaa9f73d24d4392ed203cbf12d6c90d0cdabcbc29ec9c87151041aa355
tracked release notes normalized sha256 at publication = 1fed8bfaa9f73d24d4392ed203cbf12d6c90d0cdabcbc29ec9c87151041aa355
normalized release body matched tracked release notes at publication = true
release body normalized line count = 101
tracked release notes normalized line count at publication = 101
release body raw sha256 = 92bd2f48a42fc706173aa1971efe95c1f3559a86fa57e54f71b8d3692b922744
tracked release notes raw sha256 at publication = 92bd2f48a42fc706173aa1971efe95c1f3559a86fa57e54f71b8d3692b922744
raw release body matched tracked release notes at publication = true
raw hash equality is diagnostic, not the acceptance rule
GitHub Release body released-state reconciliation issue = V311-003 / #1038
```

## Current Body Reconciliation

```text
GitHub Release body released-state reconciliation = V311-003 / #1038
GitHub Release body updated from tracked release notes = true
current release body status = RELEASED
current release body normalized sha256 = 2b951baf48c01209b10a9b3ec70b9d452739fa21fcf3685c098c90fdf00f0fcb
current tracked release notes normalized sha256 = 2b951baf48c01209b10a9b3ec70b9d452739fa21fcf3685c098c90fdf00f0fcb
current normalized release body matches tracked release notes = true
current release body raw sha256 = c8c83713945d6d42b3421a72a423bc5ab19e148fcd1342f3629da71ef76763b0
current tracked release notes raw sha256 = c8c83713945d6d42b3421a72a423bc5ab19e148fcd1342f3629da71ef76763b0
current raw release body matches tracked release notes = true
```

## Issue Scope

```text
V310 final release issue set = 11/11 required
V310 exact milestone issue set = #1006-#1015 plus #1033
#1006 V310-000 = closed
#1007 V310-001 = closed
#1008 V310-002 = closed
#1009 V310-003 = closed
#1010 V310-004 = closed
#1011 V310-005 = closed
#1012 V310-006 = closed
#1013 V310-007 = closed
#1014 V310-008 = closed
#1015 V310-009 = closed
#1033 V310-010 = closed
V310 final release issue set = 11/11 closed
V310 final release scope issue count = 11
V310 final release scope evidence count = 11
V310 registered corrective-scope exception count = 1
corrective release-publication scope changes runtime behavior = false
corrective release-publication scope changes trading behavior = false
```

## Milestone Closeout

```text
v0.31.0 milestone = #28
v0.31.0 milestone title = v0.31.0
v0.31.0 milestone state = closed
v0.31.0 open_issues = 0
v0.31.0 closed_issues = 11
v0.31.0 closed_at = 2026-07-13T20:18:01Z

v0.31.1 milestone = #29
v0.31.1 start rule = governance closeout patch
v0.32.0 milestone = #30
v0.32.0 backend closeout start rule = blocked until all V311 issues close and ntpro-rust-only-v0.31.1 release evidence is published
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

## Verification

```text
v31.1 post-publication closeout evidence = scripts/ai/verify_v31_1_post_publication_closeout_evidence.sh
source mode = required
live mode = required for V311-001 PR evidence
```

## Reconstruction Commands

```text
gh release view ntpro-rust-only-v0.31.0 --repo atxinbao/NTPRO --json tagName,name,isDraft,isPrerelease,url,publishedAt,createdAt,targetCommitish,author,body,databaseId,id
gh run view 29285960500 --repo atxinbao/NTPRO --json status,conclusion,headSha,url,createdAt,updatedAt,workflowName,jobs
gh run view 29290691138 --repo atxinbao/NTPRO --json status,conclusion,headSha,url,createdAt,updatedAt,workflowName,jobs
gh api repos/atxinbao/NTPRO/milestones/28 --jq '{number,title,state,open_issues,closed_issues,closed_at}'
gh issue list --repo atxinbao/NTPRO --milestone v0.31.0 --state all --limit 100 --json number,title,state
git rev-parse ntpro-rust-only-v0.31.0
git rev-parse 'ntpro-rust-only-v0.31.0^{}'
git ls-remote --tags origin refs/tags/ntpro-rust-only-v0.31.0 'refs/tags/ntpro-rust-only-v0.31.0^{}'
scripts/ai/verify_v31_1_post_publication_closeout_evidence.sh source
scripts/ai/verify_v31_1_post_publication_closeout_evidence.sh live
```
