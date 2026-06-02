# RREM-020 - Remove runtime crate PyO3 and Python residue

Date: 2026-06-02
Executor: Codex

## Summary

Remove scoped PyO3/Python residue from runtime-facing Rust crates after
RREM-019 left 81 Rust crate files with remaining Python/PyO3 hits.

## Scope

- Remove stale `#[cfg_attr(feature = "python", ...)]` PyO3 and stub
  annotations from:
  - `crates/execution`
  - `crates/backtest`
  - `crates/live`
  - `crates/trading`
  - `crates/risk`
  - `crates/portfolio`
  - `crates/data`
- Remove or rewrite scoped crate docs, examples, tests, and comments that still
  describe active Python/PyO3 product surfaces.
- Remove Python-only runtime bridge code in the scoped crates only after
  confirming it is not part of the Rust-only runtime contract.
- Update the Rust crate Python residue inventory and task evidence for this
  slice.
- Close local RREM-019 task state after PR #115 was merged.

## Out of Scope

- Do not modify remaining residue in `crates/persistence`,
  `crates/infrastructure`, `crates/plugin`, `crates/testkit`,
  `crates/cryptography`, `crates/model`, or `crates/system`.
- Do not change backtest, live, trading, execution, risk, portfolio, or data
  runtime semantics.
- Do not change order matching, reconciliation, risk rejection, PnL, accounting,
  event ordering, actor scheduling, or adapter behavior.
- Do not remove Rust-native tests that are not Python/PyO3 gated.
- Do not execute `RREL-008`.

## Owner and Review

- Owner role: `rust_core_runtime_agent`
- Review role: `verification_release_gatekeeper`
- Risk level: `critical`

## Allowed Paths

- `crates/execution/**`
- `crates/backtest/**`
- `crates/live/**`
- `crates/trading/**`
- `crates/risk/**`
- `crates/portfolio/**`
- `crates/data/**`
- `docs/rust-cutover/tasks/RREM-020.md`
- `docs/rust-cutover/evidence/RREM-020.md`
- `docs/rust-cutover/inventory/rust_crate_python_residue.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RREM-019.json`
- `.agentflow/leases/RREM-020.json`

## Required Evidence

- `rg` checks showing no PyO3/Python binding residue remains in the scoped
  runtime-facing crates.
- `cargo check -p nautilus-execution --all-targets`
- `cargo check -p nautilus-backtest --all-targets`
- `cargo check -p nautilus-live --all-targets`
- `cargo check -p nautilus-trading --all-targets`
- `cargo check -p nautilus-risk --all-targets`
- `cargo check -p nautilus-portfolio --all-targets`
- `cargo check -p nautilus-data --all-targets`
- `cargo fmt --check`
- `scripts/ai/validate_agentflow_roles.py`
- `git diff --check`
- `scripts/ai/verify_fast.sh`
- `scripts/ai/check_rust_only_runtime.sh` recorded as expected blocker if
  remaining non-scoped Rust crate residue still exists.

## Acceptance

- Scoped runtime-facing crates no longer contain active PyO3 attributes,
  `pyo3_stub_gen` annotations, `#[cfg(feature = "python")]` blocks, PyO3
  imports, Python extension feature documentation, Python-facing examples, or
  Python-only runtime bridge code.
- Scoped Rust cargo checks pass or any failure is documented with a concrete
  non-scoped blocker.
- Remaining PyO3/Python residue outside this task scope is classified and not
  silently deleted.
- The PR stops at `REVIEW_REQUIRED`; auto-merge is disabled.
