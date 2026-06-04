# RHARD-004 - Sandbox demo

Milestone: v0.2.0 Hardening
Priority: P1
Default role: Rust Product Surface
Risk: medium

## Goal

Provide a minimal sandbox or paper-like run path.

## Scope

- Use simulated data.
- Use simulated execution.
- Show node start and stop.
- Show event flow.
- Expose basic risk, portfolio, and cache state.

## Likely files

- `examples/rust/`
- `crates/live/examples/`
- `docs/`
- `docs/rust-cutover/evidence/RHARD-004.md`

## Non-goals

- Do not submit real orders.
- Do not require exchange credentials.
- Do not implement dashboard controls.

## Dependencies

- `RHARD-006`

## Acceptance criteria

- A minimal sandbox path is documented or executable.
- Smoke evidence proves the path starts and stops safely.
- Deferred items are recorded with owner-visible notes.

## Required commands

```bash
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RHARD-004.md`.
