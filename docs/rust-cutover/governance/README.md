# Backend Freeze Governance

Date: 2026-07-15
Executor: Codex

This directory contains governance applied after the published v0.32.0 backend
baseline. It does not replace or rewrite the release package under
`docs/rust-cutover/release/`.

Plain Chinese summary: v0.32.0 是后端基线冻结点。本目录负责基线后的治理、清理、
变更入口和下一阶段立项规则；历史发布事实仍由 v0.32.0 tag、GitHub Release、hosted
release gate 和已跟踪 release package 共同证明。

## Authority

- `backend_freeze_registry.json` - machine-readable v0.32.0 baseline identity,
  hashes, release proof, exact scope, and forbidden capability flags.
- `backend_freeze_policy.md` - rules for immutability, errata, exceptions,
  post-freeze cleanup, and separately scoped v0.33+ work.
- `v0_32_0_errata.md` - post-baseline clarification of stale current-route
  wording without changing the tagged release package.
- `../release/v0_32_0_release_closeout_evidence.md` - source-controlled release
  closeout contract.

## Planned Governance Work

- BFG-001: baseline registry and policy.
- BFG-002: current documentation and version-route cleanup.
- BFG-003: deterministic backend-freeze drift guard.
- BFG-004: generated artifact hygiene.
- BFG-005: GitHub issue and PR intake hardening.
- BFG-006: v0.33+ separately scoped intake and governance closeout.

The milestone is `backend-freeze-governance` (#31). It is governance work, not
a v0.32.1 backend release.
