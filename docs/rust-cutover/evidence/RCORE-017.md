# RCORE-017 Evidence - Rust Trading Strategy / Algorithm API Tests

Date: 2026-05-30
Executor: Codex
Task ID: RCORE-017
Risk: high

## Summary

Added direct Rust tests for the backtest engine strategy and execution algorithm
registration API.

The new tests instantiate minimal Rust-native strategy and execution algorithm
types, register them through `BacktestEngine::add_strategy` and
`BacktestEngine::add_exec_algorithm`, and assert that the underlying trader
records the expected strategy and execution algorithm identifiers.

Existing wider API coverage was also verified across `nautilus-trading`,
`nautilus-system`, and `nautilus-plugin`.

## Files Changed

- `crates/backtest/src/engine.rs`
- `docs/rust-cutover/evidence/RCORE-017.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RCORE-017.json`

## Tests Added Or Updated

- Added `engine::tests::test_add_strategy_registers_strategy_with_trader`.
- Added `engine::tests::test_add_exec_algorithm_registers_algorithm_with_trader`.

## Commands Run

Targeted discovery and verification:

```bash
cargo test -p nautilus-trading --features examples
cargo test -p nautilus-system --lib
cargo test -p nautilus-plugin --tests --features host
cargo fmt --check
cargo test -p nautilus-backtest --lib
```

Required full verification:

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc scripts/ai/verify_full.sh
```

## Command Results

- `cargo test -p nautilus-trading --features examples`: passed; 315 library
  tests passed, 0 failed, and 4 doctests were ignored.
- `cargo test -p nautilus-system --lib`: passed; 44 tests passed, 0 failed.
- `cargo test -p nautilus-plugin --tests --features host`: passed; crate unit
  tests and integration tests passed, including strategy execution dispatch and
  surface alignment coverage.
- `cargo fmt --check`: passed.
- `cargo test -p nautilus-backtest --lib`: passed; 68 tests passed, including
  the two new RCORE-017 registration tests.
- `scripts/ai/verify_full.sh`: passed with `== verify_full complete ==`. The
  full run completed fast checks, clippy, Rust workspace tests, log-global
  tests, golden trace validation, and Rust docs.

## Behavior Impact

No runtime behavior changed. This task only adds tests and task evidence.

No strategy logic, execution algorithm logic, execution routing, risk behavior,
portfolio behavior, adapter behavior, persistence format, public API shape,
Python API, PyO3 binding, or Cython surface changed.

## Public API Impact

No public API change.

## Python / PyO3 / Cython Impact

No new Python, PyO3, or Cython dependency was introduced. Existing workspace
surfaces remain unchanged and gated for later cutover/removal tasks.

## Migration Note Status

No migration note is required because this PR only adds test coverage.

## Gate Status

RCORE-017 is high risk because strategy and execution algorithm registration sit
on the trading-runtime boundary. Even though this PR is test-only, the tested
surface controls strategy activation and execution algorithm availability inside
backtest runtime setup.

This PR must stop at `REVIEW_REQUIRED`. Auto-merge must not be enabled.

## Rollback Plan

Revert the two test additions in `crates/backtest/src/engine.rs`, this evidence
file, and the RCORE-017 task state/lease updates. No runtime, persisted data,
adapter, schema, Python, PyO3, Cython, or public API rollback is required.
