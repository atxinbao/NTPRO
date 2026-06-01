# RREM-003 Cython Source And Build Surface Inventory

Date: 2026-06-01
Executor: Codex
Task ID: RREM-003

## Scope

This inventory covers Cython source, interface, build, packaging, test, and
documentation surfaces that must be removed, ported, or explicitly scoped
before Rust-only cutover can pass. It is non-destructive: no Python, PyO3,
Cython, build, runtime, adapter, or public API files were deleted or edited.

## Summary

Cython remains a broad legacy runtime/build surface. All `.pyx` and `.pxd`
files are under `nautilus_trader/`, while root packaging and build metadata
still compile Cython extensions through `build.py`. Tests and docs also keep
Cython, PyO3/Cython interop, and legacy conversion paths visible.

| Surface | Evidence command | Count | Removal decision |
| --- | --- | ---: | --- |
| Cython source files | `find . -name '*.pyx'` | 110 files | Blocked pending Rust runtime parity and migration notes. |
| Cython interface files | `find . -name '*.pxd'` | 133 files | Blocked pending replacement of Cython cimports and type contracts. |
| Full generated CSV inventory | `python3 scripts/ai/inventory_cython.py` | 243 rows | Use as staged-removal source of truth. |
| Source files with `cdef class` | CSV field `has_cdef_class` | 204 files | High-risk removal; requires Rust type/API replacement evidence. |
| Source files with `cpdef` | CSV field `has_cpdef` | 151 files | High-risk removal; requires call-site migration evidence. |
| Source files with `cimport` | CSV field `has_cimport` | 216 files | High-risk removal; requires dependency graph planning. |
| Files mentioning Cython or Cython build terms | `rg -l 'Cython|cython|\\.pyx|\\.pxd|as_legacy_cython|cython-compat' ...` | 327 files | Blocked pending docs/tests/build cleanup tasks. |
| Test files mentioning Cython/PyO3 interop | `rg -l 'Cython|cython|as_legacy_cython|nautilus_pyo3' tests -g '*.py'` | 156 files | Blocked pending Rust replacement tests or explicit parity deferral. |
| Docs mentioning Cython | `rg -l 'Cython|cython|\\.pyx|\\.pxd|cython-compat' README.md docs -g '*.md'` | 235 files | Blocked pending Rust-only docs and migration notes. |

## Source Layout

The Cython source and interface files are concentrated in these
`nautilus_trader/` families:

| Path family | File count |
| --- | ---: |
| `nautilus_trader/model` | 95 |
| `nautilus_trader/core` | 24 |
| `nautilus_trader/accounting` | 18 |
| `nautilus_trader/indicators` | 18 |
| `nautilus_trader/execution` | 17 |
| `nautilus_trader/backtest` | 16 |
| `nautilus_trader/common` | 13 |
| `nautilus_trader/cache` | 9 |
| `nautilus_trader/data` | 9 |
| `nautilus_trader/portfolio` | 5 |
| `nautilus_trader/risk` | 5 |
| `nautilus_trader/serialization` | 5 |
| `nautilus_trader/persistence` | 3 |
| `nautilus_trader/trading` | 3 |
| `nautilus_trader/adapters` | 2 |
| `nautilus_trader/__init__.pxd` | 1 |

The generated detailed inventory is stored at:

- `docs/rust-cutover/inventory/cython_inventory.csv`

That CSV records each `.pyx`/`.pxd` path, suffix, line count, Cython imports,
and whether the file contains `cdef class`, `cpdef`, or `cimport`.

## Build And Packaging Surfaces

Root `pyproject.toml` still makes Cython part of packaging and testing:

- build-system dependency `cython==3.2.4`;
- dev dependency `cython==3.2.4`;
- docs dependency `cython`;
- Poetry build script `build.py`;
- pytest doctest glob for `*.pyx`;
- coverage plugin `Cython.Coverage`.

`build.py` remains the Cython build orchestrator:

- imports `build_ext`, `cythonize`, `Options`, and
  `cython_compiler_version` from Cython;
- defines Cython compiler directives in `CYTHON_COMPILER_DIRECTIVES`;
- discovers extension sources with
  `Path("nautilus_trader").rglob("*.pyx")`;
- calls `cythonize(...)` to create extension modules;
- runs `build_ext` and optionally copies compiled extensions back into source;
- prints the active Cython compiler version;
- supports `PYO3_ONLY` as a build-time escape hatch, but keeps Cython as the
  default build path unless that environment variable is set.

Decision: removal of `build.py`, Cython dependencies, doctest settings, and
coverage settings must be a dedicated gated build cleanup task. It should not
be merged into source deletion tasks.

## PyO3/Cython Interop Surfaces

Cython source still calls into PyO3 and Rust-facing bindings:

- 30 `.pyx`/`.pxd` files reference `nautilus_pyo3`;
- `build.py` enables `cython-compat` and `extension-module` while building the
  PyO3 crate;
- tests cover capsule roundtrips, `as_legacy_cython`, and PyO3-to-Cython data
  conversions.

Decision: Cython removal must be coordinated with RREM-002. Removing Cython
without first replacing PyO3/Cython interop paths would leave mixed data
conversion workflows without parity evidence.

## Test And Documentation Surfaces

Tests still reference Cython, legacy Cython conversions, or PyO3/Cython
interop across unit, integration, performance, and memory leak suites. Notable
families include:

- `tests/unit_tests/**`;
- `tests/integration_tests/adapters/**`;
- `tests/performance_tests/**`;
- `tests/mem_leak_tests/**`.

Public and developer docs still describe Cython as part of install,
development, high-precision limitations, indicator implementation, and hybrid
Rust/Python/Cython workflows.

Decision: do not delete tests or docs to force the release gate through. Each
test/doc family must be replaced, rewritten as Rust-only, or explicitly marked
as legacy migration material.

## Removal Readiness Classification

| Class | Paths | Status |
| --- | --- | --- |
| Cython source | `nautilus_trader/**/*.pyx` | Blocked pending Rust runtime parity and staged deletion plan. |
| Cython interface | `nautilus_trader/**/*.pxd` | Blocked pending cimport/type replacement plan. |
| Build orchestrator | `build.py` | Blocked pending dedicated build cleanup task. |
| Build dependencies | `pyproject.toml` | Blocked pending packaging and test config cleanup. |
| PyO3/Cython bridge | `crates/pyo3`, `nautilus_trader/core/nautilus_pyo3*`, `cython-compat` | Blocked pending RREM-002/RREM-003 combined removal gate. |
| Tests | `tests/**/*.py` | Blocked pending Rust replacement coverage or explicit parity deferral. |
| Docs | `README.md`, `docs/**/*.md` | Blocked pending Rust-only docs and migration notes. |

## Blockers Before Removal

- Rust runtime parity must cover model, backtest, accounting, execution,
  portfolio, risk, persistence, data, and cache behavior currently implemented
  in Cython.
- Cython `cimport` dependencies must be planned as a graph, not removed by
  directory order.
- PyO3/Cython interop paths must have Rust-only replacements or explicit
  deferred scope decisions.
- Build cleanup must remove Cython dependencies, pytest settings, coverage
  plugin configuration, and `build.py` behavior together.
- Migration notes must explain removed Cython/Python extension modules and
  replacement Rust workflows.

## Next Tasks

- Map Cython source families to Rust replacement/parity tasks.
- Split staged removal into source, interface, build, test, docs, and
  migration-note tasks.
- Keep `docs/rust-cutover/inventory/cython_inventory.csv` updated if Cython
  files move before removal begins.
