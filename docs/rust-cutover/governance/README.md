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
- `../../../scripts/ai/check_backend_freeze_baseline.sh` - deterministic guard
  for tag, commit, source hashes, explicit-false boundaries, and v0.33+
  non-inheritance, including fail-closed negative selftests.
- `generated_artifact_policy.md` - tracked, remote-reconstructable, and
  ephemeral-local artifact classification plus worktree hygiene rules.
- `docs_examples_authority_map.md` - post-baseline ownership, retention,
  rewrite, removal, and generated-artifact boundaries for `docs/` and
  `examples/`.
- `github_intake_policy.md` - required issue/PR declarations, label meanings,
  and fail-closed routing for post-freeze changes.
- `v0_33_plus_intake_policy.md` - mandatory entry contract for separately
  scoped frontend, UX, deployment, or new-module work.
- `backend_freeze_governance_closeout.md` - source-controlled closeout contract
  for milestone #31 and its live GitHub reconstruction.
- `docs_examples_governance_closeout.md` - source-controlled closeout contract
  for milestone #32 and its live GitHub reconstruction.
- `python_tooling_baseline.json` - machine-readable corrected Python execution
  baseline for milestone #33.
- `python_tooling_authority_map.md` - replacement, retirement, retention, and
  zero-Python closeout authority for repository tooling.
- `../release/v0_32_0_release_closeout_evidence.md` - source-controlled release
  closeout contract.

## Completed Backend Freeze Governance

- BFG-001: baseline registry and policy.
- BFG-002: current documentation and version-route cleanup.
- BFG-003: deterministic backend-freeze drift guard.
- BFG-004: generated artifact hygiene.
- BFG-005: GitHub issue and PR intake hardening.
- BFG-006: v0.33+ separately scoped intake and governance closeout.

The milestone is `backend-freeze-governance` (#31). It is governance work, not
a v0.32.1 backend release.

## Docs And Examples Governance

The follow-up milestone is `post-backend-docs-examples-governance` (#32), with
the exact issue set `#1080-#1087`. It cleans the current documentation and
examples surfaces after the backend freeze. It does not modify the v0.32.0
release package or authorize a new backend version.

## Completed Docs And Examples Governance

- DEXG-001: authority map and cleanup boundaries.
- DEXG-002: canonical Rust examples integrity.
- DEXG-003: legacy Python API appendix and docs-python retirement.
- DEXG-004: unsupported Python-first guide and asset retirement.
- DEXG-005: integration documentation authority.
- DEXG-006: concept documentation authority and source links.
- DEXG-007: stable Rust docs build and deterministic governance gate.
- DEXG-008: source plus live GitHub governance closeout.

The closeout contract is `docs_examples_governance_closeout.md`. Final merge,
issue, workflow, main-branch, and milestone facts are reconstructed from live
GitHub after the DEXG-008 PR merges.

## Python Tooling Closeout

The follow-up milestone is `python-tooling-closeout` (#33), with exact issue
set `#1096-#1103`. It removes repository Python execution only after current
validation authority is replaced in Rust. Historical `docs/rust-cutover/`
evidence remains retained, and v0.32.0 remains the frozen backend baseline.

The final source-controlled contract is `python_tooling_closeout.md`. The
repository drift gate is `scripts/ai/check_zero_python_closeout.sh`; post-merge
issue, PR, workflow, main, and milestone facts are reconstructed from live
GitHub instead of being guessed inside the closing PR.
