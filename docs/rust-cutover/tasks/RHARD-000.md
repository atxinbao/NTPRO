# RHARD-000 - Public release surface cleanup

Milestone: v0.2.0 Hardening
Priority: P0
Default role: Rust Product Surface
Risk: low

## Goal

Clean the public appearance after the formal Rust-only release.

## Scope

- Update README wording from release-candidate language to formal Rust-only
  release workspace language.
- Clean remaining Python/PyPI user-path references from docs.
- State that Python is allowed only for local repository helper scripts, not as
  a product entrypoint.
- Keep release notes, installation docs, and getting-started docs consistent.

## Likely files

- `README.md`
- `docs/getting_started/`
- `docs/tutorials/`
- `docs/rust-cutover/`

## Non-goals

- Do not change Rust runtime behavior.
- Do not create release tags or GitHub Releases.
- Do not add Python/PyO3/Cython product surfaces.

## Dependencies

- none

## Acceptance criteria

- Public release docs describe NTPRO as a formal Rust-only release workspace.
- Python/PyPI install paths are only mentioned as unsupported paths or
  historical migration evidence.
- Public surface audit evidence is recorded.

## Required commands

```bash
git diff --check
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RHARD-000.md`.
