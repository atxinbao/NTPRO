# RREM-021 - Remove support crate PyO3 and Python residue

Date: 2026-06-02
Executor: Codex

## Summary

Remove scoped PyO3/Python residue from support Rust crates after RREM-020
cleared runtime-facing crates.

## Scope

- Remove Python/PyO3 product-surface residue from:
  - `crates/persistence`
  - `crates/infrastructure`
  - `crates/plugin`
  - `crates/testkit`
  - `crates/cryptography`
- Remove PyO3/stub annotations and Python-gated blocks from the scoped crates.
- Remove PyO3 generation support from `crates/persistence/macros` while keeping
  Rust-only `custom_data` generation behavior.
- Rewrite scoped docs, READMEs, tests, and comments that still describe active
  Python/PyO3 product surfaces.
- Update the Rust crate Python residue inventory and task evidence.
- Close local RREM-020 task state after PR #116 was merged.

## Out of Scope

- Do not modify remaining residue in `crates/model` or `crates/system`.
- Do not change catalog storage schemas, Arrow/Parquet formats, database
  behavior, Redis behavior, plug-in ABI behavior, or cryptographic algorithms.
- Do not remove Rust-native tests that are not Python/PyO3 gated.
- Do not execute `RREL-008`.

## Owner and Review

- Owner role: `rust_core_runtime_agent`
- Review role: `verification_release_gatekeeper`
- Risk level: `critical`

## Allowed Paths

- `crates/persistence/**`
- `crates/infrastructure/**`
- `crates/plugin/**`
- `crates/testkit/**`
- `crates/cryptography/**`
- `docs/rust-cutover/tasks/RREM-021.md`
- `docs/rust-cutover/evidence/RREM-021.md`
- `docs/rust-cutover/inventory/rust_crate_python_residue.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RREM-020.json`
- `.agentflow/leases/RREM-021.json`

## Required Evidence

- `rg` checks showing no PyO3/Python binding residue remains in the scoped
  support crates.
- `cargo check -p nautilus-persistence-macros --all-targets`
- `cargo check -p nautilus-persistence --all-targets`
- `cargo check -p nautilus-infrastructure --all-targets`
- `cargo check -p nautilus-plugin --all-targets`
- `cargo check -p nautilus-testkit --all-targets`
- `cargo check -p nautilus-cryptography --all-targets`
- `cargo fmt --check`
- `scripts/ai/validate_agentflow_roles.py`
- `git diff --check`
- `scripts/ai/verify_fast.sh`
- `scripts/ai/check_rust_only_runtime.sh` recorded as expected blocker if
  remaining non-scoped Rust crate residue still exists.

## Acceptance

- Scoped support crates no longer contain active PyO3 attributes,
  `pyo3_stub_gen` annotations, `#[cfg(feature = "python")]` blocks, PyO3
  imports, Python extension feature documentation, Python-facing macro options,
  or Python-only tests.
- Rust-only `custom_data` macro behavior still compiles.
- Scoped Rust cargo checks pass or any failure is documented with a concrete
  non-scoped blocker.
- Remaining PyO3/Python residue outside this task scope is classified and not
  silently deleted.
- The PR stops at `REVIEW_REQUIRED`; auto-merge is disabled.
