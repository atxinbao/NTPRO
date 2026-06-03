# RREM-006 Python Package Surface Removal Stage

Date: 2026-06-01
Executor: Codex
Task ID: RREM-006

Updated: 2026-06-04
Executor: Codex
Follow-up ID: RC-CLEANUP-001

## Scope

This document originally staged removal of the Python package surfaces under
`python/` and top-level `nautilus_trader/`. It was a removal plan, not a
deletion PR.

After the Rust-only release gate passed, RREL-008 completion was merged, and
`ntpro-rust-only-rc.1` was created as a tag-only release candidate, RC cleanup
removed the top-level legacy Python tests under `tests/**/*.py`.

## Summary

The table below is the original RREM-006 staging snapshot. It is retained as
historical evidence. The current release surface no longer includes Python,
PyO3, Cython, or top-level Python test product paths.

RREM-006 therefore stages removal into explicit lanes:

| Lane | Candidate paths | Current status | Removal decision |
| --- | --- | --- | --- |
| Python overlay package | `python/**` | Blocked. The overlay still contains import stubs, package docs, tests, and maturin metadata. | Remove only after Rust CLI/API/docs are the supported product surface and PyO3 package removal is approved. |
| Top-level Python package | `nautilus_trader/**/*.py`, `nautilus_trader/**/*.pyi` | Blocked. It still hosts public Python modules, adapters, examples, test kit, and runtime wrappers. | Remove by domain after Rust replacements and migration notes exist. |
| Top-level Cython package | `nautilus_trader/**/*.pyx`, `nautilus_trader/**/*.pxd` | Blocked. RREM-003 records 243 Cython source/interface files. | Do not remove in the Python package lane; coordinate with Cython/build removal tasks. |
| Python package tests | historical `python/tests/**`, historical `tests/**/*.py` | Top-level `tests/**/*.py` removed by RC cleanup. Python package overlay tests were removed with the retired package surface in earlier removal tasks. | Removed with approved Rust-only surface deletion. |
| Python examples/docs | `examples/**/*.py`, `examples/**/*.ipynb`, Python-facing docs | Blocked. RREM-004 shows Rust help/runtime smokes but product run paths are incomplete. | Replace with Rust examples/docs or mark as legacy migration material before deletion. |
| Packaging/build bridge | `pyproject.toml`, `python/pyproject.toml`, `build.py` | Blocked. These are shared with PyO3/Cython build cleanup. | Leave to dedicated build-path and final release tasks. |

## Current Surface Size

Current counts from this task run:

| Surface | Count | Evidence command |
| --- | ---: | --- |
| `python/` files | 178 | `find python -type f` |
| `python/nautilus_trader` shallow module directories | 50 | `find python/nautilus_trader -maxdepth 2 -type d` |
| top-level `nautilus_trader/` Python/Cython/interface files | 642 | `find nautilus_trader -type f ...` |
| Python examples and notebooks | 137 | `find examples -type f ...` |
| top-level Python test files | 534 at RREM-006 time; 0 tracked after RC cleanup | `find tests -type f -name '*.py'` |
| README/docs Python-facing references | 77 | `rg -l 'from nautilus_trader|import nautilus_trader|pip install|Python 3|```python' docs README.md` |

The top-level `nautilus_trader/` package currently includes:

| Extension | Count |
| --- | ---: |
| `.py` | 395 |
| `.pxd` | 133 |
| `.pyx` | 110 |
| `.pyi` | 4 |

The `python/` overlay currently includes:

| Extension | Count |
| --- | ---: |
| `.py` | 123 |
| `.pyi` | 50 |
| `.md` | 2 |
| `.typed` | 1 |
| `.toml` | 1 |
| `.lock` | 1 |

## Gate Sequence

Hard removal must follow this sequence:

1. Rust product gate: CLI/API/docs/examples cover the supported backtest,
   sandbox/live, data, config, and adapter workflows without requiring Python.
2. Runtime parity gate: scoped order lifecycle, risk, execution, portfolio,
   model, cache/message-bus, backtest/live, and persistence behavior has Rust
   test or golden-trace evidence.
3. Adapter parity gate: each supported venue/data provider has fixture, mock,
   dry-run, schema, or sandbox evidence and a supported/deferred/removed
   decision.
4. Test scope gate: Python tests are ported, superseded, deferred, archived as
   migration-only, or removed with the approved surface. RC cleanup applies
   `removed_with_surface` to top-level `tests/**/*.py`.
5. PyO3/Cython/build gate: `crates/pyo3`, `crates/**/src/python/**`,
   `nautilus_trader/**/*.pyx`, `nautilus_trader/**/*.pxd`, `pyproject.toml`,
   `python/pyproject.toml`, and `build.py` are handled by their dedicated
   removal tasks.
6. Migration note gate: public breaking changes for Python imports, examples,
   docs, build commands, and package installation are written before deletion.
