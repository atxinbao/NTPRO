# RCTL-008 - Validate overlay and task graph

Milestone: R0 Control Plane
Priority: P0
Default role: Architect/CI

## Goal

Validate overlay and task graph

## Scope

Run package/task validation and record any overlay conflicts

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

- `RCTL-007`

## Acceptance criteria

Control-plane file exists, task evidence is written, and no runtime behavior changed

## Required commands

```bash
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RCTL-008.md` with:

- task ID;
- summary;
- files changed;
- commands run;
- command results;
- tests added/updated;
- behavior impact;
- rollback plan.
