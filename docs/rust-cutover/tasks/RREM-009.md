# RREM-009 - Run Rust-only runtime gate

Milestone: R6 Python PyO3 Cython Removal
Priority: P0
Default role: Coding/CI/Architect

## Goal

Run Rust-only runtime gate

## Scope

Run check_rust_only_runtime.sh and record blockers

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

- `RREM-008`

## Acceptance criteria

Removal advances only after Rust usability and parity evidence; blockers are documented

## Required commands

```bash
scripts/ai/verify_full.sh
```
```bash
scripts/ai/check_rust_only_runtime.sh
```
```bash
scripts/ai/check_cython_removed.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RREM-009.md` with:

- task ID;
- summary;
- files changed;
- commands run;
- command results;
- tests added/updated;
- behavior impact;
- rollback plan.
