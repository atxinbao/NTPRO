# NTRACE-001 - Trace and performance expansion plan

Milestone: v0.2.0 Verification
Priority: P1
Default role: Verification
Risk: medium

## Goal

Define v0.2.0 trace and performance evidence expansion.

## Scope

- Backtest trace expansion.
- Live and sandbox lifecycle trace expansion.
- Data source trace expansion.
- Execution order lifecycle trace expansion.
- Risk rejection trace expansion.
- Adapter payload trace expansion.
- Performance smoke scope.

## Likely files

- `docs/rust-cutover/trace_performance_expansion_plan.md`
- `docs/rust-cutover/evidence/NTRACE-001.md`

## Non-goals

- Do not implement the full trace runner changes.
- Do not weaken existing golden trace gates.
- Do not change trading semantics.

## Dependencies

- `NBIN-001`

## Acceptance criteria

- Trace and performance expansion plan exists.
- Scope separates required, deferred, and future evidence.
- Performance smoke is defined without becoming a release blocker by accident.

## Required commands

```bash
git diff --check
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/NTRACE-001.md`.
