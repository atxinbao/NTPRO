# RREM-008 Cython Removal Blockers

Date: 2026-06-01
Executor: Codex
Task ID: RREM-008

Updated: 2026-06-04
Executor: Codex
Follow-up ID: RC-CLEANUP-001

## Current Status Update

This document is a historical RREM-008 blocker snapshot. It is not the current
release state.

The Cython blocker state recorded here was superseded by later RREM cleanup
tasks, RREL-009 release verification, RREL-008 owner-approved completion, and
the current tag-only `ntpro-rust-only-rc.2` release candidate.

For current release status, read:

- `docs/rust-cutover/release/rust_only_release_notes.md`
- `docs/rust-cutover/release/final_release_verification.md`
- `docs/rust-cutover/migration/rust_only_migration_guide.md`

## Scope

This document records the Cython source/build removal gate result. It does not
delete, move, skip, or weaken `.pyx`, `.pxd`, `build.py`, `pyproject.toml`,
tests, docs, Rust runtime code, or build metadata.

## Summary

RREM-008 is not ready for hard deletion. The task allows Cython source and
build removal only after parity gates are satisfied. Current evidence shows
that the no-Cython runtime gate fails immediately because Cython source and
interface files remain, and active build/runtime paths still reference Cython.

Historical RREM-008 decision: blocked. No Cython files were removed in that PR.

## Gate Result

`scripts/ai/check_no_cython_runtime.sh` failed as expected for the current
repository state.

Primary blockers:

| Blocker | Evidence |
| --- | --- |
| Retained Cython source/interface files | 243 `.pyx` / `.pxd` files remain outside ignored build/cache paths. |
| Top-level Cython package remains | Files remain under `nautilus_trader/**`. |
| Active Cython build script remains | `build.py` imports `Cython.Build`, calls `cythonize`, discovers `nautilus_trader/**/*.pyx`, and prints Cython compiler version. |
| Active test/coverage config remains | `pyproject.toml` still has doctest glob for `*.pyx` and `Cython.Coverage`. |
| Rust build metadata still generates Cython headers | `crates/*/build.rs` and `cbindgen_cython.toml` still reference Cython definitions. |
| PyO3/Cython interop remains | `crates/pyo3` and `cython-compat` remain staged but not deleted. |

## Current Cython Surface Size

Current counts from this task run:

| Surface | Count |
| --- | ---: |
| retained `.pyx` / `.pxd` files | 243 |
| `nautilus_trader/model` Cython files | 95 |
| `nautilus_trader/core` Cython files | 24 |
| `nautilus_trader/indicators` Cython files | 18 |
| `nautilus_trader/accounting` Cython files | 18 |
| `nautilus_trader/execution` Cython files | 17 |
| `nautilus_trader/backtest` Cython files | 16 |
| `nautilus_trader/common` Cython files | 13 |
| `nautilus_trader/data` Cython files | 9 |
| `nautilus_trader/cache` Cython files | 9 |

## Required Before Deletion

Before a later deletion PR removes Cython source/build paths, these gates must
be green or explicitly deferred by the release gatekeeper:

1. Runtime parity for model, accounting, backtest, cache, common, data,
   execution, indicators, portfolio, risk, serialization, persistence, and
   trading behavior currently represented in Cython.
2. PyO3/Cython interop removal or replacement, including `cython-compat`.
3. Build cleanup plan for `build.py`, root `pyproject.toml`, Cython doctest
   glob, coverage plugin, generated Cython headers, and `cbindgen_cython.toml`.
4. Python test scope decisions from RREM-005 applied to Cython/PyO3 interop
   tests.
5. Migration notes for removed extension modules and replacement Rust
   workflows.
6. `scripts/ai/check_no_cython_runtime.sh` passes after the actual deletion
   PR.

## Allowed Later Outcomes

Later RREM/RREL tasks may classify each Cython family as:

- `ready_for_deletion`: all required gates are green;
- `ported_to_rust`: Rust tests/golden traces cover the behavior;
- `superseded_by_rust_product_contract`: the Cython workflow is intentionally
  replaced by a Rust-only workflow;
- `deferred_by_scope`: release gatekeeper approves deferral;
- `blocked`: deletion remains unsafe.

## Next Tasks

- RREM-009 should run the Rust-only runtime gate and record remaining blockers.
- RREM-010 should use RREM-008 and RREM-009 evidence for the final removal
  gate decision.
