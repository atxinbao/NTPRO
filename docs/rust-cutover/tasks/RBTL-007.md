# RBTL-007 - Close Rust live config/client gaps

Milestone: R4 Rust Backtest and Live
Priority: P0
Default role: Coding/Test

## Goal

Close Rust live config/client gaps

## Scope

Implement or scope Rust-only live config/client registration gaps

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

- `RBTL-006`

## Acceptance criteria

Rust backtest/live behavior is usable or blocker is scoped with evidence

## Required commands

```bash
scripts/ai/verify_full.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RBTL-007.md` with:

- task ID;
- summary;
- files changed;
- commands run;
- command results;
- tests added/updated;
- behavior impact;
- rollback plan.
