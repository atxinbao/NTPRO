# RCORE-004 - Inventory Rust serialization/data/persistence gaps

Milestone: R3 Rust Core Runtime
Priority: P0
Default role: Inventory/Coding

## Goal

Inventory Rust serialization/data/persistence gaps

## Scope

Inspect crates/serialization/, crates/data/, and crates/persistence/ for Rust-only parity gaps

## Likely files

- `crates/`
- `docs/rust-cutover/inventory/`

## Non-goals

- Do not refactor unrelated modules.
- Do not change trading semantics unless this task explicitly requires it.
- Do not delete Python, PyO3, or Cython product surfaces unless this task explicitly allows it and required parity evidence already exists.
- Do not change public APIs without a migration note.

## Dependencies

- `RCORE-003`

## Acceptance criteria

Gap list for serialization/data/persistence exists with evidence

## Required commands

```bash
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RCORE-004.md` with:

- task ID;
- summary;
- files changed;
- commands run;
- command results;
- tests added/updated;
- behavior impact;
- rollback plan.
