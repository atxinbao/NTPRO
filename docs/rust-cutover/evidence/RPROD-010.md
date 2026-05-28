# RPROD-010 Evidence

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-010

## Summary

Added the `examples/rust/` layout for Rust-first product workflows. The layout
documents the currently supported command contracts for backtest, sandbox, live,
data/catalog, and shared config validation without adding non-runnable Rust
source or Python fallback behavior.

Runtime execution remains intentionally blocked by the product-surface CLI
until later Rust config-parser and runtime wiring tasks connect these commands
to Rust models.

## Files Changed

- `examples/rust/README.md`
- `examples/rust/backtest/README.md`
- `examples/rust/sandbox/README.md`
- `examples/rust/live/README.md`
- `examples/rust/data/README.md`
- `examples/rust/config/README.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RPROD-010.json`
- `docs/rust-cutover/evidence/RPROD-010.md`

## Commands Run

```bash
python3 scripts/ai/lease.py claim RPROD-010 --force --branch ai/RPROD-010-add-rust-examples-layout --agent-id Codex --path docs/rust-cutover/tasks/RPROD-010.md --path examples/rust/README.md --path examples/rust/backtest/README.md --path examples/rust/sandbox/README.md --path examples/rust/live/README.md --path examples/rust/data/README.md --path examples/rust/config/README.md --path docs/rust-cutover/evidence/RPROD-010.md --path .agentflow/state/task_status.json --path .agentflow/leases/RPROD-010.json
mkdir -p examples/rust/backtest examples/rust/sandbox examples/rust/live examples/rust/data examples/rust/config
find examples/rust -maxdepth 2 -type f -print
git diff --check
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
```

## Command Results

- Lease claim succeeded for branch
  `ai/RPROD-010-add-rust-examples-layout`.
- Directory creation succeeded.
- `find examples/rust -maxdepth 2 -type f -print` listed the six README files
  under the new Rust examples layout.
- `git diff --check` passed.
- `scripts/ai/verify_fast.sh` passed:
  - rust toolchain check passed;
  - `cargo fmt --check` passed;
  - fast-mode cargo check and clippy stayed skipped by policy unless their
    opt-in environment variables are set.

## Tests Added Or Updated

No unit or integration tests were added. This task is a docs/examples layout
task and does not add runnable runtime code.

## Behavior Impact

- Adds a Rust-first examples directory owners can use as the canonical location
  for future Rust product examples.
- Does not change CLI behavior, runtime behavior, adapter behavior, persistence,
  public Rust APIs, Python APIs, PyO3, or Cython paths.
- Keeps current execution blockers explicit instead of adding placeholder
  runnable examples.

## Public API Impact

No public API change.

## Migration Note Status

No migration note required because this task adds documentation-only examples
layout and does not remove or change existing examples.

## Rollback Plan

Revert the commit to remove `examples/rust/` and this evidence file. No runtime
or generated data rollback is required.
