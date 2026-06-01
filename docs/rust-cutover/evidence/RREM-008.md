# RREM-008 Evidence - Cython Removal Gate Blocked

Date: 2026-06-01
Executor: Codex
Task ID: RREM-008
Risk: medium
Release gatekeeper approval: user explicitly approved executing RREM-007
through RREM-010 in this thread.

## Summary

Ran the Cython removal gate and recorded blockers. The gate is not green:
retained `.pyx` / `.pxd` files and active Cython build/runtime references are
still present. Because the task requires parity evidence before deletion, this
PR does not delete Cython files or build configuration.

No Python, PyO3, Cython, runtime, adapter, test, packaging, build, or public
API files were deleted or modified.

## Files Changed

- `docs/rust-cutover/migration/cython_removal_blockers.md`
- `docs/rust-cutover/evidence/RREM-008.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RREM-008.json`

## Commands Run

- `sed -n '1,220p' docs/rust-cutover/tasks/RREM-008.md`
- `sed -n '1,260p' scripts/ai/check_no_cython_runtime.sh`
- `find . ... \( -name '*.pyx' -o -name '*.pxd' \) -type f -print | sort | wc -l`
- `find nautilus_trader -type f \( -name '*.pyx' -o -name '*.pxd' \) | awk -F/ '{print $1"/"$2}' | sort | uniq -c | sort -nr`
- `rg -n 'Cython|cythonize|Cython\\.Build|\\.pyx|\\.pxd' pyproject.toml build.py setup.py setup.cfg Makefile Cargo.toml crates`
- `scripts/ai/check_no_cython_runtime.sh`
- `PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_fast.sh`
- `python3 -m json.tool .agentflow/state/task_status.json`
- `python3 -m json.tool .agentflow/leases/RREM-008.json`
- `python3 scripts/ai/validate_agentflow_roles.py`
- `git diff --check`

## Command Results

- Retained Cython source/interface files: 243.
- Largest Cython families: `nautilus_trader/model` 95,
  `nautilus_trader/core` 24, `nautilus_trader/indicators` 18,
  `nautilus_trader/accounting` 18, `nautilus_trader/execution` 17,
  `nautilus_trader/backtest` 16.
- Active Cython references remain in `build.py`, `pyproject.toml`, `Cargo.toml`,
  `crates/pyo3`, Rust parity comments, generated-header build scripts, and
  `cbindgen_cython.toml`.
- `scripts/ai/check_no_cython_runtime.sh`: failed, because retained Cython
  source/interface files were found. This is the RREM-008 blocker being
  recorded.
- `scripts/ai/verify_fast.sh`: passed. Cargo check and clippy remained
  skipped by the script defaults.
- `.agentflow/state/task_status.json`: valid JSON.
- `.agentflow/leases/RREM-008.json`: valid JSON.
- `scripts/ai/validate_agentflow_roles.py`: passed.
- `git diff --check`: passed.

## Tests Added Or Updated

No tests were added or updated. This is a gate/blocker evidence task.

## Behavior Impact

No runtime behavior changed. No trading semantics changed. No Python, PyO3,
Cython, build, packaging, adapter, or public API files were removed or edited.

## Public API Impact

None in this PR.

## Migration Note Status

No end-user migration note is required for this blocker PR because it does not
remove public APIs. A future deletion PR must include migration notes for
removed Cython/Python extension modules and replacement Rust workflows.

## Rollback Plan

Revert `docs/rust-cutover/migration/cython_removal_blockers.md`,
`docs/rust-cutover/evidence/RREM-008.md`, and the RREM-008 state/lease updates.
