# RPROD-011 Evidence

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-011

## Summary

Verified that NTPRO has a runnable Rust backtest Cargo smoke. The smoke uses
the existing `nautilus-backtest` Cargo example `engine-ema-cross`, which runs a
synthetic AUD/USD EMA crossover backtest through Rust `BacktestEngine` and the
Rust `EmaCross` strategy.

The Rust CLI `backtest validate` and `backtest run` command surfaces remain
blocked until later config-parser and runtime wiring tasks. This task records
the current runnable Cargo path instead of adding a duplicate placeholder
example.

## Files Changed

- `examples/rust/backtest/README.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RPROD-011.json`
- `docs/rust-cutover/evidence/RPROD-011.md`

## Commands Run

```bash
python3 scripts/ai/lease.py claim RPROD-011 --force --branch ai/RPROD-011-add-rust-backtest-example-smoke --agent-id Codex --path docs/rust-cutover/tasks/RPROD-011.md --path examples/rust/backtest/README.md --path docs/rust-cutover/evidence/RPROD-011.md --path .agentflow/state/task_status.json --path .agentflow/leases/RPROD-011.json
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -p nautilus-backtest --features examples --example engine-ema-cross
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_full.sh
```

## Command Results

- Lease claim succeeded for branch
  `ai/RPROD-011-add-rust-backtest-example-smoke`.
- Cargo built and ran the Rust backtest smoke successfully.
- Cargo finished the dev build and executed
  `target/debug/examples/engine-ema-cross`.
- Backtest runtime evidence from the command output:
  - `Run ID: 9acba90e-e256-4758-822f-a4c3c8e6d354`;
  - `Data elements: 745`;
  - `Iterations: 745`;
  - `Total events: 36`;
  - `Total orders: 12`;
  - `Total positions: 12`;
  - process exit code: `0`.
- Required full verification passed with `scripts/ai/verify_full.sh`.
- Full verification included fast checks, `cargo fmt --check`, targeted Rust
  checks, `cargo clippy -p nautilus-cli`, Rust test execution, golden trace
  schema validation, and Rust documentation generation.
- No verification failures were observed.

## Tests Added Or Updated

No Rust tests were added. The verification target for this task is a Cargo
example smoke run.

## Behavior Impact

- Documents the current runnable Rust backtest smoke under
  `examples/rust/backtest/`.
- Does not change runtime behavior, trading semantics, adapter behavior,
  persistence, public Rust APIs, Python APIs, PyO3, or Cython paths.
- Keeps CLI execution blockers unchanged until the later CLI runtime wiring
  tasks can connect config files to Rust backtest models.

## Public API Impact

No public API change.

## Migration Note Status

No migration note required because this task records and verifies an existing
Rust Cargo example path.

## Rollback Plan

Revert the commit to remove the backtest README smoke reference and this
evidence file. No runtime or generated data rollback is required.
