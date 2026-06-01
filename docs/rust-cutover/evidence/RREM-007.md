# RREM-007 Evidence - Stage Removal Of PyO3 Surfaces

Date: 2026-06-01
Executor: Codex
Task ID: RREM-007
Risk: medium
Release gatekeeper approval: user explicitly approved executing RREM-007
through RREM-010 in this thread.

## Summary

Created a non-destructive staged removal plan for PyO3 surfaces. The plan
classifies the `crates/pyo3` aggregator, per-crate `src/python*` bindings,
Cargo metadata, Python package bridge, build bridge, and tests/docs/import
references into removal lanes with explicit gates and blockers.

No Python, PyO3, Cython, runtime, adapter, test, packaging, build, or public
API files were deleted or modified.

## Files Changed

- `docs/rust-cutover/migration/pyo3_surface_removal_stage.md`
- `docs/rust-cutover/evidence/RREM-007.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RREM-007.json`

## Commands Run

- `sed -n '1,220p' docs/rust-cutover/tasks/RREM-007.md`
- `sed -n '1,220p' docs/rust-cutover/TASK_EXECUTION.md`
- `sed -n '1,220p' docs/rust-cutover/CONTRACT.md`
- `sed -n '1,220p' docs/rust-cutover/DEFINITION_OF_DONE.md`
- `sed -n '1,220p' docs/rust-cutover/AGENT_ROLES.md`
- `sed -n '1,240p' docs/rust-cutover/inventory/pyo3_product_surfaces.md`
- `find crates/pyo3 -type f | wc -l`
- `find crates -path '*/src/python*' -type f | wc -l`
- `find crates -path '*/src/python*' -type d | wc -l`
- `rg -l 'pyo3|PyO3|wrap_pymodule|pymodule|pyclass|pymethods' crates -g '*.rs' -g 'Cargo.toml' | wc -l`
- `find crates -path '*/src/python*' -type f | awk -F/ '{print $1"/"$2}' | sort | uniq -c | sort -nr`
- `rg -n 'nautilus-pyo3|pyo3|pyo3-async-runtimes|pyo3-stub-gen|crates/pyo3|debug-pyo3|cython-compat|extension-module' Cargo.toml python/pyproject.toml build.py crates/pyo3/Cargo.toml crates/pyo3/src/lib.rs`
- `PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_fast.sh`
- `python3 -m json.tool .agentflow/state/task_status.json`
- `python3 -m json.tool .agentflow/leases/RREM-007.json`
- `python3 scripts/ai/validate_agentflow_roles.py`
- `git diff --check`

## Command Results

- `crates/pyo3/`: 4 files.
- `crates/**/src/python*`: 371 files.
- `crates/**/src/python*` directories: 60.
- Rust files/manifests mentioning PyO3 binding primitives: 775.
- Largest binding families: `crates/model` 114 files, `crates/adapters` 110,
  `crates/indicators` 44, `crates/analysis` 23, `crates/common` 15.
- Cargo, Python package, and build metadata still reference `nautilus-pyo3`,
  `pyo3`, `pyo3-async-runtimes`, `pyo3-stub-gen`, `crates/pyo3`,
  `debug-pyo3`, `cython-compat`, and `extension-module`.
- `scripts/ai/verify_fast.sh`: passed. Cargo check and clippy remained
  skipped by the script defaults.
- `.agentflow/state/task_status.json`: valid JSON.
- `.agentflow/leases/RREM-007.json`: valid JSON.
- `scripts/ai/validate_agentflow_roles.py`: passed.
- `git diff --check`: passed.

## Tests Added Or Updated

No tests were added or updated. This is a non-destructive staging and evidence
task.

## Behavior Impact

No runtime behavior changed. No trading semantics changed. No Python, PyO3,
Cython, build, packaging, adapter, or public API files were removed or edited.

## Public API Impact

None in this PR. Future deletion of PyO3-backed Python imports requires
migration notes and release-gate approval.

## Migration Note Status

No end-user migration note is required for this staging PR because it does not
remove public APIs. The staging document records that future deletion PRs must
include migration notes for Python imports and Rust replacement workflows.

## Rollback Plan

Revert `docs/rust-cutover/migration/pyo3_surface_removal_stage.md`,
`docs/rust-cutover/evidence/RREM-007.md`, and the RREM-007 state/lease updates.
