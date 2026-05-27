# RCORE-013 - Inventory Rust portfolio/accounting/PnL gaps

Milestone: R3 Rust Core Runtime
Priority: P0
Default role: Inventory/Coding

## Goal

Inventory Rust portfolio/accounting/PnL gaps

## Scope

Inspect crates/portfolio/ and account-related model code for Rust-only parity gaps

## Likely files

- `crates/`
- `docs/rust-cutover/inventory/`

## Non-goals

- Do not refactor unrelated modules.
- Do not change trading semantics unless this task explicitly requires it.
- Do not delete Python, PyO3, or Cython product surfaces unless this task explicitly allows it and required parity evidence already exists.
- Do not change public APIs without a migration note.

## Dependencies

- `RCORE-012`

## Acceptance criteria

Gap list for portfolio/accounting/PnL exists with evidence

## Required commands

```bash
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RCORE-013.md` with:

- task ID;
- summary;
- files changed;
- commands run;
- command results;
- tests added/updated;
- behavior impact;
- rollback plan.
