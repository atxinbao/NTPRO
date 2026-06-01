# RREM-011 - Remove Cython source and build/runtime references

Milestone: R8 Removal
Priority: P0
Default role: Runtime

## Goal

Remove Cython source files and active Cython build/runtime references.

## Scope

Delete `.pyx` and `.pxd` files, remove Cython build dependencies, and remove
active references that would keep the Cython build/runtime path alive.

## Likely files

- `nautilus_trader/**/*.pyx`
- `nautilus_trader/**/*.pxd`
- `build.py`
- `pyproject.toml`
- `Makefile`
- `docs/rust-cutover/evidence/`

## Non-goals

- Do not remove `crates/pyo3/`; that is RREM-012 scope.
- Do not remove the Python package product surface; that is RREM-013 scope.
- Do not mark the Rust-only cutover complete.
- Do not execute RREL-008.

## Dependencies

- `RREL-007`

## Acceptance criteria

- `scripts/ai/check_cython_removed.sh` passes.
- No `.pyx` or `.pxd` files remain outside ignored build/cache paths.
- Active build/runtime paths no longer require Cython.
- Residual Python/PyO3 blockers are documented for later RREM tasks.

## Required commands

```bash
scripts/ai/check_cython_removed.sh
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RREM-011.md` with:

- task ID;
- summary;
- files changed;
- commands run;
- command results;
- tests added/updated;
- behavior impact;
- rollback plan.
