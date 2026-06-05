# NAUDIT-001 - Python package metadata cleanup and Rust-only gate hardening

Milestone: v0.2.0 Audit Backlog
Priority: P0
Default role: Control & Scope
Risk: critical

## Goal

Resolve the conflict between the Rust-only public product position and the root
Python package metadata still present in `pyproject.toml`.

## Scope

- Decide whether root `pyproject.toml` is retained only as local tool config or
  moved/split into a tools-only path.
- Remove or isolate root `[project]` Python package metadata from the product
  surface.
- Remove or isolate Python classifiers, Python runtime dependencies, optional
  product extras, upstream package URLs, and Python-package publication signals.
- Strengthen `scripts/ai/check_rust_only_runtime.sh` so it fails on root Python
  package metadata while still allowing repository helper scripts.
- Update docs to explain that Python is allowed only for local control scripts,
  not as a product package.

## Likely files

- `pyproject.toml`
- `uv.lock`
- `scripts/ai/check_rust_only_runtime.sh`
- `docs/rust-cutover/`
- `README.md`

## Non-goals

- Do not reintroduce Python/PyO3/Cython product surfaces.
- Do not publish a package.
- Do not create release tags.

## Dependencies

- none

## Acceptance criteria

- Root product metadata no longer presents NTPRO as a Python package.
- Rust-only runtime check fails if root Python package metadata reappears.
- Local Python helper tooling remains documented as non-product.
- Evidence records the exact metadata decision and rollback plan.

## Required commands

```bash
git diff --check
scripts/ai/check_rust_only_runtime.sh
scripts/ai/check_cython_removed.sh
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/NAUDIT-001.md`.
