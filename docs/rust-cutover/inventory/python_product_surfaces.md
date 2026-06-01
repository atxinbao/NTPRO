# RREM-001 Python Product Surface Inventory

Date: 2026-06-01
Executor: Codex
Task ID: RREM-001

## Scope

This inventory covers Python product-facing surfaces that must be removed,
ported, or explicitly scoped before the Rust-only cutover can pass. It is
non-destructive: no Python, PyO3, Cython, build, runtime, or public API files
were deleted or edited.

## Summary

Python remains a broad product surface across package entrypoints, generated
stubs, strategy examples, live/backtest examples, acceptance and integration
tests, documentation snippets, and packaging metadata.

| Surface | Evidence command | Count | Removal decision |
| --- | --- | ---: | --- |
| `python/` package overlay | `find python -type f` | 178 files | Port or remove after Rust CLI/API/runtime parity evidence. |
| `python/nautilus_trader/` import overlay | `find python/nautilus_trader -maxdepth 2 -type d` | 47 top-level module dirs | Replace with Rust product docs or remove when PyO3 bridge is removed. |
| top-level `nautilus_trader/` Python/Cython package | `find nautilus_trader -type f` | 642 Python/Cython/interface files | Split by RREM-002/RREM-003; not removable in RREM-001. |
| `examples/` Python workflows | `find examples -type f -name '*.py' -o -name '*.ipynb'` | 137 files | Replace with Rust examples or scope as migration-only docs. |
| `tests/` Python test suites | `find tests -type f -name '*.py'` | 534 files | Port, replace with Rust tests, or defer with explicit parity gaps. |
| `docs/` Python tutorial assets | `find docs -type f -name '*.py'` | 20 files | Replace product docs with Rust flows or mark as legacy migration examples. |
| Python-facing documentation references | `rg -l 'from nautilus_trader|import nautilus_trader|pip install|Python 3|```python' README.md docs` | 75 docs | Rewrite final product docs to Rust-only usage path. |
| root Python packaging/build metadata | `pyproject.toml`, `python/pyproject.toml`, `build.py` | 3 files | Build-path removal tracked by RREM-003 and final release gate. |

## Product-Facing Package Surfaces

### `python/`

`python/` contains a separate Python package overlay and local Python tests:

- `python/nautilus_trader/__init__.py`
- `python/nautilus_trader/_fixup.py`
- `python/nautilus_trader/_libnautilus/__init__.pyi`
- `python/nautilus_trader/{analysis,backtest,common,core,data,execution,live,model,persistence,portfolio,risk,serialization,testkit,trading}/`
- adapter stub overlays such as `python/nautilus_trader/{binance,bybit,databento,deribit,dydx,hyperliquid,kraken,okx,polymarket,sandbox,tardis}/`
- `python/tests/acceptance/test_backtest.py`
- `python/tests/unit/test_live_node.py`
- `python/tests/unit/test_msgbus.py`
- `python/generate_stubs.py`
- `python/generate_docstrings.py`
- `python/pyproject.toml`

Decision: treat this as a removable product overlay only after Rust user
entrypoints and runtime parity are release-gate approved. This task records
the surface but does not remove it.

### Top-level `nautilus_trader/`

`nautilus_trader/` is still the dominant Python/Cython-facing package. The
Python product modules include:

- runtime modules: `backtest`, `live`, `trading`, `data`, `execution`, `risk`,
  `portfolio`, `persistence`, `cache`, `common`, `system`;
- public model and value modules: `model`, `accounting`, `core`, `indicators`;
- adapter modules: `adapters/{binance,bybit,databento,deribit,dydx,hyperliquid,interactive_brokers,kraken,okx,polymarket,sandbox,tardis,...}`;
- Python examples under `nautilus_trader/examples/`;
- Python test kit under `nautilus_trader/test_kit/`.

Decision: RREM-001 classifies this as a final Rust-only blocker. RREM-002
separates PyO3 import surfaces, and RREM-003 separates Cython source/build
surfaces.

## Python Examples

`examples/` still presents Python as a primary user workflow:

- backtest examples: `examples/backtest/*.py`;
- live examples: `examples/live/<venue>/*.py`;
- sandbox examples: `examples/sandbox/*.py`;
- notebooks and debug examples under `examples/backtest/notebooks/`,
  `examples/live/*/notebooks/`, and `examples/other/debugging/`;
- helper utilities under `examples/utils/`.

Decision: replace supported workflows with Rust examples before deleting or
archiving these paths. Unsupported workflows should be documented as deferred
or removed by a release-gate decision.

## Python Tests

`tests/` contains Python acceptance, docs, integration, memory leak,
performance, and unit tests. Major suites include:

- `tests/acceptance_tests/`;
- `tests/docs_tests/`;
- `tests/integration_tests/adapters/`;
- `tests/integration_tests/live/`;
- `tests/integration_tests/infrastructure/`;
- `tests/mem_leak_tests/`;
- `tests/performance_tests/`;
- `tests/unit_tests/`.

Decision: do not delete tests to make the release gate pass. Each test family
must either have Rust replacement coverage, explicit parity evidence, or a
documented scope decision.

## Python Documentation Surfaces

Python remains visible in the public docs and README:

- `README.md` installation and development sections still describe Python,
  PyPI wheels, PyO3, and Cython.
- `docs/getting_started/*.py` and `docs/getting_started/*.md` present Python
  quickstart flows.
- `docs/tutorials/*.py` and `docs/tutorials/*.md` use Python examples and
  render-panel scripts.
- `docs/integrations/*.md` reference Python adapter usage.
- `docs/concepts/*.md` and `docs/developer_guide/*.md` document Python APIs,
  testing, FFI, and development paths.

Decision: final Rust-only docs must make Rust CLI/examples/docs the product
surface. Python docs can remain only as migration notes until final removal.

## Blockers Before Removal

- Rust-only CLI/API/docs must cover supported backtest, live/sandbox, data,
  config, and adapter workflows.
- Runtime parity must cover scoped Python workflows before Python tests are
  removed or rewritten.
- Adapter parity must remain traceable for venues previously exposed through
  Python examples/tests.
- Public migration notes must explain breaking changes for Python import paths.
- RREM-002 must classify PyO3 surfaces and RREM-003 must classify Cython/build
  surfaces before staged deletion tasks begin.

## Removal Readiness Classification

| Class | Paths | Status |
| --- | --- | --- |
| Python package overlay | `python/**` | Blocked pending Rust-only product and PyO3 decisions. |
| Legacy Python/Cython package | `nautilus_trader/**` | Blocked pending RREM-002/RREM-003 split and Rust parity. |
| Python examples | `examples/**/*.py`, `examples/**/*.ipynb` | Blocked pending Rust example replacements. |
| Python tests | `tests/**/*.py` | Blocked pending Rust parity/replacement tests. |
| Python docs | `README.md`, `docs/**` | Blocked pending Rust-only migration docs. |
| Packaging/build | `pyproject.toml`, `python/pyproject.toml`, `build.py` | Blocked pending build-path and Cython inventory. |

## Next Tasks

- RREM-002: isolate PyO3-facing product surfaces under `crates/pyo3` and
  `crates/**/src/python`.
- RREM-003: isolate Cython source, interface, generated-extension, and build
  surfaces.
- Later staged-removal tasks may only delete paths after release gatekeeper
  approval and replacement evidence.
