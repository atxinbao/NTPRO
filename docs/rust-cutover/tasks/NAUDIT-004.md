# NAUDIT-004 - Product-reachable runtime panic cleanup

Milestone: v0.2.0 Audit Backlog
Priority: P0
Default role: Rust Core Runtime
Risk: high

## Goal

Replace product-reachable runtime `panic!()` paths in trading, execution, and
backtest flows with explicit errors, rejection events, or unsupported statuses.

## Scope

- Audit at least:
  - Mark price bar handling in the matching engine.
  - Missing OTO parent order handling.
  - Missing contingent linked order handling.
  - Backtest exchange unknown trading command handling.
  - Backtest exchange missing latency model handling.
- Convert product-reachable panics to owner-visible failure paths.
- Add focused regression tests for each changed path.

## Likely files

- `crates/execution/src/matching_engine/engine.rs`
- `crates/backtest/src/exchange.rs`
- `crates/execution/tests/`
- `crates/backtest/tests/`

## Non-goals

- Do not change matching semantics beyond replacing crash behavior with
  explicit rejection/error behavior.
- Do not expand adapter behavior.
- Do not remove tests to pass validation.

## Dependencies

- `GH-158`

## Acceptance criteria

- Product-reachable paths no longer abort the process for the scoped conditions.
- Regression tests cover the changed paths.
- Behavior changes are documented in evidence.

## Required commands

```bash
cargo test -p nautilus-execution
cargo test -p nautilus-backtest
git diff --check
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/NAUDIT-004.md`.
