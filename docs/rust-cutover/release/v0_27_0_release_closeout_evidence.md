# NTPRO v0.27.0 Release Closeout Evidence

Date: 2026-07-08
Executor: Codex
Task: `V271-001` / GitHub issue `#887`
Milestone: `v0.27.1`
Status: CLOSEOUT EVIDENCE RECORDED

## Summary

This document records the live GitHub closeout facts for the completed
`v0.27.0` release. It is a release governance ledger only: it does not change
runtime behavior, adapter behavior, Dashboard behavior, Admin Workbench
behavior, public API behavior, or trading semantics.

Plain Chinese summary: 本文档把 `v0.27.0` 已发布后的真实 GitHub 状态写回源码树。
`ntpro-rust-only-v0.27.0` 已公开发布，hosted release gate 已成功，GitHub Release
body 与源码内 release notes 的哈希一致，`#853-#861/#883/#885` 全部关闭，
`v0.27.0` milestone 已关闭。这个版本仍只是 Product Operations Runtime
Integration Foundation；它不是 automatic remediation runtime，不新增 submit，
不允许 production order mutation，不开放 Dashboard/Admin 交易控件，也不宣称产品级
实盘交易终端。

## Publication Closeout

```text
release tag = ntpro-rust-only-v0.27.0
release name = NTPRO Rust-only v0.27.0
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.27.0
GitHub Release draft = false
GitHub Release prerelease = false
published at = 2026-07-08T07:31:08Z
GitHub Release updated at = 2026-07-08T07:31:08Z
target commitish = main
annotated tag object = 9f647dc2ee78b4e97435a7a3282ce4f366123a33
annotated tag peeled commit = 67db6d2c4d2f5b922f9e52e7d20588cb41f972f3
origin/main release source = 67db6d2c4d2f5b922f9e52e7d20588cb41f972f3
hosted release gate = https://github.com/atxinbao/NTPRO/actions/runs/28921344889
hosted release gate workflow = Rust Cutover Release Gate
hosted release gate status = completed
hosted release gate conclusion = success
hosted release gate head SHA = 67db6d2c4d2f5b922f9e52e7d20588cb41f972f3
hosted release gate created at = 2026-07-08T06:04:46Z
hosted release gate completed at = 2026-07-08T07:29:57Z
hosted release gate jobs = 82/82 success
release publication after gate = pass
release publication evidence status = published_after_gate
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
remote reconstruction required = true
release-publication-evidence/ntpro-rust-only-v0.27.0.json = generated artifact, not sole proof
```

## Publication Entry Provenance

```text
publication entry provenance = docs/rust-cutover/release/v0_27_0_publication_entry_provenance.md
publication path = local_publish_script_after_hosted_gate
publication entrypoint = scripts/ai/publish_ntpro_release_after_gate.sh
publication entrypoint mode = local Codex shell with authenticated gh
GitHub Release API author login = atxinbao
GitHub Release API author type = User
GitHub Release API release id = 350764398
GitHub Release API created_at = 2026-07-08T06:03:41Z
GitHub Release API published_at = 2026-07-08T07:31:08Z
hosted release-publish workflow used for v0.27.0 = false
v0.27.0 matching hosted release-publish workflow run = none
bounded non-workflow publication path = true
public publication after hosted release gate success = true
created_at is public publication proof = false
published_at is public publication proof = true
```

## Release Body Hash

```text
release body sha256 = 91184074bab30a50f69147697aecf19d91977d615ad313eef96fbcb2c470138b
tracked release notes sha256 = 91184074bab30a50f69147697aecf19d91977d615ad313eef96fbcb2c470138b
release body matches tracked release notes = true
release body normalized line count = 88
tracked release notes normalized line count = 88
strict release body match required = true
```

## Final Release Scope

