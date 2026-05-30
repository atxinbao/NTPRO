# RBTL-004 Evidence - Close Rust Backtest Result/Report Gaps

Date: 2026-05-30
Executor: Codex

## 大白话说明

这次修了两个点。第一，把 Rust 回测结果对象的报告字段用测试盯住，确认能拿到 PnL、PnL% 和 Long Ratio 这些报告数据。第二，修掉 `verify_full` 卡住的 live timer 问题：以前 live timer 会为了补偿启动开销提前 1ms 触发，结果事件可能比它自己的计划时间还早；现在不再提前触发，保证事件不会早于计划时间。没有碰 Python、PyO3、Cython。

## Task

- Task ID: RBTL-004
- Goal: Close Rust backtest result/report gaps
- Risk: medium by task metadata; runtime-impacting timer fix called out for review
- Branch: `ai/RBTL-004-close-rust-backtest-result-report-gaps`

## Files Changed

- `crates/backtest/tests/backtest_node.rs`
- `crates/common/src/live/timer.rs`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RBTL-004.json`
- `docs/rust-cutover/evidence/RBTL-004.md`

## Summary

Added regression coverage for Rust backtest report fields on `BacktestResult`, including total event/order consistency, USDT PnL report keys, and the general Long Ratio report value.

Fixed the known `verify_full` blocker in `live::clock::tests::test_live_timer_short_delay_not_early`. `LiveTimer` no longer subtracts a fixed startup-overhead estimate from the scheduled delay, so timer events are not deliberately scheduled before their target timestamp.

## Commands Run

```bash
cargo fmt --check -p nautilus-common -p nautilus-backtest
```

Result: passed.

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc cargo test -p nautilus-common --features live --lib live::clock::tests::test_live_timer_short_delay_not_early -- --exact --nocapture
```

Result: passed, 1 test passed.

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc RUST_TEST_THREADS=1 cargo test -p nautilus-common --features live --lib -- --skip logging::logger::tests::serial_tests --skip logging::macros::tests::test_colored_logging_macros --skip logging::macros::tests::test_default_macro_captures_module_path --skip serial_tests
```

Result: passed, 950 tests passed, 2 ignored, 13 filtered out.

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc cargo test -p nautilus-backtest --features streaming --test backtest_node
```

Result: passed, 43 tests passed.

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc RUST_TEST_THREADS=1 scripts/ai/verify_full.sh
```

Result: passed. The script completed with `== verify_full complete ==`.

## Tests Added or Updated

- Updated `test_generates_orders` to assert Rust `BacktestResult` report data is present and populated for USDT PnL and Long Ratio.
- Re-ran the exact live timer blocker test to prove timer events no longer fire before the scheduled event timestamp.

## Behavior Impact

Backtest behavior is unchanged except for stronger regression coverage around returned report data.

Live timer scheduling no longer subtracts a fixed 1ms startup overhead from the requested delay. This removes an early-fire path and makes the timer respect the scheduled event timestamp more strictly.

## Public API Impact

None. No public function signatures, Python APIs, PyO3 bindings, or CLI contracts changed.

## Migration Note

Not required. There is no user-facing API or configuration migration.

## Rollback Plan

Revert this PR. That restores the previous live timer delay compensation and removes the new backtest report assertions.
