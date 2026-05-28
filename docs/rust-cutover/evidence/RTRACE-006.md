# RTRACE-006 Evidence - Bind live/sandbox lifecycle trace

Date: 2026-05-29
Executor: Codex
Task ID: RTRACE-006

## Summary

Added an executable `backtest_live` golden trace fixture for a Rust sandbox
`LiveNode` lifecycle and bound it to the standard golden trace command.

## Files Changed

- `tests/golden/live_sandbox_lifecycle_schema.jsonl`
- `crates/live/tests/golden_trace_live_sandbox.rs`
- `docs/rust-cutover/evidence/RTRACE-006.md`
- `scripts/ai/run_golden_traces.sh`
- `docs/rust-cutover/golden_trace/SCHEMA.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RTRACE-006.json`

## Commands Run

```bash
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo test -p nautilus-live --test golden_trace_live_sandbox
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/run_golden_traces.sh
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
python3 scripts/ai/validate_agentflow_roles.py
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RTRACE-006.json >/dev/null
git diff --check
```

## Command Results

- `cargo test -p nautilus-live --test golden_trace_live_sandbox`: passed; 1
  Rust sandbox lifecycle replay test passed.
- `scripts/ai/run_golden_traces.sh`: passed; validated 5 golden trace JSONL
  files, ran the Rust schema harness, ran the Rust backtest replay harness, and
  ran the Rust live/sandbox lifecycle harness.
- `scripts/ai/verify_fast.sh`: passed; fast mode ran toolchain detection and
  `cargo fmt --check`.
- `scripts/ai/validate_agentflow_roles.py`: passed.
- `.agentflow` JSON validation: passed.
- `git diff --check`: passed.

## Tests Added or Updated

- Added `backtest_live.sandbox_lifecycle_stop.001` in
  `tests/golden/live_sandbox_lifecycle_schema.jsonl`.
- Added `rust_sandbox_live_node_replays_lifecycle_golden_trace` under
  `nautilus-live`.
- The Rust lifecycle test builds a `LiveNode` with `Environment::Sandbox`,
  disables startup reconciliation, observes deterministic `Idle`, `Running`,
  and `Stopped` states, and compares the normalized lifecycle envelope to the
  trace `expected` events.
- Updated `run_golden_traces.sh` so the live/sandbox lifecycle harness runs by
  default. `RUN_RUST_LIVE_SANDBOX_TRACE_REPLAY=0` is reserved for documented
  local toolchain or scoped replay blockers.

## Behavior Impact

No runtime implementation, adapter behavior, external API, secret handling,
Python, PyO3, or Cython surface changed. The verification surface is stricter
because the standard golden trace command now fails if the Rust live/sandbox
lifecycle harness fails.

## Public API Impact

None.

## Migration Note Status

No migration note is required because no public API changed.

## Rollback Plan

Revert the lifecycle fixture, the Rust lifecycle test, and the
`run_golden_traces.sh` live/sandbox lifecycle invocation. The schema and
backtest replay harnesses remain independently usable.
