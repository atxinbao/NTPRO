# NTPRO v0.24.0 Provenance Reconciliation

Date: 2026-07-05
Executor: Codex
Task: `V241-002` / GitHub issue `#771`
Milestone: `v0.24.1`
Status: TAG / MAIN / RELEASE BODY DRIFT EXPLAINED

## Summary

This report records the exact source anchors for the published `v0.24.0` tag,
the post-tag release-body sync, the V241-001 closeout commit, and the GitHub
Release body. It is a release governance record only. It does not change
runtime behavior, adapter behavior, Dashboard behavior, public API behavior, or
trading semantics.

Plain Chinese summary: 本报告解释 `v0.24.0` 发布后为什么 tag、main 和 GitHub
Release body 不完全相同。唯一的 release body 漂移来自 PR #769 的文档换行修复：
它把 Dashboard 禁用边界句合并成一行，并同步 GitHub Release body。后续 V241-001
只补发布 closeout 证据。处理策略是 patch closeout 记录，不 retag。

## Provenance Anchors

```text
release tag = ntpro-rust-only-v0.24.0
release tag commit = fff22c4e36b85098b4b32a35762a873f93d16587
release tag tree = 287adca8a02aaada2bc78d49277568751a4bbe46
release tag notes sha256 = 92cb335a5d7a071cde4be738f3d632a3b64ed56e8812f001704ae64bdd4756ca
release tag notes bytes = 5263

PR #769 release notes/body sync merge = f590023fd8e62323f3a3a5f08e970e5376ba73cb
PR #769 head = 7f33daaee9071792541e7152af3ecdd0124fb4d5
PR #769 merged at = 2026-07-05T04:03:58Z
PR #769 files changed = docs/rust-cutover/release/v0_24_0_release_notes.md only
PR #769 diff size = +1/-2
PR #769 notes sha256 = 53c7c59d2585c7b8e710c59b0707156e6c9f3107eeb9e0decf8cbc0a3c4a5570
PR #769 notes bytes = 5261

V241-001 closeout PR = #786
V241-001 closeout merge = 581d5775a3f3589e16dfbb2758432869b78a1212
V241-001 closeout head = 57966f5c44d1a10a6a43f2f0c7ecd70c352736fc
V241-001 closeout merged at = 2026-07-05T10:24:49Z
V241-001 closeout evidence = docs/rust-cutover/release/v0_24_0_release_closeout_evidence.md

GitHub Release body source = ntpro-rust-only-v0.24.0 live release body
GitHub Release body sha256 = 53c7c59d2585c7b8e710c59b0707156e6c9f3107eeb9e0decf8cbc0a3c4a5570
GitHub Release body bytes = 5261
current source-tree release notes sha256 = 53c7c59d2585c7b8e710c59b0707156e6c9f3107eeb9e0decf8cbc0a3c4a5570
current source-tree release notes equal GitHub Release body = true
```

## Explained Drift

```text
tag -> PR #769 drift = explained
drift file = docs/rust-cutover/release/v0_24_0_release_notes.md
drift reason = PR #769 repaired one release-notes line wrap so publication guard could match the complete Dashboard disabled-control sentence
drift behavior class = doc_only_release_body_sync
runtime files changed by PR #769 = false
trading behavior changed by PR #769 = false
GitHub Release body synchronized to PR #769 notes = true

PR #769 -> V241-001 drift = explained
drift reason = V241-001 added release closeout evidence and verifier after publication
release notes changed after PR #769 = false
GitHub Release body changed after PR #769 = false
runtime behavior changed by V241-001 = false
trading behavior changed by V241-001 = false
```

The only allowed release-body drift from the tag is this exact diff:

```diff
-- Dashboard order, approval, cancel, retry, submit, replace, amend, flatten, or
-  order-ticket controls;
+- Dashboard order, approval, cancel, retry, submit, replace, amend, flatten, or order-ticket controls;
```

## Retag Decision

```text
retag required = false
retag performed = false
strategy = patch_closeout_record_not_retag
reason = published tag commit passed hosted release gate and the later drift is doc-only release-body synchronization
retag escalation rule = any future retag proposal must be a separate blocking approval item
v0.24.0 tag remains unchanged = true
```

## Gate Policy

`scripts/ai/verify_release.sh v24.1-provenance-reconciliation` fails if any of
the following are true:

- the tag SHA or tag release-notes hash changes;
- PR #769 is not merged, changes any file besides `v0_24_0_release_notes.md`,
  or no longer has the documented `+1/-2` diff;
- the tag-to-PR #769 diff is not the documented Dashboard disabled-control line
  wrapping change;
- current source-tree release notes diverge from the GitHub Release body;
- the GitHub Release body hash changes without updating this reconciliation;
- V241-001 closeout is not an ancestor of the checked-out source tree;
- the manifest no longer records patch closeout instead of retag.

## Boundary Statement

```text
new_submit_capability = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
execution_adapter_call_allowed = false
live_exchange_request_allowed = false
retry_scheduler_enabled = false
dashboard_operation_controls_enabled = false
dashboard_trading_controls_enabled = false
product_grade_live_trading_terminal = false
```
