# RPROD-002 - Add Rust CLI help and version smoke

Milestone: R1 Rust Product Surface
Priority: P0
Default role: Coding/CI

## Goal

Add Rust CLI help and version smoke

## Scope

Add local smoke command for nautilus-cli help/version output

## Likely files

- `crates/cli/`
- `crates/backtest/`
- `crates/live/`
- `examples/rust/`
- `docs/rust-cutover/`
- `scripts/ai/`

## Non-goals

- Do not refactor unrelated modules.
- Do not change trading semantics unless this task explicitly requires it.
- Do not delete Python, PyO3, or Cython product surfaces unless this task explicitly allows it and required parity evidence already exists.
- Do not change public APIs without a migration note.

## Dependencies

- `RPROD-001`

## Acceptance criteria

Rust product surface is usable or an owner-visible blocker is recorded with evidence

## Required commands

```bash
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RPROD-002.md` with:

- task ID;
- summary;
- files changed;
- commands run;
- command results;
- tests added/updated;
- behavior impact;
- rollback plan.
