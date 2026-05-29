# RCORE-005 Evidence - Add Rust Tests for Serialization/Data/Persistence

Date: 2026-05-29
Executor: Codex
Task ID: RCORE-005
Risk: high

## Summary

Added focused Rust regression tests for serialization/data/persistence fixed
precision boundaries.

The serialization test covers Arrow fixed-size binary precision byte validation
for the current build mode and rejects the other precision-mode width. The
persistence test writes `QuoteTick` values through `ParquetDataCatalog`, reads
them back through both generic and typed catalog query paths, and verifies that
price and quantity values plus precision metadata are preserved.

RCORE-005 is high risk by role policy because serialization, market data, and
persistence define the stored and replayed trading data path. This PR must stop
at `REVIEW_REQUIRED`, must not enable auto-merge, and requires Verification &
Release Gatekeeper review before merge.

## Files Changed

- `crates/serialization/src/arrow/mod.rs`
- `crates/persistence/tests/test_catalog.rs`
- `docs/rust-cutover/evidence/RCORE-005.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RCORE-005.json`

## Commands Run

Initial dispatch command:

```bash
python3 scripts/control/dispatch_next.py --max-risk high
```

Manual dispatch fallback, after confirming local `main` matched GitHub `main`:

```bash
git switch -c ai/RCORE-005-add-rust-tests-for-serialization-data-persistence
python3 scripts/ai/lease.py claim RCORE-005 --branch ai/RCORE-005-add-rust-tests-for-serialization-data-persistence --agent-id Codex --path docs/rust-cutover/tasks/RCORE-005.md --path docs/rust-cutover/evidence/RCORE-005.md --path .agentflow/state/task_status.json --path .agentflow/leases/RCORE-005.json
```

Focused validation commands:

```bash
rustup run 1.95.0-aarch64-apple-darwin cargo fmt --check
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc rustup run 1.95.0-aarch64-apple-darwin cargo test -p nautilus-serialization --features arrow test_validate_precision_bytes --lib
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc rustup run 1.95.0-aarch64-apple-darwin cargo test -p nautilus-persistence --test test_catalog test_rust_quote_tick_catalog_roundtrip_preserves_fixed_precision
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_full.sh
```

## Command Results

- `dispatch_next.py --max-risk high`: failed during `git fetch --prune origin`
  because the local machine timed out while reaching GitHub over HTTPS. Local
  `main` was then compared with GitHub API branch metadata and both pointed to
  commit `e0ce83581d26f74b4e3dfb61ebf4d70a62973a42`, so the branch, lease, and
  task state were created with equivalent local steps.
- `cargo fmt --check`: passed.
- `cargo test -p nautilus-serialization --features arrow test_validate_precision_bytes --lib`:
  passed, 2 tests.
- `cargo test -p nautilus-persistence --test test_catalog test_rust_quote_tick_catalog_roundtrip_preserves_fixed_precision`:
  passed, 1 test.
- `scripts/ai/verify_full.sh`: passed; fast checks, clippy, workspace Rust
  tests, golden trace validation, and rustdoc completed.

The default shell `cargo` resolved to Homebrew Rust 1.87.0, but this workspace
requires Rust 1.95.0. Validation therefore used the installed rustup toolchain
`1.95.0-aarch64-apple-darwin` with `RUSTC` pinned to the matching compiler.

## Tests Added or Updated

- Added `test_validate_precision_bytes_accepts_current_build_width` in
  `crates/serialization/src/arrow/mod.rs`.
- Added `test_validate_precision_bytes_rejects_other_precision_mode_width` in
  `crates/serialization/src/arrow/mod.rs`.
- Added `test_rust_quote_tick_catalog_roundtrip_preserves_fixed_precision` in
  `crates/persistence/tests/test_catalog.rs`.

## Behavior Impact

No runtime behavior changed. No trading semantics changed. No adapter behavior
changed. No data format, schema, persistence writer, persistence reader, Cargo
feature, Python, PyO3, Cython, or FFI file was deleted or moved.

## Public API Impact

None. The change adds tests only and does not alter function signatures,
public types, product commands, Python bindings, PyO3 surfaces, or Cython
surfaces.

## Migration Note Status

No migration note is required because no public API or persisted data format
changed.

## Rollback Plan

Revert the two Arrow precision validation unit tests, the `QuoteTick` catalog
roundtrip test, this evidence file, and the RCORE-005 `.agentflow` metadata
updates. No persisted data migration is required.
