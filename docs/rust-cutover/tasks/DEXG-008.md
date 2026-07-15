# DEXG-008 Post-Backend Docs And Examples Governance Closeout

Date: 2026-07-15
Executor: Codex
GitHub issue: #1087
Milestone: post-backend-docs-examples-governance
Status: READY FOR PR

## Goal

Close the post-v0.32.0 docs/examples governance milestone with
source-controlled and live GitHub proof.

Plain Chinese summary: 本任务收口文档与 examples 清理，不发布后端新版本。PR 合并后
关闭 #1087 和 milestone #32，并从 GitHub live state 重建最终 issue、PR、workflow、
main 和 milestone 证据。

## Dependencies

- DEXG-001 through DEXG-007 / #1080-#1086 are closed;
- PR #1088 through PR #1094 are merged;
- DEXG-007 hosted smoke and security checks completed successfully.

## Scope

- add the source-controlled docs/examples closeout contract;
- record the exact DEXG issue and merged-PR set;
- verify docs/examples governance, Rust examples, generated artifacts, and the
  backend-freeze boundary;
- merge this PR, close issue #1087, close milestone #32, and run the final live
  GitHub audit.

Not included:

- modifying the frozen v0.32.0 release package;
- creating v0.32.1 or a new backend capability version;
- deleting retained Rust cutover audit evidence;
- changing runtime, adapter, trading, or public API behavior.

## Acceptance Criteria

- source contract records the exact eight-issue scope and known merge commits;
- deterministic docs/examples and backend-freeze gates pass;
- generated repository artifacts remain absent after validation;
- after merge, all eight DEXG issues are closed and no milestone PR remains;
- milestone #32 is closed only after the final issue closes;
- live GitHub state and `origin/main` agree with the source closeout contract.

## Result

Local and live dependency validation passed. Final completion still requires
this PR to merge, issue #1087 to close, milestone #32 to close, and the
post-merge GitHub audit to pass.
