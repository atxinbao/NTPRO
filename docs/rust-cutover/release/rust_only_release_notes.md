# RREL-002 Rust-Only Release Notes

Date: 2026-06-01
Executor: Codex
Task ID: RREL-002

## Release State

This is a draft Rust-only release note package for the current cutover state.
It must not be published as a completed Rust-only release note until the final
release verification gate passes.

Current decision: release blocked.

## What Changed In This Release Track

- Rust-first product, runtime, adapter, trace, removal, and release evidence has
  been collected under `docs/rust-cutover/evidence/`.
- Migration notes now describe the intended Rust-only path and the remaining
  Python/PyO3/Cython blockers.
- Final removal gate evidence exists in `RREM-010` and records that the
  repository is not yet Rust-only.

## Breaking Change Plan

The following breaking changes are planned for a future Rust-only candidate,
but they are not complete in the current repository state:

- Remove Python product package/import surfaces after Rust replacements and
  migration notes are complete.
- Remove PyO3 aggregator and per-crate binding surfaces after native Rust
  product workflows cover the required paths.
- Remove Cython source/interface files and active Cython build references after
  parity evidence is green.
- Remove Python/Cython packaging assumptions from build and release workflows
  after Rust-only verification passes.

## Replacement Workflows

| Legacy workflow | Replacement workflow | Current note |
| --- | --- | --- |
| Python package usage | Rust CLI/API/docs/examples | Target documented, not fully cut over. |
| PyO3 bridge usage | Native Rust product/runtime access | Pending removal evidence. |
| Cython implementation/build path | Rust workspace build and tests | Blocked by remaining `.pyx` / `.pxd` files. |
| Mixed Python release packaging | Rust-only release gate | Blocked by final verification failures. |

## Known Blockers

- `python/`, `nautilus_trader/`, `crates/pyo3/`, and `build.py` still exist.
- `crates/**/src/python` binding directories remain.
- Cython `.pyx` and `.pxd` files remain.
- `verify_full.sh` did not complete in the RREM-010 final-gate run.
- `check_rust_only_runtime.sh` and `check_cython_removed.sh` failed in the
  final-gate evidence.

## Validation Summary

The release note package itself is documentation-only and does not change
runtime behavior. Release validation remains blocked until the RREL-006
verification evidence proves otherwise.

## Release Recommendation

Do not tag or publish a Rust-only release from the current state.

The recommended next step is to finish the remaining release documentation and
run RREL-006 as blocker evidence, then prepare an owner signoff packet that
clearly states that final owner signoff is still pending.
