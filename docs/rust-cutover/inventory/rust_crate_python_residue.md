# Rust Crate Python/PyO3 Residue Inventory

Date: 2026-06-02
Executor: Codex
Task ID: RREM-018

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

`crates/adapters`, `crates/indicators`, `crates/common`, and `crates/core`
no longer contain PyO3/Python binding residue after RREM-018:

```bash
rg -n -i "pyo3|pyo3_stub_gen|feature = \"python\"|cfg\\(feature = \"python\"\\)|cfg_attr\\([^\\n]*python|python|nautilus_pyo3|extension-module|custom_data\\([^\\)]*pyo3|stub_module|PyObject|PyAny|Py<" \
  crates/adapters crates/indicators crates/common crates/core
```

Result: no matches.

## Remaining Rust Crate Python/PyO3 Hits

The broader Rust-only runtime scan still finds Python/PyO3 residue outside the
RREM-018 implementation scope. File-level count:

```text
109 files
```

Top path groups:

```text
 19 crates/serialization
 14 crates/trading
 12 crates/execution
 11 crates/persistence
  9 crates/network
  8 crates/backtest
  8 crates/live
  7 crates/infrastructure
  5 crates/model
  3 crates/data
  3 crates/portfolio
  3 crates/system
  2 crates/plugin
  2 crates/risk
  2 crates/testkit
  1 crates/cryptography
```

RREM-018 cargo checks also surfaced `unexpected cfg` warnings in these
remaining non-scoped crates. Those warnings are intentionally not fixed in
RREM-018 because the task path scope only authorizes `adapters`, `indicators`,
`common`, and `core`.

## Follow-up Slices

Recommended cleanup order:

1. `crates/serialization` and `crates/network`: shared adapter dependencies
   that already emit `feature = "python"` warnings during adapter checks.
2. `crates/execution`, `crates/backtest`, `crates/live`, `crates/trading`,
   `crates/risk`, `crates/portfolio`, and `crates/data`: runtime-facing
   config/model cleanup, with targeted cargo checks per crate.
3. `crates/persistence`, `crates/infrastructure`, `crates/plugin`,
   `crates/testkit`, and `crates/cryptography`: support crate cleanup after
   runtime surfaces are handled.
4. Residual `crates/model` documentation/comment cleanup that was outside the
   already merged active binding removal slice.

## Boundary

This inventory is not a deletion approval for all remaining matches. Each
crate family needs its own PR with scoped path authority, targeted cargo
checks, and release-gate review.
