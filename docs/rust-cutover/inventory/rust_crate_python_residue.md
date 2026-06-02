# Rust Crate Python/PyO3 Residue Inventory

Date: 2026-06-02
Executor: Codex
Task ID: RREM-021

## Purpose

Track Rust crate Python/PyO3 residue after the Rust-only removal slices that
have already landed:

- Python workspace and package surface cleanup.
- `crates/pyo3` workspace cleanup.
- Cython generation cleanup.
- `crates/analysis` annotation cleanup.
- `crates/model` active PyO3 binding cleanup.
- RREM-018 cleanup for `crates/adapters`, `crates/indicators`,
  `crates/common`, and `crates/core`.
- RREM-019 cleanup for `crates/serialization` and `crates/network`.
- RREM-020 cleanup for `crates/execution`, `crates/backtest`, `crates/live`,
  `crates/trading`, `crates/risk`, `crates/portfolio`, and `crates/data`.
- RREM-021 cleanup for `crates/persistence`, `crates/infrastructure`,
  `crates/plugin`, `crates/testkit`, and `crates/cryptography`.

## Current State

The active Cython scan is clean for Rust crate product paths:

```bash
rg -n "Cython|cythonize|\\.pyx|\\.pxd|cbindgen_cython|nautilus_trader/core/rust|cython" \
  crates Cargo.toml Makefile pyproject.toml --glob '!docs/**'
```

Result: no matches.

`crates/serialization` and `crates/network` no longer contain PyO3/Python
binding residue after RREM-019:

```bash
rg -n -i "pyo3|pyo3_stub_gen|feature = \"python\"|cfg\\(feature = \"python\"\\)|cfg_attr\\([^\\n]*python|python|nautilus_pyo3|extension-module|custom_data\\([^\\)]*pyo3|stub_module|PyObject|PyAny|Py<" \
  crates/serialization crates/network
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

`crates/adapters`, `crates/indicators`, `crates/common`, and `crates/core`
no longer contain PyO3/Python binding residue after RREM-018:

```bash
rg -n -i "pyo3|pyo3_stub_gen|feature = \"python\"|cfg\\(feature = \"python\"\\)|cfg_attr\\([^\\n]*python|python|nautilus_pyo3|extension-module|custom_data\\([^\\)]*pyo3|stub_module|PyObject|PyAny|Py<" \
  crates/adapters crates/indicators crates/common crates/core
```

Result: no matches.

`crates/execution`, `crates/backtest`, `crates/live`, `crates/trading`,
`crates/risk`, `crates/portfolio`, and `crates/data` no longer contain
PyO3/Python binding residue after RREM-020:

```bash
rg -n -i "pyo3|pyo3_stub_gen|feature = \"python\"|cfg\\(feature = \"python\"\\)|cfg_attr\\([^\\n]*python|python|nautilus_pyo3|extension-module|custom_data\\([^\\)]*pyo3|stub_module|PyObject|PyAny|Py<" \
  crates/execution crates/backtest crates/live crates/trading crates/risk crates/portfolio crates/data
```

Result: no matches.

`crates/persistence`, `crates/infrastructure`, `crates/plugin`,
`crates/testkit`, and `crates/cryptography` no longer contain PyO3/Python
binding residue after RREM-021:

```bash
rg -n -i "pyo3|pyo3_stub_gen|feature = \"python\"|cfg\\(feature = \"python\"\\)|cfg_attr\\([^\\n]*python|python|nautilus_pyo3|extension-module|custom_data\\([^\\)]*pyo3|stub_module|PyObject|PyAny|Py<" \
  crates/persistence crates/infrastructure crates/plugin crates/testkit crates/cryptography
```

Result: no matches.

## Remaining Rust Crate Python/PyO3 Hits

The broader Rust-only runtime scan still finds Python/PyO3 residue outside the
RREM-021 implementation scope. File-level count:

```text
8 files
```

Top path groups:

```text
  5 crates/model
  3 crates/system
```

RREM-021 targeted cargo checks passed for the scoped support crates. The
Rust-only runtime gate still fails on non-scoped `crates/system` and
`crates/model` residue, which remains intentionally outside this task.

## Follow-up Slices

Recommended cleanup order:

1. Residual `crates/model` documentation/comment/cbindgen cleanup that was outside the
   already merged active binding removal slice.
2. `crates/system` cleanup for the remaining test-gated Python cfg warning and
   crate-level feature documentation.

## Boundary

This inventory is not a deletion approval for all remaining matches. Each
crate family needs its own PR with scoped path authority, targeted cargo
checks, and release-gate review.
