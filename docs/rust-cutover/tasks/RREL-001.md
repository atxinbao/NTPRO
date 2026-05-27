# RREL-001 - Write Rust-only migration guide

Milestone: R7 Release
Priority: P0
Default role: Release

## Goal

Write Rust-only migration guide

## Scope

Document supported Rust workflows and removed Python/PyO3/Cython surfaces

## Likely files

- `docs/rust-cutover/migration/`
- `docs/rust-cutover/release/`
- `completion/`
- `scripts/ai/`

## Non-goals

- Do not refactor unrelated modules.
- Do not change trading semantics unless this task explicitly requires it.
- Do not delete Python, PyO3, or Cython product surfaces unless this task explicitly allows it and required parity evidence already exists.
- Do not change public APIs without a migration note.

## Dependencies

- `RREM-010`

## Acceptance criteria

Release evidence exists and all blockers are documented

## Required commands

```bash
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RREL-001.md` with:

- task ID;
- summary;
- files changed;
- commands run;
- command results;
- tests added/updated;
- behavior impact;
- rollback plan.
