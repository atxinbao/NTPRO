# NBIN-001 - Release binary and install path

Milestone: v0.2.0 Product Delivery
Priority: P1
Default role: Rust Product Surface
Risk: medium

## Goal

Define how users install and run the Rust CLI.

## Scope

- Decide source build path.
- Decide whether `cargo install` is supported.
- Decide release artifact strategy.
- Decide binary naming.
- Decide platform scope.
- Explicitly defer Docker delivery for now.

## Likely files

- `docs/getting_started/`
- `docs/rust-cutover/`
- `docs/rust-cutover/evidence/NBIN-001.md`

## Non-goals

- Do not publish binaries.
- Do not create release tags.
- Do not introduce Docker delivery as a v0.2.0 requirement.

## Dependencies

- `NADAPT-001`

## Acceptance criteria

- Install and run path is documented.
- Binary and release artifact decision record exists.
- Platform scope is explicit.

## Required commands

```bash
git diff --check
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/NBIN-001.md`.
