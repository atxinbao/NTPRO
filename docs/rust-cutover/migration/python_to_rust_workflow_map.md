# RREM-004 Python To Rust Workflow Map

Date: 2026-06-01
Executor: Codex
Task ID: RREM-004

Updated: 2026-06-04
Executor: Codex
Follow-up ID: RC-CLEANUP-001

## Scope

This document maps current Python-facing workflows to Rust-first replacement
paths for the Rust-only cutover. It does not delete Python, PyO3, Cython,
examples, tests, docs, packaging, or runtime code.

## Summary

The Rust product surface now has clear CLI contracts and help-level commands,
plus two runnable Cargo smokes for deterministic backtest and sandbox
live-node lifecycle. Full CLI `run` workflows are still blocked until Rust
config parsing, strategy selection, data/catalog wiring, and live-node runtime
integration are connected.

| Python workflow family | Python surface today | Rust replacement path | Current status | Gate decision |
| --- | --- | --- | --- | --- |
| Backtest run scripts | `examples/backtest/**/*.py`, docs backtest guides, Python `BacktestNode` and `BacktestEngine` examples | `nautilus backtest validate/run` contract plus `cargo run -p nautilus-backtest --features examples --example engine-ema-cross` | Partial: Cargo smoke exists; CLI run path returns explicit blocker. | Do not remove Python backtest workflows until CLI config/run smoke passes without Python. |
| Sandbox workflows | `examples/sandbox/**/*.py` and Python sandbox adapter examples | `nautilus sandbox validate/run` contract plus `cargo run -p nautilus-live --no-default-features --features node --example sandbox-node-smoke` | Partial: Cargo sandbox smoke exists; CLI run path returns explicit blocker. | Do not remove Python sandbox examples until CLI lifecycle smoke is runnable. |
| Live adapter workflows | `examples/live/<venue>/**/*.py` and integration docs | `nautilus live validate/run` contract, adapter Rust examples, and RADP fixture manifests | Deferred for production live; fixture/mock/dry-run evidence exists by adapter family, but not a unified live product command. | Keep production live behavior gated; remove only after adapter support decisions and Rust live smoke. |
| Data/catalog workflows | Python catalog, wrangler, loader, Databento/Tardis examples, docs data guides | `nautilus data inspect/validate/load` contract and `examples/rust/data/README.md` | Partial: GH-156 supports local file/directory inspect and validate; loader/catalog ingest remains blocked. | Do not treat data migration complete until Rust load/catalog ingest path is implemented and scoped. |
| Config validation workflows | Python config objects and example config scripts | `nautilus config validate --kind <backtest|sandbox|live|data>` | Partial: scoped Rust TOML validation exists for backtest, sandbox, live-smoke, and data/catalog configs; unified workflow config models remain incomplete. | Do not treat config migration complete until shared Rust config models validate real product examples and runtime-specific blockers are closed. |
| Strategy examples | Python strategies under `examples/backtest/**/strategy.py`, `examples/other/**`, and Python live examples | Rust strategy APIs under `nautilus-trading`, Rust `EmaCross` example smoke, future strategy registry | Partial: one deterministic Rust example path exists; arbitrary user strategy loading is not a stable product contract. | Keep Python strategy examples until Rust strategy registry or scoped built-in strategy contract lands. |
| Adapter tester scripts | Python live data/exec tester scripts per venue | Adapter crate examples and tests under `crates/adapters/<venue>/examples` and `tests`; RADP support/deferred decisions | Partial by venue; fixture evidence is stronger than product CLI evidence. | Treat tester scripts as deferred migration material until live CLI and adapter scopes converge. |
| Notebooks and debugging | `examples/backtest/notebooks/**`, `examples/other/debugging/**` | No Rust notebook replacement; Rust docs/examples and cargo smokes only | Deferred. | Keep or archive as legacy migration docs; do not claim Rust-only replacement yet. |
| Python-only tests | Historical `tests/**/*.py`; local Python helper scripts under `scripts/` remain | RREM-005 test scope map, Rust crate tests, golden trace, adapter fixture tests | Top-level Python tests removed by RC cleanup after Rust-only completion; the cleanup is included in `ntpro-rust-only-rc.2`; `scripts/` helpers remain non-product automation. | Do not restore Python product tests. New release evidence should be Rust-native. |

## Current Rust Product Evidence

