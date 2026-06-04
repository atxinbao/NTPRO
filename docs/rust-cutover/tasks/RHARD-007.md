# RHARD-007 - Verification cleanup

Milestone: v0.2.0 Hardening
Priority: P0
Default role: Verification
Risk: low

## Goal

Make verification choices clear for users and agents.

## Scope

- Document when to run `verify_fast`.
- Document when to run `verify_full`.
- Document when to run `verify_release`.
- Document `check_rust_only_runtime`.
- Document `check_cython_removed`.
- Document golden trace checks.
- Explain why release build is slow and expected to take substantially longer.

## Likely files

- `docs/rust-cutover/`
- `scripts/ai/`
- `docs/rust-cutover/evidence/RHARD-007.md`

## Non-goals

- Do not weaken release verification.
- Do not make CI the only validation source.
- Do not change trading behavior.

## Dependencies

- `RHARD-002`

## Acceptance criteria

- Verification guide explains fast, full, release, Rust-only, Cython-removal,
  and golden trace checks.
- Slow checks are clearly labeled.
- Evidence records local validation.

## Required commands

```bash
git diff --check
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RHARD-007.md`.
