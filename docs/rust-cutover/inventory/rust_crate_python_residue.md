# Rust Crate Python/PyO3 Residue Inventory

Date: 2026-06-02
Executor: Codex
Task ID: RREM-022

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
- RREM-022 cleanup for final `crates/model` and `crates/system` residue.

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

`crates/model` and `crates/system` no longer contain PyO3/Python binding
residue after RREM-022:

```bash
rg -n -i "pyo3|pyo3_stub_gen|feature = \"python\"|cfg\\(feature = \"python\"\\)|cfg_attr\\([^\\n]*python|python|nautilus_pyo3|extension-module|custom_data\\([^\\)]*pyo3|stub_module|PyObject|PyAny|Py<" \
  crates/model crates/system --glob "*.rs" --glob "*.toml" --glob "*.md"
```

Result: no matches.

## Remaining Rust Crate Python/PyO3 Hits

The broader Rust-only runtime scan no longer finds Python/PyO3 residue in Rust
crate product paths. File-level count:

```text
0 files
```

Top path groups:

```text
none
```

RREM-022 targeted cargo checks passed for the final scoped crates. The
Rust-only runtime gate now passes.

## Follow-up Slices

No additional Rust crate Python/PyO3 residue cleanup slices are currently
recommended. The next release step should use release-gate evidence rather than
starting `RREL-008` automatically.

## Boundary

This inventory is not a deletion approval for all remaining matches. Each
crate family needs its own PR with scoped path authority, targeted cargo
checks, and release-gate review.
