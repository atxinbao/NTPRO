# RREM-022 - Remove final Rust crate Python residue

Date: 2026-06-02
Executor: Codex

## Summary

Remove the remaining Python/PyO3 residue from Rust crate product paths after
RREM-021 cleared support crates.

## Scope

- Remove residual Python references from `crates/model`.
- Remove residual Python feature gates and crate docs from `crates/system`.
- Update the Rust crate Python residue inventory and task evidence.

## Out of Scope

- Do not modify `RREL-008`.
- Do not change trading semantics, model enum values, FFI function names,
  kernel behavior, storage formats, or Cargo workspace membership.
- Do not remove C FFI support.

## Owner and Review

- Owner role: `rust_core_runtime_agent`
- Review role: `verification_release_gatekeeper`
- Risk level: `critical`

## Allowed Paths

- `crates/model/**`
- `crates/system/**`
- `docs/rust-cutover/tasks/RREM-022.md`
- `docs/rust-cutover/evidence/RREM-022.md`
- `docs/rust-cutover/inventory/rust_crate_python_residue.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RREM-022.json`

## Required Evidence

- `rg` checks showing no Python/PyO3 residue remains under `crates`.
- `cargo check -p nautilus-model --all-targets`
- `cargo check -p nautilus-system --all-targets`
- `cargo fmt --check`
- `git diff --check`
- `scripts/ai/validate_agentflow_roles.py`
- `scripts/ai/check_rust_only_runtime.sh`
- `scripts/ai/verify_fast.sh`

## Acceptance

- `scripts/ai/check_rust_only_runtime.sh` passes.
- No Python/PyO3/Cython residue remains in Rust crate product paths.
- Rust model C FFI remains available without `Python.h`.
- System builder and kernel tests are no longer gated behind `feature = "python"`.
- The PR stops at `REVIEW_REQUIRED`; auto-merge is disabled.
