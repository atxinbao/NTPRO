# RBTL-001 - Inventory Rust backtest runtime gaps

Milestone: R4 Rust Backtest and Live
Priority: P0
Default role: Coding/Test

## Goal

Inventory Rust backtest runtime gaps

## Scope

Inspect crates/backtest for Rust-only data loading, config, execution, and results gaps

## Likely files

- `crates/backtest/`
- `crates/live/`
- `crates/cli/`
- `examples/rust/`
- `tests/`
- `docs/rust-cutover/evidence/`

## Non-goals

- Do not refactor unrelated modules.
- Do not change trading semantics unless this task explicitly requires it.
- Do not delete Python, PyO3, or Cython product surfaces unless this task explicitly allows it and required parity evidence already exists.
- Do not change public APIs without a migration note.

## Dependencies

- `RCORE-018`

## Acceptance criteria

Rust backtest/live behavior is usable or blocker is scoped with evidence

## Required commands

```bash
scripts/ai/verify_full.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RBTL-001.md` with:

- task ID;
- summary;
- files changed;
- commands run;
- command results;
- tests added/updated;
- behavior impact;
- rollback plan.
