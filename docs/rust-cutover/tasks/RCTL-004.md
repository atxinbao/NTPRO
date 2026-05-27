# RCTL-004 - Inventory current Rust product surface

Milestone: R0 Control Plane
Priority: P0
Default role: Architect/CI

## Goal

Inventory current Rust product surface

## Scope

Record existing Rust crates, CLI commands, examples, docs, and runnable tests

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

- `RCTL-003`

## Acceptance criteria

Control-plane file exists, task evidence is written, and no runtime behavior changed

## Required commands

```bash
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RCTL-004.md` with:

- task ID;
- summary;
- files changed;
- commands run;
- command results;
- tests added/updated;
- behavior impact;
- rollback plan.
