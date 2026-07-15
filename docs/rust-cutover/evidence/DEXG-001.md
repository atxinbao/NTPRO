# DEXG-001 Docs And Examples Authority Evidence

Date: 2026-07-15
Executor: Codex
GitHub issue: #1080
Milestone: post-backend-docs-examples-governance
Status: LOCAL VALIDATION PASSED

## Summary

This evidence covers the post-baseline docs/examples authority map and cleanup
boundaries. It changes governance documentation only and does not remove
tracked public documentation or alter runtime behavior.

Plain Chinese summary: 本任务已建立文档和 examples 的清理归属表，保护 v0.32.0
冻结文件与完整 Rust cutover 审计链，并把后续删除、重写和门禁工作拆到独立 issue。

## Files

- `docs/rust-cutover/governance/README.md`
- `docs/rust-cutover/governance/docs_examples_authority_map.md`
- `docs/rust-cutover/tasks/DEXG-001.md`
- `docs/rust-cutover/evidence/DEXG-001.md`

## Validation

```text
find docs examples -name .DS_Store -print = PASS (no output)
frozen v0.32.0 release file diff = PASS (no changes)
scripts/ai/check_backend_freeze_baseline.sh = PASS
backend freeze negative selftest = PASS (20 cases)
scripts/ai/verify_fast.sh = PASS
git diff --check = PASS
```

## Behavior Impact

None. Documentation and governance metadata only.
