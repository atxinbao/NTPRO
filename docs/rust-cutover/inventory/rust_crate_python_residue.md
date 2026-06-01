# Rust Crate Python/PyO3 Residue Inventory

Date: 2026-06-02
Executor: Codex
Task ID: RREM-017

## Purpose

Track the Rust crate Python/PyO3 residue that remains after removing the active
Cython build residue, the `crates/analysis` PyO3 annotations, and the
`crates/model` PyO3/Python binding residue.

## Current State

The active Cython scan is clean for Rust crate product paths:

```bash
rg -n "Cython|cythonize|\\.pyx|\\.pxd|cbindgen_cython|nautilus_trader/core/rust|cython" \
  crates Cargo.toml Makefile pyproject.toml --glob '!docs/**'
```

Result: no matches.

The old generated header path scan is also clean outside historical inventory
records:

```bash
rg -n "nautilus_trader/core/includes|nautilus_trader/core/rust" \
  crates Cargo.toml Makefile pyproject.toml docs/developer_guide \
  docs/rust-cutover/inventory/core_model_value_types.md
```

Result: no matches.

`crates/analysis` no longer contains PyO3/Python annotation residue:

```bash
rg -n "pyo3|pyo3_stub_gen|PyO3|feature = \"python\"|python" crates/analysis
```

Result: no matches.

`crates/model` no longer contains PyO3/Python binding residue:

```bash
rg -n "pyo3|pyo3_stub_gen|PyO3|feature = \"python\"|python|nautilus_trader" crates/model
```

Result: no matches.

## Remaining Rust Crate Python/PyO3 Hits

The broader Rust-only runtime scan still finds Python/PyO3 residue outside the
RREM-017 implementation scope. File-level count:

```text
325 files
```

Top path groups:

```text
162 crates/adapters
 41 crates/indicators
 27 crates/common
 14 crates/trading
 10 crates/persistence
  9 crates/network
  8 crates/execution
  8 crates/backtest
  7 crates/live
  7 crates/infrastructure
  6 crates/core
  4 crates/system
  4 crates/portfolio
  3 crates/risk
  3 crates/plugin
  3 crates/data
  2 crates/testkit
  2 crates/serialization
  2 crates/cryptography
  1 crates/event_store
  1 crates/cli
  1 crates/analysis
```

## Follow-up Slices

Recommended cleanup order:

1. `crates/indicators`: isolated metric/indicator structs with repeated
   annotation patterns.
2. `crates/core`, `crates/common`, and runtime support crates.
3. `crates/backtest`, `crates/execution`, `crates/risk`, `crates/portfolio`, and
   `crates/trading`.
4. Adapter crates, grouped by venue family to avoid changing adapter behavior in
   a single oversized PR.
5. Remaining persistence macro and serialization bridge residue after adapter
   custom-data usage is retargeted or removed.

## Boundary

This inventory is intentionally not a deletion approval for all remaining
matches. Many matches are product-code annotations or migration notes. Each
crate family needs a dedicated PR with targeted cargo checks.
