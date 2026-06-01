# RREM-018 - Remove adapters indicators common core PyO3 and Python residue

Date: 2026-06-02
Executor: Codex

## Summary

Remove remaining PyO3/Python residue from `crates/adapters`,
`crates/indicators`, `crates/common`, and `crates/core` after the Python
workspace, PyO3 workspace, Cython generation, and model crate cleanup slices
have already landed.

## Scope

- Remove stale `#[cfg_attr(feature = "python", ...)]` PyO3 and stub
  annotations from the four scoped crate areas.
- Remove unreachable `#[cfg(feature = "python")]` implementation blocks,
  imports, helper methods, callback variants, and Python runtime hooks from
  these crates.
- Remove or rewrite scoped README, crate docs, inline comments, and adapter
  fixture notes that still describe active Python/PyO3 product surfaces.
- Update the Rust crate Python residue inventory and task evidence for this
  slice.
- Close out RREM-017 local task state now that PR #113 is merged.

## Out of Scope

- Do not modify Python/PyO3 residue outside `crates/adapters`,
  `crates/indicators`, `crates/common`, and `crates/core`.
- Do not change adapter trading behavior, market-data parsing behavior, order
  routing behavior, indicator formulas, clock semantics, message bus behavior,
  or public Rust runtime semantics.
- Do not delete adapter fixture coverage or parity evidence; update historical
  references instead of removing audit trail.
- Do not execute `RREL-008`.

## Owner and Review

- Owner role: `rust_core_runtime_agent`
- Review role: `verification_release_gatekeeper`
- Risk level: `critical`

## Allowed Paths

- `crates/adapters/**`
- `crates/indicators/**`
- `crates/common/**`
- `crates/core/**`
- `docs/rust-cutover/tasks/RREM-018.md`
- `docs/rust-cutover/evidence/RREM-018.md`
- `docs/rust-cutover/inventory/rust_crate_python_residue.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RREM-017.json`
- `.agentflow/leases/RREM-018.json`

## Required Evidence

- `rg` checks showing no PyO3/Python binding residue remains in the four scoped
  crate areas.
- `cargo check -p nautilus-core --all-targets`
- `cargo check -p nautilus-common --all-targets`
- `cargo check -p nautilus-indicators --all-targets`
- Adapter crate checks that are feasible without external services.
- `cargo fmt --check`
- `scripts/ai/validate_agentflow_roles.py`
- `git diff --check`
- `scripts/ai/verify_fast.sh`
- `scripts/ai/check_rust_only_runtime.sh` recorded as expected blocker if
  remaining non-scoped Rust crate residue still exists.

## Acceptance

- The four scoped crate areas no longer contain active PyO3 attributes,
  `pyo3_stub_gen` annotations, `#[cfg(feature = "python")]` blocks, PyO3
  imports, or Python extension feature documentation.
- Rust-only compile checks pass for the scoped core/common/indicator crates.
- Adapter cleanup preserves Rust adapter behavior and fixture audit history.
- Remaining PyO3/Python residue outside this task scope is classified and not
  silently deleted.
- The PR stops at `REVIEW_REQUIRED`; auto-merge is disabled.
