# RREL-007 - Prepare human owner signoff packet

Milestone: R7 Release
Priority: P0
Default role: Release

## Goal

Prepare human owner signoff packet

## Scope

Populate signoff template with evidence and residual risk

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

- `RREL-006`

## Acceptance criteria

Release evidence exists and all blockers are documented

## Required commands

```bash
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RREL-007.md` with:

- task ID;
- summary;
- files changed;
- commands run;
- command results;
- tests added/updated;
- behavior impact;
- rollback plan.
