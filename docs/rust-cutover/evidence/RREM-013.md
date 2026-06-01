# RREM-013 Evidence - Remove Python Product Package Surfaces

Date: 2026-06-02
Executor: Codex
Task ID: RREM-013
Risk: critical

## Summary

Removed the remaining Python package product surfaces from the Rust-only
cutover workspace:

- removed `python/` (178 tracked files);
- removed top-level `nautilus_trader/` (405 tracked files);
- removed `build.py`;
- retired active references to those paths in Makefile targets, pre-commit
  config, CI guardrails, security-audit lockfile inputs, and the v2 Python
  package build workflow;
- converted root `pyproject.toml` from Python package build metadata to local
  tooling metadata by removing the Poetry build backend, wheel include paths,
  and `build.py` build script reference.

This is a critical removal task. It stops at `REVIEW_REQUIRED`; auto-merge is
not enabled.

## Files Changed

- Deleted:
  - `python/**`
  - `nautilus_trader/**`
  - `build.py`
- Updated:
  - `.agentflow/leases/RREM-013.json`
  - `.agentflow/state/task_status.json`
  - `.github/CODEOWNERS`
  - `.github/workflows/build-v2.yml`
  - `.github/workflows/build.yml`
  - `.github/workflows/security-audit.yml`
  - `.pre-commit-config.yaml`
  - `Makefile`
  - `pyproject.toml`
  - `scripts/check-no-build-packages.sh`
  - `scripts/ci/security-audit-gate.sh`
  - `scripts/ci/verify-ci-inputs.sh`
  - `docs/rust-cutover/migration/python_product_surface_removed.md`
  - `docs/rust-cutover/tasks/RREM-013.md`
  - `docs/rust-cutover/evidence/RREM-013.md`

## Commands Run

```bash
for p in python nautilus_trader build.py crates/pyo3; do
  if [ -e "$p" ]; then echo "present $p"; else echo "removed $p"; fi
done
cargo metadata --format-version=1
scripts/ai/check_rust_only_runtime.sh
scripts/ai/verify_fast.sh
python3 -m json.tool .agentflow/state/task_status.json
python3 -m json.tool .agentflow/leases/RREM-013.json
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
cargo fmt --check
python3 - <<'PY'
from pathlib import Path
import tomllib
for path in ['pyproject.toml', '.cargo/config.toml', 'Cargo.toml']:
    tomllib.loads(Path(path).read_text())
    print(f'OK {path}')
PY
scripts/check-no-build-packages.sh
scripts/ai/check_cython_removed.sh
scripts/ai/check_no_cython_runtime.sh
ruby -e 'require "yaml"; %w[.github/workflows/build-v2.yml .github/workflows/build.yml .github/workflows/security-audit.yml .pre-commit-config.yaml].each { |p| YAML.load_file(p); puts "YAML OK #{p}" }'
bash -n scripts/check-no-build-packages.sh scripts/ci/security-audit-gate.sh scripts/ci/verify-ci-inputs.sh
uv lock --locked
taplo fmt --check pyproject.toml
```

## Command Results

- Product path check: `python`, `nautilus_trader`, `build.py`, and
  `crates/pyo3` are absent.
- `cargo metadata --format-version=1`: passed.
- `scripts/ai/verify_fast.sh`: passed.
- `python3 -m json.tool .agentflow/state/task_status.json`: passed.
- `python3 -m json.tool .agentflow/leases/RREM-013.json`: passed.
- `python3 scripts/ai/validate_agentflow_roles.py`: passed.
- `git diff --check`: passed.
- `cargo fmt --check`: passed.
- TOML syntax check via Python `tomllib`: passed for `pyproject.toml`,
  `.cargo/config.toml`, and `Cargo.toml`.
- `scripts/check-no-build-packages.sh`: passed after adding the locked
  `cython` package to root `no-build-package`.
- `scripts/ai/check_cython_removed.sh`: passed.
- Workflow/pre-commit YAML syntax via Ruby `YAML.load_file`: passed.
- Shell syntax via `bash -n` for edited scripts: passed.
- `scripts/ai/check_rust_only_runtime.sh`: failed as expected because active
  Rust source paths still retain PyO3 annotations and Cython generation/parity
  references.
- `scripts/ai/check_no_cython_runtime.sh`: failed as expected because active
  Rust source paths still retain Cython generation/parity references.
- `uv lock --locked`: not usable in the local shell because local `uv` is
  `0.11.12` while `pyproject.toml` requires `0.11.14`.
- `taplo fmt --check pyproject.toml`: not run because `taplo` is not installed
  in the local shell.

## Residual Release Blockers

- Rust-only runtime gate remains blocked by PyO3 annotations and Python module
  metadata still embedded across Rust crates, for example
  `pyo3::pyclass(...)`, `pyo3_stub_gen::derive::*`, and
  `nautilus_trader.core.nautilus_pyo3.*` module strings.
- Cython runtime gate remains blocked by Rust crate build/parity references,
  including `crates/core/build.rs`, `crates/backtest/build.rs`,
  `crates/common/build.rs`, `crates/model/build.rs`, and
  `crates/*/cbindgen_cython.toml`.
- Root `pyproject.toml` and `uv.lock` remain for Python tooling metadata used
  by pre-commit and helper scripts. They are no longer package build metadata,
  but final release may still choose to remove or replace them.
- Historical Python-facing docs, examples, issue templates, and release scripts
  still contain migration references. They are not active product package
  paths in this task.

## Behavior Impact

Python package imports and Python wheel/package builds are removed from this
workspace. Rust workspace metadata and fast verification continue to work.

No trading semantics, matching logic, risk rules, portfolio accounting, adapter
runtime behavior, or persistence format was intentionally changed.

## Public API Impact

Breaking change: the Python package product API is removed. Users should no
longer rely on `import nautilus_trader` or the `python/nautilus_trader` overlay
package in this cutover workspace.

Rust crate and CLI APIs were not intentionally changed by this task.

## Migration Note Status

Added `docs/rust-cutover/migration/python_product_surface_removed.md`.

## Rollback Plan

Revert this PR to restore the removed Python product package surfaces and any
build or verification references needed for them.
