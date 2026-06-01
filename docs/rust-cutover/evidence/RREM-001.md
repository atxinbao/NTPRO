# RREM-001 Evidence - Inventory Python Product Surfaces

Date: 2026-06-01
Executor: Codex
Task ID: RREM-001
Risk: medium
Release gatekeeper approval: user approved RREM-001/RREM-002/RREM-003 automation in this thread.

## Summary

Created a non-destructive inventory of Python product-facing surfaces that
block the Rust-only cutover. The inventory records package overlays, top-level
Python package modules, Python examples, Python tests, Python docs, and
packaging/build files that must be ported, scoped, or removed in later gated
tasks.

No Python, PyO3, Cython, runtime, adapter, or public API files were deleted or
modified.

## Files Changed

- `docs/rust-cutover/inventory/python_product_surfaces.md`
- `docs/rust-cutover/evidence/RREM-001.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RREM-001.json`

## Commands Run

- `sed -n '1,240p' AGENTS.md`
- `sed -n '1,240p' docs/rust-cutover/CONTRACT.md`
- `sed -n '1,240p' docs/rust-cutover/DEFINITION_OF_DONE.md`
- `sed -n '1,240p' docs/rust-cutover/TASK_EXECUTION.md`
- `sed -n '1,260p' docs/rust-cutover/AGENT_ROLES.md`
- `sed -n '1,220p' docs/rust-cutover/tasks/RREM-001.md`
- `find python -type f | sed -n 's/.*\\.//p' | sort | uniq -c | sort -nr`
- `find nautilus_trader -type f \\( -name '*.py' -o -name '*.pyi' -o -name '*.pyx' -o -name '*.pxd' \\) | sed -n 's/.*\\.//p' | sort | uniq -c | sort -nr`
- `find examples -type f \\( -name '*.py' -o -name '*.ipynb' \\) | sed -n 's/.*\\.//p' | sort | uniq -c | sort -nr`
- `find tests -type f \\( -name '*.py' -o -name '*.ipynb' \\) | sed -n 's/.*\\.//p' | sort | uniq -c | sort -nr`
- `find docs -type f \\( -name '*.py' -o -name '*.ipynb' \\) | sed -n 's/.*\\.//p' | sort | uniq -c | sort -nr`
- `rg -l 'from nautilus_trader|import nautilus_trader|pip install|Python 3|```python' docs README.md | wc -l`
- `scripts/ai/verify_fast.sh`

## Command Results

- `python/`: 178 files, including 123 `.py`, 50 `.pyi`, `python/pyproject.toml`, and package docs/lock files.
- `nautilus_trader/`: 642 Python/Cython/interface files, including 395 `.py`, 133 `.pxd`, 110 `.pyx`, and 4 `.pyi`.
- `examples/`: 137 Python-facing example files, including 136 `.py` and 1 `.ipynb`.
- `tests/`: 534 Python test files.
- `docs/`: 20 Python tutorial/render files.
- README/docs Python-facing references: 75 files.
- `scripts/ai/verify_fast.sh`: passed.

## Tests Added Or Updated

No tests were added or updated. This is an inventory-only task.

## Behavior Impact

No runtime behavior changed. No trading semantics changed. No Python, PyO3, or
Cython product surfaces were removed.

## Public API Impact

None.

## Migration Note Status

No migration note required for this task because it only records inventory.
Later removal tasks will require migration notes for removed Python import paths
and user workflows.

## Rollback Plan

Revert `docs/rust-cutover/inventory/python_product_surfaces.md`,
`docs/rust-cutover/evidence/RREM-001.md`, and the RREM-001 state/lease updates.
