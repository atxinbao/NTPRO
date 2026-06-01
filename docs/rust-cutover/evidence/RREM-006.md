# RREM-006 Evidence - Stage Removal Of Python Package Surfaces

Date: 2026-06-01
Executor: Codex
Task ID: RREM-006
Risk: medium
Release gatekeeper approval: user explicitly approved executing RREM-006 in
this thread.

## Summary

Created a non-destructive staged removal plan for the Python package surfaces
under `python/` and top-level `nautilus_trader/`. The plan defines removal
lanes, gate order, blockers, later allowed outcomes, and a checklist for any
future deletion PR.

No Python, PyO3, Cython, runtime, adapter, test, packaging, build, or public
API files were deleted or modified.

## Files Changed

- `docs/rust-cutover/migration/python_package_surface_removal_stage.md`
- `docs/rust-cutover/evidence/RREM-006.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RREM-006.json`

## Commands Run

- `rg -n "NTPRO|RREM|rust-cutover|auto-dispatch" /Users/mac/.codex/memories/MEMORY.md || true`
- `git status --short --branch`
- `rg -n "RREM-006|RREM-005|RREM-007" docs .agentflow /Users/mac/.codex/shrimp-data/NTPRO/tasks.json`
- Shrimp `query_task` for `RREM-006`
- Shrimp `get_task_detail` for `RREM-006`
- Shrimp `analyze_task` and `reflect_task`
- Shrimp `process_thought`
- `sed -n '1,220p' docs/rust-cutover/tasks/RREM-006.md`
- `sed -n '1,220p' docs/rust-cutover/TASK_EXECUTION.md`
- `sed -n '1,220p' docs/rust-cutover/CONTRACT.md`
- `sed -n '1,220p' docs/rust-cutover/DEFINITION_OF_DONE.md`
- `sed -n '1,220p' docs/rust-cutover/AGENT_ROLES.md`
- `sed -n '1340,1415p' .agentflow/state/task_status.json`
- `sed -n '1,240p' .agentflow/policies/path_scope.yaml`
- `sed -n '1,240p' .agentflow/roles.yaml`
- `ls .agentflow/leases && find .agentflow/leases -maxdepth 1 -type f -name '*.json' -print`
- `sed -n '1,220p' docs/rust-cutover/migration/python_to_rust_workflow_map.md`
- `sed -n '1,220p' docs/rust-cutover/migration/python_test_scope_map.md`
- `sed -n '1,240p' docs/rust-cutover/inventory/python_product_surfaces.md`
- `sed -n '1,220p' docs/rust-cutover/evidence/RREM-001.md`
- `sed -n '1,220p' docs/rust-cutover/inventory/pyo3_product_surfaces.md`
- `sed -n '1,220p' docs/rust-cutover/inventory/cython_source_build_surfaces.md`
- `find python -type f | wc -l`
- `find python/nautilus_trader -maxdepth 2 -type d | wc -l`
- `find nautilus_trader -type f \( -name '*.py' -o -name '*.pyi' -o -name '*.pyx' -o -name '*.pxd' \) | wc -l`
- `find python -type f | sed -n 's/.*\\.//p' | sort | uniq -c | sort -nr`
- `find nautilus_trader -type f \( -name '*.py' -o -name '*.pyi' -o -name '*.pyx' -o -name '*.pxd' \) | sed -n 's/.*\\.//p' | sort | uniq -c | sort -nr`
- `find examples -type f \( -name '*.py' -o -name '*.ipynb' \) | wc -l`
- `find tests -type f -name '*.py' | wc -l`
- `rg -l 'from nautilus_trader|import nautilus_trader|pip install|Python 3|```python' docs README.md | wc -l`
- `PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_fast.sh`
- `python3 -m json.tool .agentflow/state/task_status.json`
- `python3 -m json.tool .agentflow/leases/RREM-006.json`
- `python3 scripts/ai/validate_agentflow_roles.py`
- `git diff --check`

## Command Results

- Shrimp task `RREM-006 Stage removal of Python package surfaces` was found
  as pending before execution.
- `docs/rust-cutover/tasks/RREM-006.md` requires staging removal and explicitly
  says not to delete Python, PyO3, or Cython product surfaces unless the task
  allows it and required parity evidence already exists.
- `python/`: 178 files.
- `python/nautilus_trader`: 50 shallow module directories.
- top-level `nautilus_trader/`: 642 Python/Cython/interface files.
- top-level `nautilus_trader/` extension counts: 395 `.py`, 133 `.pxd`, 110
  `.pyx`, 4 `.pyi`.
- `python/` overlay extension counts: 123 `.py`, 50 `.pyi`, 2 `.md`, 1
  `.typed`, 1 `.toml`, 1 `.lock`.
- Python examples and notebooks: 137 files.
- top-level Python tests: 534 files.
- README/docs Python-facing references: 77 files.
- `scripts/ai/verify_fast.sh`: passed. Cargo check and clippy remained
  skipped by the script defaults.
- `.agentflow/state/task_status.json`: valid JSON.
- `.agentflow/leases/RREM-006.json`: valid JSON.
- `scripts/ai/validate_agentflow_roles.py`: passed.
- `git diff --check`: passed.

## Tests Added Or Updated

No tests were added or updated. This is a non-destructive staging and evidence
task.

## Behavior Impact

No runtime behavior changed. No trading semantics changed. No Python, PyO3,
Cython, build, packaging, adapter, or public API files were removed or edited.

## Public API Impact

None in this PR. Future deletion of Python imports or package paths will
require migration notes and release-gate approval.

## Migration Note Status

No end-user migration note is required for this staging PR because it does not
remove or change a public API. The staged-removal document records that future
deletion PRs must include migration notes for Python imports, examples, docs,
installation commands, and build workflows.

## Rollback Plan

Revert `docs/rust-cutover/migration/python_package_surface_removal_stage.md`,
`docs/rust-cutover/evidence/RREM-006.md`, and the RREM-006 state/lease updates.
