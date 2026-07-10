# NTPRO v0.29.0 Release Closeout Evidence

Date: 2026-07-11
Executor: Codex
Task: `V291-001` / GitHub issue `#963`
Milestone: `v0.29.1`
Status: CLOSEOUT EVIDENCE RECORDED

## Summary

This source-controlled closeout ledger records the live GitHub closeout facts
for the completed `v0.29.0` release. It is not a generated artifact and must
not be replaced by `release-publication-evidence/*` as the sole proof.

Plain Chinese summary: 本文档把 `v0.29.0` 已发布后的真实 GitHub 状态写回源码树。
`ntpro-rust-only-v0.29.0` 已公开发布，hosted release gate 已成功，GitHub Release
body 与源码内 release notes 的 normalized/raw hash 一致，`#926-#936` 和 `#961`
全部关闭，`v0.29.0` milestone 已关闭。未跟踪的 generated evidence 不能单独作为
v0.30.0 intake 或后续发布依据。

## Publication Closeout

```text
release tag = ntpro-rust-only-v0.29.0
release name = NTPRO Rust-only v0.29.0
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.29.0
GitHub Release id = 352121462
GitHub Release node id = RE_kwDOSox1D84U_PJ2
GitHub Release draft = false
GitHub Release prerelease = false
GitHub Release created at = 2026-07-10T12:10:49Z
published at = 2026-07-10T13:44:23Z
GitHub Release updated at = 2026-07-10T13:44:23Z
target commitish = main
GitHub Release author login = atxinbao
GitHub Release author type = User
annotated tag object = 25cccef7a99c6f231dac7f915f24abe882ad7f2c
annotated tag peeled commit = 85110d29867763f8d3b6395f4ff8154378b475b9
release tag tree = 8c6529cba8366e191ee5b301254b1acdc7dab74a
origin/main release source = 85110d29867763f8d3b6395f4ff8154378b475b9
HEAD release source = 85110d29867763f8d3b6395f4ff8154378b475b9
remote tag object = 25cccef7a99c6f231dac7f915f24abe882ad7f2c
remote tag peeled commit = 85110d29867763f8d3b6395f4ff8154378b475b9
hosted release gate = https://github.com/atxinbao/NTPRO/actions/runs/29091765148
hosted release gate workflow = Rust Cutover Release Gate
hosted release gate status = completed
hosted release gate conclusion = success
hosted release gate head SHA = 85110d29867763f8d3b6395f4ff8154378b475b9
hosted release gate created at = 2026-07-10T12:11:26Z
hosted release gate completed at = 2026-07-10T13:43:15Z
hosted release gate jobs = 88/88 success
release publication after gate = pass
publication status = published_after_gate
published after hosted gate = true
release_gate_run_id = 29091765148
published_at >= release_gate_completed_at = true
release body matches tracked release notes = true
source-controlled closeout evidence = docs/rust-cutover/release/v0_29_0_release_closeout_evidence.md
release-publication-evidence/ntpro-rust-only-v0.29.0.json = generated artifact, not sole proof
generated publication evidence sole proof allowed = false
v0.30.0 intake requires this source-controlled closeout evidence = true
```

## Release Body Hash

```text
release body hash semantics = normalized_sha256
release body normalization = line_rstrip_and_outer_strip
release body normalized sha256 = 7812d750cb4f5fe3d4a71b041c6cf9e4c652a938cf4a654dfc70e1e22776ef43
tracked release notes normalized sha256 = 7812d750cb4f5fe3d4a71b041c6cf9e4c652a938cf4a654dfc70e1e22776ef43
normalized release body matches tracked release notes = true
release body normalized line count = 101
tracked release notes normalized line count = 101
release body raw sha256 = ccd4811acfc48cbca4514aa936e4f225428e0fd129db1371c88738ad8ec5c356
tracked release notes raw sha256 = ccd4811acfc48cbca4514aa936e4f225428e0fd129db1371c88738ad8ec5c356
raw release body matches tracked release notes = true
raw hash equality is diagnostic, not the acceptance rule
```

## Final Release Scope

```text
#926 V290-000 v0.29.0 intake gate and v0.28.1 dependency proof = CLOSED
#927 V290-001 backend production readiness boundary contract = CLOSED
#928 V290-002 persistent audit storage production readiness = CLOSED
#929 V290-003 telemetry SLO ingestion production readiness = CLOSED
#930 V290-004 permission source production readiness = CLOSED
#931 V290-005 read-only backend API production readiness = CLOSED
#932 V290-006 deployment config and runbook production readiness = CLOSED
#933 V290-007 monitoring alert incident production readiness = CLOSED
#934 V290-008 canary rollback DR preflight readiness = CLOSED
#935 V290-009 backend production readiness fail-closed hardening = CLOSED
#936 V290-010 v29 release gates and v30 go-live candidate handoff = CLOSED
#961 V290-011 v29 hosted release gate JSON payload fix = CLOSED
V290 final release issue set = 12/12 closed
V290 final release scope issue count = 12
V290 final release scope evidence count = 12
V290 exact milestone issue set = #926-#936, #961
V290 registered corrective-scope exception count = 1
corrective release-publication scope changes runtime behavior = false
corrective release-publication scope changes trading behavior = false
```

## Milestone Closeout

```text
v0.29.0 milestone = #24
v0.29.0 milestone title = v0.29.0
v0.29.0 milestone state = closed
v0.29.0 open_issues = 0
v0.29.0 closed_issues = 12
v0.29.0 closed_at = 2026-07-10T13:45:02Z

v0.29.1 milestone = #25
v0.29.1 state = open
v0.29.1 start rule = patch closeout only; no backend go-live or trading controls
v0.30.0 milestone = #26
v0.30.0 start rule = hard-blocked until v0.29.1 release evidence is published
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

## Reconstruction Commands

```text
gh api repos/atxinbao/NTPRO/releases/tags/ntpro-rust-only-v0.29.0
gh run view 29091765148 --repo atxinbao/NTPRO --json status,conclusion,headSha,url,createdAt,updatedAt,workflowName,jobs
gh api repos/atxinbao/NTPRO/milestones/24 --jq '{number,title,state,open_issues,closed_issues,closed_at}'
gh issue list --repo atxinbao/NTPRO --milestone v0.29.0 --state all --limit 100 --json number,title,state
git rev-parse ntpro-rust-only-v0.29.0
git rev-parse 'ntpro-rust-only-v0.29.0^{}'
git ls-remote --tags origin refs/tags/ntpro-rust-only-v0.29.0 'refs/tags/ntpro-rust-only-v0.29.0^{}'
NTPRO_CURRENT_RELEASE_VERSION=v0.29.0 NTPRO_CURRENT_RELEASE_TAG=ntpro-rust-only-v0.29.0 NTPRO_CURRENT_RELEASE_NAME="NTPRO Rust-only v0.29.0" scripts/ai/verify_release.sh release-publication-guard
scripts/ai/verify_release.sh v29.1-release-closeout-evidence
```

## Next Step

After this evidence is merged through issue `#963`, proceed to `#964`
`V291-002 v29 release-publish-after-gate current-release binding` on its own
branch and PR. No later V291 or V300 task may claim submit, adapter send, live
exchange request, backend go-live, or product-grade live trading readiness
unless its own issue adds explicit release-gated evidence.
