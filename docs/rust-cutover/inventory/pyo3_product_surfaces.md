# RREM-002 PyO3 Product Surface Inventory

Date: 2026-06-01
Executor: Codex
Task ID: RREM-002

## Scope

This inventory covers PyO3-facing product surfaces that must be removed,
ported, or explicitly scoped before Rust-only cutover can pass. It is
non-destructive: no Python, PyO3, Cython, build, runtime, adapter, or public
API files were deleted or edited.

## Summary

PyO3 remains a first-class product bridge across the workspace. The dedicated
`nautilus-pyo3` crate aggregates module bindings, `crates/**/src/python`
contains the per-crate binding modules, and the build/package metadata still
targets maturin, Python extension modules, type stubs, and Cython
compatibility.

| Surface | Evidence command | Count | Removal decision |
| --- | --- | ---: | --- |
| `crates/pyo3/` aggregator crate | `find crates/pyo3 -type f` | 4 files | Blocked pending Rust-only route and replacement import/API evidence. |
| workspace Python binding modules | `find crates -path '*/src/python*' -type f` | 371 files | Blocked pending per-crate Rust API parity and public migration notes. |
| Rust files or manifests referencing PyO3 binding primitives | `rg -l 'pyo3|PyO3|wrap_pymodule|pymodule|pyclass|pymethods' crates -g '*.rs' -g 'Cargo.toml'` | 775 files | Blocked pending staged removal plan. |
| root workspace membership/dependencies | `Cargo.toml` | 1 member, 4 workspace deps/profiles | Remove only in a gated Cargo workspace restructuring task. |
| Python packaging bridge | `python/pyproject.toml` | maturin manifest to `../crates/pyo3/Cargo.toml` | Blocked pending Python package removal decision. |
| local build bridge | `build.py` | PyO3 build/copy paths and `debug-pyo3` handling | Blocked pending RREM-003 build-path inventory and removal gate. |

## Aggregator Crate: `crates/pyo3`

`crates/pyo3` is the workspace-level Python extension crate:

- `crates/pyo3/Cargo.toml`
- `crates/pyo3/README.md`
- `crates/pyo3/src/lib.rs`
- `crates/pyo3/bin/stub_gen.rs`

`crates/pyo3/Cargo.toml` defines:

- package name `nautilus-pyo3`;
- library name `nautilus_pyo3`;
- crate types `rlib` and `cdylib`;
- feature `extension-module`, forwarding extension-module features into core,
  runtime, adapter, and support crates;
- feature `cython-compat`, which keeps a Cython-compatible import path alive;
- feature `defi`, which includes the blockchain adapter Python module;
- dependencies on all major Rust runtime crates with their `python` feature
  enabled;
- dependencies on adapter crates with their `python` feature enabled;
- `python-stub-gen`, which emits Python type stubs through `pyo3-stub-gen`.

Decision: this crate is a direct product surface, not an internal-only helper.
It can only be removed after Rust product entrypoints, runtime parity, adapter
scope decisions, and migration notes are approved.

## Workspace Binding Modules

The workspace contains 371 files under `crates/**/src/python*`.

| Crate family | File count |
| --- | ---: |
| `crates/adapters/**/src/python` | 110 |
| `crates/model/src/python` | 114 |
| `crates/indicators/src/python` | 44 |
| `crates/analysis/src/python` | 23 |
| `crates/common/src/python` | 15 |
| `crates/persistence/src/python` | 11 |
| `crates/core/src/python` | 10 |
| `crates/infrastructure/src/python` | 7 |
| `crates/backtest/src/python` | 6 |
| `crates/execution/src/python` | 6 |
| `crates/network/src/python` | 4 |
| `crates/trading/src/python` | 4 |
| `crates/data/src/python` | 3 |
| `crates/live/src/python` | 3 |
| `crates/cryptography/src/python` | 2 |
| `crates/risk/src/python` | 2 |
| `crates/serialization/src/python` | 2 |
| `crates/system/src/python` | 2 |
| `crates/testkit/src/python` | 2 |
| `crates/portfolio/src/python` | 1 |

