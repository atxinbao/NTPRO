# NARCH-004 - Observability state model

Milestone: v0.2.0 Architecture Foundation
Priority: P1
Default role: Control & Scope
Risk: medium

## Goal

Define the future dashboard-readable state model.

## Scope

Areas:

- system status;
- data source status;
- execution gateway status;
- risk status;
- portfolio summary;
- alert summary.

This task defines the model only. It does not implement UI.

## Likely files

- `docs/architecture/`
- `docs/rust-cutover/evidence/NARCH-004.md`

## Non-goals

- Do not implement telemetry emitters.
- Do not build a dashboard UI.
- Do not expose secrets or credentials.

## Dependencies

- `NARCH-003`

## Acceptance criteria

- Observability state model exists.
- Sensitive fields and non-goals are explicitly scoped out.
- Evidence records current sources and future gaps.

## Required commands

```bash
git diff --check
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/NARCH-004.md`.
