# v0.32.0 Backend Freeze Policy

Date: 2026-07-15
Executor: Codex
Baseline: `ntpro-rust-only-v0.32.0`
Baseline commit: `2b955cb8a989827e3351c08c3d82d9578253e1f6`
Status: ACTIVE

## Policy Decision

NTPRO freezes `v0.32.0` as the Backend Production Closeout baseline. Backend
mainline development ends at this baseline. Post-baseline governance may make
the current repository easier to operate and audit, but it must not rewrite the
published tag, GitHub Release, release package, or capability boundary.

Plain Chinese summary: v0.32.0 已经是后端基线。后续可以清理当前文档、增加防漂移
检查、治理本地生成物并收紧 GitHub 入口，但这些都是“基线后治理”，不是继续做后端
版本。默认不创建 v0.32.1，也不能从 v0.32.0 自动推导出实盘交易能力。

## Baseline Authority

The baseline is valid only when the following sources reconstruct the same
release identity:

1. the annotated tag `ntpro-rust-only-v0.32.0` peeled to commit
   `2b955cb8a989827e3351c08c3d82d9578253e1f6`;
2. the published, non-draft, non-prerelease GitHub Release for that tag;
3. hosted release gate run `29371898609`, completed successfully for the same
   commit;
4. closed milestone `v0.32.0` (#30) with exact issues `#1042-#1051`;
5. the tracked release manifest, notes, readiness report, and closeout evidence
   whose hashes are pinned in `backend_freeze_registry.json`.

The audit strategy is `source_tree_plus_github_remote`. A local generated
artifact is never sufficient as the sole publication or baseline proof.

## Immutable Surface

- The published tag and release are immutable governance facts.
- Files matching `docs/rust-cutover/release/v0_32_0_*` are frozen baseline
  evidence. Routine cleanup must not edit them.
- The registered boundary flags must remain explicit `false`.
- Historical evidence remains historical even when current route documents are
  cleaned up after the freeze.

Documentary clarification belongs in a post-baseline errata file under this
governance directory. An errata may explain a stale sentence or PR-time output;
it must not claim that the tagged source contained a change that it did not
contain.

## Allowed Post-Baseline Work

- current README, roadmap, versioning, and governance-index cleanup;
- deterministic guards that compare the active tree with the frozen registry;
- generated artifact hygiene and reproducible audit tooling;
- issue and PR intake rules;
- frontend Trader Terminal, trader UX, product deployment experience, or a new
  module that has its own approved scope.

These changes must continue to follow one issue, one branch, and one PR.

## Forbidden Inheritance

No later track inherits any of the following from v0.32.0:

- backend go-live or actual production go-live authority;
- production submit, cancel, replace, amend, flatten, or mutation authority;
- execution adapter call/send or live exchange request authority;
- implicit retry, retry scheduler, automatic remediation, automatic operation,
  or automatic recovery authority;
- Dashboard, Admin Workbench, or Trader Terminal trading controls;
- frontend completion or product-grade live trading terminal claims.

`v0.33.0+` is `separately_scoped_only`. Its issue and PR must state its owner,
boundary, dependencies, evidence, rollback, telemetry, and forbidden capability
claims without relying on v0.32.0 as authorization.

## Exception and Patch Rule

There is no scheduled v0.32.1 backend patch. A patch proposal is permitted only
when evidence proves that the frozen baseline itself is invalid, for example:

- tag, peeled commit, release body, or hosted gate identity cannot be reconciled;
- a registered boundary flag is missing or true in the tagged baseline;
- the exact release issue scope or publication ordering cannot be reconstructed.

Such a proposal requires a dedicated `backend-freeze-exception` issue, explicit
owner authorization, impact and rollback analysis, corrected audit evidence,
and a separately reviewed release decision. Documentation cleanup or generated
artifact noise is not sufficient reason to publish a patch.

## Change Classification

Every post-freeze issue and PR must be classified as one of:

- `baseline-preserving-governance`;
- `v33-separately-scoped`;
- `backend-freeze-exception`.

Anything unclassified or claiming inherited production trading authority fails
closed and must not merge.
