# V02 Release Logging Guard Fallback Evidence

Date: 2026-06-06
Executor: Codex

## Task

Local release blocker cleanup for v0.2 readiness.

## Goal

Fix a release-gate failure where `nautilus-backtest` unit tests could fail when
the process-global Rust `log` facade had already been claimed by another logger.

## Why This Was Needed

The v0.2 release verification run on `main@703a55851b` failed during:

```bash
scripts/ai/verify_release.sh
```

The failing command was the workspace Rust test phase. The direct failing crate
was:

```bash
cargo test -p nautilus-backtest --lib
```

Three backtest engine tests failed while constructing `BacktestEngine`:

- `engine::tests::test_add_strategy_registers_strategy_with_trader`
- `engine::tests::test_run_impl_event_store_replay_skips_trader_start`
- `engine::tests::test_add_exec_algorithm_registers_algorithm_with_trader`

The panic was not a trading behavior failure. The helper used
`BacktestEngine::new(...).unwrap()`, and kernel initialization returned:

```text
A non-Nautilus logger is already registered; cannot initialize Nautilus logging
```

These tests already configure `bypass_logging=true`, so they should not fail
because another test or test harness registered the process-global logger first.

## Files Changed

- `crates/common/src/logging/logger.rs`
- `crates/system/src/kernel.rs`
- `docs/rust-cutover/evidence/V02-RELEASE-LOGGING-GUARD-FALLBACK.md`

## Change Summary

- Added `LogGuard::noop()` for contexts where Nautilus logging is explicitly bypassed.
- Made `LogGuard` drop a no-op guard without touching the global guard counter or
  logger sender.
- Updated kernel logging initialization to use the no-op guard only when all of
  these are true:
  - logger registration failed with `SetLoggerError`;
  - no active Nautilus `LogGuard` can be created;
  - `LoggerConfig::bypass_logging` is `true`.
- Kept non-bypass logging initialization errors as hard errors.
- Added a unit test proving no-op guard drop does not mutate the active guard count.

## Commands Run

- `cargo fmt`
  - Result: passed.
- `cargo test -p nautilus-common --lib noop_log_guard_drop_does_not_touch_global_guard_count -- --nocapture`
  - Result: passed, 1 test.
- `cargo test -p nautilus-backtest --lib -- --nocapture`
  - Result: passed, 82 tests.
  - This directly reran the release-gate failure point.
- `cargo check -p nautilus-system`
  - Result: passed.
- `scripts/ai/verify_fast.sh`
  - Result: passed; this is fast smoke only.
- `git diff --check`
  - Result: passed.

## Behavior Impact

Only logger initialization fallback behavior changes for explicitly bypassed
logging contexts. Trading semantics, matching, risk, portfolio, adapters,
serialization, persistence, and execution behavior are unchanged.

## Public API Impact

Adds `LogGuard::noop()` as a Rust API helper for internal/system integration
code. Existing public APIs are not removed or changed.

## Migration Note

No user migration is required.

## Rollback Plan

Revert this PR to restore the previous behavior where bypassed logging still
failed when a non-NTPRO logger had already claimed the global `log` facade.
