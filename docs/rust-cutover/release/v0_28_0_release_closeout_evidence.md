# NTPRO v0.28.0 Release Closeout Evidence

Date: 2026-07-09
Executor: Codex
Task: `V281-001` / GitHub issue `#919`
Milestone: `v0.28.1`
Status: CLOSEOUT EVIDENCE RECORDED

## Summary

This document records the live GitHub closeout facts for the completed
`v0.28.0` release. It is a release governance ledger only: it does not change
runtime behavior, adapter behavior, Dashboard behavior, Admin Workbench
behavior, Trader Terminal behavior, public API behavior, or trading semantics.

Plain Chinese summary: 本文档把 `v0.28.0` 已发布后的真实 GitHub 状态写回源码树。
`ntpro-rust-only-v0.28.0` 已公开发布，hosted release gate 已成功，GitHub Release
body 与源码内 release notes 的哈希一致，`#893-#902` 全部关闭，`v0.28.0`
milestone 已关闭。这个版本仍只是 Backend Closure / Product Operations Runtime
Finalization；它不是产品级实盘交易终端，不新增 submit，不允许 production order
mutation，不开放 Dashboard/Admin/Trader Terminal 交易控件，也不访问 live exchange。

## Publication Closeout

```text
release tag = ntpro-rust-only-v0.28.0
release name = NTPRO Rust-only v0.28.0
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.28.0
GitHub Release id = 351169027
GitHub Release node id = RE_kwDOSox1D84U7moD
GitHub Release draft = false
GitHub Release prerelease = false
GitHub Release created at = 2026-07-08T19:13:58Z
published at = 2026-07-08T20:39:19Z
GitHub Release updated at = 2026-07-08T20:39:19Z
target commitish = main
GitHub Release author login = atxinbao
GitHub Release author type = User
annotated tag object = e511d7e1ed4945beb7331060c6850fc04eebff0d
annotated tag peeled commit = 41ef23417a4f21226cbc069de8cc31d0fa5e696e
origin/main release source = 41ef23417a4f21226cbc069de8cc31d0fa5e696e
hosted release gate = https://github.com/atxinbao/NTPRO/actions/runs/28969059200
hosted release gate workflow = Rust Cutover Release Gate
hosted release gate status = completed
hosted release gate conclusion = success
hosted release gate head SHA = 41ef23417a4f21226cbc069de8cc31d0fa5e696e
hosted release gate created at = 2026-07-08T19:14:51Z
hosted release gate completed at = 2026-07-08T20:38:03Z
hosted release gate jobs = 84/84 success
release publication after gate = pass
release publish after gate current-release binding = pass
release_gate_run_id = 28969059200
published_at >= release_gate_completed_at = true
historical fixture-only current-release proof allowed = false
release publication evidence status = published_after_gate
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
remote reconstruction required = true
release-publication-evidence/ntpro-rust-only-v0.28.0.json = generated artifact, not sole proof
```

## Publication Entry Provenance

```text
publication path = local_publish_script_after_hosted_gate
publication entrypoint = scripts/ai/publish_ntpro_release_after_gate.sh
publication entrypoint mode = local Codex shell with authenticated gh
hosted release-publish workflow used for v0.28.0 = false
v0.28.0 matching hosted release-publish workflow run = none
bounded non-workflow publication path = true
public publication after hosted release gate success = true
created_at is public publication proof = false
published_at is public publication proof = true
```

## Release Body Hash

```text
release body hash semantics = normalized_sha256
release body normalization = line_rstrip_and_outer_strip
release body normalized sha256 = a0586ed49c0154ab1bae4ceff46dcf582139a1b0324efc379662f96a092ee219
tracked release notes normalized sha256 = a0586ed49c0154ab1bae4ceff46dcf582139a1b0324efc379662f96a092ee219
normalized release body matches tracked release notes = true
release body normalized line count = 115
tracked release notes normalized line count = 115
release body raw sha256 = fb4ee2773151b5685a6b89fd75da53d4c496116fdd3bf315f532525fc1ce6b00
tracked release notes raw sha256 = fb4ee2773151b5685a6b89fd75da53d4c496116fdd3bf315f532525fc1ce6b00
raw release body matches tracked release notes = true
raw hash equality is diagnostic, not the acceptance rule
strict release body match required = normalized-exact
accepted trailing-newline-only drift = true
accepted content drift beyond normalization = false
```

