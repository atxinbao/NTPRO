# RCTL-004 Evidence

Date: 2026-05-27 10:24:47 CST
Executor: Codex
Task ID: RCTL-004
Branch: `ai/RCTL-004-rust-product-surface-inventory`

## Summary

Recorded the current Rust product surface inventory for NTPRO. The inventory
covers Cargo workspace packages, CLI commands, Rust examples, Rust-facing docs,
and runnable Rust test entrypoints. This task also synced task metadata so
RCTL-003 is `DONE` and RCTL-004 is `RUNNING` in `.agentflow/state/task_status.json`.

## Files Changed

- Created `.agentflow/leases/RCTL-004.json`.
- Updated `.agentflow/state/task_status.json`.
- Created `docs/rust-cutover/inventory/RUST_PRODUCT_SURFACE.md`.
- Updated `docs/rust-cutover/inventory/README.md`.
- Created `docs/rust-cutover/evidence/RCTL-004.md`.

## Commands Run

```bash
sed -n '1,220p' docs/rust-cutover/tasks/RCTL-004.md
sed -n '1,220p' docs/rust-cutover/AGENT_ROLES.md
sed -n '1,180p' docs/rust-cutover/TASK_EXECUTION.md
python3 scripts/ai/lease.py claim RCTL-004 --branch ai/RCTL-004-rust-product-surface-inventory --agent-id Codex --path docs/rust-cutover/inventory/RUST_PRODUCT_SURFACE.md --path docs/rust-cutover/evidence/RCTL-004.md --path .agentflow/state/task_status.json --path .agentflow/leases/RCTL-004.json
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo metadata --no-deps --format-version=1
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- --help
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- database --help
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli --features defi -- --help
find crates -maxdepth 4 -path '*/examples/*' -type f
find crates -maxdepth 4 -path '*/tests/*' -type f
find examples -type f -name '*.py'
find docs -maxdepth 3 -type f \( -name '*.md' -o -name '*.rst' \)
scripts/ai/validate_agentflow_roles.py
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
python3 scripts/ai/lease.py release RCTL-004 --status PR_READY
```

## Command Results

- RCTL-004 task and role/task protocol reads completed.
- Lease was claimed at `.agentflow/leases/RCTL-004.json` and released as `PR_READY`.
- `cargo metadata --no-deps --format-version=1` completed.
  - Workspace packages: 42.
  - Adapter packages: 17.
  - Non-adapter packages: 25.
  - `nautilus-pyo3` remains present.
- `cargo run -q -p nautilus-cli -- --help` passed.
  - Default top-level command surface includes `database`.
- `cargo run -q -p nautilus-cli -- database --help` passed.
  - Database subcommands include `init` and `drop`.
- `cargo run -q -p nautilus-cli --features defi -- --help` passed.
  - Feature-gated command surface includes `blockchain`.
- Inventory counts recorded:
  - Rust crate example files under `crates/**/examples`: 52.
  - Rust crate test files under `crates/**/tests`: 114.
  - Python example files under `examples/**`: 136.
  - Docs markdown/reStructuredText files under `docs/**`: 238.
- `validate_agentflow_roles.py`: passed.
- `verify_fast.sh`: passed with `PATH="/opt/homebrew/opt/rustup/bin:$PATH"`.
  - `cargo fmt --check` passed.
  - Default fast mode skipped optional cargo check and clippy by design.

## Tests Added or Updated

No runtime tests were added or updated. RCTL-004 is an inventory/evidence task.

## Behavior Impact

No runtime behavior impact. No trading semantics, public APIs, adapters, CLI
behavior, precision behavior, Python/PyO3/Cython product surfaces, or Cargo
workspace configuration were changed.

## Public API Impact

None.

## Migration Note Status

Not required.

## Rollback Plan

- Remove `docs/rust-cutover/inventory/RUST_PRODUCT_SURFACE.md`.
- Revert `docs/rust-cutover/inventory/README.md`.
- Remove `docs/rust-cutover/evidence/RCTL-004.md`.
- Revert `.agentflow/state/task_status.json` to the previous task-state snapshot.
- Release or remove `.agentflow/leases/RCTL-004.json` if abandoning this branch.
