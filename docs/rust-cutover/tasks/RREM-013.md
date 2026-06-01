# RREM-013 - Remove Python product package surfaces

Milestone: R8 Removal
Priority: P0
Default role: Runtime

## Goal

Remove the remaining Python package product surfaces from the Rust-only
cutover workspace.

## Scope

Delete `python/`, delete the top-level `nautilus_trader/` package surface, and
delete `build.py`. Remove active build, verification, and release references
that keep those paths in the Rust-only product path.

## Likely files

- `python/**`
- `nautilus_trader/**`
- `build.py`
- `pyproject.toml`
- `Makefile`
- `.github/workflows/**`
- `scripts/**`
- `docs/rust-cutover/evidence/`
- `docs/rust-cutover/tasks/`

## Non-goals

- Do not remove Python examples, Python tests, or Python-facing docs outside
  the package product paths in this task.
- Do not mark the Rust-only cutover complete.
- Do not execute RREL-008.

## Dependencies

- `RREM-012`

## Acceptance criteria

- `python/` is removed.
- `nautilus_trader/` is removed.
- `build.py` is removed.
- Active Rust-only build and verification paths no longer require those product
  paths.
- Remaining release blockers are documented for the final release gate.

## Required commands

```bash
cargo metadata --format-version=1
scripts/ai/check_rust_only_runtime.sh
scripts/ai/verify_fast.sh
python3 -m json.tool .agentflow/state/task_status.json
python3 -m json.tool .agentflow/leases/RREM-013.json
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
```

## Evidence required

Create `docs/rust-cutover/evidence/RREM-013.md` with:

- task ID;
- summary;
- files changed;
- commands run;
- command results;
- residual release blockers;
- behavior impact;
- public API impact;
- migration note status;
- rollback plan.
