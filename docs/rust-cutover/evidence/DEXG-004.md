# DEXG-004 Legacy Python Guides Retirement Evidence

Date: 2026-07-15
Executor: Codex
GitHub issue: #1083
Milestone: post-backend-docs-examples-governance
Status: LOCAL VALIDATION PASSED

## Summary

This evidence covers five retired Python-first public pages, their orphaned
assets, and repaired navigation. Runtime behavior is unchanged.

Plain Chinese summary: 本任务从当前文档入口删除 5 个 Python-first 页面和 12 张
孤儿图片，并把入站链接改到有当前依据的 Rust example、guide、contract 或迁移说明。

## Removed Surface

- five Markdown pages (1,745 lines before deletion);
- three tutorial asset directories containing 12 PNG files.

## Validation

```text
retired page filesystem check = PASS (5 absent)
active retired-route reference search = PASS (no matches)
tutorial/how-to Python product-code search = PASS (no matches)
tutorial asset reference comparison = PASS (20 files, 20 references)
changed non-legacy Markdown link check = PASS (9 files)
known retired Python API URL dependency scan = 2 links, assigned to DEXG-006
scripts/ai/check_backend_freeze_baseline.sh = PASS
backend freeze negative selftest = PASS (20 cases)
scripts/ai/verify_fast.sh = PASS
frozen v0.32.0 release file diff = PASS (no changes)
git diff --check = PASS
```

The first broad changed-file link scan also reported two pre-existing retired
Python API URLs in `concepts/execution.md` and `concepts/live.md`. They are not
links to a DEXG-004 retired page and remain in the explicit DEXG-006 scope. The
scoped check fails on every other missing local target and passed without
exclusions beyond that exact legacy URL family.

## Behavior Impact

Unsupported documentation routes are retired. Rust runtime, public API, and
backend capability are unchanged.
