# RADP-006 - Close Rust adapter gaps for Bybit OKX Kraken

Milestone: R5 Rust Adapter Parity
Priority: P0
Default role: Coding

## Goal

Close Rust adapter gaps for Bybit OKX Kraken

## Scope

Implement or scope Rust-only adapter gaps for Bybit OKX Kraken

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

- `RADP-005`

## Acceptance criteria

Bybit OKX Kraken adapter parity resolved or explicitly scoped with evidence

## Required commands

```bash
scripts/ai/verify_full.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RADP-006.md` with:

- task ID;
- summary;
- files changed;
- commands run;
- command results;
- tests added/updated;
- behavior impact;
- rollback plan.