Decision: these modules are removal candidates, but removal must be staged by
crate family. Model, adapter, runtime, persistence, and indicator bindings are
too broad to delete in one pass without replacement coverage.

## Aggregated Python Modules

`crates/pyo3/src/lib.rs` registers Python modules from the workspace through
`pyo3::wrap_pymodule!`. The always-on module set includes:

- core/runtime families: `analysis`, `core`, `common`, `cryptography`, `data`,
  `execution`, `indicators`, `infrastructure`, `live`, `model`, `network`,
  `persistence`, `portfolio`, `risk`, `serialization`, `testkit`, `trading`,
  `backtest`;
- adapter families: `architect`, `binance`, `bitmex`, `bybit`, `coinbase`,
  `databento`, `deribit`, `dydx`, `hyperliquid`, `kraken`,
  `interactive_brokers`, `okx`, `polymarket`, `sandbox`, `tardis`;
- optional DeFi family: `blockchain` behind feature `defi`;
- runtime shutdown hook `_shutdown_nautilus_runtime`.

The same file switches the module import name based on `cython-compat`:

- without `cython-compat`: `nautilus_trader._libnautilus`;
- with `cython-compat`: `nautilus_trader.core.nautilus_pyo3`.

Decision: the import-path switch is a bridge between PyO3 and Cython and must
remain blocked until RREM-003 completes the Cython source/build inventory.

## Workspace And Build Metadata

The root `Cargo.toml` still exposes PyO3 at workspace level:

- workspace member `crates/pyo3`;
- workspace dependency `nautilus-pyo3`;
- third-party dependencies `pyo3`, `pyo3-async-runtimes`, and
  `pyo3-stub-gen`;
- profile `debug-pyo3`.

`python/pyproject.toml` still builds the Python package with maturin:

- `manifest-path = "../crates/pyo3/Cargo.toml"`;
- `module-name = "nautilus_trader._libnautilus"`;
- features including `extension-module`, `arrow`, `high-precision`, `redis`,
  `postgres`, `defi`, and `hypersync`.

`build.py` still has PyO3-specific behavior:

- `debug-pyo3` profile handling;
- cargo package selection for `nautilus-pyo3`;
- features `cython-compat` and `extension-module`;
- copy from the built Rust dynamic library into `nautilus_trader/core`.

Decision: Cargo/workspace/build metadata removal must be a dedicated gated
task because it can affect local builds, wheel builds, and import paths.

## Removal Readiness Classification

| Class | Paths | Status |
| --- | --- | --- |
| PyO3 aggregator crate | `crates/pyo3/**` | Blocked pending Rust-only release gate. |
| Per-crate Python binding modules | `crates/**/src/python/**` | Blocked pending per-crate replacement/parity evidence. |
| Workspace PyO3 membership and deps | `Cargo.toml`, `Cargo.lock` | Blocked pending Cargo restructuring task. |
| Python package maturin bridge | `python/pyproject.toml` | Blocked pending Python product surface removal. |
| Build bridge | `build.py` | Blocked pending Cython/build inventory and release gate. |
| Type stub generation | `crates/pyo3/bin/stub_gen.rs`, `python/generate_stubs.py` | Blocked pending Python API removal and migration notes. |

## Blockers Before Removal

- RREM-003 must classify Cython source and build surfaces before removing the
  `cython-compat` bridge.
- Rust CLI/API/docs must replace Python import workflows for supported
  backtest, live, sandbox, data, config, and adapter use cases.
- Adapter support decisions must identify which PyO3 adapter bindings are
  supported, deferred, or removed.
- Public migration notes must list removed Python import paths and replacement
  Rust workflows.
- Cargo workspace changes must be reviewed as a medium/high-risk structural
  change, not folded into inventory-only tasks.

## Next Tasks

- RREM-003: inventory Cython source and build surfaces.
- Later staged-removal tasks can split PyO3 deletion by product surface:
  aggregator crate, per-crate binding modules, packaging bridge, type stubs,
  and workspace metadata.
