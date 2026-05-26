# RCORE-005 - Add Rust tests for serialization/data/persistence

Milestone: R3 Rust Core Runtime
Priority: P0
Default role: Test/Coding

## Goal

Add Rust tests for serialization/data/persistence

## Scope

Add or identify Rust tests covering serialization/data/persistence

## Likely files

- `crates/`
- `tests/`
- `docs/rust-cutover/evidence/`

## Non-goals

- Do not refactor unrelated modules.
- Do not change trading semantics unless this task explicitly requires it.
- Do not delete Python, PyO3, or Cython product surfaces unless this task explicitly allows it and required parity evidence already exists.
- Do not change public APIs without a migration note.

## Dependencies

- `RCORE-004`

## Acceptance criteria

Rust tests for serialization/data/persistence exist or blocker is scoped

## Required commands

```bash
scripts/ai/verify_full.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RCORE-005.md` with:

- task ID;
- summary;
- files changed;
- commands run;
- command results;
- tests added/updated;
- behavior impact;
- rollback plan.
