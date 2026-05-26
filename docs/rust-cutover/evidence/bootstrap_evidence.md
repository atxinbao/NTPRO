# Bootstrap Evidence

Date: 2026-05-27 06:27:58 CST
Executor: Codex
Package: v1.06 (`1.0.6-rust-first`)
Target repository: `/Users/mac/Documents/NTPRO`
Target branch: `rust-first-cutover-agentflow`

## Summary

The old NTPRO control-pack workspace was moved out of the repository path before any runtime refactor work started. `/Users/mac/Documents/NTPRO` is now a fresh clone of `https://github.com/atxinbao/nautilus_trader` on the cutover branch, with the v1.06 control-plane overlay applied without overwriting existing target files.

## Filesystem Changes

- Moved v1.06 package from `/Users/mac/Documents/NTPRO/tmp` to `/Users/mac/Documents/nautilus_trader_rust_first_pack_v1_0_6`.
- Archived the old control repository at `/Users/mac/Documents/NTPRO_control_repo_archive_20260527_060425`.
- Cloned `https://github.com/atxinbao/nautilus_trader` into `/Users/mac/Documents/NTPRO`.
- Created branch `rust-first-cutover-agentflow`.
- Applied `repo_overlay/` into the target repository with `rsync --ignore-existing`.

## Overlay Conflict

The overlay detected one existing target file and did not overwrite it:

- `.github/PULL_REQUEST_TEMPLATE.md`

The conflict report is stored at `.agentflow/bootstrap_overlay_conflicts.md`. The target repository currently keeps its original pull request template.

## Tooling Activated

- Code-Index indexed `/Users/mac/Documents/NTPRO`.
- Code-Index Web UI is running at `http://127.0.0.1:6070`.
- Shrimp task data was reset from the old MTP queue and loaded with 100 v1.06 tasks.
- Shrimp GUI is running at `http://localhost:6071?lang=zh-TW`.
- Shrimp task map is stored at `/Users/mac/.codex/shrimp-data/ntpro_v1_06_task_id_map.json`.
- Full pre-import Shrimp backup is stored under `/Users/mac/.codex/shrimp-data/memory/`.

## Validation

Commands run:

```bash
python3 scripts/control/validate_task_graph.py backlog/task_graph.yaml
python3 scripts/control/package_self_check.py
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
```

Results:

- `validate_task_graph.py`: `task graph ok: 100 tasks`
- `package_self_check.py`: `package self-check ok`
- Initial `verify_fast.sh` failed because Homebrew `rustc 1.87.0` did not satisfy workspace `rust-version = "1.95.0"`.
- Installed Homebrew `rustup` and Rust toolchain `1.95.0`.
- Re-ran `verify_fast.sh` with rustup first in `PATH`; result passed:
  - `cargo fmt --check` completed with stable rustfmt warnings for unstable import grouping settings.
  - `cargo check --workspace --features arrow,ffi,high-precision,streaming,defi` completed.
  - clippy was skipped by script default (`VERIFY_FAST_CLIPPY=0`).

## Behavior Impact

No runtime behavior was changed during bootstrap. The applied overlay adds cutover control-plane documents, scripts, GitHub templates/workflows, task specifications, and golden-test scaffolding.

## Rollback Plan

- Restore the pre-bootstrap control workspace from `/Users/mac/Documents/NTPRO_control_repo_archive_20260527_060425` if needed.
- Remove uncommitted overlay files from `/Users/mac/Documents/NTPRO` or reset the `rust-first-cutover-agentflow` branch before staging.
- Restore Shrimp task data from `/Users/mac/.codex/shrimp-data/memory/tasks_full_backup_before_ntpro_v1_06_import_20260526T221636Z.json` if the v1.06 queue needs to be reverted.
