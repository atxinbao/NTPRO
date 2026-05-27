# RCORE-009 - Close Rust common cache/message bus/component lifecycle gaps

Milestone: R3 Rust Core Runtime
Priority: P0
Default role: Coding

## Goal

Close Rust common cache/message bus/component lifecycle gaps

## Scope

Implement or scope remaining Rust-only gaps for common cache/message bus/component lifecycle

## Likely files

- `crates/`
- `docs/rust-cutover/evidence/`

## Non-goals

- Do not refactor unrelated modules.
- Do not change trading semantics unless this task explicitly requires it.
- Do not delete Python, PyO3, or Cython product surfaces unless this task explicitly allows it and required parity evidence already exists.
- Do not change public APIs without a migration note.

## Dependencies

- `RCORE-008`

## Acceptance criteria

common cache/message bus/component lifecycle parity is implemented or explicitly scoped with evidence

## Required commands

```bash
scripts/ai/verify_full.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RCORE-009.md` with:

- task ID;
- summary;
- files changed;
- commands run;
- command results;
- tests added/updated;
- behavior impact;
- rollback plan.