## Final Release Scope

```text
#893 V280-000 v0.28.0 intake gate and v0.27.1 dependency proof = CLOSED
#894 V280-001 backend closure boundary contract and readiness matrix = CLOSED
#895 V280-002 identity and permission runtime closure = CLOSED
#896 V280-003 persistent audit storage runtime closure = CLOSED
#897 V280-004 deployment upgrade rollback orchestration runtime closure = CLOSED
#898 V280-005 telemetry SLO ingestion runtime closure = CLOSED
#899 V280-006 Admin Workbench backend state bridge closure = CLOSED
#900 V280-007 backend API contract for Trader Terminal handoff = CLOSED
#901 V280-008 backend closure fail-closed hardening = CLOSED
#902 V280-009 v28 release gates strict provenance and backend closure handoff = CLOSED
release gate / publication closeout PR #918 merge commit = 41ef23417a4f21226cbc069de8cc31d0fa5e696e
V280 final release issue set = 10/10 closed
V280 final release scope issue count = 10
V280 final release scope evidence count = 10
corrective release-publication scope changes runtime behavior = false
corrective release-publication scope changes trading behavior = false
```

## Milestone Closeout

```text
v0.28.0 milestone = #22
v0.28.0 milestone title = v0.28.0
v0.28.0 milestone state = closed
v0.28.0 open_issues = 0
v0.28.0 closed_issues = 10
v0.28.0 closed_at = 2026-07-08T19:13:33Z

v0.28.1 milestone = #23
v0.28.1 state = open
v0.28.1 start rule = patch closeout before v0.29.0 capability work

v0.29.0 milestone = #24
v0.29.0 state = open
v0.29.0 start rule = blocked until all V281 issues are closed and v0.28.1 release evidence is published
```

## Boundary Statement

```text
v0.28.0 published but runtime capability = Backend Closure / Product Operations Runtime Finalization
backend_closure_product_operations_runtime_finalization = true
frontend_product_work_complete = false
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
gh release view ntpro-rust-only-v0.28.0 --repo atxinbao/NTPRO --json tagName,name,isDraft,isPrerelease,publishedAt,targetCommitish
gh run view 28969059200 --repo atxinbao/NTPRO --json status,conclusion,headSha,url,createdAt,updatedAt,workflowName,jobs
gh api repos/atxinbao/NTPRO/milestones/22 --jq '{number,title,state,open_issues,closed_issues,closed_at}'
gh issue list --repo atxinbao/NTPRO --milestone v0.28.0 --state closed --limit 30 --json number,title,state
git rev-parse ntpro-rust-only-v0.28.0
git rev-parse 'ntpro-rust-only-v0.28.0^{}'
git ls-remote --tags origin refs/tags/ntpro-rust-only-v0.28.0 refs/tags/ntpro-rust-only-v0.28.0^{}
NTPRO_CURRENT_RELEASE_VERSION=v0.28.0 NTPRO_CURRENT_RELEASE_TAG=ntpro-rust-only-v0.28.0 NTPRO_CURRENT_RELEASE_NAME="NTPRO Rust-only v0.28.0" NTPRO_RELEASE_PUBLICATION_STRICT_BODY=1 scripts/ai/check_github_release_published.sh
```

## Evidence Sources

```text
GitHub issue #919 body
GitHub milestone #22 live state
GitHub Release ntpro-rust-only-v0.28.0
GitHub Actions run 28969059200
docs/rust-cutover/release/v0_28_0_release_notes.md
docs/rust-cutover/release/v0_28_0_readiness_report.md
docs/rust-cutover/release/v0_28_0_release_manifest.json
docs/rust-cutover/evidence/V280-009.md
release-publication-evidence/ntpro-rust-only-v0.28.0.json
```

## Next Step

After this evidence is merged through issue `#919`, proceed to `#920`
`V281-002 stale V280-009 evidence cleanup` on its own branch and PR. No
`v0.29.0` implementation starts until all V281 issues are closed and
`v0.28.1` release evidence is published.
