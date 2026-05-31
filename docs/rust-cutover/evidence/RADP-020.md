# RADP-020 Evidence - Add Rust Adapter Fixtures For Betfair Architect AX

Date: 2026-05-31
Executor: Codex
Task ID: RADP-020
Risk: medium

## Summary

Added compact Rust fixture manifests for the Betfair and Architect AX adapters
and added manifest-level Rust tests that verify every listed fixture and
primary parser/test path is resolvable.

No adapter runtime behavior changed. The new tests pin parser, data, execution,
session, reconnect, and scoped blocker evidence for the later RADP-021 closure
task.

## Files Changed

- `crates/adapters/betfair/test_data/rust_fixture_manifest.json`
- `crates/adapters/betfair/tests/fixture_manifest.rs`
- `crates/adapters/architect_ax/test_data/rust_fixture_manifest.json`
- `crates/adapters/architect_ax/tests/fixture_manifest.rs`
- `docs/rust-cutover/evidence/RADP-020.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RADP-020.json`

## Commands Run

```bash
sed -n '1,220p' docs/rust-cutover/tasks/RADP-020.md
sed -n '1,260p' docs/rust-cutover/TASK_EXECUTION.md
sed -n '1,260p' docs/rust-cutover/AGENT_ROLES.md
sed -n '1,260p' docs/rust-cutover/CONTRACT.md
sed -n '1,260p' docs/rust-cutover/DEFINITION_OF_DONE.md
find crates/adapters/betfair/test_data -type f
find crates/adapters/architect_ax/test_data -type f
rg -n "fn .*test|#\\[rstest|#\\[test|#\\[tokio::test" crates/adapters/betfair/tests crates/adapters/betfair/src/http crates/adapters/betfair/src/stream crates/adapters/betfair/src/common crates/adapters/betfair/src/data.rs crates/adapters/betfair/src/execution.rs
rg -n "fn .*test|#\\[rstest|#\\[test|#\\[tokio::test" crates/adapters/architect_ax/tests crates/adapters/architect_ax/src/http crates/adapters/architect_ax/src/websocket crates/adapters/architect_ax/src/common crates/adapters/architect_ax/src/data.rs crates/adapters/architect_ax/src/execution.rs
python3 -m json.tool crates/adapters/betfair/test_data/rust_fixture_manifest.json >/dev/null
python3 -m json.tool crates/adapters/architect_ax/test_data/rust_fixture_manifest.json >/dev/null
cargo fmt --check
cargo test -p nautilus-betfair --test fixture_manifest
cargo test -p nautilus-architect-ax --test fixture_manifest
scripts/ai/verify_full.sh
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RADP-020.json >/dev/null
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
```

## Command Results

- Betfair fixture manifest JSON validation: passed.
- Architect AX fixture manifest JSON validation: passed.
- `cargo fmt --check`: passed.
- `cargo test -p nautilus-betfair --test fixture_manifest`: the default local
  Rust toolchain is `rustc 1.87.0` and the workspace requires `rustc 1.95`, so
  the first local attempt stopped at toolchain selection. Re-ran with
  `/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin` on `PATH`;
  passed with 2 tests.
- `cargo test -p nautilus-architect-ax --test fixture_manifest`: re-ran with
  the same Rust 1.95 toolchain; passed with 2 tests.
- `scripts/ai/verify_full.sh`: re-ran with the same Rust 1.95 toolchain; passed
  and ended with `== verify_full complete ==`.
- Final JSON validation, agentflow role validation, and `git diff --check`:
  passed.

## Tests Added Or Updated

Added:

- `crates/adapters/betfair/tests/fixture_manifest.rs`
- `crates/adapters/architect_ax/tests/fixture_manifest.rs`

The tests require each manifest to:

- declare the RADP-020 task and RADP-019 inventory dependency;
- list expected adapter fixture surfaces;
- reference only existing fixture files;
- reference only existing primary parser/test files;
- include every RADP-019 blocker for the adapter;
- leave no blocker with `open` status;
- document scoped/deferred boundary decisions for RADP-021.

## Behavior Impact

No runtime behavior, adapter behavior, public API, trading semantics, Python,
PyO3, Cython, or Cargo feature behavior changed. The change adds machine
checkable fixture evidence and tests for existing adapter fixture coverage.

## Public API Impact

None.

## Migration Note Status

No migration note required because there is no public API or runtime behavior
change.

## Rollback Plan

Revert the two fixture manifest files, the two manifest test files, this
evidence file, and the RADP-020 agentflow state and lease updates.
