# RREM-017 - Remove model crate PyO3 and Python residue

Date: 2026-06-02
Executor: Codex

## Summary

Remove PyO3/Python binding residue from `crates/model` now that the model crate
no longer declares a `python` Cargo feature or depends on PyO3.

## Scope

- Remove `#[cfg_attr(feature = "python", ...)]` PyO3 and stub annotations from
  model data, identifiers, enums, events, accounts, orders, instruments,
  orderbook, reports, DeFi, and type definitions.
- Remove `#[cfg(feature = "python")]` model custom-data wrappers, Python
  extractor registries, Python conversion helpers, and Python-only tests.
- Convert model `#[custom_data(pyo3)]` declarations to Rust-only
  `#[custom_data]` declarations while preserving JSON and Arrow custom-data
  behavior.
- Remove stale model crate documentation and comments that still describe
  Python/PyO3 feature flags or Python-specific constructor requirements.
- Update Rust-cutover evidence and task state for this slice.

## Out of Scope

- Do not modify PyO3/Python residue outside `crates/model`.
- Do not modify the persistence `custom_data` procedural macro in this task.
- Do not delete adapter, indicator, common, execution, trading, live, backtest,
  network, infrastructure, or persistence crate residue.
- Do not change trading semantics, serialization semantics, adapter behavior, or
  public Rust runtime behavior.
- Do not execute `RREL-008`.

## Owner and Review

- Owner role: `rust_core_runtime_agent`
- Review role: `verification_release_gatekeeper`
- Risk level: `critical`

## Allowed Paths

- `crates/model/**`
- `docs/rust-cutover/tasks/RREM-017.md`
- `docs/rust-cutover/evidence/RREM-017.md`
- `docs/rust-cutover/inventory/rust_crate_python_residue.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RREM-016.json`
- `.agentflow/leases/RREM-017.json`

## Required Evidence

- `rg` checks showing no PyO3/Python binding residue remains in
  `crates/model`.
- `cargo check -p nautilus-model --all-targets`
- `cargo check -p nautilus-model --all-targets --features ffi`
- `cargo check -p nautilus-model --all-targets --features arrow`
- `cargo fmt --check`
- `scripts/ai/validate_agentflow_roles.py`
- `git diff --check`
- `scripts/ai/verify_fast.sh`
- `scripts/ai/check_rust_only_runtime.sh` recorded as expected blocker until
  remaining non-model PyO3/Python residue is removed.

## Acceptance

- `crates/model` no longer contains PyO3 or Python binding attributes, imports,
  wrappers, registry helpers, feature gates, tests, or documentation.
- Rust JSON custom-data registration remains available.
- Rust Arrow custom-data registration remains available behind the `arrow`
  feature.
- Rust model FFI still compiles behind the `ffi` feature.
- Remaining PyO3/Python residue outside `crates/model` is classified and not
  silently deleted.
- The PR stops at `REVIEW_REQUIRED`; auto-merge is disabled.
