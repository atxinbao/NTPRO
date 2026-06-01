# RREM-002 Evidence - Inventory PyO3 Product Surfaces

Date: 2026-06-01
Executor: Codex
Task ID: RREM-002
Risk: medium
Release gatekeeper approval: user approved RREM-001/RREM-002/RREM-003 automation in this thread.

## Summary

Created a non-destructive inventory of PyO3 product-facing surfaces that block
the Rust-only cutover. The inventory records the `crates/pyo3` aggregator
crate, the workspace `crates/**/src/python` binding modules, PyO3 workspace
dependencies/features, stub generation, maturin packaging, and build-script
bridge points.

No Python, PyO3, Cython, runtime, adapter, Cargo behavior, or public API files
were deleted or modified.

## Files Changed

- `docs/rust-cutover/inventory/pyo3_product_surfaces.md`
- `docs/rust-cutover/evidence/RREM-002.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RREM-002.json`

## Commands Run

- `sed -n '1,220p' docs/rust-cutover/tasks/RREM-002.md`
- `find crates/pyo3 -type f | sort`
- `find crates/pyo3 -type f | wc -l`
- `find crates -path '*/src/python*' -type f | wc -l`
- `find crates -path '*/src/python*' -type f | sed -E 's#^crates/([^/]+)/.*#\\1#' | sort | uniq -c | sort -n`
- `rg -l 'pyo3|PyO3|wrap_pymodule|pymodule|pyclass|pymethods' crates -g '*.rs' -g 'Cargo.toml' | wc -l`
- `sed -n '1,220p' crates/pyo3/Cargo.toml`
- `sed -n '1,430p' crates/pyo3/src/lib.rs`
- `rg -n 'pyo3|PyO3|nautilus-pyo3|extension-module|cython-compat|pyo3-stub|stub_gen|stub' Cargo.toml crates/pyo3 README.md pyproject.toml python/pyproject.toml build.py -g '*'`
- `scripts/ai/verify_fast.sh`
- `python3 -m json.tool .agentflow/state/task_status.json`
- `python3 -m json.tool .agentflow/leases/RREM-002.json`
- `python3 scripts/ai/validate_agentflow_roles.py`
- `git diff --check`

## Command Results

- `crates/pyo3/`: 4 files.
- `crates/**/src/python*`: 371 files.
- Rust files or manifests referencing PyO3 binding primitives: 775 files.
- Largest per-crate binding surfaces: `model` 114 files, `adapters` 110 files,
  `indicators` 44 files, `analysis` 23 files, `common` 15 files,
  `persistence` 11 files, `core` 10 files.
- `crates/pyo3/Cargo.toml` confirms package `nautilus-pyo3`, library
  `nautilus_pyo3`, `cdylib` output, feature `extension-module`, feature
  `cython-compat`, adapter/runtime dependencies with `python` features, and
  `python-stub-gen`.
- `crates/pyo3/src/lib.rs` confirms the aggregated Python modules and the
  `cython-compat` import path switch.
- `python/pyproject.toml` confirms maturin builds from
  `../crates/pyo3/Cargo.toml` into `nautilus_trader._libnautilus`.
- `build.py` confirms `debug-pyo3`, `nautilus-pyo3`, `cython-compat`,
  `extension-module`, and copy-to-`nautilus_trader/core` build behavior.
- `scripts/ai/verify_fast.sh`: passed.
- JSON validation for state and lease files: passed.
- `python3 scripts/ai/validate_agentflow_roles.py`: passed.
- `git diff --check`: passed.

## Tests Added Or Updated

No tests were added or updated. This is an inventory-only task.

## Behavior Impact

No runtime behavior changed. No trading semantics changed. No Python, PyO3, or
Cython product surfaces were removed.

## Public API Impact

None.

## Migration Note Status

No migration note required for this task because it only records inventory.
Later removal tasks will require migration notes for removed PyO3/Python import
paths and Rust replacement workflows.

## Rollback Plan

Revert `docs/rust-cutover/inventory/pyo3_product_surfaces.md`,
`docs/rust-cutover/evidence/RREM-002.md`, and the RREM-002 state/lease updates.
