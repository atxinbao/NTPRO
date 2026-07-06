# v0.25.0 Post-Release Gate Split

Date: 2026-07-06
Executor: Codex
Task: `V251-005` / GitHub issue `#810`

## Summary

This contract separates v25 PR-stage validation from tag/release-stage and
post-release closeout validation. PR-stage checks may document missing tag or
open current issue state only as pre-release validation. Once
`NTPRO_RELEASE_GATE=1` or post-release closeout proof is required, missing tags,
offline publication proof, pre-publication state, open closeout issues, open
milestones, or failed hosted runs fail closed.

Plain Chinese summary: 本文档把 v25 的 PR 阶段 gate 和发布后 closeout gate 明确拆开。
PR 阶段可以在显式 pre-release 语境下出现 missing tag 或当前 issue open；发布/tag 阶段
和 post-release closeout 阶段不能用这些状态冒充发布完成证据。

## Phase Contract

```text
pre_release_phase = v25_pre_release_pr_gate
pre_release_missing_tag = allowed_only_when_NTPRO_RELEASE_GATE_is_not_1
pre_release_current_issue_open = allowed_until_tag_publication
pre_release_offline_publication = allowed_only_with_explicit_pr_mode
pre_release_output = historical_pre_release_validation_not_closeout_evidence

tag_release_phase = v25_tag_release_gate
tag_release_NTPRO_RELEASE_GATE = required
tag_release_missing_tag = fail_closed
tag_release_head_tag_match = required
tag_release_github_release = required_non_draft_non_prerelease
tag_release_hosted_run_success = required
tag_release_current_issue_state_OPEN = fail_closed
tag_release_corrective_issue_804_OPEN = fail_closed
tag_release_milestone_open = fail_closed
tag_release_output = released_tag_gate_verified

post_release_closeout_phase = v25_post_release_closeout_gate
post_release_missing_tag = fail_closed
post_release_offline_publication = fail_closed
post_release_pre_publication_state = fail_closed
post_release_current_issue_state_OPEN = fail_closed
post_release_corrective_issue_804_OPEN = fail_closed
post_release_milestone_open = fail_closed
post_release_output = released_closeout_verified

v0_26_start_gate_without_v25_1_release_evidence = fail_closed
```

## Runtime Boundary

This is release governance only. It does not change runtime behavior, public
API behavior, adapter behavior, Dashboard behavior, or trading semantics.

```text
new_submit_capability = false
production_order_mutation_allowed = false
execution_adapter_call_allowed = false
adapter_send_allowed = false
live_exchange_request_allowed = false
retry_scheduler_enabled = false
automatic_remediation_allowed = false
dashboard_operation_controls_enabled = false
dashboard_trading_controls_enabled = false
product_grade_live_trading_terminal_claim = false
```

## Validation

```text
scripts/ai/verify_release.sh v25.1-post-release-gate-split
```
