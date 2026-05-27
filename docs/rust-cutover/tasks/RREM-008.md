# RREM-008 - Remove Cython source and stale build config

Milestone: R6 Python PyO3 Cython Removal
Priority: P0
Default role: Coding/CI/Architect

## Goal

Remove Cython source and stale build config

## Scope

Remove .pyx/.pxd, build.py, and stale Cython references after parity gates

## Likely files

- `python/`
- `nautilus_trader/`
- `crates/pyo3/`
- `crates/`
- `pyproject.toml`
- `build.py`
- `docs/rust-cutover/`
- `scripts/ai/`

## Non-goals

- Do not refactor unrelated modules.
- Do not change trading semantics unless this task explicitly requires it.
- Do not delete Python, PyO3, or Cython product surfaces unless this task explicitly allows it and required parity evidence already exists.
- Do not change public APIs without a migration note.

## Dependencies

- `RREM-007`

## Acceptance criteria

Removal advances only after Rust usability and parity evidence; blockers are documented

## Required commands

```bash
scripts/ai/verify_fast.sh
```
```bash
scripts/ai/check_no_cython_runtime.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RREM-008.md` with:

- task ID;
- summary;
- files changed;
- commands run;
- command results;
- tests added/updated;
- behavior impact;
- rollback plan.
