# NARCH-006 - Module boundary audit

Milestone: v0.2.0 Architecture Foundation
Priority: P1
Default role: Control & Scope
Risk: low

## Goal

Audit current module boundaries before refactoring.

## Scope

- Check whether current crates match the target architecture boundaries.
- Identify mixed concerns.
- Identify internal implementation details that Dashboard code must not read
  directly.
- Identify state that should later be exposed through a stable telemetry
  surface.

## Likely files

- `docs/architecture/`
- `docs/rust-cutover/evidence/NARCH-006.md`

## Non-goals

- Do not refactor crate boundaries.
- Do not add dashboard implementation.
- Do not change runtime behavior.

## Dependencies

- `NARCH-001`

## Acceptance criteria

- Module boundary audit exists.
- Mixed concerns and dashboard boundary risks are recorded.
- Follow-up refactor candidates are separated from executable changes.

## Required commands

```bash
git diff --check
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/NARCH-006.md`.
