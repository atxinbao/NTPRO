# DEXG-001 Docs And Examples Authority Map

Date: 2026-07-15
Executor: Codex
GitHub issue: #1080
Milestone: post-backend-docs-examples-governance
Status: READY FOR PR

## Goal

Classify every docs/examples cleanup surface before tracked deletion begins,
while preserving the v0.32.0 backend baseline and the complete Rust cutover
audit chain.

Plain Chinese summary: 本任务先固定清理边界，不删除 tracked 公共文档。它明确保留
完整 `docs/rust-cutover/` 审计体系和 `examples/rust/`，把 legacy Python API、教程、
集成和概念文档交给后续独立 issue，并确认 Finder/Python/build 生成物不是发布证据。

## Dependency

None. This is the root task for GitHub issue #1080 and milestone #32.

## Scope

Included:

- add the docs/examples authority map;
- record protected, retained, rewrite, removable, and ephemeral surfaces;
- bind DEXG-001 through DEXG-008 to explicit ownership and dependencies;
- verify that `docs/` and `examples/` contain no `.DS_Store`.

Not included:

- deleting tracked public documentation;
- changing the frozen v0.32.0 release package;
- changing runtime behavior, trading semantics, public APIs, or capability.

## Acceptance Criteria

- every planned cleanup surface has one owning DEXG issue;
- the complete Rust cutover audit chain is retained;
- `docs/rust-cutover/release/v0_32_0_*` remains unchanged;
- the backend freeze guard passes;
- `docs/` and `examples/` contain no `.DS_Store`.

## Validation

```bash
find docs examples -name .DS_Store -print
scripts/ai/check_backend_freeze_baseline.sh
scripts/ai/verify_fast.sh
git diff --check
```
