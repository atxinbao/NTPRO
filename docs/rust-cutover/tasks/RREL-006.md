# RREL-006 - Run final Rust-only release verification

Milestone: R7 Release
Priority: P0
Default role: Release

## Goal

Run final Rust-only release verification

## Scope

Run verify_release.sh and final gates

## Likely files

- `docs/rust-cutover/migration/`
- `docs/rust-cutover/release/`
- `completion/`
- `scripts/ai/`

## Non-goals

- Do not refactor unrelated modules.
- Do not change trading semantics unless this task explicitly requires it.
- Do not delete Python, PyO3, or Cython product surfaces unless this task explicitly allows it and required parity evidence already exists.
- Do not change public APIs without a migration note.

## Dependencies

- `RREL-005`

## Acceptance criteria

Release evidence exists and all blockers are documented

## Required commands

```bash
scripts/ai/verify_release.sh
```
```bash
scripts/ai/check_rust_only_runtime.sh
```
```bash
scripts/ai/check_cython_removed.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RREL-006.md` with:

- task ID;
- summary;
- files changed;
- commands run;
- command results;
- tests added/updated;
- behavior impact;
- rollback plan.
