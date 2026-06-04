# NARCH-004 Observability State Model Evidence

Date: 2026-06-04
Executor: Codex
Task ID: NARCH-004
Risk: medium

## Scope

NARCH-004 defines the future dashboard-readable observability state model.

No telemetry emitters, dashboard UI, control endpoints, runtime code, adapter
behavior, or live trading behavior changed.

## Context Reviewed

- `docs/rust-cutover/tasks/NARCH-004.md`
- `docs/architecture/rust_only_architecture_map.md`
- `docs/architecture/module_contracts.md`
- `docs/architecture/node_lifecycle_state_machine.md`
- `docs/integrations/adapter_support_matrix.md`
- `crates/live/src/node.rs`
- `crates/system/src/trader.rs`
- `crates/data/src`
- `crates/execution/src/client/core.rs`
- `crates/risk/src/engine/mod.rs`
- `crates/portfolio/src/portfolio.rs`
- `crates/common/src/cache`

## Runtime Findings

- NARCH-003 defines lifecycle states that can feed future system status.
- `ExecutionClientCore` exposes connected and started flags, but there is no
  unified gateway status DTO.
- `RiskEngine` tracks trading state and internal counters, but there is no
  stable dashboard summary DTO for rejection counts or last rejection.
- `Portfolio` exposes PnL, exposure, and snapshot methods, but portfolio output
  is sensitive and needs an explicit display/redaction policy.
- DataEngine and cache paths expose subscription, instrument status, order, and
  position information, but there is no unified data-source freshness/status
  DTO.
- Alerts are currently implied by logs/errors; there is no alert aggregator or
  severity policy.

## Changes

- Added `docs/architecture/observability_state_model.md`.
- Defined a future top-level `ObservabilitySnapshot`.
- Defined read-only model sections for:
  - system status;
  - data source status;
  - execution gateway status;
  - risk status;
  - portfolio summary;
  - alert summary;
  - explicit observability gaps.
- Explicitly scoped out secrets, credentials, raw signed payloads, raw venue
  payloads, raw orders, raw account objects, and direct mutable engine access.

## Commands Run

```bash
git diff --check
scripts/ai/validate_agentflow_roles.py
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/NARCH-004.json >/dev/null
scripts/ai/verify_fast.sh
```

## Results

- `git diff --check`: passed.
- `scripts/ai/validate_agentflow_roles.py`: passed.
- `.agentflow/state/task_status.json` and `.agentflow/leases/NARCH-004.json`
  JSON validation: passed.
- `scripts/ai/verify_fast.sh`: passed with Cargo/Rust `1.95.0`.

## Behavior Impact

No runtime behavior changed.

## Public API Impact

No public API changed.

## Migration Note Status

No migration note is required. This task defines a future observability model
and does not expose a new runtime or dashboard API.

## Rollback Plan

Revert this PR to remove the observability model document, evidence, and
agentflow state changes.
