# RREM-015 - Remove Cap'n Proto feature and audit residual Python surfaces

Date: 2026-06-02
Executor: Codex

## Summary

Remove the optional Cap'n Proto serialization feature from the Rust workspace and
record a file-level audit of the remaining Python/PyO3/Cython cleanup surface.

## Scope

- Remove active Cap'n Proto feature wiring from Cargo manifests.
- Remove Cap'n Proto schema files, generated Rust files, conversion modules,
  tests, benches, regeneration scripts, CI hooks, Makefile targets, and public
  docs that advertise the removed feature.
- Update Rust serialization docs to describe the remaining supported formats.
- Record the remaining Python files by surface area, especially `tests/**/*.py`
  and control scripts.
- Keep `RREL-008` paused.

## Out of Scope

- Do not delete `tests/**/*.py` in this task.
- Do not delete `scripts/control/*.py` or `scripts/ai/*.py` unless they are
  Cap'n Proto-specific.
- Do not make trading semantic changes.
- Do not execute `RREL-008`.

## Owner and Review

- Owner role: `rust_core_runtime_agent`
- Review role: `verification_release_gatekeeper`
- Risk level: `critical`

## Allowed Paths

- `Cargo.toml`
- `Cargo.lock`
- `Makefile`
- `.github/**`
- `.pre-commit-config.yaml`
- `crates/serialization/**`
- `crates/common/**`
- `scripts/*capnp*`
- `docs/**`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RREM-015.json`
- `docs/rust-cutover/tasks/RREM-015.md`
- `docs/rust-cutover/evidence/RREM-015.md`
- `docs/rust-cutover/inventory/residual_python_capnp_cleanup.md`
- `docs/rust-cutover/migration/capnp_feature_removed.md`

## Required Evidence

- `cargo metadata --format-version=1 --no-deps`
- `cargo check -p nautilus-serialization --all-targets`
- `cargo check -p nautilus-common --all-targets`
- `cargo fmt --check`
- `scripts/ai/verify_fast.sh`
- `rg` checks showing no active Cap'n Proto feature/build/test/doc entrypoints
  remain.
- File-count audit for remaining tracked Python files.

## Acceptance

- `nautilus-serialization` no longer exposes a `capnp` feature.
- `nautilus-common` no longer forwards a `capnp` feature.
- Cap'n Proto schema/generated/conversion files and Cap'n Proto-only tests or
  benches are removed.
- CI, Makefile, pre-commit, and docs no longer advertise Cap'n Proto commands.
- Remaining Python files are classified and not silently deleted.
- The PR stops at `REVIEW_REQUIRED`; auto-merge is disabled.
