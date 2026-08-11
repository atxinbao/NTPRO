# S2-DM-003 Demo 模拟结果与跨环境比较迁移说明

Date: 2026-08-11
Executor: Codex

## 行为变化

`GET /api/product/v1/runs/{run_id}/demo-snapshot` 的响应 schema 从
`ntpro.product_api.demo_run_snapshot.response.v1` 升级为 `response.v2`，内部结果 schema
同步升级为 `ntpro.product_api.demo_run_result.v2`。结果新增必需但可空的 `simulation`：
created 或失败且没有可信模拟工件时为 `null`；running 和可信 stopped Run 返回模拟摘要、
成交、持仓与资金曲线。

`GET /api/product/v1/run-comparisons` 从 Backtest-only 的 `response.v1` 升级为统一 Run
`response.v2`。比较项新增 `environment`、`strategy_id`，指标、风险和来源改为 Backtest 与
Demo 共用结构；兼容性新增 `same_environment` 和 `behaviorally_comparable`。

## 消费方迁移

- 重新从权威 OpenAPI 生成客户端，使用 `compareRuns`，不再调用 `compareBacktestRuns`；
- Demo 页面只把 `simulation_only=true` 且所有真实交易边界为 false 的记录显示为模拟结果；
- `simulation` 非空时，摘要计数必须与三个明细数组精确一致；不一致时客户端拒绝响应；
- Backtest 与 Demo 可在同一请求中比较，但只有策略、版本、标的和币种相同时才具有行为
  可比性；数据哈希不同不应标记为可直接比较；
- Demo Run 不支持确定性复现，客户端必须禁用对应操作；
- 客户端不得把 Demo 成交、持仓或资金曲线标记为真实交易结果。

## 兼容边界

Demo 创建、start/stop action、Backtest 创建和 Backtest 确定性复现合同保持不变。真实
Live、外部 Venue、真实订单提交与撤改、自动重试、自动补救和交易控件继续关闭；任何
Backtest 或 Demo 结果都不会自动开启 Live 权限。
