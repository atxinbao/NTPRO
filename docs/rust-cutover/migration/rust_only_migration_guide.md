# RREL-001 Rust-Only Migration Guide

Date: 2026-06-01
Executor: Codex
Task ID: RREL-001

Updated: 2026-06-04
Executor: Codex
Follow-up ID: RC-CLEANUP-001

Updated: 2026-06-04
Executor: Codex
Follow-up ID: RC2-PRERELEASE-DOCS

Updated: 2026-06-04
Executor: Codex
Follow-up ID: FORMAL-RELEASE-V0.1.0

## Scope

This guide describes the Rust-only migration path for NTPRO after the
Rust-only release gate passed and `ntpro-rust-only-v0.1.0` was prepared as the
first formal GitHub Release.

## Current Release Status

The Rust-only cutover is published as a formal GitHub Release.

Current state:

- RREL-009 made `scripts/ai/verify_release.sh` pass.
- RREL-008 recorded human owner approval for Rust-only completion.
- `ntpro-rust-only-rc.3` records the final pre-release candidate after RC2
  documentation correction.
- `ntpro-rust-only-v0.1.0` records the first formal Rust-only GitHub Release.
- Top-level legacy Python tests under `tests/**/*.py` were removed during RC
  public-surface cleanup.

Older RREM-009/RREM-010 blocker documents remain in the repository as
historical snapshots. They are superseded by the later RREM cleanup tasks,
RREL-009 release verification, and RREL-008 owner completion approval.

## Supported Rust-First Workflows

The documented Rust-only user path is:

1. Use the Rust CLI and Cargo workspace as the primary product entry.
2. Use Rust examples, Rust docs, and release evidence as the migration source
   of truth.
3. Use local gate scripts to verify whether Rust-only criteria are satisfied.
4. Treat Python/PyO3/Cython paths as unsupported product surfaces.

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
| Python package imports under `python/` and `nautilus_trader/` | Rust CLI/API/docs/examples | Removed from the Rust-only product surface. |
| PyO3 bridge under `crates/pyo3/` | Native Rust product surface | Removed from the Rust-only product surface. |
| Per-crate `src/python` bindings | Native Rust runtime/API access | Removed or scoped out by staged RREM cleanup. |
| Cython `.pyx` / `.pxd` implementation files | Rust implementation and tests | Removed from the Rust-only product surface. |
| Python/Cython build and packaging path | Rust workspace build/release path | Removed from the Rust-only release path. |
| Top-level Python tests under `tests/**/*.py` | Rust crate tests, golden traces, adapter fixture evidence, release gates | Removed from the public release surface by RC cleanup. |

## Active Release Gates

NTPRO is considered Rust-only only when these gates remain green:

- `verify_release.sh`, `check_rust_only_runtime.sh`, and
  `check_cython_removed.sh` pass without ignored product paths.
- Golden trace validation passes in standard and final release mode.
- The Rust CLI product surface remains available through Cargo.
- Later GitHub Release promotion remains owner-approved only.

## User Impact

Users should treat NTPRO as a Rust-only release candidate. Python package
installation, Python imports, PyO3 bindings, Cython builds, Python wheels, and
mixed Rust/Python packaging are not supported product paths.

## References

- `docs/rust-cutover/release/rust_only_release_notes.md`
- `docs/rust-cutover/release/final_release_verification.md`
- `docs/rust-cutover/release/final_completion_report.md`
- `docs/rust-cutover/migration/python_to_rust_workflow_map.md`
- `docs/rust-cutover/migration/python_test_scope_map.md`
- `docs/rust-cutover/evidence/RREL-009.md`
