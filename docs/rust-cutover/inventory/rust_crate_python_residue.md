# Rust Crate Python/PyO3 Residue Inventory

Date: 2026-06-02
Executor: Codex
Task ID: RREM-016

## Purpose

Track the Rust crate Python/PyO3 residue that remains after removing the active
Cython build residue and the `crates/analysis` PyO3 annotations.

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

## Remaining Rust Crate Python/PyO3 Hits

The broader Rust-only runtime scan still finds Python/PyO3 residue outside the
RREM-016 implementation scope. File-level count:

```text
406 files
```

Top path groups:

```text
143 crates/adapters
126 crates/model
 40 crates/indicators
 24 crates/common
 13 crates/trading
  8 crates/persistence
  8 crates/network
  7 crates/execution
  7 crates/backtest
  6 crates/live
  6 crates/infrastructure
  4 crates/core
  3 crates/system
  2 crates/risk
  2 crates/portfolio
  1 crates/cryptography
  1 crates/data
  1 crates/plugin
  1 crates/serialization
  1 crates/testkit
  1 Makefile
  1 pyproject.toml
```

## Follow-up Slices

Recommended cleanup order:

1. `crates/model`: largest remaining cluster and central to value/type
   annotations.
2. `crates/indicators`: isolated metric/indicator structs with repeated
   annotation patterns.
3. `crates/common`, `crates/core`, and runtime support crates.
4. `crates/backtest`, `crates/execution`, `crates/risk`, `crates/portfolio`, and
   `crates/trading`.
5. Adapter crates, grouped by venue family to avoid changing adapter behavior in
   a single oversized PR.
6. Root build metadata such as `Makefile` and `pyproject.toml` after dependent
   crate references are gone.

## Boundary

This inventory is intentionally not a deletion approval for all remaining
matches. Many matches are product-code annotations or migration notes. Each
crate family needs a dedicated PR with targeted cargo checks.
