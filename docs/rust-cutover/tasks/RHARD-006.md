# RHARD-006 - Backtest CLI minimal path

Milestone: v0.2.0 Hardening
Priority: P0
Default role: Rust Product Surface
Risk: medium

## Goal

Let a user run one minimal backtest path.

## Scope

- Define input data.
- Define config.
- Define command.
- Define output path.
- Define expected result.

## Likely files

- `crates/cli/`
- `examples/rust/`
- `docs/`
- `docs/rust-cutover/evidence/RHARD-006.md`

## Non-goals

- Do not add large datasets.
- Do not change trading semantics.
- Do not restore Python backtest entrypoints.

## Dependencies

- `RHARD-003`

## Acceptance criteria

- One minimal backtest path is documented or executable.
- Smoke evidence records command, input, output, and result.
- Missing runtime work is documented as a blocker instead of hidden.

## Required commands

```bash
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RHARD-006.md`.
