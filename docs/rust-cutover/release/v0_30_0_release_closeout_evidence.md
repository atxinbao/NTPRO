# NTPRO v0.30.0 Release Closeout Evidence

Date: 2026-07-11
Executor: Codex
Task: `V301-001` / GitHub issue `#999`
Milestone: `v0.30.1`
Status: CLOSEOUT EVIDENCE RECORDED

## Summary

This source-controlled closeout ledger records the live GitHub closeout facts
for the completed `v0.30.0` release. It is not a generated artifact and must
not be replaced by `release-publication-evidence/*` as the sole proof.

Plain Chinese summary: 本文档把 `v0.30.0` 已发布后的真实 GitHub 状态写回源码树。
`ntpro-rust-only-v0.30.0` 已公开发布，hosted release gate 已成功，GitHub Release
body 与源码内 release notes 的 normalized/raw hash 一致，`#969-#980` 全部关闭，
`v0.30.0` milestone 已关闭。未跟踪的 generated evidence 不能单独作为 v0.31.0
intake 或后续发布依据。

## Publication Closeout

```text
release tag = ntpro-rust-only-v0.30.0
release name = NTPRO Rust-only v0.30.0
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.30.0
GitHub Release id = 352435565
GitHub Release node id = RE_kwDOSox1D84VAb1t
GitHub Release draft = false
GitHub Release prerelease = false
GitHub Release created at = 2026-07-11T04:17:26Z
published at = 2026-07-11T05:37:06Z
GitHub Release updated at = 2026-07-11T05:37:06Z
target commitish = main
GitHub Release author login = atxinbao
GitHub Release author type = User
annotated tag object = e1c50b6189790322998fee9ee3d6d00e850b8c79
annotated tag peeled commit = 0f0949156401fa6e6016c0160697e7090a6da788
release tag tree = 242ac7360f5fe2357a158e11b202ecf4dbd49c3b
origin/main release source = 0f0949156401fa6e6016c0160697e7090a6da788
HEAD release source = 0f0949156401fa6e6016c0160697e7090a6da788
remote tag object = e1c50b6189790322998fee9ee3d6d00e850b8c79
remote tag peeled commit = 0f0949156401fa6e6016c0160697e7090a6da788
hosted release gate = https://github.com/atxinbao/NTPRO/actions/runs/29139384219
hosted release gate workflow = Rust Cutover Release Gate
hosted release gate status = completed
hosted release gate conclusion = success
hosted release gate head SHA = 0f0949156401fa6e6016c0160697e7090a6da788
hosted release gate created at = 2026-07-11T04:17:58Z
hosted release gate completed at = 2026-07-11T05:35:59Z
hosted release gate jobs = 92/92 success
release publication after gate = pass
release publish after gate current-release binding = pass
post-publication closeout gate = required
release_gate_ready-only artifacts after publication accepted = false
source_tree_plus_github_remote reconstruction accepted = true
generated-evidence-only proof accepted = false
release body hash semantics = normalized_sha256
publication status = published_after_gate
published after hosted gate = true
release_gate_run_id = 29139384219
published_at is public publication proof = true
published_at >= release_gate_completed_at = true
release body matches tracked release notes = true
source-controlled closeout evidence = docs/rust-cutover/release/v0_30_0_release_closeout_evidence.md
release-publication-evidence/ntpro-rust-only-v0.30.0.json = generated artifact, not sole proof
generated publication evidence sole proof allowed = false
historical fixture-only current-release proof allowed = false
published_release manifest field populated = true
post_publication_closeout manifest field populated = true
v0.31.0 intake requires v0.30.1 release evidence = true
v0.30.0 publication evidence alone unlocks v0.31.0 = false
v0.31.0 start gate = blocked_until_v301_release_evidence_published
v0.30.1 release evidence required before v0.31.0 intake = true
```

## Release Body Hash

