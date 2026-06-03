# RREM-009 Rust-Only Runtime Gate Blockers

Date: 2026-06-01
Executor: Codex
Task ID: RREM-009

Updated: 2026-06-04
Executor: Codex
Follow-up ID: RC-CLEANUP-001

## Current Status Update

This document is a historical RREM-009 blocker snapshot. It is not the current
release state.

The blocker state recorded here was superseded by later RREM cleanup tasks,
RREL-009 release verification, RREL-008 owner-approved completion, and the
tag-only `ntpro-rust-only-rc.1` release candidate.

For current release status, read:

- `docs/rust-cutover/release/rust_only_release_notes.md`
- `docs/rust-cutover/release/final_release_verification.md`
- `docs/rust-cutover/release/final_completion_report.md`
- `docs/rust-cutover/migration/rust_only_migration_guide.md`

## Scope

This document records the Rust-only runtime gate result. It does not delete,
move, skip, or weaken Python, PyO3, Cython, build, runtime, adapter, test, or
documentation files.

## Summary

The Rust-only runtime gate is not green. `scripts/ai/check_rust_only_runtime.sh`
fails because Python, PyO3, Cython, and build/runtime product surfaces remain
in active paths. `scripts/ai/check_cython_removed.sh` also fails because
`.pyx` and `.pxd` files remain. `scripts/ai/verify_full.sh` was run with the
project Rust 1.95.0 toolchain and timed out at the bounded 180 second limit
during workspace clippy.

Historical RREM-009 decision: blocked. This task records gate evidence only.

## Gate Results

| Command | Result | Summary |
| --- | --- | --- |
| `scripts/ai/verify_full.sh` with default environment | Failed | Default rustc was below the workspace requirement. |
| `scripts/ai/verify_full.sh` with Rust 1.95.0 | Timed out after 180 seconds | Reached workspace clippy and was still checking crates. |
| `scripts/ai/check_rust_only_runtime.sh` | Failed | Python/PyO3/Cython product paths and active references remain. |
| `scripts/ai/check_cython_removed.sh` | Failed | Cython `.pyx` / `.pxd` files remain. |

## Current Blocker Counts

| Blocker | Count |
| --- | ---: |
| retained product paths among `python`, `nautilus_trader`, `crates/pyo3`, `build.py` | 4 |
| retained `crates/**/src/python` directories | 36 |
| retained `.pyx` / `.pxd` files | 243 |
| active files matching Python/PyO3/Cython build/runtime reference patterns | 839 |

## Required Before Gate Can Pass

- Remove or replace `python/` as a product surface.
- Remove or replace top-level `nautilus_trader/` product runtime/API surface.
- Remove `crates/pyo3` and per-crate `src/python` modules after replacement
  evidence is complete.
- Remove Cython `.pyx` / `.pxd` source and generated-header build paths after
  parity gates are complete.
- Remove or rewrite active `pyo3`, `maturin`, `Cython`, `cythonize`, `.pyx`,
  and `.pxd` references in product build/runtime paths.
- Make `verify_full.sh` complete under the pinned Rust toolchain.
- Re-run `scripts/ai/check_rust_only_runtime.sh` and
  `scripts/ai/check_cython_removed.sh` after deletion/replacement PRs.

## Next Task Input

RREM-010 should use this evidence as the final removal gate input. Based on
current results, the final gate cannot truthfully pass yet.
