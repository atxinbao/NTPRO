# V02 Control Sync Evidence

Date: 2026-06-06
Executor: Codex
Task ID: V02-CONTROL-SYNC
Branch: `codex/dispatch-v02-task-id-parser`

## 中文总结

这次是为 V02 自动化执行补控制面，不是实现产品功能。

发现的问题是：Shrimp 队列里已经有 `V02-001` 到 `V02-010`，但 `dispatch_next.py` 不能识别带数字前缀的任务 ID，而且仓库里的 `.agentflow` 和 task docs 还没有 V02 元数据。这样会导致自动化无法调度 V02-001。

## 修改内容

- `scripts/control/dispatch_next.py` 支持 `V02-001` 这类带数字前缀的任务 ID，并优先读取 Shrimp task 的 `id` 字段。
- 新增 `docs/rust-cutover/tasks/V02-001.md` 到 `V02-010.md`。
- 在 `.agentflow/state/task_status.json` 中登记 `V02-001` 到 `V02-010`。
- 本 evidence 记录控制面同步结果。

## 验证计划

- `python3 -m json.tool .agentflow/state/task_status.json`
- `python3 scripts/control/dispatch_next.py --dry-run --branch-prefix codex --max-risk medium`
- `scripts/ai/verify_fast.sh`
- `git diff --check`

## 行为影响

无 runtime 行为影响。无交易语义影响。无 public API 影响。

## Rollback plan

回滚本 PR 会恢复旧调度解析，并移除 V02 task docs / `.agentflow` 元数据。回滚后自动化仍不能调度 V02 任务。
