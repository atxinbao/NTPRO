# MVP-010 控制中心生命周期动作迁移说明

Date: 2026-08-04
Executor: Codex
Task: MVP-010 / issue #1225

## 变化

控制中心运维快照 schema 从 `ntpro.mvp_control_center_snapshot.v1` 升级为
`ntpro.mvp_control_center_snapshot.v2`。v2 保留既有最小化、脱敏运维字段，并新增
`lifecycle_actions`，其中必须且只能包含当前唯一节点的 `start` 和 `stop` capability。

新增版本化动作端点：

```text
POST /api/mvp/v1/control-center/nodes/{node_id}/actions/start
POST /api/mvp/v1/control-center/nodes/{node_id}/actions/stop
```

动作响应 schema 为 `ntpro.mvp_control_center_lifecycle_action.response.v1`，contract 为
`ntpro.mvp_control_center_lifecycle_action.v1`。调用者必须校验 top-level node/action、
result 内的 action id、前后状态和显式边界，不能只根据 HTTP 2xx 推断动作成功。

## 调用方迁移

- 继续使用 v1 schema 的控制中心客户端必须升级为 v2，否则应 fail closed；
- 不得从生命周期状态自行推导未知动作，也不得调用旧 Dashboard 的 unversioned API；
- action 成功后应重新读取共享状态、运维快照和事件关联，等待三者身份与状态一致；
- 三个投影一致仍不够，最终节点生命周期必须等于 start 的 `running` 或 stop 的 `stopped`；
- `409`、`404`、`403`、`405` 或无法验证的响应都表示动作未被产品合同确认；
- 单节点 sandbox 或任一外部/真实交易边界无法证明时返回 `503`，动作不会进入 Supervisor；
- 服务端对 scope 与动作前置状态使用锁内同一份快照，不接受校验后重新读取的未验证状态；
- action message 和 error code 已脱敏，不应尝试获取原始进程或 adapter 错误。

## 保持关闭的能力

本迁移不开放 pause、resume、restart、reconnect、retry、remediation 或任何交易控制。
机构工作台继续没有 Supervisor action；外部 Venue、真实订单、订单提交和变更全部为
false。该端点只控制当前本地 sandbox 节点进程生命周期。

## 回滚

回滚 MVP-010 提交会恢复只读 v1 运维投影并移除版本化 start/stop 产品端点。既有旧
Dashboard 本地 API 和 Supervisor registry/store 数据格式不变，不需要迁移注册表。
