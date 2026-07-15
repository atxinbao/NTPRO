# DEXG-003 Retire Legacy Python API Appendix

Date: 2026-07-15
Executor: Codex
GitHub issue: #1082
Milestone: post-backend-docs-examples-governance
Status: READY FOR PR

## Goal

Remove the active legacy upstream Python API appendix, its broken build entry,
and dependencies used only by that entry, while preserving migration history.

Plain Chinese summary: 本任务删除 39 个 legacy Python API appendix 文件，删除
无法工作的 `docs-python` target，并将 `make docs` 收敛到 Rust docs。迁移 tombstone
保留历史路由；公开文档中的旧 Python API 链接由 DEXG-006 独立修复。

## Dependency

DEXG-001 / #1080 is merged and closed.

## Scope

Included:

- delete `docs/api_reference/`;
- remove `docs-python` and the Sphinx-only dependency group;
- regenerate `uv.lock` and keep `no-build-package` synchronized;
- update the developer docs authority statement;
- add a migration tombstone.

Not included:

- rewriting concept, how-to, tutorial, or integration content;
- changing Rust runtime, public API, or trading semantics;
- editing frozen v0.32.0 release files.

## Acceptance Criteria

- no tracked file remains under `docs/api_reference/`;
- no Make target or workflow invokes `docs-python` or `sphinx-build`;
- `make docs` resolves to the supported Rust docs target;
- the helper dependency manifest and lockfile are consistent;
- the backend freeze guard and local smoke pass.

## Validation

```bash
git ls-files docs/api_reference
rg -n 'docs-python|sphinx-build|docs/api_reference' Makefile .github scripts pyproject.toml
uv lock --check
scripts/check-no-build-packages.sh
scripts/ai/check_backend_freeze_baseline.sh
scripts/ai/verify_fast.sh
git diff --check
```