```text
#853 V270-000 v0.27.0 intake gate and v0.26.1 dependency proof = CLOSED
#854 V270-001 product operations runtime integration boundary contract = CLOSED
#855 V270-002 external identity and permission integration foundation = CLOSED
#856 V270-003 persistent operation audit storage integration foundation = CLOSED
#857 V270-004 deployment upgrade rollback runtime orchestration foundation = CLOSED
#858 V270-005 long-run telemetry ingestion and SLO runtime evidence = CLOSED
#859 V270-006 admin workbench runtime state bridge read-only surface = CLOSED
#860 V270-007 runtime integration fail-closed and no-trading-control hardening = CLOSED
#861 V270-008 v27 release gates and strict provenance = CLOSED
#883 V270-009 v27 release publication guard version support = CLOSED
#885 V270-010 release governance golden trace category support = CLOSED
release gate / publication closeout PR #882 merge commit = 2f65a5618d69b4fb38fde6beb9e457aaa4fa0780
release gate / publication closeout PR #884 merge commit = d580a2b2bd70464d7dd89158969aba5c7dea2173
release gate / publication closeout PR #886 merge commit = 67db6d2c4d2f5b922f9e52e7d20588cb41f972f3
V270 final release issue set = 11/11 closed
V270 final release scope issue count = 11
V270 final release scope evidence count = 11
corrective release-publication scope changes runtime behavior = false
corrective release-publication scope changes trading behavior = false
```

## Milestone Closeout

```text
v0.27.0 milestone = #20
v0.27.0 milestone title = v0.27.0
v0.27.0 milestone state = closed
v0.27.0 open_issues = 0
v0.27.0 closed_issues = 11
v0.27.0 closed_at = 2026-07-08T06:02:57Z

v0.27.1 milestone = #21
v0.27.1 state = open
v0.27.1 start rule = patch closeout before v0.28.0 capability work

v0.28.0 milestone = #22
v0.28.0 state = open
v0.28.0 start rule = blocked until all V271 issues are closed and v0.27.1 release evidence is published
```

## Boundary Statement

```text
v0.27.0 published but runtime capability = Product Operations Runtime Integration Foundation
product_operations_runtime_integration_foundation = true
automatic_remediation_runtime = false
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
admin_workbench_operation_controls_enabled = false
admin_workbench_trading_controls_enabled = false
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
gh release view ntpro-rust-only-v0.27.0 --repo atxinbao/NTPRO --json tagName,name,isDraft,isPrerelease,publishedAt,targetCommitish
gh run view 28921344889 --repo atxinbao/NTPRO --json status,conclusion,headSha,url,createdAt,updatedAt,workflowName,jobs
gh api repos/atxinbao/NTPRO/milestones/20 --jq '{number,title,state,open_issues,closed_issues,closed_at}'
gh issue list --repo atxinbao/NTPRO --milestone v0.27.0 --state closed --limit 30 --json number,title,state
git rev-parse ntpro-rust-only-v0.27.0^{}
git ls-remote --tags origin refs/tags/ntpro-rust-only-v0.27.0 refs/tags/ntpro-rust-only-v0.27.0^{}
NTPRO_CURRENT_RELEASE_VERSION=v0.27.0 NTPRO_CURRENT_RELEASE_TAG=ntpro-rust-only-v0.27.0 NTPRO_CURRENT_RELEASE_NAME="NTPRO Rust-only v0.27.0" NTPRO_RELEASE_PUBLICATION_STRICT_BODY=1 scripts/ai/check_github_release_published.sh
```

## Evidence Sources

```text
GitHub issue #887 body
GitHub milestone #20 live state
GitHub Release ntpro-rust-only-v0.27.0
GitHub Actions run 28921344889
docs/rust-cutover/release/v0_27_0_publication_entry_provenance.md
docs/rust-cutover/release/v0_27_0_release_notes.md
docs/rust-cutover/release/v0_27_0_readiness_report.md
docs/rust-cutover/release/v0_27_0_release_manifest.json
docs/rust-cutover/evidence/V270-008.md
docs/rust-cutover/evidence/V270-009.md
docs/rust-cutover/evidence/V270-010.md
release-publication-evidence/ntpro-rust-only-v0.27.0.json
```

## Next Step

After this evidence is merged through issue `#887`, proceed to `#888`
`V271-002 v0.27.0 publication entry provenance` on its own branch and PR. No
`v0.28.0` implementation starts until all V271 issues are closed and `v0.27.1`
release evidence is published.
