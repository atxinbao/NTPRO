# RTRACE-005 Evidence - Bind backtest replay trace

Date: 2026-05-29
Executor: Codex
Task ID: RTRACE-005

## Summary

Added an executable `backtest_live` golden trace fixture and bound it to a Rust
`BacktestEngine` replay test. The standard golden trace command now validates
the JSONL schema, runs the Rust schema harness, and runs the Rust backtest
replay harness.

## Files Changed

- `tests/golden/backtest_replay_schema.jsonl`
- `crates/backtest/tests/golden_trace_backtest.rs`
- `scripts/ai/run_golden_traces.sh`
- `docs/rust-cutover/golden_trace/SCHEMA.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RTRACE-005.json`

## Commands Run

```bash
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo test -p nautilus-backtest --test golden_trace_backtest
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/run_golden_traces.sh
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
python3 scripts/ai/validate_agentflow_roles.py
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RTRACE-005.json >/dev/null
git diff --check
```

## Command Results

- `cargo test -p nautilus-backtest --test golden_trace_backtest`: passed; 1
  Rust backtest replay test passed.
- `scripts/ai/run_golden_traces.sh`: passed; validated 4 golden trace JSONL
  files, ran the Rust schema harness, and ran the Rust backtest replay harness.
- `scripts/ai/verify_fast.sh`: passed; fast mode ran toolchain detection and
  `cargo fmt --check`.
- `scripts/ai/validate_agentflow_roles.py`: passed.
- `.agentflow` JSON validation: passed.
- `git diff --check`: passed.

## Tests Added or Updated

- Added `backtest_live.single_quote_replay.001` in
  `tests/golden/backtest_replay_schema.jsonl`.
- Added `rust_backtest_engine_replays_single_quote_golden_trace` under
  `nautilus-backtest`.
- The Rust replay test reads the golden trace case, constructs a
  `BacktestEngine` with a simulated `BINANCE` venue, replays one deterministic
  `QuoteTick`, normalizes deterministic `BacktestResult` fields, and compares
  the result envelope to the trace `expected` event.
- Updated `run_golden_traces.sh` so the backtest replay harness runs by
  default. `RUN_RUST_BACKTEST_TRACE_REPLAY=0` is reserved for documented local
  toolchain or scoped replay blockers.

## Behavior Impact

No trading runtime implementation, adapter behavior, Python, PyO3, or Cython
surface changed. The verification surface is stricter because the standard
golden trace command now fails if the Rust backtest replay harness fails.

## Public API Impact

None.

## Migration Note Status

No migration note is required because no public API changed.

## Rollback Plan

Revert the backtest fixture, the Rust replay test, and the
`run_golden_traces.sh` backtest replay invocation. The schema harness and
existing Python JSONL validator remain independently usable.