Help-level Rust CLI commands exist and run without executing Python workflows:

```bash
cargo run -q -p nautilus-cli -- backtest --help
cargo run -q -p nautilus-cli -- sandbox --help
cargo run -q -p nautilus-cli -- live --help
cargo run -q -p nautilus-cli -- data --help
cargo run -q -p nautilus-cli -- config --help
```

Runnable Rust smokes already documented by product tasks:

```bash
cargo run -p nautilus-backtest --features examples --example engine-ema-cross
cargo run -p nautilus-live --no-default-features --features node --example sandbox-node-smoke
```

These are not equivalent to complete product CLI replacements. They prove Rust
runtime entrypoints exist, while the CLI run paths still need config/runtime
wiring before Python workflows can be removed.

## Python Workflow Inventory Inputs

RREM-001 recorded broad Python product surfaces:

- `examples/`: 137 Python-facing example files;
- `tests/`: 534 Python test files at RREM-004/RREM-005 time; 0 tracked Python
  test files after RC cleanup;
- `python/`: 178 package overlay files;
- `nautilus_trader/`: 642 Python/Cython/interface files;
- README/docs Python-facing references: 75 files.

RREM-002 recorded PyO3 surfaces:

- `crates/pyo3/`: 4 files;
- `crates/**/src/python*`: 371 files;
- Rust files or manifests referencing PyO3 binding primitives: 775 files.

RREM-003 recorded Cython surfaces:

- 110 `.pyx` files;
- 133 `.pxd` files;
- 243 generated CSV inventory rows.

## Example Distribution

Current Python examples by top-level family:

| Family | Count |
| --- | ---: |
| `examples/live` | 60 |
| `examples/backtest` | 49 |
| `examples/sandbox` | 8 |
| `examples/other` | 4 |
| `examples/utils` | 2 |

Current Rust example docs under `examples/rust`:

| Family | Current artifact |
| --- | --- |
| `backtest` | README and Cargo smoke pointer. |
| `sandbox` | README and Cargo smoke pointer. |
| `live` | README and CLI contract pointer. |
| `data` | README and CLI contract pointer. |
| `config` | README and CLI contract pointer. |

## Replacement Rules

Use these rules when a later RREM task removes or archives Python workflows:

- A Python workflow is replaceable only when the matching Rust CLI/API path is
  supported, validated, and documented.
- Help output alone is not enough to remove a Python workflow.
- A Cargo example smoke is valid runtime evidence, but not a full CLI product
  replacement until config/run wiring exists.
- Adapter workflows must be classified by venue as supported, deferred, or
  removed, using RADP evidence.
- Production live workflows require explicit release-gate approval and must not
  be inferred from fixture or dry-run tests.
- Notebooks can be retained as legacy migration docs until Rust docs cover the
  same user story.
- Python-only tests were handled by RREM-005 planning and RC cleanup removal.
  Future release evidence should be Rust-native unless a local helper script is
  explicitly scoped as non-product automation.

## Workflow Readiness Matrix

| Workflow | Rust help | Rust runtime smoke | Config validation | Product run | Removal readiness |
| --- | --- | --- | --- | --- | --- |
| Backtest | Yes | Yes, via Cargo example | Blocked | Blocked | Rust-only release evidence exists; full product run remains future work. |
| Sandbox | Yes | Yes, via Cargo example | Blocked | Blocked | Rust-only release evidence exists; full product run remains future work. |
| Live | Yes | Partial, sandbox/live-node evidence only | Blocked | Blocked | Rust-only release evidence exists; production live remains gated. |
| Data/catalog | Yes | Partial crate/API evidence | Blocked | Blocked | Rust-only release evidence exists; full catalog workflow remains future work. |
| Config validation | Yes | Not a runtime workflow | Blocked | Not applicable | Rust-only CLI contract exists; shared config workflow remains future work. |
| Adapter tester scripts | Not unified | Partial per adapter | Deferred by venue | Blocked/deferred | Python tester scripts are not product surfaces; adapter Rust evidence remains active. |
| Notebooks/debugging | No direct replacement | No | No | No | Python notebooks/debugging are not product surfaces. |

## Next Tasks

- Keep README and release notes aligned with the Rust-only public surface.
- Keep local Python helper scripts documented as non-product automation.
- Publish a GitHub pre-release only after current checks, Rust CLI entrypoint
  evidence, and repository language display are reviewed.
