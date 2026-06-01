# RADP-023 Evidence - Add Rust Adapter Fixtures For Polymarket Sandbox

Date: 2026-06-01
Executor: Codex
Task ID: RADP-023
Risk: medium

## Summary

Added machine-checkable Rust fixture manifests for the Polymarket and Sandbox
adapter scope recorded in RADP-022.

Polymarket now has a crate-local manifest grouping existing Gamma, CLOB, Data
API, WebSocket market-data, WebSocket user-order, execution order lifecycle,
auth signing, fee, and precision fixtures. Sandbox now has a crate-local
manifest plus an internal lifecycle descriptor that explicitly records why
Sandbox has executable Rust lifecycle fixtures rather than external venue
REST/WebSocket payload fixtures.

No adapter runtime behavior changed. This task only adds fixture inventory,
manifest validation tests, and evidence for RADP-024.

## Files Changed

- `crates/adapters/polymarket/test_data/rust_fixture_manifest.json`
- `crates/adapters/polymarket/tests/fixture_manifest.rs`
- `crates/adapters/sandbox/Cargo.toml`
- `crates/adapters/sandbox/test_data/rust_fixture_manifest.json`
- `crates/adapters/sandbox/test_data/sandbox_lifecycle_fixture.json`
- `crates/adapters/sandbox/tests/fixture_manifest.rs`
- `Cargo.lock`
- `docs/rust-cutover/evidence/RADP-023.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RADP-023.json`

## Commands Run

```bash
sed -n '1,220p' docs/rust-cutover/tasks/RADP-023.md
find crates/adapters/polymarket/test_data -maxdepth 2 -type f | sort
find crates/adapters/sandbox -maxdepth 3 -type f | sort
python3 -m json.tool crates/adapters/polymarket/test_data/rust_fixture_manifest.json >/dev/null
python3 -m json.tool crates/adapters/sandbox/test_data/rust_fixture_manifest.json >/dev/null
python3 -m json.tool crates/adapters/sandbox/test_data/sandbox_lifecycle_fixture.json >/dev/null
git diff --check
cargo fmt --check
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-polymarket --test fixture_manifest
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-sandbox --test fixture_manifest
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 scripts/ai/verify_full.sh
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RADP-023.json >/dev/null
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
```

## Command Results

- Polymarket fixture manifest JSON validation: passed.
- Sandbox fixture manifest JSON validation: passed.
- Sandbox lifecycle descriptor JSON validation: passed.
- `git diff --check`: passed before and after evidence/state updates.
- `cargo fmt --check`: passed.
- `cargo test -p nautilus-polymarket --test fixture_manifest`: passed, 2 tests.
- `cargo test -p nautilus-sandbox --test fixture_manifest`: passed, 3 tests.
- The first full validation attempt was not used as final evidence after a
  sandbox clippy resource stall. The final run used `CARGO_BUILD_JOBS=1` and
  `CARGO_INCREMENTAL=0` and passed with `== verify_full complete ==`.
- Final JSON validation and agentflow role validation: passed.

## Tests Added Or Updated

Added:

- `crates/adapters/polymarket/tests/fixture_manifest.rs`
- `crates/adapters/sandbox/tests/fixture_manifest.rs`

The new tests require:

- manifest schema, task ID, inventory task, and adapter identity to match
  RADP-023 and RADP-022;
- every fixture group to resolve to existing crate-local paths;
- expected primary tests to be present;
- Polymarket scoped blockers `PM-ADP-001` through `PM-ADP-008` to remain owned
  by RADP-024;
- Sandbox scoped blockers `SBX-ADP-001` through `SBX-ADP-005` to remain owned
  by RADP-024;
- Sandbox fixture scope to explicitly declare no external payload requirement.

## Behavior Impact

No runtime behavior changed. No Polymarket or Sandbox market-data parsing,
order execution, signing, fee calculation, precision handling, matching,
account handling, reconciliation, public API, Python API, PyO3 binding, Cython
surface, or Cargo feature behavior changed.

Sandbox adds `serde_json` as a dev-only dependency so its manifest test can read
JSON fixtures. `Cargo.lock` only records that existing workspace dependency for
the sandbox test target.

The practical impact is release-gate clarity: existing Polymarket fixtures and
Sandbox lifecycle evidence are now grouped and checked by Rust tests, while
remaining adapter closure items are explicitly scoped to RADP-024.

## Public API Impact

No public API change.

## Migration Note Status

No migration note is required because this PR adds fixture manifests and tests
without changing runtime behavior or public APIs.

## Gate Status

RADP-023 is medium risk. It adds adapter fixture evidence and test coverage
only. Auto-merge is allowed after local validation and required PR checks pass.

## Rollback Plan

Revert the Polymarket and Sandbox fixture manifest files, the Sandbox lifecycle
descriptor, the related manifest tests, the sandbox dev-dependency lockfile
entry, this evidence file, and the RADP-023 agentflow state and lease updates.
No runtime, persisted data, adapter protocol, matching behavior, schema,
Python, PyO3, Cython, Cargo feature, or public API rollback is required.
