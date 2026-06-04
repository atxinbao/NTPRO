# NARCH-003 - Node lifecycle state machine

Milestone: v0.2.0 Architecture Foundation
Priority: P1
Default role: Control & Scope
Risk: medium

## Goal

Define the node lifecycle model.

## Scope

States:

```text
stopped
starting
running
pausing
paused
resuming
stopping
error
```

## Likely files

- `docs/architecture/`
- `docs/rust-cutover/evidence/NARCH-003.md`

## Non-goals

- Do not implement lifecycle code changes.
- Do not add dashboard controls.
- Do not change live trading behavior.

## Dependencies

- `NARCH-002`

## Acceptance criteria

- Lifecycle states and transitions are documented.
- Invalid transitions and error handling expectations are recorded.
- Evidence links the model to current runtime concepts.

## Required commands

```bash
git diff --check
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/NARCH-003.md`.
