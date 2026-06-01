# RREL-001 Rust-Only Migration Guide

Date: 2026-06-01
Executor: Codex
Task ID: RREL-001

## Scope

This guide describes the intended Rust-only migration path for NTPRO and the
current release blockers. It is a migration guide and evidence index only. It
does not delete Python, PyO3, Cython, build, packaging, adapter, runtime, or
public API files.

## Current Release Status

The Rust-only release is not ready from the current repository state.

`RREM-010` recorded the final removal gate as blocked because Python, PyO3,
Cython, build, runtime/API, and product surfaces still remain. This guide must
therefore be read as the target migration plan, not as a declaration that the
cutover is complete.

## Supported Rust-First Workflows

The documented Rust-first user path is:

1. Use the Rust CLI and Cargo workspace as the primary product entry.
2. Use Rust examples, Rust docs, and release evidence as the migration source
   of truth.
3. Use local gate scripts to verify whether Rust-only criteria are satisfied.
4. Treat Python/PyO3/Cython paths as legacy or pending-removal surfaces until
   destructive removal tasks provide green evidence.

Expected validation entry points:

- `scripts/ai/verify_fast.sh` for fast repository checks.
- `scripts/ai/verify_full.sh` for broader workspace validation.
- `scripts/ai/verify_release.sh` for release gate validation.
- `scripts/ai/check_rust_only_runtime.sh` for residual runtime/product surface
  checks.
- `scripts/ai/check_cython_removed.sh` for Cython removal checks.

## Migration Map

| Old workflow | Rust-only target | Current status |
| --- | --- | --- |
| Python package imports under `python/` and `nautilus_trader/` | Rust CLI/API/docs/examples | Pending. Source paths still exist. |
| PyO3 bridge under `crates/pyo3/` | Native Rust product surface | Pending. Aggregator crate still exists. |
| Per-crate `src/python` bindings | Native Rust runtime/API access | Pending. `crates/**/src/python` directories remain. |
| Cython `.pyx` / `.pxd` implementation files | Rust implementation and tests | Pending. Cython files remain. |
| Python/Cython build and packaging path | Rust workspace build/release path | Pending. Build metadata still references legacy surfaces. |

## Required Removal Gates Before Rust-Only Release

Before NTPRO can be marked Rust-only, the following must be true:

- Python product/import surfaces are removed or explicitly archived as
  non-product compatibility material.
- `nautilus_trader/` runtime/API surfaces have Rust replacements or documented
  removal decisions.
- `crates/pyo3/` and per-crate Python binding directories are removed after
  replacement evidence is available.
- Cython source/interface files and active build references are removed.
- Migration notes cover removed imports, examples, install commands, and build
  workflows.
- `verify_release.sh`, `check_rust_only_runtime.sh`, and
  `check_cython_removed.sh` pass without ignored product paths.

## User Impact

No user-facing runtime behavior changes in this guide. It only documents the
expected migration direction and the current blockers.

Users should not treat the current repository as a completed Rust-only release.
The repository remains in Rust-first cutover mode until the final release gate
passes and the owner signoff is recorded.

## References

- `docs/rust-cutover/migration/final_rust_only_removal_gate.md`
- `docs/rust-cutover/migration/rust_only_runtime_gate_blockers.md`
- `docs/rust-cutover/migration/cython_removal_blockers.md`
- `docs/rust-cutover/migration/python_to_rust_workflow_map.md`
- `docs/rust-cutover/evidence/RREM-010.md`
