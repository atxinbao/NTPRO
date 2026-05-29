# RBTL-003 Evidence - Close Rust Backtest Data/Catalog Gaps

Date: 2026-05-30
Executor: Codex

## 大白话说明

这次不是大改 catalog，而是把 Rust 回测最容易踩坑的两个 catalog 边界补成测试：显式 `file://` 本地 catalog 能读；指定 instrument 但 catalog 里没有时会明确报错，不会悄悄跑空回测。没有改交易逻辑，也没有碰 Python、PyO3、Cython。

## Task

- Task ID: RBTL-003
- Goal: Close Rust backtest data/catalog gaps
- Risk: medium

## Files Changed

- `crates/backtest/tests/backtest_node.rs`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RBTL-003.json`
- `docs/rust-cutover/evidence/RBTL-003.md`

## Summary

Added Rust `BacktestNode` catalog boundary coverage for:

- explicit local catalog protocol handling through `catalog_fs_protocol = "file"`;
- requested instrument IDs that are absent from the catalog.

The missing-instrument case now has regression coverage proving the build fails with a clear error that names the missing requested instrument. The explicit `file` protocol case proves the local catalog path still loads deterministically.

This PR also carries forward the already-merged RBTL-002 bookkeeping from `PR_OPEN` to `DONE`, because `main` still had stale local task state after PR #56 merged.

## Commands Run

```bash
cargo fmt --check -p nautilus-backtest
```

Result: passed.

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc cargo test -p nautilus-backtest --features streaming --test backtest_node test_build_rejects_missing_requested_instrument -- --exact
```

Result: passed, 1 test passed.

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc cargo test -p nautilus-backtest --features streaming --test backtest_node test_load_data_config_accepts_explicit_file_protocol -- --exact
```

Result: passed, 1 test passed.

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc cargo test -p nautilus-backtest --features streaming --test backtest_node
```

Result: passed, 43 tests passed.

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc RUST_TEST_THREADS=1 scripts/ai/verify_full.sh
```

Result: passed. The script completed with `== verify_full complete ==`.

```bash
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RBTL-003.json >/dev/null
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
gh auth status
git ls-remote origin refs/heads/main
```

Result: passed. Role validation reported `agentflow role protocol validation passed`; GitHub auth and the `origin/main` network probe succeeded.

## Tests Added or Updated

- Added `test_build_rejects_missing_requested_instrument`.
- Added `test_load_data_config_accepts_explicit_file_protocol`.

## Behavior Impact

No production behavior change. This is test coverage and task bookkeeping only.

## Public API Impact

None.

## Migration Note

Not required. No public API or user-facing configuration changed.

## Rollback Plan

Revert this PR. That removes the new catalog boundary tests and restores task metadata.
