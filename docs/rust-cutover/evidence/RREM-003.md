# RREM-003 Evidence - Inventory Cython Source And Build Surfaces

Date: 2026-06-01
Executor: Codex
Task ID: RREM-003
Risk: medium
Release gatekeeper approval: user approved RREM-001/RREM-002/RREM-003 automation in this thread.

## Summary

Created a non-destructive inventory of Cython source, interface, build,
packaging, test, and documentation surfaces that block the Rust-only cutover.
The inventory records `.pyx`/`.pxd` counts and module distribution, generated a
CSV source inventory, and classified `build.py`, `pyproject.toml`, pytest,
coverage, docs, and PyO3/Cython interop references as later gated cleanup
work.

No Python, PyO3, Cython, runtime, adapter, Cargo behavior, packaging behavior,
or public API files were deleted or modified.

## Files Changed

- `docs/rust-cutover/inventory/cython_source_build_surfaces.md`
- `docs/rust-cutover/inventory/cython_inventory.csv`
- `docs/rust-cutover/evidence/RREM-003.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RREM-003.json`

## Commands Run

- `sed -n '1,220p' docs/rust-cutover/tasks/RREM-003.md`
- `find . -path './target' -prune -o -path './.git' -prune -o -type f \\( -name '*.pyx' -o -name '*.pxd' \\) -print | wc -l`
- `find . -path './target' -prune -o -path './.git' -prune -o -type f -name '*.pyx' -print | wc -l`
- `find . -path './target' -prune -o -path './.git' -prune -o -type f -name '*.pxd' -print | wc -l`
- `find nautilus_trader -type f \\( -name '*.pyx' -o -name '*.pxd' \\) | sed -E 's#^nautilus_trader/([^/]+).*#\\1#' | sort | uniq -c | sort -nr`
- `python3 scripts/ai/inventory_cython.py`
- `python3 - <<'PY' ... csv.DictReader(...) ... PY`
- `rg -n 'RUST_INCLUDES|PYO3_ONLY|BUILD_MODE == \"debug-pyo3\"|cython-compat|extension-module|Path\\(\"nautilus_trader\"\\).rglob\\(\"\\*\\.pyx\"\\)|cythonize|build_ext|Cython:|CYTHON_COMPILER_DIRECTIVES|Cython.Build' build.py`
- `rg -n 'cython==|Cython.Coverage|doctest-glob|\\.pyx|build.py|cython' pyproject.toml Cargo.toml README.md docs/getting_started/installation.md`
- `rg -l 'Cython|cython|\\.pyx|\\.pxd|as_legacy_cython|cython-compat' build.py pyproject.toml Cargo.toml README.md docs scripts tests nautilus_trader -g '*' | wc -l`
- `rg -l 'Cython|cython|as_legacy_cython|nautilus_pyo3' tests -g '*.py' | wc -l`
- `rg -l 'Cython|cython|\\.pyx|\\.pxd|cython-compat' README.md docs -g '*.md' | wc -l`
- `rg -l 'nautilus_pyo3' nautilus_trader -g '*.pyx' -g '*.pxd' | wc -l`
- `scripts/ai/verify_fast.sh`
- `python3 -m json.tool .agentflow/state/task_status.json`
- `python3 -m json.tool .agentflow/leases/RREM-003.json`
- `python3 scripts/ai/validate_agentflow_roles.py`
- `git diff --check`

## Command Results

- Total Cython source/interface files: 243.
- `.pyx` files: 110.
- `.pxd` files: 133.
- `nautilus_trader/**/*.pyx`: 110.
- `nautilus_trader/**/*.pxd`: 133.
- Generated `docs/rust-cutover/inventory/cython_inventory.csv`: 243 data rows,
  244 lines including header.
- CSV aggregate: 204 files contain `cdef class`, 151 files contain `cpdef`,
  216 files contain `cimport`, and total Cython inventory line count is 117633.
- Largest source/interface families: `model` 95 files, `core` 24,
  `accounting` 18, `indicators` 18, `execution` 17, `backtest` 16,
  `common` 13.
- `build.py` confirms Cython imports, Cython compiler directives,
  `Path("nautilus_trader").rglob("*.pyx")`, `cythonize`, `build_ext`,
  `PYO3_ONLY`, and the `cython-compat` PyO3 build feature.
- `pyproject.toml` confirms `cython==3.2.4` in build and dev dependencies,
  `build.py` as the build script, pytest doctests for `*.pyx`, and
  `Cython.Coverage`.
- Files mentioning Cython/Cython build terms: 327.
- Test files mentioning Cython, `as_legacy_cython`, or `nautilus_pyo3`: 156.
- Docs mentioning Cython/Cython source/build terms: 235.
- Cython source/interface files referencing `nautilus_pyo3`: 30.
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
Later deletion tasks must add migration notes for removed Cython extension
modules, Python import paths, and Rust replacement workflows.

## Rollback Plan

Revert `docs/rust-cutover/inventory/cython_source_build_surfaces.md`,
`docs/rust-cutover/inventory/cython_inventory.csv`,
`docs/rust-cutover/evidence/RREM-003.md`, and the RREM-003 state/lease updates.
