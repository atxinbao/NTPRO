# RTRACE-002 - Capture market data trace fixtures

Milestone: R2 Golden Trace
Priority: P1
Default role: Test

## Goal

Capture market data trace fixtures

## Scope

Add market data fixtures for Rust replay or schema validation

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

- `RTRACE-001`

## Acceptance criteria

Golden trace evidence exists and is executable or explicitly scoped

## Required commands

```bash
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RTRACE-002.md` with:

- task ID;
- summary;
- files changed;
- commands run;
- command results;
- tests added/updated;
- behavior impact;
- rollback plan.
