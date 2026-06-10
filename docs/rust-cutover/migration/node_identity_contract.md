# Node Identity Contract Migration

Date: 2026-06-10
Executor: Codex
Task: P0-006

## 变更内容

`NodeStatus.node_id` 和 `NodeMetrics.node_id` 现在代表节点工件的规范运行身份。

- supervisor 管理的节点必须使用注册时的 `node_id`。
- supervisor 启动 `ntpro-node` 时会通过 `--run-id` 传入该身份。
- 独立运行 `nautilus live run` 且未传入 `--run-id` 时，使用配置中的
  `run.id`。
- `system.node_name` 和 `system.instance_id` 仍可作为 LiveNode 的显示名称，
  但不再覆盖 status/metrics 工件中的 `node_id`。

## 不兼容点

旧版本可能把配置中的 `system.node_name` 或 `system.instance_id` 写入
`status.json` 和 `metrics.json` 的 `node_id`。升级后，这两个工件字段会改为
运行 ID。

supervisor 命令输出标签也更明确：

```text
status_node_id  -> runtime_node_id
metrics_node_id -> runtime_node_id
```

依赖这些文本标签的本地脚本需要同步更新。

## 拒绝策略

supervisor 和 Dashboard 不再接受身份与注册记录不一致的 status/metrics 工件：

- status 工件会标记为 `invalid`，不会覆盖最后一次可信状态；
- metrics 查询会返回明确的 identity mismatch 错误；
- Dashboard 会把错绑工件显示为 unavailable，并记录 gap/error；
- 启动期间出现错绑 status 工件会立即失败，不再等到超时。
- supervisor 启动新进程前会清理上一轮 status/metrics 工件，避免旧身份阻塞
  正常重启或冒充新进程状态。

## 回滚

回滚 P0-006 PR 会恢复配置显示名可覆盖工件 `node_id` 的旧行为。回滚后应重新
运行 `scripts/ai/v02_two_node_supervisor_smoke.sh`，确认调用方能够接受旧的身份
歧义。
