# RHARD-005 - Live init smoke

Milestone: v0.2.0 Hardening
Priority: P1
Default role: Rust Product Surface
Risk: medium

## Goal

Verify live node initialization and shutdown without real orders.

## Scope

- Define live config.
- Initialize kernel.
- Register adapter.
- Start and shut down.

## Likely files

- `examples/rust/`
- `crates/live/examples/`
- `docs/`
- `docs/rust-cutover/evidence/RHARD-005.md`

## Non-goals

- Do not call real trading endpoints.
- Do not require live credentials.
- Do not change production adapter behavior.

## Dependencies

- `RHARD-004`

## Acceptance criteria

- Live initialization and shutdown path is documented or executable.
- Evidence records whether the smoke is full, fixture-backed, or deferred.
- No real order flow is introduced.

## Required commands

```bash
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RHARD-005.md`.
