# v0.32.0 Post-Baseline Errata

Date: 2026-07-15
Executor: Codex
Applies after baseline commit: `2b955cb8a989827e3351c08c3d82d9578253e1f6`

This errata records current-route clarifications made after the published
v0.32.0 backend baseline. It does not alter the tagged source, GitHub Release,
release body, release manifest, readiness report, or closeout evidence.

Plain Chinese summary: v0.32.0 的发布事实不变。本文件只说明基线后的当前文档清理：
README 曾把 v0.31.0 与 v0.32.0 tag 混写，并把 v0.32.1 写成默认 patch。现在统一为
v0.32.0 后端冻结基线、无计划后端 patch、backend-freeze-governance 治理路线，以及
v0.33.0+ 单独立项能力路线。

## Clarifications

1. The current GitHub Release URL points to the v0.32.0 release target, not a
   v0.31.0 release target.
2. No v0.32.1 backend patch is scheduled. A patch requires proven baseline
   invalidity and the exception process in `backend_freeze_policy.md`.
3. `backend-freeze-governance` is the active post-baseline governance track.
4. v0.33.0+ work is separately scoped and inherits no forbidden production or
   trading-control capability from v0.32.0.

## Unchanged Facts

- tag: `ntpro-rust-only-v0.32.0`;
- peeled commit: `2b955cb8a989827e3351c08c3d82d9578253e1f6`;
- hosted release gate run: `29371898609`, completed successfully;
- release milestone: v0.32.0 #30, closed;
- exact release issue set: #1042-#1051, closed;
- publication evidence strategy: `source_tree_plus_github_remote`;
- all registered production and trading-control boundary flags remain false.
