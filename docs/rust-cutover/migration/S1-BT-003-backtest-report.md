# S1-BT-003 Backtest 明细合同迁移说明

Date: 2026-08-09
Executor: Codex

## 变更

新创建的 Backtest Run 除不可变 `summary.json` 外，还会写入同一次 `BacktestEngine`
执行产生的不可变 `details.json`。Run 的 `result.report_ref` 指向该产物，机构用户和运维
只读角色可通过 `GET /api/product/v1/runs/{run_id}/report` 读取交易、最终持仓和账户权益
时间序列。

服务端在返回数据前校验普通文件类型、目录 containment、SHA-256、Run 与
StrategyVersion 身份、数据和配置哈希、时间范围、排序、币种、数量、价格以及全部禁止
能力边界。校验失败时 fail closed，不返回部分明细。

## 兼容性

`summary.json` 的 `ntpro.backtest_result.v1` 合同保持不变。公开 Run 合同新增必需但可空的
`result.report_ref`：历史 Run 没有明细产物时返回 `null`，原有 detail 和 metrics 路由继续
可用；重新创建 Backtest Run 后才会生成明细。调用方不得根据 `result_ref` 推导
`details.json` 路径，应仅在 `report_ref` 非空时调用 report 路由。

## 边界

本变更不修改回测撮合和统计语义，不连接外部 Venue，不提交或修改真实订单，也不开放
Demo、Live、停止、重试、自动补救或交易控件。
