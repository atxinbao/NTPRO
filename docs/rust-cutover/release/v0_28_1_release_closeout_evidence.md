# NTPRO v0.28.1 Release Closeout Evidence

Date: 2026-07-09
Executor: Codex
Task: `V281-010` / GitHub issue `#948`
Milestone: `v0.28.1`
Status: CLOSEOUT EVIDENCE RECORDED

## Summary

This source-controlled closeout ledger records the live GitHub closeout facts
for the completed `v0.28.1` release. It is not a generated artifact and must
not be replaced by `release-publication-evidence/*` as the sole proof.

Plain Chinese summary: 本文档把 `v0.28.1` 已发布后的真实 GitHub 状态写回源码树。
`ntpro-rust-only-v0.28.1` 已公开发布，hosted release gate 已成功，GitHub Release
body 与源码内 release notes 的 normalized hash 一致，`#919-#925`、`#944`、`#946`、
`#948` 全部关闭，`v0.28.1` milestone 已关闭。未跟踪的 generated evidence 不能单独
作为 v0.29.0 intake 依据。

## Publication Closeout

```text
release tag = ntpro-rust-only-v0.28.1
release name = NTPRO Rust-only v0.28.1
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.28.1
GitHub Release id = 351780311
GitHub Release node id = RE_kwDOSox1D84U973X
GitHub Release draft = false
GitHub Release prerelease = false
GitHub Release created at = 2026-07-09T19:26:17Z
published at = 2026-07-09T20:57:07Z
GitHub Release updated at = 2026-07-09T20:57:07Z
target commitish = main
GitHub Release author login = atxinbao
GitHub Release author type = User
annotated tag object = 08daf1c7df9ee102722622ce927ee5c0f6635380
annotated tag peeled commit = 8b42671d5095ad5f32bc7947002900019eeb8269
origin/main release source = 8b42671d5095ad5f32bc7947002900019eeb8269
hosted release gate = https://github.com/atxinbao/NTPRO/actions/runs/29044397184
hosted release gate workflow = Rust Cutover Release Gate
hosted release gate status = completed
hosted release gate conclusion = success
hosted release gate head SHA = 8b42671d5095ad5f32bc7947002900019eeb8269
hosted release gate created at = 2026-07-09T19:27:35Z
hosted release gate completed at = 2026-07-09T20:53:43Z
hosted release gate jobs = 86/86 success
release publication after gate = pass
publication status = published_after_gate
published after hosted gate = true
release_gate_run_id = 29044397184
published_at >= release_gate_completed_at = true
release body matches tracked release notes = true
source-controlled closeout evidence = docs/rust-cutover/release/v0_28_1_release_closeout_evidence.md
release-publication-evidence/ntpro-rust-only-v0.28.1.json = generated artifact, not sole proof
generated publication evidence sole proof allowed = false
v0.29.0 intake requires this source-controlled closeout evidence = true
```

## Release Body Hash

```text
release body hash semantics = normalized_sha256
release body normalization = line_rstrip_and_outer_strip
release body normalized sha256 = 7817ff5c9d448f608cb7352cbe34d337ddad5c5538b1a2ec7298e5a6e846c3bf
tracked release notes normalized sha256 = 7817ff5c9d448f608cb7352cbe34d337ddad5c5538b1a2ec7298e5a6e846c3bf
normalized release body matches tracked release notes = true
release body normalized line count = 94
tracked release notes normalized line count = 94
release body raw sha256 = c2342e35d3b7495be83dfd686c4871d29f96cfdc5acdfe0fbdba0c7b3f249819
tracked release notes raw sha256 = c2342e35d3b7495be83dfd686c4871d29f96cfdc5acdfe0fbdba0c7b3f249819
raw release body matches tracked release notes = true
raw hash equality is diagnostic, not the acceptance rule
```

## Final Release Scope

```text
#919 V281-001 v0.28.0 release closeout evidence backfill = CLOSED
#920 V281-002 stale V280-009 evidence cleanup = CLOSED
#921 V281-003 v0.27.1 base release closeout reconciliation = CLOSED
#922 V281-004 release body hash normalization contract = CLOSED
#923 V281-005 runtime-closed terminology hardening = CLOSED
#924 V281-006 release-publish-after-gate current-release binding = CLOSED
#925 V281-007 v28.1 release gates and post-publication strict provenance = CLOSED
#944 V281-008 v28.1 release tag gate prepublication publish-after-gate fix = CLOSED
#946 V281-009 v28.1 prepublish live-current require semantics = CLOSED
#948 V281-010 v28.1 nested base gate tag-scope isolation = CLOSED
V281 final release issue set = 10/10 closed
V281 final release scope issue count = 10
V281 final release scope evidence count = 10
corrective release-publication scope changes runtime behavior = false
corrective release-publication scope changes trading behavior = false
```

## Milestone Closeout

```text
v0.28.1 milestone = #23
v0.28.1 milestone title = v0.28.1
v0.28.1 milestone state = closed
v0.28.1 open_issues = 0
v0.28.1 closed_issues = 10
v0.28.1 closed_at = 2026-07-09T20:57:58Z

v0.29.0 milestone = #24
v0.29.0 state = open
v0.29.0 start rule = gated Backend Production Readiness Foundation only
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
backend_go_live_claim = false
product_grade_trading_terminal_claim = false
```

## Reconstruction Commands

```text
gh release view ntpro-rust-only-v0.28.1 --repo atxinbao/NTPRO --json tagName,name,isDraft,isPrerelease,publishedAt,targetCommitish
gh run view 29044397184 --repo atxinbao/NTPRO --json status,conclusion,headSha,url,createdAt,updatedAt,workflowName,jobs
gh api repos/atxinbao/NTPRO/milestones/23 --jq '{number,title,state,open_issues,closed_issues,closed_at}'
gh issue list --repo atxinbao/NTPRO --milestone v0.28.1 --state closed --limit 30 --json number,title,state
git rev-parse ntpro-rust-only-v0.28.1
git rev-parse 'ntpro-rust-only-v0.28.1^{}'
git ls-remote --tags origin refs/tags/ntpro-rust-only-v0.28.1 refs/tags/ntpro-rust-only-v0.28.1^{}
NTPRO_CURRENT_RELEASE_VERSION=v0.28.1 NTPRO_CURRENT_RELEASE_TAG=ntpro-rust-only-v0.28.1 NTPRO_CURRENT_RELEASE_NAME="NTPRO Rust-only v0.28.1" scripts/ai/verify_release.sh release-publication-guard
scripts/ai/verify_release.sh v29-intake-gate
```

## Next Step

After this evidence is merged through issue `#926`, proceed to `#927`
`V290-001 backend production readiness boundary contract` on its own branch and
PR. No later V290 task may claim submit, adapter send, live exchange request,
backend go-live, or product-grade live trading readiness unless its own issue
adds explicit release-gated evidence.