```text
release body hash semantics = normalized_sha256
release body normalization = line_rstrip_and_outer_strip
release body normalized sha256 = 5d0e93c8f56c71b19a7ca8d8eeaa328bcecd1d185305b07930de1e53f55564e4
tracked release notes normalized sha256 = 5d0e93c8f56c71b19a7ca8d8eeaa328bcecd1d185305b07930de1e53f55564e4
normalized release body matches tracked release notes = true
release body normalized line count = 113
tracked release notes normalized line count = 113
release body raw sha256 = 41354e181696c095c383e1f8be07cf5383b563634f8002a437b7fdfc5e3d3e24
tracked release notes raw sha256 = 41354e181696c095c383e1f8be07cf5383b563634f8002a437b7fdfc5e3d3e24
raw release body matches tracked release notes = true
raw hash equality is diagnostic, not the acceptance rule
```

## Final Release Scope

```text
#969 V300-000 v0.30.0 intake gate and v0.29.1 dependency proof = CLOSED
#970 V300-001 backend go-live candidate boundary contract = CLOSED
#971 V300-002 production deployment plan and environment readiness = CLOSED
#972 V300-003 runtime enablement boundary and controlled feature flags = CLOSED
#973 V300-004 operator approval freeze and change-window lifecycle = CLOSED
#974 V300-005 canary execution preflight and no-default-execution gate = CLOSED
#975 V300-006 rollback and disaster recovery execution boundary = CLOSED
#976 V300-007 production config provenance and venue connectivity readiness = CLOSED
#977 V300-008 telemetry SLO gate and incident freeze integration = CLOSED
#978 V300-009 audit retention and evidence export readiness = CLOSED
#979 V300-010 go-no-go runbook and live readiness decision record = CLOSED
#980 V300-011 v30 release gates and v31 production enablement handoff = CLOSED
V300 final release issue set = 12/12 closed
V300 final release scope issue count = 12
V300 final release scope evidence count = 12
V300 exact milestone issue set = #969-#980
V300 registered corrective-scope exception count = 0
corrective release-publication scope changes runtime behavior = false
corrective release-publication scope changes trading behavior = false
```

## Milestone Closeout

```text
v0.30.0 milestone = #26
v0.30.0 milestone title = v0.30.0
v0.30.0 milestone state = closed
v0.30.0 open_issues = 0
v0.30.0 closed_issues = 12
v0.30.0 closed_at = 2026-07-11T05:37:42Z

v0.30.1 milestone = #27
v0.30.1 state = open
v0.30.1 start rule = patch closeout only; no backend go-live or trading controls
v0.31.0 milestone = #28
v0.31.0 start rule = hard-blocked until v0.30.1 release evidence is published
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
actual_backend_production_go_live_allowed = false
product_grade_trading_terminal_claim = false
product_grade_live_trading_terminal_claim = false
```

## Reconstruction Commands

```text
gh api repos/atxinbao/NTPRO/releases/tags/ntpro-rust-only-v0.30.0
gh run view 29139384219 --repo atxinbao/NTPRO --json status,conclusion,headSha,url,createdAt,updatedAt,workflowName,jobs
gh api repos/atxinbao/NTPRO/milestones/26 --jq '{number,title,state,open_issues,closed_issues,closed_at}'
gh issue list --repo atxinbao/NTPRO --milestone v0.30.0 --state all --limit 100 --json number,title,state
git rev-parse ntpro-rust-only-v0.30.0
git rev-parse 'ntpro-rust-only-v0.30.0^{}'
git ls-remote --tags origin refs/tags/ntpro-rust-only-v0.30.0 'refs/tags/ntpro-rust-only-v0.30.0^{}'
NTPRO_CURRENT_RELEASE_VERSION=v0.30.0 NTPRO_CURRENT_RELEASE_TAG=ntpro-rust-only-v0.30.0 NTPRO_CURRENT_RELEASE_NAME="NTPRO Rust-only v0.30.0" scripts/ai/verify_release.sh release-publication-guard
scripts/ai/verify_v30_1_release_closeout_evidence.sh
scripts/ai/verify_v30_1_release_publish_after_gate_current_binding.sh
scripts/ai/verify_v30_1_post_publication_closeout_gate.sh source
scripts/ai/verify_v30_1_post_publication_closeout_gate.sh live
```

## Next Step

After issue `#1001` is merged, proceed to `#1002`
`V301-004 V300-011 stale pre-tag evidence cleanup` on its own branch and PR.
No later V301, V310, or v0.31.0 task may claim submit, adapter send, live
exchange request, backend go-live, or product-grade live trading readiness
unless its own issue adds explicit release-gated evidence.
