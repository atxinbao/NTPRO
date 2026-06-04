# NDASH-001 Dashboard MVP Scope Contract Evidence

Date: 2026-06-04
Executor: Codex
Task ID: NDASH-001
Risk: low

## Scope

NDASH-001 locks the first dashboard MVP scope before implementation starts.

No dashboard UI, runtime control endpoint, telemetry emitter, order-entry
workflow, adapter behavior, or live trading behavior changed.

## Context Reviewed

- `docs/rust-cutover/tasks/NDASH-001.md`
- `docs/architecture/node_lifecycle_state_machine.md`
- `docs/architecture/observability_state_model.md`
- `docs/architecture/control_api_contract.md`
- `docs/architecture/module_contracts.md`

## Changes

- Added `docs/architecture/dashboard_mvp_scope_contract.md`.
- Defined the dashboard MVP goal as local operator visibility plus a small
  lifecycle-control boundary.
- Included:
  - status viewing;
  - alert viewing;
  - node lifecycle viewing;
  - start/stop/pause/resume control contract surface.
- Excluded:
  - manual order entry;
  - order modification/cancellation;
  - strategy parameter hot reload;
  - multi-user permissions;
  - complex asset management;
  - full trading frontend scope;
  - Docker delivery as a requirement.
- Recorded redaction/data policy and future implementation acceptance criteria.

## Commands Run

```bash
git diff --check
scripts/ai/validate_agentflow_roles.py
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/NDASH-001.json >/dev/null
scripts/ai/verify_fast.sh
```

## Results

- `git diff --check`: passed.
- `scripts/ai/validate_agentflow_roles.py`: passed.
- `.agentflow/state/task_status.json` and `.agentflow/leases/NDASH-001.json`
  JSON validation: passed.
- `scripts/ai/verify_fast.sh`: passed with Cargo/Rust `1.95.0`.

## Behavior Impact

No runtime behavior changed.

## Public API Impact

No public API changed.

## Migration Note Status

No migration note is required. This task locks dashboard scope only.

## Rollback Plan

Revert this PR to remove the dashboard MVP scope contract, evidence, and
agentflow state changes.
