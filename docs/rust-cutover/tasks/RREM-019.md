# RREM-019 - Remove serialization and network PyO3 and Python residue

Date: 2026-06-02
Executor: Codex

## Summary

Remove scoped PyO3/Python residue from `crates/serialization` and
`crates/network` after adapter compilation exposed remaining
`feature = "python"` warnings in these shared dependency crates.

## Scope

- Remove stale `#[cfg_attr(feature = "python", ...)]` PyO3 and stub
  annotations from `crates/network`.
- Remove unreachable `#[cfg(feature = "python")]` imports, error variants,
  cleanup blocks, and test modules from `crates/network` and
  `crates/serialization`.
- Rewrite scoped crate docs and comments that still describe active
  Python/PyO3 product surfaces.
- Update the Rust crate Python residue inventory and task evidence for this
  slice.
- Close local RREM-018 task state after PR #114 was merged.

## Out of Scope

- Do not modify remaining Python/PyO3 residue outside `crates/serialization`
  and `crates/network`.
- Do not change Arrow wire schemas, SBE decoding, HTTP request behavior,
  WebSocket backend selection, TCP reconnect semantics, rate limiter behavior,
  or adapter trading behavior.
- Do not remove Rust tests that are not Python/PyO3 gated.
- Do not execute `RREL-008`.

## Owner and Review

- Owner role: `rust_core_runtime_agent`
- Review role: `verification_release_gatekeeper`
- Risk level: `critical`

## Allowed Paths

- `crates/serialization/**`
- `crates/network/**`
- `docs/rust-cutover/tasks/RREM-019.md`
- `docs/rust-cutover/evidence/RREM-019.md`
- `docs/rust-cutover/inventory/rust_crate_python_residue.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RREM-018.json`
- `.agentflow/leases/RREM-019.json`

## Required Evidence

- `rg` checks showing no PyO3/Python binding residue remains in
  `crates/serialization` or `crates/network`.
- `cargo check -p nautilus-serialization --all-targets`
- `cargo check -p nautilus-network --all-targets`
- `cargo fmt --check`
- `scripts/ai/validate_agentflow_roles.py`
- `git diff --check`
- `scripts/ai/verify_fast.sh`
- `scripts/ai/check_rust_only_runtime.sh` recorded as expected blocker if
  remaining non-scoped Rust crate residue still exists.

## Acceptance

- `crates/serialization` and `crates/network` no longer contain active PyO3
  attributes, `pyo3_stub_gen` annotations, `#[cfg(feature = "python")]`
  blocks, PyO3 imports, Python extension feature documentation, or Python-only
  test modules.
- Scoped Rust cargo checks pass.
- Remaining PyO3/Python residue outside this task scope is classified and not
  silently deleted.
- The PR stops at `REVIEW_REQUIRED`; auto-merge is disabled.
