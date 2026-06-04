# NARCH-001 Rust-Only Architecture Map Evidence

Date: 2026-06-04
Executor: Codex
Task ID: NARCH-001
Risk: low

## Scope

NARCH-001 documents the current Rust-only architecture map. It does not
refactor crates, change runtime behavior, implement dashboard telemetry, or add
control APIs.

## Context Reviewed

- `docs/rust-cutover/tasks/NARCH-001.md`
- `docs/rust-cutover/scope/v0_2_0_roadmap.md`
- `docs/rust-cutover/CONTRACT.md`
- `docs/rust-cutover/post-release-gap-list.md`
- `docs/rust-cutover/product/RUST_API_ENTRYPOINTS.md`
- `docs/integrations/adapter_support_matrix.md`
- `docs/concepts/architecture.md`
- `Cargo.toml`
- `crates/*/Cargo.toml`
- `crates/adapters/*/Cargo.toml`

## Audit Commands

```bash
find docs -maxdepth 3 -type f | sort | rg 'architecture|module|contract|component|runtime|system|kernel|engine|adapter|observability|control'
find crates -maxdepth 2 -name Cargo.toml -print | sort
find crates/system/src crates/live/src crates/backtest/src crates/common/src crates/data/src crates/execution/src crates/risk/src crates/portfolio/src crates/model/src -maxdepth 2 -type f | sort
rg -n 'struct (NautilusKernel|Trader|DataEngine|ExecutionEngine|RiskEngine|Portfolio|Cache|MessageBus)|pub struct (NautilusKernel|Trader|DataEngine|ExecutionEngine|RiskEngine|Portfolio|Cache|MessageBus)|enum Environment|pub enum Environment' crates/system/src crates/live/src crates/backtest/src crates/common/src crates/data/src crates/execution/src crates/risk/src crates/portfolio/src crates/model/src
rg -n 'NautilusKernel|Trader|DataEngine|ExecutionEngine|RiskEngine|Portfolio|MessageBus|Cache|EventStore|Adapter|LiveNode|BacktestNode' docs/rust-cutover docs/concepts docs/developer_guide docs/getting_started README.md
git diff --check
scripts/ai/validate_agentflow_roles.py
scripts/ai/verify_fast.sh
```

## Changes

- Added `docs/architecture/rust_only_architecture_map.md`.
- Mapped:
  - product surface;
  - node runtime;
  - system kernel and trader;
  - DataEngine;
  - ExecutionEngine;
  - RiskEngine;
  - Portfolio;
  - MessageBus;
  - Cache;
  - persistence and event store;
  - adapter layer;
  - verification gates.
- Recorded known unknowns and follow-up questions for:
  - module boundary audit;
  - module contracts;
  - node lifecycle;
  - observability state;
  - control actions;
  - dashboard scope;
  - persistence boundary;
  - adapter fixture manifests.

## Results

- Architecture and crate inventory commands completed and identified the
  current product, runtime, engine, adapter, persistence, and verification
  surfaces used by the map.
- `git diff --check`: passed.
- `scripts/ai/validate_agentflow_roles.py`: passed.
- `.agentflow/state/task_status.json` and `.agentflow/leases/NARCH-001.json`
  JSON validation: passed.
- `scripts/ai/verify_fast.sh`: passed with Cargo/Rust `1.95.0`.

## Behavior Impact

No runtime behavior changed. This task is documentation-only.

## Public API Impact

No public API changed.

## Migration Note Status

No migration note is required. The map states that Python, PyO3, and Cython are
not current Rust-only product architecture surfaces.

## Rollback Plan

Revert this PR to remove the architecture map, evidence, and agentflow state
changes.
