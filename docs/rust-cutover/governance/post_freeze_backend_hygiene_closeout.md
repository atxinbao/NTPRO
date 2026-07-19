# Post-Freeze Backend Hygiene Closeout Contract

Date: 2026-07-19
Executor: Codex
Milestone: `post-freeze-backend-hygiene` (#34)
Status: SOURCE-CONTROLLED CLOSEOUT CONTRACT

## Summary

This contract closes phase-1 backend hygiene after BFH-001 through BFH-007
merge. It is post-v0.32.0 governance, not v0.32.1 and not a backend capability
release. Final BFH-007 merge, hosted run, issue, main-branch, and milestone
state must be reconstructed from live GitHub after the BFH-007 PR merges.

Plain Chinese summary: 本阶段清除了确认可删除的本地噪声，修复了失效配置和贡献入口，
把清理命令限制为明确 allowlist，并为大 fixture 建立 owner/hash/reconstruction 权威。
最终 Rust guard 阻止这些规则回退。BFH-007 合并后，GitHub live state 必须证明 7 个
issue 全部关闭、7 个 PR 全部合并、smoke 成功且 milestone #34 已关闭。

## Exact Scope And Mapping

```text
BFH issue set = #1112-#1118
BFH issue count = 7
BFH-001 issue #1112 = PR #1127, merge 6e49dab600a3c961d1f9823ab85096631c845576
BFH-002 issue #1113 = PR #1128, merge 1cb02ec11eb7d5cecbba6488002b3696bb8ba80b
BFH-003 issue #1114 = PR #1129, merge 0c5152a469348079640bd4d0de9cb0a2a2d58092
BFH-004 issue #1115 = PR #1130, merge 9016624b3ef39329492ff14291a53b2da9c440df
BFH-005 issue #1116 = PR #1131, merge db977627f7f61d464c98e2fa8ca754f85ccc0750
BFH-006 issue #1117 = PR #1132, merge 6a1651b36b7832b3b1e08ebdd0bfc975a8a23da3
BFH-007 issue #1118 = current PR, must merge before closeout
```

Prior hosted smoke mapping:

```text
PR #1127 = run 29691591818, completed/success
PR #1128 = run 29691788108, completed/success
PR #1129 = run 29692047972, completed/success
PR #1130 = run 29693295304, completed/success
PR #1131 = run 29693770385, completed/success
PR #1132 = run 29694649134, completed/success
BFH-007 PR = live hosted run required before merge
```

## Delivered Controls

- explicit authority classes for frozen source, retained audit, product/test
  input, generated output, user-owned local state, and separately scoped work;
- removal of eight Finder caches while retaining local agent state, reports,
  and large test data;
- removal of unreachable ignore exceptions and the unused `ci-pr-wheel`
  profile without deleting supported tooling;
- current Rust-only contribution routes based on `main`, Cargo, tracked
  validation, and the GitHub task protocol;
- guarded Make cleanup based only on declared build/generated allowlists, with
  dry-runs and `FORCE=1` for generated output;
- a 17-entry large fixture inventory with owner, consumer, disposition, hash,
  metadata, and deterministic reconstruction policy;
- `scripts/ai/check_backend_hygiene.sh`, wired into current governance and
  every pull-request smoke run with negative self-tests.

## Required Post-Merge Reconstruction

After the BFH-007 PR merges, every condition is required:

```text
origin/main contains all seven BFH merge commits = true
issues #1112-#1118 closed = 7/7
PRs #1127 through BFH-007 PR merged = all
required hosted smoke checks completed/success = all
milestone #34 open issues = 0
milestone #34 closed issues = 7
milestone #34 state = closed
open post-freeze-backend-hygiene PRs = 0
backend hygiene guard and negative self-tests = pass
current governance = pass
backend freeze baseline = pass, boundaries=27
v0.32.0 frozen release files changed = false
backend patch scheduled = false
```

The BFH-007 PR cannot source-control its own merge SHA, final hosted
conclusion, or post-merge milestone state without circular proof. Those facts
are bound through `source_tree_plus_github_remote` and recorded in final
GitHub issue and milestone closeout comments.

## Preserved Backend Boundary

All 27 v0.32.0 boundary flags remain explicit false. Phase 1 does not authorize
backend go-live, submit, mutation, adapter call/send, live exchange request,
retry, remediation, recovery, Dashboard/Admin/Trader Terminal trading
controls, or a product-grade live trading claim. Frozen
`docs/rust-cutover/release/v0_32_0_*` files remain unchanged.

## Next State

- v0.32.0 remains the Backend Production Closeout and backend freeze baseline;
- milestone #34 closes without creating v0.32.1;
- phase-2 issues #1120-#1126 become dependency-unblocked only after BFH-007
  merges and the milestone closeout is reconstructed;
- every phase-2 runtime change remains separately scoped and inherits no
  forbidden production or trading capability.
