# RTRACE-001 - Define Rust golden trace schema

Milestone: R2 Golden Trace
Priority: P0
Default role: Test

## Goal

Define Rust golden trace schema

## Scope

Document the Rust trace schema and required event categories

## Likely files

- `tests/golden/`
- `scripts/ai/`
- `docs/rust-cutover/golden_trace/`
- `crates/`

## Non-goals

- Do not refactor unrelated modules.
- Do not change trading semantics unless this task explicitly requires it.
- Do not delete Python, PyO3, or Cython product surfaces unless this task explicitly allows it and required parity evidence already exists.
- Do not change public APIs without a migration note.

## Dependencies

- `RPROD-014`

## Acceptance criteria

Golden trace evidence exists and is executable or explicitly scoped

## Required commands

```bash
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RTRACE-001.md` with:

- task ID;
- summary;
- files changed;
- commands run;
- command results;
- tests added/updated;
- behavior impact;
- rollback plan.
