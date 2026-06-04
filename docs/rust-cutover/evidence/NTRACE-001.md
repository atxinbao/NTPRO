# NTRACE-001 Trace And Performance Expansion Plan Evidence

Date: 2026-06-04
Executor: Codex
Task ID: NTRACE-001
Risk: medium

## Scope

NTRACE-001 defines v0.2.0 trace and performance evidence expansion. It does
not implement trace runner changes, change trading semantics, weaken existing
golden trace gates, or make performance smoke a release blocker by accident.

## Context Reviewed

- `docs/rust-cutover/tasks/NTRACE-001.md`
- `docs/rust-cutover/scope/v0_2_0_roadmap.md`
- `docs/rust-cutover/golden_trace/SCHEMA.md`
- `docs/rust-cutover/golden_trace/GATE_EVIDENCE.md`
- `docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json`
- `docs/rust-cutover/release/final_release_verification.md`
- `docs/rust-cutover/verification/README.md`
- `scripts/ai/run_golden_traces.sh`
- `scripts/ai/verify_release.sh`

## Changes

- Added `docs/rust-cutover/trace_performance_expansion_plan.md`.
- Recorded the current golden trace baseline:
  - 8 JSONL trace files;
  - 18 rows;
  - executable replay for backtest, live/sandbox lifecycle,
    cache/message-bus, OKX adapter payload, and scoped backtest/live parity;
  - schema-only scoped rows for market data and order lifecycle.
- Defined required v0.2.0 trace expansion areas:
  - backtest;
  - live and sandbox lifecycle;
  - data source;
  - execution order lifecycle;
  - risk rejection;
  - adapter payload.
- Separated deferred and future evidence from required v0.2.0 evidence.
- Defined performance smoke as non-blocking by default unless a later
  owner-approved task promotes a metric to a release blocker.

## Commands Run

```bash
rg --files crates tests | rg '(^|/)benches/|bench.*\\.rs$|tests/golden' | sort
git diff --check
scripts/ai/validate_agentflow_roles.py
scripts/ai/verify_fast.sh
```

## Results

- Bench and golden trace inventory command completed and confirmed existing
  Rust bench sources plus `tests/golden/*.jsonl` trace files.
- `git diff --check`: passed.
- `scripts/ai/validate_agentflow_roles.py`: passed.
- `.agentflow/state/task_status.json` and `.agentflow/leases/NTRACE-001.json`
  JSON validation: passed.
- `scripts/ai/verify_fast.sh`: passed with Cargo/Rust `1.95.0`.

## Behavior Impact

No trading behavior changed. This is a planning and evidence task only.

## Public API Impact

No public API changed.

## Migration Note Status

No migration note is required. The plan keeps Python/PyO3/Cython out of the
Rust-only trace evidence path.

## Rollback Plan

Revert this PR to remove the NTRACE-001 plan, evidence, and agentflow state
changes.
