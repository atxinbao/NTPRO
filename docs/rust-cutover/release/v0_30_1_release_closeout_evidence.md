# v0.30.1 Release Closeout Evidence

Date: 2026-07-12
Executor: Codex
Release: `ntpro-rust-only-v0.30.1`
Status: CLOSEOUT EVIDENCE RECORDED

## Summary

This source-controlled closeout ledger records the live GitHub publication facts
for the completed `v0.30.1` release. It is not a generated artifact and must not
be replaced by `release-publication-evidence/*` as the sole proof.

Plain Chinese summary: 本文档把 `v0.30.1` 已发布后的真实 GitHub 状态写回源码树。
`ntpro-rust-only-v0.30.1` 已公开发布，hosted release gate 已成功，GitHub Release
body 与源码内 release notes 的 normalized/raw hash 一致，`#999-#1005` 全部关闭，
`v0.30.1` milestone 已关闭。未跟踪的 generated evidence 不能单独作为 v0.31.0
intake 或后续发布依据。

## Closeout Target

```text
release tag = ntpro-rust-only-v0.30.1
release name = NTPRO Rust-only v0.30.1
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.30.1
source-controlled closeout evidence = docs/rust-cutover/release/v0_30_1_release_closeout_evidence.md
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
generated publication evidence sole proof allowed = false
release gate before publication required = true
publication after hosted gate required = true
same tag commit hosted gate required = true
v0.31.0 start rule = hard-blocked until v0.30.1 release evidence is published
```

## Publication Closeout

```text
GitHub Release id = 352801301
GitHub Release node id = RE_kwDOSox1D84VB1IV
GitHub Release draft = false
GitHub Release prerelease = false
GitHub Release created at = 2026-07-12T13:17:36Z
GitHub Release published at = 2026-07-12T17:07:13Z
GitHub Release updated at = 2026-07-12T17:10:14Z
target commitish = main
GitHub Release author login = atxinbao
GitHub Release author type = User
annotated tag object = 17d2b48ed4df2b21f1a0b20bf739fd46f33659be
annotated tag peeled commit = 5b66335a8f625062dbcdd4f7441cfacab57b5ede
release tag tree = 21f31d20a0f1c316127b68e0f0dc797170b87cb2
origin/main release source = 5b66335a8f625062dbcdd4f7441cfacab57b5ede
HEAD release source = 5b66335a8f625062dbcdd4f7441cfacab57b5ede
remote tag object = 17d2b48ed4df2b21f1a0b20bf739fd46f33659be
remote tag peeled commit = 5b66335a8f625062dbcdd4f7441cfacab57b5ede
hosted release gate = https://github.com/atxinbao/NTPRO/actions/runs/29194173422
hosted release gate workflow = Rust Cutover Release Gate
hosted release gate status = completed
hosted release gate conclusion = success
hosted release gate head SHA = 5b66335a8f625062dbcdd4f7441cfacab57b5ede
hosted release gate created at = 2026-07-12T13:18:43Z
hosted release gate completed at = 2026-07-12T14:53:56Z
hosted release gate jobs = 94/94 success
release publication after gate = pass
release publish after gate current-release binding = pass
post-publication closeout gate = required
release_gate_ready-only artifacts after publication accepted = false
source_tree_plus_github_remote reconstruction accepted = true
generated-evidence-only proof accepted = false
release body hash semantics = normalized_sha256
publication status = published_after_gate
published after hosted gate = true
release_gate_run_id = 29194173422
published_at is public publication proof = true
published_at >= release_gate_completed_at = true
release body matches tracked release notes = true
v0.30.1 milestone = must be closed before tag gate
v0.31.0 intake gate = dependency proof may be recorded, scoped approval still required
v0.31.0 start gate contract = docs/rust-cutover/release/v0_30_1_v31_start_gate.json
```

## Release Body Hash

```text
release body hash semantics = normalized_sha256
release body normalization = line_rstrip_and_outer_strip
release body normalized sha256 = 1a9a71278ca7716a681b17667f5f7ef9c174f9eebacae0683a3c5a91cc4de4f9
tracked release notes normalized sha256 = 1a9a71278ca7716a681b17667f5f7ef9c174f9eebacae0683a3c5a91cc4de4f9
normalized release body matches tracked release notes = true
release body normalized line count = 118
tracked release notes normalized line count = 118
release body raw sha256 = 112045169e1cc733db164a19ceafe94406fb2fe93154a488e053a5b58c96e982
tracked release notes raw sha256 = 112045169e1cc733db164a19ceafe94406fb2fe93154a488e053a5b58c96e982
raw release body matches tracked release notes = true
raw hash equality is diagnostic, not the acceptance rule
```

## Issue Scope

```text
V301 final release issue set = 7/7 required
V301 exact milestone issue set = #999-#1005
#999 V301-001 = must be closed before tag gate
#1000 V301-002 = must be closed before tag gate
#1001 V301-003 = must be closed before tag gate
#1002 V301-004 = must be closed before tag gate
#1003 V301-005 = must be closed before tag gate
#1004 V301-006 = must be closed before tag gate
#1005 V301-007 = must be closed before tag gate
V301 final release issue set = 7/7 closed
V301 final release scope issue count = 7
V301 final release scope evidence count = 7
V301 exact milestone issue set = #999-#1005
V301 registered corrective-scope exception count = 0
corrective release-publication scope changes runtime behavior = false
corrective release-publication scope changes trading behavior = false
```

## Milestone Closeout

```text
v0.30.1 milestone = #27
v0.30.1 milestone title = v0.30.1
v0.30.1 milestone state = closed
v0.30.1 open_issues = 0
v0.30.1 closed_issues = 7
v0.30.1 closed_at = 2026-07-12T17:08:50Z

v0.31.0 milestone = #28
v0.31.0 start rule = scoped intake may record dependency proof but no execution authority is inherited
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
dashboard_operation_controls_enabled = false
dashboard_trading_controls_enabled = false
admin_workbench_operation_controls_enabled = false
admin_workbench_trading_controls_enabled = false
trader_terminal_order_ticket_enabled = false
manual_operation_submit_allowed = false
backend_go_live_claim = false
product_grade_trading_terminal_claim = false
```

## Verification

```text
publication guard = pass
v30.1 v31 start gate = v0.30.1 release evidence published, explicit scoped approval still required
v31 dependency proof = scripts/ai/verify_v30_1_v31_start_gate.sh
```

## Reconstruction Commands

```text
gh api repos/atxinbao/NTPRO/releases/tags/ntpro-rust-only-v0.30.1
gh run view 29194173422 --repo atxinbao/NTPRO --json status,conclusion,headSha,url,createdAt,updatedAt,workflowName,jobs
gh api repos/atxinbao/NTPRO/milestones/27 --jq '{number,title,state,open_issues,closed_issues,closed_at}'
gh issue list --repo atxinbao/NTPRO --milestone v0.30.1 --state all --limit 100 --json number,title,state
git rev-parse ntpro-rust-only-v0.30.1
git rev-parse 'ntpro-rust-only-v0.30.1^{}'
git ls-remote --tags origin refs/tags/ntpro-rust-only-v0.30.1 'refs/tags/ntpro-rust-only-v0.30.1^{}'
NTPRO_CURRENT_RELEASE_VERSION=v0.30.1 NTPRO_CURRENT_RELEASE_TAG=ntpro-rust-only-v0.30.1 NTPRO_CURRENT_RELEASE_NAME="NTPRO Rust-only v0.30.1" scripts/ai/verify_release.sh release-publication-guard
scripts/ai/verify_v31_intake_gate.sh
```
