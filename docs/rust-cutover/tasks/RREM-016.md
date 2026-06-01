# RREM-016 - Remove Rust crate Cython build residue and analysis PyO3 annotations

Date: 2026-06-02
Executor: Codex

## Summary

Remove the remaining active Cython generation wiring from Rust crate build
scripts and clear the no-longer-declared PyO3/Python annotation residue from the
`nautilus-analysis` crate.

## Scope

- Remove `cbindgen_cython.toml` files from Rust crates.
- Remove Cython `.pxd` generation from Rust crate `build.rs` files.
- Move remaining C header generation output from the removed Python package
  tree into Cargo's `OUT_DIR`.
- Remove the stale `cython` build dependency from `pyproject.toml`.
- Remove stale Cython wording from Rust source comments where it points to
  removed files or removed build surfaces.
- Remove developer-guide instructions that still tell contributors to add
  Python/PyO3 binding features or generated `.pxd` files.
- Remove `#[cfg_attr(feature = "python", ...)]` PyO3 and stub annotations from
  `crates/analysis`, which no longer declares a `python` Cargo feature.
- Record remaining Rust crate PyO3/Python residue for follow-up cleanup.

## Out of Scope

- Do not delete remaining PyO3/Python annotations outside `crates/analysis`.
- Do not delete Rust crates or adapter modules.
- Do not change trading semantics, serialization semantics, adapter behavior, or
  public Rust runtime behavior.
- Do not execute `RREL-008`.

## Owner and Review

- Owner role: `rust_core_runtime_agent`
- Review role: `verification_release_gatekeeper`
- Risk level: `critical`

## Allowed Paths

- `Cargo.toml`
- `pyproject.toml`
- `crates/core/build.rs`
- `crates/core/cbindgen_cython.toml`
- `crates/common/build.rs`
- `crates/common/cbindgen_cython.toml`
- `crates/backtest/build.rs`
- `crates/backtest/cbindgen_cython.toml`
- `crates/model/build.rs`
- `crates/model/cbindgen_cython.toml`
- `crates/analysis/**`
- Rust source files with comment-only Cython wording updates
- `docs/developer_guide/rust.md`
- `docs/rust-cutover/inventory/core_model_value_types.md`
- `docs/rust-cutover/tasks/RREM-016.md`
- `docs/rust-cutover/evidence/RREM-016.md`
- `docs/rust-cutover/inventory/rust_crate_python_residue.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RREM-016.json`

## Required Evidence

- `rg` checks showing no active Cython/Cython-build references remain in Rust
  crate product paths.
- `scripts/ai/check_cython_removed.sh`
- `scripts/ai/check_no_cython_runtime.sh`
- `rg` checks showing `crates/analysis` has no PyO3/Python annotation residue.
- `cargo check -p nautilus-analysis --all-targets`
- Targeted cargo checks for crates whose build scripts changed.
- `cargo fmt --check`
- `scripts/ai/validate_agentflow_roles.py`
- `scripts/ai/verify_fast.sh`
- `scripts/ai/check_rust_only_runtime.sh` recorded as expected blocker until
  remaining PyO3/Python residue is removed.

## Acceptance

- Rust crate build scripts no longer generate Cython `.pxd` files.
- Rust crate build scripts no longer regenerate `nautilus_trader/` source-tree
  paths for C headers.
- First-party Cython config files for Rust crate cbindgen generation are removed.
- `pyproject.toml` no longer declares `cython`.
- `crates/analysis` no longer contains PyO3/Python binding annotations.
- Remaining PyO3/Python residue is classified and not silently deleted.
- The PR stops at `REVIEW_REQUIRED`; auto-merge is disabled.
