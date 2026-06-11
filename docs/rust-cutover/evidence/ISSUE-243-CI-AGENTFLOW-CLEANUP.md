# Issue 243 Evidence - CI smoke optimization and local agentflow state

Date: 2026-06-11
Executor: Codex
Issue: https://github.com/atxinbao/NTPRO/issues/243
Status: PR_OPEN

## Plain Chinese Summary

这次修改解决两个流程问题。

第一，GitHub 的 `Fast verification` 名字和实际行为不一致。它以前在 CI 里还会跑整个 workspace 的 `cargo check` 和 `clippy`，而且关闭了 Rust cache，所以每次 PR 都像冷启动编译一样慢。现在把它拆清楚：快速检查只做工具链和格式，重型 Rust 检查单独命名，并打开 cache。

第二，`.agentflow/` 是本地任务状态，不应该每次状态收口都传到 GitHub。现在 Git 停止跟踪 `.agentflow/`，并在 `.gitignore` 里忽略它。这样本地 Codex/Shrimp 仍能使用这些文件，但后续不会再为了本地状态收口开 PR。

## Files Changed

- `.github/workflows/rust-cutover-smoke.yml`
  - Enables Rust/Cargo cache.
  - Renames fast check to `Format and toolchain check`.
  - Splits workspace cargo check and clippy into explicit steps.
  - Adds changed-file classification so docs/local-state-only PRs skip heavy Rust checks.
- `.gitignore`
  - Adds `.agentflow/`.
- `.agentflow/**`
  - Removed from Git tracking with `git rm --cached`.
  - Local files remain on disk for Codex/Shrimp use.
- `docs/rust-cutover/evidence/ISSUE-243-CI-AGENTFLOW-CLEANUP.md`
  - Records this evidence.

## Behavior Impact

- Runtime behavior: none.
- CLI behavior: none.
- Trading semantics: none.
- CI behavior:
  - Code/workflow/script PRs still run workspace cargo check, clippy, CLI unit tests, and supervisor smoke.
  - Docs/local-state-only PRs keep the required smoke job green but skip heavy Rust compile checks.
- Local workflow behavior:
  - `.agentflow/` becomes local-only state.

## Public API Impact

None.

## Migration Note Status

No user-facing migration note is required. This is repository workflow cleanup.

## Commands Run

### `scripts/ai/verify_fast.sh`

Result: PASS

Summary:

- Toolchain check passed with Cargo/Rust 1.95.0.
- `cargo fmt --check` passed.
- Workspace `cargo check` and clippy were skipped by the script default, as expected for the fast path.

### `.agentflow` local-only check

Result: PASS

Commands:

- `test -d .agentflow`
- `git ls-files .agentflow | wc -l`
- `git check-ignore -v .agentflow/state/task_status.json`

Summary:

- `.agentflow/` still exists locally.
- Git now tracks zero `.agentflow` files.
- `.gitignore` ignores `.agentflow/`.

### Workflow marker check

Result: PASS

Summary:

- The workflow contains the expected split steps:
  - `Classify changed files`
  - `Format and toolchain check`
  - `Workspace cargo check`
  - `Workspace clippy`
- Rust cache is enabled with `cache: true`.

### `python3 -m json.tool /Users/mac/.codex/shrimp-data/NTPRO/tasks.json`

Result: PASS

Summary:

- The isolated NTPRO Shrimp queue remains valid JSON.

### `git diff --check`

Result: PASS

Summary:

- No whitespace errors were found.

## Rollback Plan

Revert this PR to restore the previous workflow and tracked `.agentflow/` files.

If only the CI split causes trouble, revert `.github/workflows/rust-cutover-smoke.yml` while keeping `.agentflow/` ignored.

If only local-state cleanup causes trouble, remove `.agentflow/` from `.gitignore` and re-add required `.agentflow` files explicitly.
