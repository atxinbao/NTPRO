# NARCH-001 - Rust-only architecture map

Milestone: v0.2.0 Architecture Foundation
Priority: P1
Default role: Control & Scope
Risk: low

## Goal

Document the current Rust-only architecture.

## Scope

Map these areas:

- Product Surface;
- Node Runtime;
- System Kernel / Trader;
- DataEngine;
- ExecutionEngine;
- RiskEngine;
- Portfolio;
- MessageBus;
- Cache;
- Persistence / Event Store;
- Adapter Layer;
- Verification Gates.

## Likely files

- `docs/architecture/`
- `docs/rust-cutover/`
- `docs/rust-cutover/evidence/NARCH-001.md`

## Non-goals

- Do not refactor crates.
- Do not change runtime behavior.
- Do not implement dashboard telemetry.

## Dependencies

- `NTRACE-001`

## Acceptance criteria

- Architecture map exists and links major runtime and product surfaces.
- Unknowns and follow-up questions are recorded.
- Evidence records audit commands and files reviewed.

## Required commands

```bash
git diff --check
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/NARCH-001.md`.
