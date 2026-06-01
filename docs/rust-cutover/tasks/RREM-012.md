# RREM-012 - Remove PyO3 workspace and Python binding surfaces

Milestone: R8 Removal
Priority: P0
Default role: Runtime

## Goal

Remove the PyO3 workspace crate, per-crate Rust Python binding modules, and
active build/runtime references that keep the PyO3 bridge alive.

## Scope

Delete `crates/pyo3/`, delete `crates/**/src/python`, and remove active PyO3,
maturin, `extension-module`, and Python-binding Cargo/build/CI references from
the Rust v2 product path.

## Likely files

- `crates/pyo3/**`
- `crates/**/src/python/**`
- `crates/**/Cargo.toml`
- `Cargo.toml`
- `Cargo.lock`
- `build.py`
- `Makefile`
- `python/pyproject.toml`
- `.github/workflows/**`
- `scripts/**`
- `docs/rust-cutover/evidence/`

## Non-goals

- Do not remove the Python package product surface under `python/`.
- Do not remove the legacy `nautilus_trader/` package surface.
- Do not mark the Rust-only cutover complete.
- Do not execute RREL-008.

## Dependencies

- `RREM-011`

## Acceptance criteria

- `crates/pyo3/` is removed.
- No `crates/**/src/python` directories remain.
- Cargo metadata no longer exposes `nautilus-pyo3` or PyO3 dependencies.
- Active build/CI paths no longer invoke maturin or PyO3 extension-module builds.
- Remaining blockers for Python package/product removal are documented for
  RREM-013.

## Required commands

```bash
cargo metadata --format-version=1
scripts/ai/verify_fast.sh
python3 -m json.tool .agentflow/state/task_status.json
python3 -m json.tool .agentflow/leases/RREM-012.json
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
```

## Evidence required

Create `docs/rust-cutover/evidence/RREM-012.md` with:

- task ID;
- summary;
- files changed;
- commands run;
- command results;
- residual Python product blockers;
- behavior impact;
- rollback plan.
