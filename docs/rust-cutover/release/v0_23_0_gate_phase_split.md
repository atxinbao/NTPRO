# v0.23.0 Gate Phase Split

Date: 2026-07-04
Executor: Codex
Task: `V231-003`

## Purpose

Split the v23 release gates into two explicit phases so post-release closeout
checks do not reuse pre-release wording.

Plain Chinese summary: 本文件把 `v0.23.0` 的发布前 gate 和发布后 closeout gate 分开。
发布前可以允许 `#718` 保持 open，等待 tag、hosted gate、public release 和 publication
evidence；发布后必须要求 `#718` closed、`v0.23.0` milestone closed、GitHub Release
存在且非 draft/prerelease、hosted release gate 成功。

## Pre-Release Gate Contract

```text
pre_release_phase = v23_pre_release_gate
pre_release_issue_718_state = open_allowed_until_publication
pre_release_milestone_state = open_allowed_until_publication
pre_release_github_release = not_required_before_publication
pre_release_hosted_run_success = not_required_before_tag_gate
pre_release_publication_evidence = not_required_before_publication
pre_release_output = waiting_for_tag_hosted_gate_public_release_publication_evidence
```

## Post-Release Closeout Gate Contract

```text
post_release_phase = v23_post_release_closeout_gate
post_release_issue_718_state = closed_required
post_release_milestone_state = closed_required
post_release_github_release = required_non_draft_non_prerelease
post_release_hosted_run_success = required
post_release_publication_evidence = required_published_after_gate
post_release_output = released_closeout_verified
```

## Live Post-Release Inputs

```text
release tag = ntpro-rust-only-v0.23.0
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.23.0
hosted release gate run = 28673868094
hosted release gate expected conclusion = success
tag SHA = 783b024621116d50feaf418f12cb95fb95f87575
issue closeout set = #711-#718 all closed
milestone closeout = v0.23.0 closed with 0 open issues
```

## Boundary

```text
new_submit_capability = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
dashboard_operation_controls_enabled = false
product_grade_trading_terminal_claim = false
```
