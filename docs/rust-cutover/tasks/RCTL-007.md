# RCTL-007 - Write bootstrap evidence

Milestone: R0 Control Plane
Priority: P0
Default role: Architect/CI

## Goal

Write bootstrap evidence

## Scope

Write docs/rust-cutover/evidence/bootstrap_evidence.md

## Likely files

- `docs/rust-cutover/`
- `scripts/ai/`
- `.github/`
- `AGENTS.md`

## Non-goals

- Do not refactor unrelated modules.
- Do not change trading semantics unless this task explicitly requires it.
- Do not delete Python, PyO3, or Cython product surfaces unless this task explicitly allows it and required parity evidence already exists.
- Do not change public APIs without a migration note.

## Dependencies

- `RCTL-006`

## Acceptance criteria

Control-plane file exists, task evidence is written, and no runtime behavior changed

## Required commands

```bash
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RCTL-007.md` with:

- task ID;
- summary;
- files changed;
- commands run;
- command results;
- tests added/updated;
- behavior impact;
- rollback plan.
