# RTRACE-005 - Bind backtest replay trace

Milestone: R2 Golden Trace
Priority: P1
Default role: Test

## Goal

Bind backtest replay trace

## Scope

Run one backtest trace through Rust replay or record scoped blocker

## Likely files

- `tests/golden/`
- `scripts/ai/`
- `docs/rust-cutover/golden_trace/`
- `crates/`

## Non-goals

- Do not refactor unrelated modules.
- Do not change trading semantics unless this task explicitly requires it.
- Do not delete Python, PyO3, or Cython product surfaces unless this task explicitly allows it and required parity evidence already exists.
- Do not change public APIs without a migration note.

## Dependencies

- `RTRACE-004`

## Acceptance criteria

Golden trace evidence exists and is executable or explicitly scoped

## Required commands

```bash
scripts/ai/verify_fast.sh
```
```bash
scripts/ai/run_golden_traces.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RTRACE-005.md` with:

- task ID;
- summary;
- files changed;
- commands run;
- command results;
- tests added/updated;
- behavior impact;
- rollback plan.
