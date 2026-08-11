# S2-DM-002 Demo snapshot 迁移说明

Date: 2026-08-11
Executor: Codex

## 行为变化

Product API 新增 `GET /api/product/v1/runs/{run_id}/demo-snapshot`。Demo Run 创建后返回
`not_started`；节点运行时返回真实策略工件投影；停止或失败后返回哈希绑定的 `frozen`
结果。客户端不应再从通用 Run 详情推测实时市场、信号或风险状态。

Demo 终态新增不可变 `demo-result.json`，`terminal-state.json` schema 升级为
`ntpro.product_api.demo_run_terminal_state.v2` 并增加 `demo_result_sha256`。终态文件是内部
运行工件，不是供客户端直接读取的公共接口。

## 消费方迁移

- 使用 OpenAPI 生成的 `getDemoRunSnapshot`，不要自行拼装响应类型；
- 仅在 `snapshot_status=running` 时轮询；`not_started` 和 `frozen` 应停止轮询；
- `latest_order_intent` 是被 Sandbox 风险边界阻止的意图，不是成交；
- `session.actual_submission_count`、`latest_order_intent.submission_allowed` 或
  `latest_risk_decision.actual_submission` 出现禁止值时，客户端必须拒绝响应；
- `404` 表示 Run 不存在或不是 Demo，来源缺失、篡改和身份漂移按 Product error fail closed。

## 兼容边界

Backtest、Demo 创建和生命周期 action 合同保持不变。真实 Live、外部 Venue、订单提交、
撤改、自动重试和自动补救仍未开放。
