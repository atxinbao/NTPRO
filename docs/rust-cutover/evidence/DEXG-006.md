# DEXG-006 Concept Link Cleanup Evidence

Date: 2026-07-15
Executor: Codex
GitHub issue: #1085
Milestone: post-backend-docs-examples-governance
Status: LOCAL VALIDATION PASSED

## Summary

This evidence covers removal of the retired Python API URL family and Rust
source authority normalization for nine concept pages.

Plain Chinese summary: 本任务把 26 个失效 Python API 链接替换为 tracked Rust
source，给 9 个 concept 页增加 Rust-only authority，并保留 Python 内容仅作为明确
标注的历史 lineage。

## Affected Pages

- execution, instruments, live, logging, orders;
- portfolio, positions, strategies, synthetics.

## Validation

```text
retired Python API URL repository search = PASS (no matches)
concept Rust-only authority audit = PASS (9 pages)
Rust source link audit = PASS (30 unique links in affected concept surface)
changed Markdown local-link audit = PASS (12 files)
current concept links replaced = 26
historical URL literals normalized = 2
scripts/ai/check_backend_freeze_baseline.sh = PASS
backend freeze negative selftest = PASS (20 cases)
scripts/ai/verify_fast.sh = PASS
frozen v0.32.0 release file diff = PASS (no changes)
git diff --check = PASS
```

## Behavior Impact

None. Documentation links and authority semantics only.
