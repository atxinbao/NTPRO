# RREM-012 Evidence - Remove PyO3 Workspace And Python Binding Surfaces

Date: 2026-06-02
Executor: Codex
Task ID: RREM-012
Risk: critical

## Summary

Removed the Rust PyO3 bridge from the active workspace path. This deletes the
`crates/pyo3` workspace crate, removes all `crates/**/src/python` binding
modules, drops PyO3/maturin/extension-module Cargo and build references, and
updates fast/full/release verification scripts so they no longer select the
deleted PyO3 crate.

This is a critical removal task and must stop at REVIEW_REQUIRED. Auto-merge is
not allowed.

## Files Changed

- Deleted `crates/pyo3/**`.
- Deleted `crates/**/src/python/**`.
- Updated root `Cargo.toml` and `Cargo.lock` to remove `nautilus-pyo3`, PyO3,
  PyO3 async runtime, PyO3 stub generation, and the debug PyO3 profile.
- Updated per-crate `Cargo.toml` files to remove Python binding features and
  PyO3 dependency wiring.
- Updated `build.py`, `Makefile`, `python/pyproject.toml`,
  `.github/workflows/build-v2.yml`, and verification scripts to stop invoking
  maturin/PyO3 extension builds.
- Updated crate README/lib docs where they advertised removed Python/PyO3
  feature flags.
- Added local non-Python `NautilusDataType` enums for persistence conversion
  binaries that previously imported the enum from the removed Python session
  module.
- Added `RecordFlag::value()` for Rust callers that previously depended on the
  Python-side helper path.
- Updated task metadata and lease records for RREM-012.

## Commands Run

```bash
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" \
RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" \
VERIFY_FAST_CARGO_CHECK=1 scripts/ai/verify_fast.sh

PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" \
RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" \
cargo fmt --check

PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" \
RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" \
cargo metadata --format-version=1

git diff --check
python3 -m json.tool .agentflow/state/task_status.json
python3 -m json.tool .agentflow/leases/RREM-012.json
python3 scripts/ai/validate_agentflow_roles.py
scripts/ai/check_rust_only_runtime.sh
```

## Command Results

- `scripts/ai/verify_fast.sh`: passed. Cargo check completed in 6m42s. The
  run emitted `unexpected cfg condition value: python` warnings from residual
  inactive `#[cfg(feature = "python")]` / PyO3 annotations, but no compile
  error remained.
- `cargo fmt --check`: passed.
- `cargo metadata --format-version=1`: passed.
- Metadata package scan: no package name containing `pyo3` remains.
- `git diff --check`: passed.
- JSON validation for `.agentflow/state/task_status.json` and
  `.agentflow/leases/RREM-012.json`: passed.
- `python3 scripts/ai/validate_agentflow_roles.py`: passed.
- `scripts/ai/check_rust_only_runtime.sh`: failed as expected for this phase.
  Remaining blockers include retained Python product paths (`python/`,
  `nautilus_trader/`, `build.py`), Cython references documented from previous
  release gates, and residual inactive PyO3 source annotations outside the
  deleted `src/python` binding modules.

## Residual Python Product Blockers

The `python/`, `nautilus_trader/`, and `build.py` product surfaces are handled
by RREM-013 and later release gates unless this task explicitly removes only
PyO3-specific build references from those files.

Additional residual cleanup before final Rust-only release:

- Remove or rewrite inactive `#[cfg(feature = "python")]` / PyO3 annotations
  still present in active Rust source files.
- Remove remaining Python product package directories in RREM-013.
- Re-run `scripts/ai/check_rust_only_runtime.sh` after RREM-013 and the
  residual annotation cleanup.

## Behavior Impact

Rust workspace metadata and fast cargo check no longer depend on PyO3. The Rust
runtime path still compiles. Python extension-module builds through PyO3 are no
longer supported by the active workspace after this change.

## Public API Impact

This removes the Rust PyO3 binding bridge from the active product surface. Any
consumer relying on `nautilus-pyo3`, per-crate Rust `src/python` modules, or
maturin extension-module builds must migrate to the Rust-first runtime path.

## Migration Note Status

Migration notes remain incomplete until RREM-013 removes or retires the Python
package product surface. This PR records the PyO3 removal evidence and leaves
final Rust-only release signoff blocked.

## Rollback Plan

Revert this PR to restore the PyO3 bridge crate, per-crate Rust Python binding
modules, and removed build metadata.
