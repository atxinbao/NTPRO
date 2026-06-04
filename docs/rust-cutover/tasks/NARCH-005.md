# NARCH-005 - Control API contract

Milestone: v0.2.0 Architecture Foundation
Priority: P1
Default role: Control & Scope
Risk: medium

## Goal

Define control actions without implementing live control.

## Scope

Actions:

```text
start
stop
restart
pause_trading
resume_trading
reconnect_data
reconnect_execution
```

This task is contract-only. Runtime implementation belongs to a later
dashboard or control-plane phase.

## Likely files

- `docs/architecture/`
- `docs/rust-cutover/evidence/NARCH-005.md`

## Non-goals

- Do not implement live control endpoints.
- Do not build dashboard UI.
- Do not add order-entry controls.

## Dependencies

- `NARCH-004`

## Acceptance criteria

- Control API contract exists.
- Allowed actions, preconditions, effects, and failure modes are documented.
- Evidence records that no runtime control implementation was added.

## Required commands

```bash
git diff --check
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/NARCH-005.md`.
