# RBTL-002 Evidence - Add Rust BacktestNode Smoke

Date: 2026-05-30
Executor: Codex

## 大白话说明

这次是在 Rust 的回测节点上补了一个很小但稳定的冒烟测试。它临时生成一份固定的 quote tick catalog，然后让 `BacktestNode` 用 4 条一批的方式跑完 12 条数据，确认节点能启动、读数据、跑完并留下可检查的 engine。没有改交易逻辑，也没有改 Python、PyO3 或 Cython。

## Task

- Task ID: RBTL-002
- Goal: Add Rust BacktestNode smoke
- Risk: medium

## Files Changed

- `crates/backtest/tests/backtest_node.rs`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RBTL-002.json`
- `docs/rust-cutover/evidence/RBTL-002.md`

## Summary

Added `test_backtest_node_smoke_streams_deterministic_catalog_quotes` to cover a deterministic Rust `BacktestNode` run over a temporary Parquet catalog. The smoke test asserts:

- one run result is returned;
- run config ID is preserved;
- the node processes exactly 12 iterations;
- no orders or positions are created;
- run metadata is present;
- the engine remains inspectable when `dispose_on_completion` is disabled.

## Commands Run

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc cargo test -p nautilus-backtest --features streaming --test backtest_node test_backtest_node_smoke_streams_deterministic_catalog_quotes -- --exact
```

Result: passed, 1 test passed.

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc cargo test -p nautilus-backtest --features streaming --test backtest_node
```

Result: passed, 41 tests passed.

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc RUST_TEST_THREADS=1 scripts/ai/verify_full.sh
```

Result: passed. The script completed with `== verify_full complete ==`.

```bash
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RBTL-002.json >/dev/null
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
gh auth status
perl -e 'alarm 20; exec @ARGV' gh api repos/atxinbao/NTPRO --jq .default_branch
```

Result: passed. GitHub API returned `main` as the default branch. A separate `git ls-remote --heads origin main` probe hung and was stopped; no repository state was changed by that probe.

## Tests Added or Updated

- Added one Rust integration smoke test in `crates/backtest/tests/backtest_node.rs`.

## Behavior Impact

No production behavior change. This is test coverage only.

## Public API Impact

None.

## Migration Note

Not required. No public API or user-facing configuration changed.

## Rollback Plan

Revert this PR. That removes the new smoke test and restores task metadata.
