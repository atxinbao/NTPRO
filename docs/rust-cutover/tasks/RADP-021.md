# RADP-021 - Close Rust adapter gaps for Betfair Architect AX

Milestone: R5 Rust Adapter Parity
Priority: P0
Default role: Coding

## Goal

Close Rust adapter gaps for Betfair Architect AX

## Scope

Implement or scope Rust-only adapter gaps for Betfair Architect AX

## Likely files

- `crates/adapters/`
- `examples/rust/`
- `docs/rust-cutover/evidence/`

## Non-goals

- Do not refactor unrelated modules.
- Do not change trading semantics unless this task explicitly requires it.
- Do not delete Python, PyO3, or Cython product surfaces unless this task explicitly allows it and required parity evidence already exists.
- Do not change public APIs without a migration note.

## Dependencies

- `RADP-020`

## Acceptance criteria

Betfair Architect AX adapter parity resolved or explicitly scoped with evidence

## Required commands

```bash
scripts/ai/verify_full.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RADP-021.md` with:

- task ID;
- summary;
- files changed;
- commands run;
- command results;
- tests added/updated;
- behavior impact;
- rollback plan.
