# NARCH-005 Control API Contract Evidence

Date: 2026-06-04
Executor: Codex
Task ID: NARCH-005
Risk: medium

## Scope

NARCH-005 defines future control actions without implementing live control.

No runtime control endpoints, dashboard UI, order-entry controls, runtime code,
adapter behavior, or live trading behavior changed.

## Context Reviewed

- `docs/rust-cutover/tasks/NARCH-005.md`
- `docs/architecture/node_lifecycle_state_machine.md`
- `docs/architecture/observability_state_model.md`
- `crates/live/src/node.rs`
- `crates/live/src/runner.rs`
- `crates/system/src/kernel.rs`
- `crates/system/src/trader.rs`
- `crates/data/src/client.rs`
- `crates/data/src/engine/mod.rs`
- `crates/execution/src/client/mod.rs`
- `crates/execution/src/engine/mod.rs`
- `crates/risk/src/engine/mod.rs`

## Runtime Findings

- `LiveNode::start` and `LiveNode::stop` exist, but there is no dashboard or
  external control API wrapper.
- `LiveNodeHandle::stop` exists as a cross-thread stop signal and state reader.
- Kernel and client paths include connect/disconnect functions for data and
  execution clients, but no stable external reconnect action exists.
- `pause_trading`, `resume_trading`, and a single `restart` action are not
  currently implemented as top-level runtime controls.
- Existing start/stop behavior is stricter than future operator control in some
  cases; for example direct `LiveNode::stop` rejects when the node is not
  running.

## Changes

- Added `docs/architecture/control_api_contract.md`.
- Documented request/response shapes for future control actions.
- Defined action contracts for:
  - `start`
  - `stop`
  - `restart`
  - `pause_trading`
  - `resume_trading`
  - `reconnect_data`
  - `reconnect_execution`
- Recorded allowed states, rejected states, expected effects, current runtime
  anchors, failure modes, and required evidence.
- Explicitly forbids order-entry controls, direct cache/message-bus mutation,
  credential changes, raw venue requests, and production endpoint validation
  without explicit later approval.

## Commands Run

```bash
git diff --check
scripts/ai/validate_agentflow_roles.py
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/NARCH-005.json >/dev/null
scripts/ai/verify_fast.sh
```

## Results

- `git diff --check`: passed.
- `scripts/ai/validate_agentflow_roles.py`: passed.
- `.agentflow/state/task_status.json` and `.agentflow/leases/NARCH-005.json`
  JSON validation: passed.
- `scripts/ai/verify_fast.sh`: passed with Cargo/Rust `1.95.0`.

## Behavior Impact

No runtime behavior changed.

## Public API Impact

No public API changed.

## Migration Note Status

No migration note is required. This task defines a future control contract and
does not expose a new runtime API.

## Rollback Plan

Revert this PR to remove the control API contract document, evidence, and
agentflow state changes.
