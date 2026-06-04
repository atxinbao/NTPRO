# NARCH-002 Module Contracts Evidence

Date: 2026-06-04
Executor: Codex
Task ID: NARCH-002
Risk: medium

## Scope

NARCH-002 writes contracts for core Rust-only modules. It does not refactor
module code, add dashboard implementation, or change public runtime behavior.

## Context Reviewed

- `docs/rust-cutover/tasks/NARCH-002.md`
- `docs/architecture/rust_only_architecture_map.md`
- `docs/architecture/module_boundary_audit.md`
- `docs/rust-cutover/scope/v0_2_0_roadmap.md`
- `docs/rust-cutover/post-release-gap-list.md`

## Changes

- Added `docs/architecture/module_contracts.md`.
- Defined contracts for:
  - product surface;
  - node runtime;
  - system kernel and trader;
  - DataEngine;
  - ExecutionEngine;
  - RiskEngine;
  - Portfolio;
  - MessageBus and Cache;
  - persistence and event store;
  - adapter layer;
  - verification.
- Each contract records:
  - responsibilities;
  - inputs;
  - outputs;
  - state;
  - lifecycle;
  - error model;
  - dependency boundaries;
  - candidate dashboard-observable fields.
- Recorded unresolved gaps for NARCH-003, NARCH-004, NARCH-005, NDASH-001,
  persistence artifact contracts, and adapter fixture manifests.

## Commands Run

```bash
git diff --check
scripts/ai/validate_agentflow_roles.py
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/NARCH-002.json >/dev/null
scripts/ai/verify_fast.sh
```

## Results

- `git diff --check`: passed.
- `scripts/ai/validate_agentflow_roles.py`: passed.
- `.agentflow/state/task_status.json` and `.agentflow/leases/NARCH-002.json`
  JSON validation: passed.
- `scripts/ai/verify_fast.sh`: passed with Cargo/Rust `1.95.0`.

## Behavior Impact

No runtime behavior changed. This is documentation-only architecture contract
work.

## Public API Impact

No public API changed.

## Migration Note Status

No migration note is required. The contracts keep Python, PyO3, and Cython out
of the product architecture surface.

## Rollback Plan

Revert this PR to remove module contracts, evidence, and agentflow state
changes.
