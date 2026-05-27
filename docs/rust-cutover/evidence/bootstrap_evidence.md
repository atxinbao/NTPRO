# Bootstrap Evidence

Date: 2026-05-27 23:00:44 CST
Executor: Codex
Package: v1.06 (`1.0.6-rust-first`)
Target repository: `/Users/mac/Documents/NTPRO`
GitHub repository: `https://github.com/atxinbao/NTPRO`
Default branch: `main`

## Summary

NTPRO is now the Rust-first cutover workspace for the NautilusTrader-derived
codebase. The initial v1.06 control-plane package was imported before runtime
refactor work began, and subsequent control-plane PRs established task
execution, roles, gates, evidence, CI smoke, and GitHub repository settings.

No runtime behavior, trading semantics, adapters, public APIs, Cargo workspace
layout, Python package surface, PyO3 crate, or Cython source was changed as
part of bootstrap control-plane work.

## Initial Import

- Moved the v1.06 package from `/Users/mac/Documents/NTPRO/tmp` to
  `/Users/mac/Documents/nautilus_trader_rust_first_pack_v1_0_6`.
- Archived the old control repository at
  `/Users/mac/Documents/NTPRO_control_repo_archive_20260527_060425`.
- Cloned `https://github.com/atxinbao/nautilus_trader` into
  `/Users/mac/Documents/NTPRO` before the new NTPRO remote was established.
- Applied `repo_overlay/` into the target repository with `rsync
  --ignore-existing`.
- Preserved the existing target `.github/PULL_REQUEST_TEMPLATE.md` during the
  initial overlay conflict.

The initial overlay conflict report was written to
`.agentflow/bootstrap_overlay_conflicts.md`.

## Current Repository State

- `origin` points to `https://github.com/atxinbao/NTPRO.git`.
- The repository is public.
- The default branch is `main`.
- GitHub auto-merge is enabled.
- Delete branch on merge is enabled.
- `main` branch protection requires the `smoke` status check with strict
  up-to-date checks.
- Force pushes and branch deletion are disabled on `main`.
- No NTPRO `develop` branch remains on the remote.
- The local `upstream` remote for `atxinbao/nautilus_trader` was removed so
  future work targets NTPRO only.

Current local baseline:

```text
commit: 03e2906b10fbcf6b15567e7500994696c13c8ad6
subject: Merge pull request #10 from atxinbao/ai/remove-develop-security-audit
remote: origin https://github.com/atxinbao/NTPRO.git
```

## Control-Plane PRs

- PR #1: bootstrapped the Rust-first cutover control plane.
- PR #2: narrowed fast smoke by skipping legacy cargo check in fast mode.
- PR #3: added RCTL-002 verification script evidence.
- PR #4: stopped legacy build workflows from running on `develop` pushes.
- PR #5: installed the agent role protocol.
- PR #6: inventoried the current Rust product surface.
- PR #7: added the scope decision log.
- PR #8: installed PR and issue templates.
- PR #9: enabled Rust cutover smoke for `main` PRs.
- PR #10: stopped security audit workflow from running on `develop` pushes.

## Tooling State

- Code-Index is indexed for `/Users/mac/Documents/NTPRO`.
- Shrimp project data is isolated under
  `/Users/mac/.codex/shrimp-data/NTPRO/tasks.json`.
- Shrimp currently tracks 100 v1.06 tasks for NTPRO:
  - `completed`: 6;
  - `in_progress`: 1 (`RCTL-007`);
  - `pending`: 93.
- The NTPRO task ID map is copied to
  `/Users/mac/.codex/shrimp-data/NTPRO/ntpro_v1_06_task_id_map.json`.

## Validation History

Bootstrap and subsequent control-plane tasks have used local validation rather
than runtime refactors. The standard fast command is:

```bash
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
```

For RCTL-001 through RCTL-006, the control-plane PRs were merged after local
evidence was written and GitHub `smoke` completed. RCTL-007 refreshes this
bootstrap evidence and records the current repository state as the baseline for
later product, runtime, adapter, removal, and release work.

## Behavior Impact

Bootstrap evidence work is documentation and control-plane state only. It does
not change runtime behavior, trading semantics, adapter behavior, public APIs,
build features, precision behavior, Python/PyO3/Cython product surfaces, or
Cargo workspace membership.

## Rollback Plan

- Revert `docs/rust-cutover/evidence/bootstrap_evidence.md`.
- Revert the specific task evidence file that introduced the change.
- Restore `.agentflow/state/task_status.json` to the previous task-state
  snapshot if task-state changes must be backed out.
- Restore Shrimp task data from
  `/Users/mac/.codex/shrimp-data/NTPRO/memory/` if project task status needs to
  be reverted.
