# RHARD-002 - Toolchain and verification path hardening

Milestone: v0.2.0 Hardening
Priority: P0
Default role: Verification
Risk: medium

## Goal

Prevent local verification from using the wrong Rust compiler.

## Scope

- Document the required Rust 1.95.0 toolchain path.
- Explain `rustup override set 1.95.0`.
- Avoid accidental use of Homebrew `rustc` when running release checks.
- Add an optional preflight check if needed.

## Likely files

- `docs/getting_started/`
- `docs/rust-cutover/`
- `scripts/ai/`

## Non-goals

- Do not change Cargo workspace structure.
- Do not change Rust runtime behavior.
- Do not add CI-only verification as the source of truth.

## Dependencies

- `RHARD-001`

## Acceptance criteria

- Toolchain selection is documented for users and agents.
- Verification docs explain how to confirm the active compiler.
- Optional preflight changes, if any, are covered by local evidence.

## Required commands

```bash
git diff --check
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RHARD-002.md`.
