# NARCH-002 - Module contracts

Milestone: v0.2.0 Architecture Foundation
Priority: P1
Default role: Control & Scope
Risk: medium

## Goal

Write contracts for core modules.

## Scope

Each contract should define:

- responsibilities;
- inputs;
- outputs;
- state;
- lifecycle;
- error model;
- dependency boundaries;
- candidate dashboard-observable fields.

## Likely files

- `docs/architecture/`
- `docs/rust-cutover/evidence/NARCH-002.md`

## Non-goals

- Do not refactor module code.
- Do not add dashboard implementation.
- Do not change public runtime behavior.

## Dependencies

- `NARCH-006`

## Acceptance criteria

- Core module contracts exist.
- Contracts distinguish current behavior from future dashboard needs.
- Evidence records reviewed modules and unresolved gaps.

## Required commands

```bash
git diff --check
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/NARCH-002.md`.
