# DEXG-002 Rust Examples Integrity Evidence

Date: 2026-07-15
Executor: Codex
GitHub issue: #1081
Milestone: post-backend-docs-examples-governance
Status: LOCAL VALIDATION PASSED

## Summary

This evidence covers Rust example path and status integrity. It does not change
runtime code, execute a workflow, or expand the frozen backend capability.

Plain Chinese summary: 本任务修复 Rust examples 的失效文档路径和过期状态说明，
保留全部 canonical examples，并增加只读完整性检查。

## Files

- `examples/rust/README.md`
- `examples/rust/config/README.md`
- `scripts/ai/check_rust_examples.sh`
- `docs/rust-cutover/tasks/DEXG-002.md`
- `docs/rust-cutover/evidence/DEXG-002.md`

## Validation

```text
scripts/ai/check_rust_examples.sh = PASS
  required_paths=14
  toml_files=7
  readme_paths=7
bash -n scripts/ai/check_rust_examples.sh = PASS
known stale example path search = PASS (no matches)
scripts/ai/check_backend_freeze_baseline.sh = PASS
backend freeze negative selftest = PASS (20 cases)
scripts/ai/verify_fast.sh = PASS
frozen v0.32.0 release file diff = PASS (no changes)
git diff --check = PASS
```

## Behavior Impact

None. Documentation and a read-only validation script only.
