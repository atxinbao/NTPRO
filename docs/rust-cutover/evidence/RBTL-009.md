# RBTL-009 Evidence

Date: 2026-05-30
Executor: Codex
Task ID: RBTL-009

## Summary

Added a scoped Rust backtest/live semantic parity trace. The new test runs the existing Rust `BacktestEngine` single-quote replay, checks it still matches the source backtest golden trace, then compares its normalized result with the Rust live sandbox lifecycle trace through a new parity JSONL fixture.

During required full verification, `nautilus-core` exposed an existing test-only Python FFI lifetime bug. The `test_pystr_to_string` test now keeps the `PyString` alive while `pystr_to_string` reads its raw pointer. This only changes test lifetime handling.

## Files Changed

- `crates/backtest/tests/backtest_live_semantic_parity.rs`
- `tests/golden/backtest_live_semantic_parity_schema.jsonl`
- `crates/core/src/ffi/string.rs`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RBTL-009.json`
- `docs/rust-cutover/evidence/RBTL-009.md`

## Commands Run

```bash
cargo fmt --check
cargo fmt
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc cargo test -p nautilus-backtest --test backtest_live_semantic_parity rust_backtest_and_live_sandbox_match_scoped_semantic_parity_trace -- --exact --nocapture
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc scripts/ai/run_golden_traces.sh
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc scripts/ai/verify_full.sh
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc cargo test -p nautilus-core --features "ffi python" --lib ffi::string::tests::test_pystr_to_string -- --exact --nocapture
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc cargo test -p nautilus-core --features "ffi python" --lib
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc scripts/ai/verify_full.sh
```

## Command Results

- Initial `cargo fmt --check` reported formatting differences; `cargo fmt` fixed them.
- Targeted RBTL-009 parity test passed: 1 passed, 0 failed.
- Golden trace validation passed and included `tests/golden/backtest_live_semantic_parity_schema.jsonl`.
- First `scripts/ai/verify_full.sh` run failed in `nautilus-core --lib` with SIGSEGV from the existing `test_pystr_to_string` raw pointer lifetime issue.
- Targeted `ffi::string::tests::test_pystr_to_string` passed after the test lifetime fix.
- Full `nautilus-core --features "ffi python" --lib` passed: 997 passed, 0 failed.
- Second `scripts/ai/verify_full.sh` completed successfully.

## Tests Added Or Updated

- Added `rust_backtest_and_live_sandbox_match_scoped_semantic_parity_trace`.
- Added golden trace case `backtest_live.semantic_parity.scope.001`.
- Updated `ffi::string::tests::test_pystr_to_string` so the Python string object remains alive for the unsafe pointer read.

## Behavior Impact

- No production trading behavior changed.
- No orders are submitted or matched by the new parity test.
- No real exchange, network, or external I/O is required.
- No public API changed.
- No Python, PyO3, or Cython product surface was removed.

## Rollback Plan

Revert the new parity test, parity JSONL fixture, task metadata/evidence updates, and the `test_pystr_to_string` lifetime fix.
