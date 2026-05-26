# RPROD-008 - Implement or close data/catalog CLI gaps

Milestone: R1 Rust Product Surface
Priority: P1
Default role: Coding/CI

## Goal

Implement or close data/catalog CLI gaps

## Scope

Implement missing data/catalog CLI command or record scoped blocker with evidence

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

- `RPROD-007`

## Acceptance criteria

Rust product surface is usable or an owner-visible blocker is recorded with evidence

## Required commands

```bash
scripts/ai/verify_full.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RPROD-008.md` with:

- task ID;
- summary;
- files changed;
- commands run;
- command results;
- tests added/updated;
- behavior impact;
- rollback plan.
