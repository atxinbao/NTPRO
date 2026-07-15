# DEXG-002 Rust Examples Path And Status Integrity

Date: 2026-07-15
Executor: Codex
GitHub issue: #1081
Milestone: post-backend-docs-examples-governance
Status: READY FOR PR

## Goal

Keep `examples/rust/` as the canonical example surface while correcting broken
paths and stale early-cutover status wording.

Plain Chinese summary: 本任务保留全部 14 个 Rust example 文件，把 config README
中不存在的两个路径替换为当前真实配置，并把早期“命令仍阻塞”说明更新为当前 scoped
runnable 状态。新增检查只验证文件、TOML 和 README 路径，不运行交易流程。

## Dependency

DEXG-001 / #1080 is merged and closed.

## Scope

Included:

- repair stale config example paths;
- align example status wording with current Rust CLI source and v0.32.0 freeze;
- add a deterministic read-only integrity check for canonical files, TOML, and
  README paths.

Not included:

- changing CLI/runtime implementation;
- adding adapter, live venue, submit, mutation, or product capability;
- deleting canonical Rust examples.

## Acceptance Criteria

- all 14 canonical files exist;
- every example TOML parses with the Python standard-library TOML parser;
- every `examples/rust/...` path in example README files resolves;
- backend freeze and fast smoke checks pass.

## Validation

```bash
scripts/ai/check_rust_examples.sh
scripts/ai/check_backend_freeze_baseline.sh
scripts/ai/verify_fast.sh
git diff --check
```
