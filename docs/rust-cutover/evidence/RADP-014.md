# RADP-014 Evidence - Add Rust Adapter Fixtures For Databento Tardis

Date: 2026-05-31
Executor: Codex
Task ID: RADP-014
Risk: medium

## Summary

Added compact Rust fixture coverage manifests and executable manifest tests for
the Databento and Tardis adapters. These manifests map existing parser,
market-data, lifecycle, and scoped blocker coverage into release-gate evidence
for RADP-015.

The task adds fixture metadata and tests only. It does not change adapter
runtime behavior, market-data decoding, data-provider protocol handling,
credential handling, public APIs, Python/PyO3 bindings, Cython surfaces, or
Cargo feature behavior.

## Files Changed

- `crates/adapters/databento/test_data/rust_fixture_manifest.json`
- `crates/adapters/databento/tests/fixture_manifest.rs`
- `crates/adapters/tardis/test_data/rust_fixture_manifest.json`
- `crates/adapters/tardis/tests/fixture_manifest.rs`
- `docs/rust-cutover/evidence/RADP-014.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RADP-014.json`

## Commands Run

Context and task protocol:

```bash
sed -n '1,220p' docs/rust-cutover/tasks/RADP-014.md
sed -n '1,220p' docs/rust-cutover/TASK_EXECUTION.md
sed -n '1,260p' docs/rust-cutover/inventory/databento_tardis_adapter_gaps.md
find crates/adapters/databento/src crates/adapters/databento/tests crates/adapters/tardis/src crates/adapters/tardis/tests -maxdepth 3 -type f | sort
find crates/adapters/databento/test_data crates/adapters/tardis/test_data -type f | sort
```

Targeted validation:

```bash
python3 -m json.tool crates/adapters/databento/test_data/rust_fixture_manifest.json >/dev/null
python3 -m json.tool crates/adapters/tardis/test_data/rust_fixture_manifest.json >/dev/null
cargo fmt --check
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-databento --test fixture_manifest
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-tardis --test fixture_manifest
```

Required and final validation:

```bash
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_full.sh
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RADP-014.json >/dev/null
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
```

## Command Results

- Manifest JSON validation: passed for Databento and Tardis.
- `cargo fmt --check`: passed.
- Initial targeted cargo tests with default `rustc 1.87.0`: failed before compiling this change because the workspace requires Rust 1.95.0.
- `cargo test -p nautilus-databento --test fixture_manifest` with Rust 1.95.0: passed; 1 test passed.
- `cargo test -p nautilus-tardis --test fixture_manifest` with Rust 1.95.0: passed; 1 test passed.
- `scripts/ai/verify_full.sh`: passed; completed with `== verify_full complete ==`.
- Final JSON validation for task state, lease, and fixture manifests: passed.
- `python3 scripts/ai/validate_agentflow_roles.py`: passed.
- `git diff --check`: passed.

## Tests Added Or Updated

Added Databento and Tardis fixture manifest tests:

- Databento manifest test validates manifest schema, required fixture surfaces,
  fixture path existence, primary parser/test path existence, and all
  RADP-013 Databento blocker IDs.
- Tardis manifest test validates manifest schema, required fixture surfaces,
  fixture path existence, primary parser/test path existence, and all
  RADP-013 Tardis blocker IDs.

## Behavior Impact

No runtime behavior changed. No Databento DBN decoder, live feed handler,
historical loader, Tardis Machine replay/stream parser, HTTP bootstrap, CSV
stream, Parquet writer, credential, public API, Python API, PyO3 binding,
Cython surface, Cargo feature, or persistence behavior changed.

The practical impact is release-gate traceability: Databento and Tardis fixture
coverage is now machine-checkable and can be used by RADP-015 to close, scope,
or defer the recorded gaps.

## Public API Impact

No public API change.

## Migration Note Status

No migration note is required because this PR adds fixture manifests and tests
without changing runtime behavior or public APIs.

## Gate Status

RADP-014 is medium risk. It adds adapter fixture metadata and manifest tests;
it does not change adapter runtime behavior or trading semantics.

Auto-merge is allowed after local validation and required PR checks pass.

## Rollback Plan

Revert the two manifest files, the two manifest test files, this evidence file,
and the RADP-014 task state/lease updates. No runtime, persisted data, adapter
protocol, schema, Python, PyO3, Cython, Cargo feature, or public API rollback is
required.