7. Release gate: `scripts/ai/check_rust_only_runtime.sh`, release checklist,
   and final owner signoff pass.

## Removal Lanes

### Lane A: `python/` Overlay

Candidate paths:

- `python/nautilus_trader/**`
- `python/tests/**`
- `python/generate_stubs.py`
- `python/generate_docstrings.py`
- `python/pyproject.toml`

Required evidence before deletion:

- Rust product docs no longer tell users to install or import the Python
  package as the product surface.
- PyO3 package bridge removal is approved.
- RREM-005 test decisions cover `python/tests/**`.
- Stub/docstring generation is either obsolete or replaced by Rust docs.
- Migration notes explain that Python import paths are removed.

Current decision: removed from the Rust-only release product surface.

### Lane B: Top-Level `nautilus_trader/` Python Modules

Historical candidate paths:

- `nautilus_trader/**/*.py`
- `nautilus_trader/**/*.pyi`
- `nautilus_trader/examples/**`
- `nautilus_trader/test_kit/**`

Required evidence before deletion:

- Matching Rust crate APIs or CLI workflows are the supported replacement.
- Python examples are mapped to Rust examples or legacy migration material.
- Test kit functionality is either unnecessary for Rust-only release or
  replaced by Rust test fixtures.
- Public migration notes cover removed import paths and user workflows.

Current decision: removed from the Rust-only release product surface.

### Lane C: Top-Level `nautilus_trader/` Cython Modules

Candidate paths:

- `nautilus_trader/**/*.pyx`
- `nautilus_trader/**/*.pxd`

Required evidence before deletion:

- RREM-003 Cython dependency graph remains current.
- Rust runtime parity covers the Cython behavior being removed.
- PyO3/Cython interop paths are no longer part of the product surface.
- Build cleanup removes Cython dependencies and doctest/coverage hooks.

Historical RREM-006 decision: blocked and owned by Cython/build removal tasks,
not by that Python package staging task. Later RREM cleanup and RREL-009
resolved the release blocker.

### Lane D: Tests, Docs, And Examples

Candidate paths:

- `tests/**/*.py`
- `examples/**/*.py`
- `examples/**/*.ipynb`
- Python-facing `README.md` and `docs/**` references

Required evidence before deletion:

- RREM-005 per-test-family decisions are applied.
- RREM-004 workflow readiness matrix moves relevant workflows to ready.
- Rust docs/examples cover supported user stories.
- Unsupported workflows are deferred or removed with release-gate approval.

Current decision: removed from the Rust-only release product surface.

## Blockers

The following blockers were recorded at RREM-006 time:

- Rust CLI `run` paths for backtest/sandbox/live/data/config remain incomplete
  compared with Python workflows.
- Runtime parity is not complete for all Python-facing behavior in model,
  execution, risk, portfolio/accounting, persistence, live, and adapters.
- Adapter support/deferred/removal decisions exist by inventory, but production
  live workflow replacement is not release-ready.
- Python-only tests required port/scope decisions before deletion. RC cleanup
  later applied the approved Rust-only `removed_with_surface` decision to
  top-level `tests/**/*.py`.
- PyO3 and Cython surfaces remain active build/import bridges.
- Public migration notes for removed Python package imports are not final.
- Final Rust-only release checks are not the current passing gate.

## Allowed Later Actions

Later RREM/RREL tasks may advance a package-surface item only by recording one
of these outcomes:

- `ready_for_deletion`: all gates for the path are green;
- `ported_to_rust`: Rust code/tests/docs replace the behavior;
- `superseded_by_rust_product_contract`: the Python workflow is intentionally
  replaced by a Rust-only workflow;
- `legacy_migration_only`: the file is retained or moved as non-product
  migration material;
- `deferred_by_scope`: release gatekeeper approves deferral from the Rust-only
  release scope;
- `blocked`: deletion remains unsafe and must not proceed.

## Package Removal Checklist

Use this checklist before any future deletion PR touches package paths:

- [ ] Source path is listed in RREM-001, RREM-002, RREM-003, RREM-004, or
      RREM-005 evidence.
- [ ] Owner task states the exact removal lane.
- [ ] Replacement Rust path or explicit deferral is linked.
- [ ] Required tests or golden traces are named.
- [ ] Public migration note exists if imports, docs, examples, or build
      commands change.
- [ ] `pyproject.toml`, `python/pyproject.toml`, `build.py`, and Cargo
      workspace effects are reviewed when packaging/build is touched.
- [ ] Release gatekeeper approval is recorded.

## Next Tasks

- RREM-007 should stage PyO3 package and per-crate binding removal.
- RREM-008/RREM-009 class tasks should handle Cython source/build cleanup only
  after Cython graph and Rust parity gates are ready.
- RREL tasks must turn the staged removal plan into user-facing migration
  notes before any final deletion PR.
