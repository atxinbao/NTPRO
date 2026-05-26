# RADP-020 - Add Rust adapter fixtures for Betfair Architect AX

Milestone: R5 Rust Adapter Parity
Priority: P0
Default role: Test/Coding

## Goal

Add Rust adapter fixtures for Betfair Architect AX

## Scope

Add parser/lifecycle fixtures for Betfair Architect AX

## Likely files

- `crates/adapters/`
- `tests/`
- `docs/rust-cutover/evidence/`

## Non-goals

- Do not refactor unrelated modules.
- Do not change trading semantics unless this task explicitly requires it.
- Do not delete Python, PyO3, or Cython product surfaces unless this task explicitly allows it and required parity evidence already exists.
- Do not change public APIs without a migration note.

## Dependencies

- `RADP-019`

## Acceptance criteria

Betfair Architect AX Rust fixtures or scoped blockers exist

## Required commands

```bash
scripts/ai/verify_full.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RADP-020.md` with:

- task ID;
- summary;
- files changed;
- commands run;
- command results;
- tests added/updated;
- behavior impact;
- rollback plan.
