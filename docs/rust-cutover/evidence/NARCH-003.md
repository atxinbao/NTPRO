# NARCH-003 Node Lifecycle State Machine Evidence

Date: 2026-06-04
Executor: Codex
Task ID: NARCH-003
Risk: medium

## Scope

NARCH-003 defines the node lifecycle state machine for future operator and
dashboard contracts. It is documentation-only work.

No runtime code, dashboard implementation, adapter behavior, or live trading
behavior changed.

## Context Reviewed

- `docs/rust-cutover/tasks/NARCH-003.md`
- `docs/architecture/rust_only_architecture_map.md`
- `docs/architecture/module_contracts.md`
- `crates/live/src/node.rs`
- `crates/live/src/runner.rs`
- `crates/common/src/enums.rs`
- `crates/common/src/component.rs`
- `crates/system/src/trader.rs`
- `docs/developer_guide/rust.md`

## Runtime Findings

- `crates/live/src/node.rs` defines the current `NodeState` values:
  `Idle`, `Starting`, `Running`, `ShuttingDown`, and `Stopped`.
- `LiveNodeHandle` stores the current node state in an `Arc<AtomicU8>` and
  exposes stop signaling through a shared stop flag.
- `LiveNode::start` moves into `Starting`, then reaches `Running` on current
  startup paths or aborts startup when shutdown/stop conditions are detected.
- `LiveNode::stop` requires a running node, moves into `ShuttingDown`, stops
  the trader, disconnects clients, finalizes stop, and sets `Stopped`.
- `crates/common/src/enums.rs` and `crates/common/src/component.rs` define
  component-level lifecycle states and transitions, including `Resuming`,
  `Faulting`, and `Faulted`.
- `crates/system/src/trader.rs` manages trader lifecycle through initialize,
  start, stop, reset, and dispose hooks.

## Changes

- Added `docs/architecture/node_lifecycle_state_machine.md`.
- Documented the contract states:
  - `stopped`
  - `starting`
  - `running`
  - `pausing`
  - `paused`
  - `resuming`
  - `stopping`
  - `error`
- Mapped current runtime states to the contract.
- Marked `pausing`, `paused`, `resuming`, and top-level `error` as future
  contract states rather than current `LiveNode` enum variants.
- Recorded valid transitions, invalid transition expectations, error handling
  expectations, and future dashboard-readable fields.

## Commands Run

```bash
git diff --check
scripts/ai/validate_agentflow_roles.py
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/NARCH-003.json >/dev/null
scripts/ai/verify_fast.sh
```

## Results

- `git diff --check`: passed.
- `scripts/ai/validate_agentflow_roles.py`: passed.
- `.agentflow/state/task_status.json` and `.agentflow/leases/NARCH-003.json`
  JSON validation: passed.
- `scripts/ai/verify_fast.sh`: passed with Cargo/Rust `1.95.0`.

## Behavior Impact

No runtime behavior changed.

## Public API Impact

No public API changed.

## Migration Note Status

No migration note is required. This PR documents future operator/dashboard
contract states and keeps unimplemented pause/resume controls explicitly out of
the current runtime surface.

## Rollback Plan

Revert this PR to remove the lifecycle state machine document, evidence, and
agentflow state changes.
