# Legacy Python API Appendix Retired

Date: 2026-07-15
Executor: Codex
Task ID: DEXG-003
Baseline: `ntpro-rust-only-v0.32.0`

## Decision

The active `docs/api_reference/` tree and `docs-python` Make target were
retired after the v0.32.0 backend freeze. The tree contained upstream Python
`nautilus_trader.*` automodule declarations but no buildable Sphinx `conf.py`,
and it was not an NTPRO Rust-only product API.

Plain Chinese summary: 原 `docs/api_reference/` 是上游 Python API 附录，不是
NTPRO Rust 产品接口，而且仓库中没有可构建它的 Sphinx `conf.py`。本次在 v0.32.0
基线后治理层删除该目录、失效的 `docs-python` 入口和专用依赖，不改变冻结后端代码或
发布事实。

## Removed Surface

- 39 tracked files under `docs/api_reference/`;
- the `docs-python` Make target and the Python/Sphinx branch of `make docs`;
- the root `docs` dependency group used only by that appendix.

The lockfile was regenerated after the dependency-group removal. Packages still
required by another helper group remain locked.

## Current Authority

Use these maintained sources instead:

- Rust crate documentation through `make docs` or `make docs-rust`;
- `nautilus-cli --help` and command-specific help;
- `docs/rust-cutover/product/` contracts;
- `examples/rust/` for bounded runnable examples;
- `docs/rust-cutover/migration/` for historical route changes.

## Follow-up Boundary

Public concept, how-to, and integration pages that still link to
`/docs/python-api-latest/` are not silently redirected. DEXG-004 through
DEXG-006 must rewrite or retire those pages against a current Rust authority.
DEXG-007 owns the final docs build and link gate after those dependencies close.

This migration note is historical routing only. It does not claim Python API
compatibility and does not authorize production submit, mutation, adapter send,
live exchange access, retry, remediation, or trading controls.
