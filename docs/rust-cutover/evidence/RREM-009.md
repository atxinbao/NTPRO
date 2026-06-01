# RREM-009 Evidence - Rust-Only Runtime Gate Blocked

Date: 2026-06-01
Executor: Codex
Task ID: RREM-009
Risk: medium
Release gatekeeper approval: user explicitly approved executing RREM-007
through RREM-010 in this thread.

## Summary

Ran the Rust-only runtime gate checks and recorded blockers. The gate is not
green: Python, PyO3, Cython, build, and runtime product surfaces remain in
active paths. This PR records the gate status and does not delete any product
surface.

## Files Changed

- `docs/rust-cutover/migration/rust_only_runtime_gate_blockers.md`
- `docs/rust-cutover/evidence/RREM-009.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RREM-009.json`

## Commands Run

- `scripts/ai/check_rust_only_runtime.sh`
- `scripts/ai/check_cython_removed.sh`
- `scripts/ai/verify_full.sh`
- `PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_full.sh`
- `for p in python nautilus_trader crates/pyo3 build.py; do [ -e "$p" ] && echo "$p"; done | wc -l`
- `find crates -path '*/src/python' -type d | wc -l`
- `find . ... \( -name '*.pyx' -o -name '*.pxd' \) -print | wc -l`
- `grep -RIlE 'pyo3|maturin|Cython|cythonize|\\.pyx|\\.pxd' Cargo.toml Cargo.lock crates Makefile pyproject.toml setup.py setup.cfg | wc -l`
- `PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_fast.sh`
- `python3 -m json.tool .agentflow/state/task_status.json`
- `python3 -m json.tool .agentflow/leases/RREM-009.json`
- `python3 scripts/ai/validate_agentflow_roles.py`
- `git diff --check`

## Command Results

- `scripts/ai/check_rust_only_runtime.sh`: failed.
  - retained product paths: `python`, `nautilus_trader`, `crates/pyo3`,
    `build.py`;
  - retained `crates/**/src/python` modules;
  - retained Cython `.pyx` / `.pxd` files;
  - active Python/PyO3/Cython build/runtime references remain.
- `scripts/ai/check_cython_removed.sh`: failed because Cython `.pyx` / `.pxd`
  files remain.
- `scripts/ai/verify_full.sh` with default environment: failed because the
  default rustc was below the workspace requirement.
- `scripts/ai/verify_full.sh` with Rust 1.95.0: timed out after 180 seconds
  during workspace clippy.
- Retained product paths among `python`, `nautilus_trader`, `crates/pyo3`,
  `build.py`: 4.
- Retained `crates/**/src/python` directories: 36.
- Retained `.pyx` / `.pxd` files: 243.
- Active files matching Python/PyO3/Cython build/runtime reference patterns:
  839.
- `scripts/ai/verify_fast.sh`: passed. Cargo check and clippy remained
  skipped by the script defaults.
- `.agentflow/state/task_status.json`: valid JSON.
- `.agentflow/leases/RREM-009.json`: valid JSON.
- `scripts/ai/validate_agentflow_roles.py`: passed.
- `git diff --check`: passed.

## Tests Added Or Updated

No tests were added or updated. This is a gate evidence task.

## Behavior Impact

No runtime behavior changed. No trading semantics changed. No Python, PyO3,
Cython, build, packaging, adapter, or public API files were removed or edited.

## Public API Impact

None in this PR.

## Migration Note Status

No end-user migration note is required for this blocker PR because it does not
remove public APIs. Future deletion/replacement PRs must include migration
notes before the final Rust-only gate can pass.

## Rollback Plan

Revert `docs/rust-cutover/migration/rust_only_runtime_gate_blockers.md`,
`docs/rust-cutover/evidence/RREM-009.md`, and the RREM-009 state/lease updates.
