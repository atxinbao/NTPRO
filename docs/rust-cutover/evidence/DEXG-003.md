# DEXG-003 Legacy Python API Retirement Evidence

Date: 2026-07-15
Executor: Codex
GitHub issue: #1082
Milestone: post-backend-docs-examples-governance
Status: LOCAL VALIDATION PASSED

## Summary

This evidence covers removal of the legacy Python API appendix and its build
tooling. It does not modify runtime code or the frozen release package.

Plain Chinese summary: 本任务移除失效的 Python API 文档目录、构建入口和专用依赖，
保留迁移说明，并继续把 v0.32.0 作为不可改写的后端基线。

## Files

- `docs/api_reference/` (39 tracked files removed)
- `Makefile`
- `pyproject.toml`
- `uv.lock`
- `docs/developer_guide/docs.md`
- `docs/rust-cutover/migration/README.md`
- `docs/rust-cutover/migration/legacy_python_api_appendix_retired.md`
- `docs/rust-cutover/tasks/DEXG-003.md`
- `docs/rust-cutover/evidence/DEXG-003.md`

## Validation

```text
docs/api_reference filesystem and Git index check = PASS (absent)
active docs-python/sphinx-build/docs/api_reference build reference search = PASS
make -n docs = PASS (cargo +nightly doc --all-features --no-deps --workspace)
uv lock --check = PASS (55 packages)
scripts/check-no-build-packages.sh = PASS (55 packages in sync)
Sphinx-only lock cleanup = PASS (29 packages removed)
scripts/ai/check_backend_freeze_baseline.sh = PASS
backend freeze negative selftest = PASS (20 cases)
scripts/ai/verify_fast.sh = PASS
frozen v0.32.0 release file diff = PASS (no changes)
git diff --check = PASS
```

The full Rust docs build is reserved for DEXG-007 so this cleanup task does not
recreate the large local `target/` tree that was removed during artifact
hygiene. This task verifies the Make route deterministically; the final gate
owns the actual supported docs build.

## Behavior Impact

The unsupported Python documentation build entry is removed. Rust runtime and
product behavior are unchanged.
