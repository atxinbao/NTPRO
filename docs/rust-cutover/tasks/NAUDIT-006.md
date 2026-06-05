# NAUDIT-006 - Live adapter cancellation contract and mock evidence

Milestone: v0.2.0 Audit Backlog
Priority: P1
Default role: Adapter & Integration
Risk: high

## Goal

Prove that live startup cancellation is safe for adapter connection futures
instead of only proving that the live node drops a pending future.

## Scope

- Define the cancellation contract for data-client connect futures.
- Add a mock adapter or fixture-backed test that verifies:
  - a pending connect future is dropped on stop/shutdown;
  - resources are released;
  - no half-connected state remains;
  - retry or cleanup behavior is explicit.
- Record which real adapters still need cancellation-safety proof.

## Likely files

- `crates/live/`
- `crates/adapters/`
- `docs/rust-cutover/evidence/`
- `docs/integrations/`

## Non-goals

- Do not connect to real exchanges.
- Do not implement dashboard UI or control API.
- Do not change adapter trading behavior without fixture evidence.

## Dependencies

- `GH-159`

## Acceptance criteria

- Data-client cancellation expectations are documented.
- Mock or fixture test proves cancellation cleanup.
- Remaining real-adapter proof gaps are listed.

## Required commands

```bash
cargo test -p nautilus-live
cargo check -p nautilus-live
git diff --check
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/NAUDIT-006.md`.
