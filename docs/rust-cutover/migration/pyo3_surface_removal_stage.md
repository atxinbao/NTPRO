# RREM-007 PyO3 Surface Removal Stage

Date: 2026-06-01
Executor: Codex
Task ID: RREM-007

Updated: 2026-06-04
Executor: Codex
Follow-up ID: RC-CLEANUP-001

## Current Status Update

This document is a historical RREM-007 staging snapshot. It is not the current
release state.

The PyO3 blocker state recorded here was superseded by later RREM cleanup
tasks, RREL-009 release verification, RREL-008 owner-approved completion, and
the tag-only `ntpro-rust-only-rc.1` release candidate.

For current release status, read:

- `docs/rust-cutover/release/rust_only_release_notes.md`
- `docs/rust-cutover/release/final_release_verification.md`
- `docs/rust-cutover/migration/rust_only_migration_guide.md`

## Scope

This document stages removal of PyO3 product surfaces. It does not delete,
move, skip, or weaken `crates/pyo3`, `crates/**/src/python*`, Python package
metadata, build scripts, Rust runtime code, tests, or documentation.

## Summary

PyO3 is still a broad product bridge. The dedicated `nautilus-pyo3` crate
aggregates Python modules, per-crate `src/python*` modules expose Rust crates
to Python, Cargo still declares PyO3 workspace dependencies, and Python build
metadata still points to `crates/pyo3`.

Current readiness: blocked. PyO3 removal can be staged, but hard deletion must
wait until Rust product usability, runtime parity, adapter parity, Cython
interop removal, packaging cleanup, and migration notes are complete.

## Current Surface Size

Current counts from this task run:

| Surface | Count | Evidence command |
| --- | ---: | --- |
| `crates/pyo3/` files | 4 | `find crates/pyo3 -type f` |
| `crates/**/src/python*` files | 371 | `find crates -path '*/src/python*' -type f` |
| `crates/**/src/python` directories | 60 | `find crates -path '*/src/python*' -type d` |
| Rust files/manifests mentioning PyO3 binding primitives | 775 | `rg -l 'pyo3|PyO3|wrap_pymodule|pymodule|pyclass|pymethods' crates ...` |

Largest `crates/**/src/python*` families:

| Family | File count |
| --- | ---: |
| `crates/model` | 114 |
| `crates/adapters` | 110 |
| `crates/indicators` | 44 |
| `crates/analysis` | 23 |
| `crates/common` | 15 |
| `crates/persistence` | 11 |
| `crates/core` | 10 |
| `crates/infrastructure` | 7 |
| `crates/execution` | 6 |
| `crates/backtest` | 6 |

## Removal Lanes

| Lane | Candidate paths | Current status | Removal decision |
| --- | --- | --- | --- |
| Aggregator crate | `crates/pyo3/**` | Blocked. It still registers Python modules and stub generation. | Remove only after per-crate bindings, package metadata, and import migration notes are ready. |
| Per-crate binding modules | `crates/**/src/python*` | Blocked. Model and adapter bindings dominate the surface. | Remove by crate family after Rust API/parity evidence is linked. |
| Cargo workspace metadata | `Cargo.toml`, `Cargo.lock` | Blocked. Workspace still includes `crates/pyo3` and PyO3 dependencies. | Remove in a dedicated Cargo restructuring PR after source lanes are ready. |
| Python package bridge | `python/pyproject.toml`, `python/generate_stubs.py` | Blocked. Maturin and stub generation still target PyO3. | Remove with Python package surface cleanup. |
| Build bridge | `build.py`, `debug-pyo3`, `cython-compat`, `extension-module` | Blocked. Still participates in mixed PyO3/Cython build flow. | Remove with Cython/build cleanup after RREM-008 gate is green. |
| Tests/docs/import references | `tests/**`, `docs/**`, `README.md` | Blocked. Import behavior remains documented and tested. | Rewrite or mark migration-only before final deletion. |

## Gate Sequence

Hard PyO3 removal must follow this order:

1. Rust API/product gate: supported user workflows are available through Rust
   CLI/API/docs/examples without Python imports.
2. Runtime parity gate: Rust tests or golden traces cover behavior currently
   asserted through PyO3 bindings.
3. Adapter gate: each adapter binding is supported, deferred, or removed with
   fixture/mock/dry-run evidence.
4. Cython interop gate: `cython-compat`, legacy Cython conversions, `.pyx`, and
   `.pxd` dependencies are gone or explicitly scoped.
5. Package gate: `python/pyproject.toml`, stub generation, and wheel build
   paths no longer require `crates/pyo3`.
6. Cargo gate: `crates/pyo3`, `nautilus-pyo3`, `pyo3`, `pyo3-async-runtimes`,
   `pyo3-stub-gen`, and `debug-pyo3` are removed from active product
   workspace/build paths.
7. Migration note gate: removed Python modules/imports and Rust replacement
   workflows are documented.
8. Final Rust-only gate: `scripts/ai/check_rust_only_runtime.sh` passes.

## Blockers

The following blockers prevent PyO3 deletion now:

- `crates/pyo3` still aggregates runtime, model, adapter, and utility modules.
- 371 per-crate Python binding files remain.
- Cargo workspace metadata still includes `crates/pyo3` and PyO3 dependencies.
- `python/pyproject.toml` still points maturin at `../crates/pyo3/Cargo.toml`.
- `build.py` still has `debug-pyo3`, `cython-compat`, `extension-module`, and
  dynamic-library copy behavior.
- Cython interop still exists, so removing PyO3 first would break the mixed
  bridge without a completed Cython removal gate.
- Public migration notes for removed Python imports are not final.

## Allowed Later Actions

Later RREM/RREL tasks may advance a PyO3 item only by recording one of:

- `ready_for_deletion`: all gates for the path are green;
- `ported_to_rust`: Rust API/tests/docs cover the behavior;
- `superseded_by_rust_product_contract`: the Python binding is intentionally
  replaced by a Rust-only workflow;
- `deferred_by_scope`: release gatekeeper approves deferral;
- `blocked`: deletion remains unsafe.

## Checklist Before Deleting PyO3 Paths

- [ ] The exact binding family is listed in RREM-002 or this staging document.
- [ ] Rust replacement API, CLI, docs, or tests are linked.
- [ ] Adapter binding removals reference RADP decisions.
- [ ] Cython interop is already removed or explicitly not affected.
- [ ] Cargo workspace/build metadata impact is reviewed.
- [ ] Python import migration notes are written.
- [ ] Release gatekeeper approval is recorded.

## Next Tasks

- RREM-008 should address Cython source/build removal gates.
- RREM-009 should run and record Rust-only runtime gate results.
- RREM-010 should make the final no-Python/PyO3/Cython product-surface
  decision from the accumulated evidence.
