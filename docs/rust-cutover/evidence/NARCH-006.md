# NARCH-006 Module Boundary Audit Evidence

Date: 2026-06-04
Executor: Codex
Task ID: NARCH-006
Risk: low

## Scope

NARCH-006 audits current module boundaries before refactoring or
dashboard-facing state extraction. It does not refactor crates, add dashboard
implementation, add runtime telemetry, add control endpoints, or change
runtime behavior.

## Context Reviewed

- `docs/rust-cutover/tasks/NARCH-006.md`
- `docs/architecture/rust_only_architecture_map.md`
- `docs/rust-cutover/scope/v0_2_0_roadmap.md`
- `docs/rust-cutover/post-release-gap-list.md`
- `docs/developer_guide/rust.md`
- `docs/concepts/dst.md`
- `Cargo.toml`
- workspace `crates/*/Cargo.toml`

## Audit Commands

```bash
source scripts/ai/toolchain_env.sh
cargo metadata --no-deps --format-version=1 >/tmp/ntpro-narch-006-metadata.json
python3 - <<'PY'
import json
m=json.load(open('/tmp/ntpro-narch-006-metadata.json'))
print('packages', len(m['packages']))
print('workspace_members', len(m['workspace_members']))
print('\n'.join(sorted(p['name'] for p in m['packages'] if p['id'] in set(m['workspace_members']))[:80]))
PY
rg -n 'NARCH-006|Module boundary|module boundary|boundary audit|Dashboard|observability|control' docs/rust-cutover docs/architecture docs/developer_guide/rust.md docs/concepts/dst.md
git diff --check
scripts/ai/validate_agentflow_roles.py
scripts/ai/verify_fast.sh
```

## Changes

- Added `docs/architecture/module_boundary_audit.md`.
- Recorded current workspace shape: 41 Rust workspace packages.
- Identified clear boundaries:
  - CLI product surface;
  - backtest runtime;
  - live/sandbox runtime;
  - system composition;
  - domain model;
  - data, risk, portfolio;
  - adapter outer boundary;
  - verification support.
- Identified mixed concerns:
  - broad `nautilus-common` shared runtime scope;
  - `nautilus-execution` routing/matching/emulation/reconciliation scope;
  - `nautilus-live` lifecycle/manager/runner/emitter scope;
  - system kernel/trader registry visibility;
  - persistence/event-store/serialization split;
  - adapter crate internal HTTP/WebSocket/parser/example/test mixes.
- Recorded internals that dashboard/control code must not read directly.
- Recorded candidate read-only telemetry surfaces for later NARCH-004 work.
- Separated refactor candidates from executable changes.

## Results

- `cargo metadata --no-deps --format-version=1`: passed and reported 41
  workspace packages.
- Scope search for NARCH/dashboard/observability/control references completed.
- `git diff --check`: passed.
- `scripts/ai/validate_agentflow_roles.py`: passed.
- `.agentflow/state/task_status.json` and `.agentflow/leases/NARCH-006.json`
  JSON validation: passed.
- `scripts/ai/verify_fast.sh`: passed with Cargo/Rust `1.95.0`.

## Behavior Impact

No runtime behavior changed. This is documentation-only architecture audit
work.

## Public API Impact

No public API changed.

## Migration Note Status

No migration note is required.

## Rollback Plan

Revert this PR to remove the module boundary audit, evidence, and agentflow
state changes.
