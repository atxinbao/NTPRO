# RADP-019 - Inventory Rust adapter gaps for Betfair Architect AX

Milestone: R5 Rust Adapter Parity
Priority: P0
Default role: Inventory/Coding

## Goal

Inventory Rust adapter gaps for Betfair Architect AX

## Scope

Inspect crates/adapters/betfair/ and crates/adapters/architect_ax/ for Rust-only parser, data, and execution gaps

## Likely files

- `crates/adapters/`
- `docs/rust-cutover/inventory/`

## Non-goals

- Do not refactor unrelated modules.
- Do not change trading semantics unless this task explicitly requires it.
- Do not delete Python, PyO3, or Cython product surfaces unless this task explicitly allows it and required parity evidence already exists.
- Do not change public APIs without a migration note.

## Dependencies

- `RADP-018`

## Acceptance criteria

Betfair Architect AX Rust adapter gap list exists

## Required commands

```bash
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RADP-019.md` with:

- task ID;
- summary;
- files changed;
- commands run;
- command results;
- tests added/updated;
- behavior impact;
- rollback plan.
