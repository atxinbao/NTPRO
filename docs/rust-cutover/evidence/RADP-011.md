# RADP-011 Evidence - Add Rust Adapter Fixtures For Deribit dYdX Hyperliquid

Date: 2026-05-31
Executor: Codex
Task ID: RADP-011
Risk: medium

## Summary

Added Rust fixture manifests for the Deribit, dYdX, and Hyperliquid adapter
parity gates. Each manifest maps existing parser, market-data, account,
execution, WebSocket, gRPC, or operational-scope fixtures to the RADP-010 gap
inventory and records scoped blockers for the RADP-012 closure task.

Added one Rust manifest test per adapter crate. The tests make the manifests
machine-checkable by asserting that every listed fixture file exists, every
listed parser or test entry exists, required adapter surfaces are covered, and
the scoped blocker IDs point back to RADP-010 and forward to RADP-012.

No adapter runtime behavior, exchange protocol handling, order routing,
credential handling, public APIs, Python/PyO3 bindings, Cython surfaces, or
Cargo feature behavior changed.

## Files Changed

- `crates/adapters/deribit/test_data/rust_fixture_manifest.json`
- `crates/adapters/deribit/tests/fixture_manifest.rs`
- `crates/adapters/dydx/test_data/rust_fixture_manifest.json`
- `crates/adapters/dydx/tests/fixture_manifest.rs`
- `crates/adapters/hyperliquid/test_data/rust_fixture_manifest.json`
- `crates/adapters/hyperliquid/tests/fixture_manifest.rs`
- `docs/rust-cutover/evidence/RADP-011.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RADP-011.json`

## Commands Run

Context and task protocol:

```bash
sed -n '1,220p' docs/rust-cutover/tasks/RADP-011.md
sed -n '1,260p' docs/rust-cutover/TASK_EXECUTION.md
sed -n '1,220p' docs/rust-cutover/CONTRACT.md
sed -n '1,220p' docs/rust-cutover/DEFINITION_OF_DONE.md
sed -n '1,240p' docs/rust-cutover/AGENT_ROLES.md
```

Manifest validation:

```bash
python3 -m json.tool crates/adapters/deribit/test_data/rust_fixture_manifest.json >/tmp/deribit_manifest_check.json
python3 -m json.tool crates/adapters/dydx/test_data/rust_fixture_manifest.json >/tmp/dydx_manifest_check.json
python3 -m json.tool crates/adapters/hyperliquid/test_data/rust_fixture_manifest.json >/tmp/hyperliquid_manifest_check.json
cargo fmt --check
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-deribit --test fixture_manifest
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-dydx --test fixture_manifest
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-hyperliquid --test fixture_manifest
```

Required and final validation:

```bash
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_full.sh
```

## Command Results

- Manifest JSON validation: passed for Deribit, dYdX, and Hyperliquid.
- `cargo fmt --check`: passed.
- `cargo test -p nautilus-deribit --test fixture_manifest`: passed; 1 test passed.
- `cargo test -p nautilus-dydx --test fixture_manifest`: passed; 1 test passed.
- `cargo test -p nautilus-hyperliquid --test fixture_manifest`: passed; 1 test passed.
- `scripts/ai/verify_full.sh`: passed; output ended with `== verify_full complete ==`.

## Tests Added Or Updated

- Added `crates/adapters/deribit/tests/fixture_manifest.rs`.
- Added `crates/adapters/dydx/tests/fixture_manifest.rs`.
- Added `crates/adapters/hyperliquid/tests/fixture_manifest.rs`.

The tests validate the new fixture manifests only. They do not add new parser
semantics, trading behavior, network calls, or live exchange access.

## Behavior Impact

No runtime behavior changed. No parser, data-client, execution-client,
WebSocket, gRPC, HTTP, transaction-broadcast, signing, outcome-settlement,
credential, order routing, public API, Python API, PyO3 binding, Cython
surface, Cargo feature, or persistence behavior changed.

The practical impact is evidence quality: RADP-012 can now close or explicitly
keep Deribit, dYdX, and Hyperliquid adapter gaps against checked fixture
manifests instead of a manual inventory only.

## Public API Impact

No public API change.

## Migration Note Status

No migration note is required because this PR adds test fixtures and evidence
without changing runtime behavior or public APIs.

## Gate Status

RADP-011 is medium risk. It adds adapter fixture manifests and tests, but does
not change adapter runtime behavior or trading semantics.

Auto-merge is allowed after local validation and required PR checks pass.

## Rollback Plan

Revert the three manifest files, the three manifest tests, this evidence file,
and the RADP-011 task state/lease updates. No runtime, persisted data, adapter
protocol, schema, Python, PyO3, Cython, or public API rollback is required.
