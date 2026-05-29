# RCORE-018 Evidence - Rust Trading Strategy / Algorithm API Gaps

Date: 2026-05-30
Executor: Codex
Task ID: RCORE-018
Risk: high

## Summary

Closed the core Rust TWAP execution algorithm API gap by adding typed
`TwapExecParams` parsing and exported parameter constants for the existing TWAP
order parameter contract.

This keeps the current order-level `exec_algorithm_params` wire shape intact,
but gives Rust callers a stable typed surface instead of repeating stringly
`horizon_secs` / `interval_secs` parsing at the execution point.

Remaining product and release gaps are explicitly scoped below rather than
silently treated as complete.

## Files Changed

- `crates/trading/src/algorithm/twap.rs`
- `crates/trading/src/algorithm/mod.rs`
- `crates/trading/src/lib.rs`
- `docs/rust-cutover/evidence/RCORE-018.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RCORE-018.json`

## Tests Added Or Updated

- Added `algorithm::twap::tests::test_twap_exec_params_parse_valid_values`.
- Added `algorithm::twap::tests::test_twap_exec_params_missing_key_returns_none`.
- Added `algorithm::twap::tests::test_twap_exec_params_invalid_number_returns_error`.
- Added `algorithm::twap::tests::test_twap_errors_on_invalid_horizon_secs_string`.

## Commands Run

Initial environment check:

```bash
cargo test -p nautilus-trading --features examples algorithm::twap
```

Targeted verification with workspace MSRV toolchain:

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc cargo test -p nautilus-trading --features examples algorithm::twap
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc cargo test -p nautilus-trading --features examples
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc cargo fmt --check
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc cargo test -p nautilus-backtest --lib
```

Required full verification:

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc scripts/ai/verify_full.sh
```

## Command Results

- Default `cargo test -p nautilus-trading --features examples algorithm::twap`:
  failed before compiling because default `rustc 1.87.0` is below the workspace
  MSRV `1.95.0`. This was an environment/toolchain mismatch only.
- MSRV `cargo test -p nautilus-trading --features examples algorithm::twap`:
  passed; 24 tests passed, 0 failed, 295 filtered out.
- MSRV `cargo test -p nautilus-trading --features examples`: passed; 319
  library tests passed, 0 failed, and 4 doctests were ignored.
- MSRV `cargo fmt --check`: passed.
- MSRV `cargo test -p nautilus-backtest --lib`: passed; 68 tests passed,
  0 failed.
- MSRV `scripts/ai/verify_full.sh`: passed with `== verify_full complete ==`.
  The full run completed fast checks, clippy/check, Rust workspace tests,
  log-global tests, golden trace validation, and Rust docs.

## Behavior Impact

No intended trading behavior changed.

TWAP still reads the same `exec_algorithm_params` keys from the order, still
requires `horizon_secs` and `interval_secs`, still returns an error for invalid
numeric strings, still logs and skips scheduling for missing or invalid TWAP
durations, still floors `horizon_secs / interval_secs` to compute child-order
intervals, and still uses the same timer duration.

## Public API Impact

Additive Rust API only:

- `TwapExecParams`
- `TWAP_HORIZON_SECS_PARAM`
- `TWAP_INTERVAL_SECS_PARAM`

These are exported from `nautilus_trading::algorithm` and the crate root. No
existing public Rust API was removed or renamed.

## Python / PyO3 / Cython Impact

No Python, PyO3, or Cython files were changed. No new Python, PyO3, or Cython
dependency was introduced.

## Scoped Remaining Gaps

- Rust CLI strategy/algorithm selection remains a Rust product-surface task.
- Stable config-to-strategy and config-to-algorithm registry mapping remains a
  follow-up runtime/product task.
- General execution algorithm plug-in registration remains a separate high-risk
  follow-up.
- Strategy plus algorithm golden trace coverage remains an RTRACE follow-up.
- Python, PyO3, and Cython cutover/removal remains gated behind dedicated RREM
  tasks and release review.

## Migration Note Status

No migration note is required because this is additive Rust API surface and does
not break an existing user contract.

## Gate Status

RCORE-018 is high risk because it touches the trading runtime boundary where
strategy code selects execution algorithms and where TWAP child-order scheduling
is derived.

This PR must stop at `REVIEW_REQUIRED`. Auto-merge must not be enabled.

## Rollback Plan

Revert the typed TWAP parameter additions and exports in `crates/trading`, this
evidence file, and the RCORE-018 task state/lease updates. No persisted data,
adapter schema, Python, PyO3, Cython, or release-contract rollback is required.
