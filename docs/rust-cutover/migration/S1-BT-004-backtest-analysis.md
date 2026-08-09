# S1-BT-004 Backtest 分析合同迁移说明

Date: 2026-08-09
Executor: Codex

## 新增合同

已完成且结果可用的新 Backtest Run 会产生不可变 `analysis.json`。Run 的
`result.analysis_ref` 指向该产物，机构用户和运维只读角色可通过
`GET /api/product/v1/runs/{run_id}/analysis` 读取风险摘要、逐点回撤、结构化运行记录和
来源哈希链。

`analysis.json` 不抓取终端 stdout。事件由同一次 `BacktestEngine` 执行的 Run 边界、账户
权益、成交和持仓确定性投影；风险与回撤可由 `details.json` 的权益序列复算。

## 兼容行为

`result.analysis_ref` 是稳定响应中的可空字段。历史 Run 没有分析产物时返回 `null`，原有
detail、metrics 和 report 路由继续可用。客户端不得自行拼接 `analysis.json` 路径，应仅在
`analysis_ref` 非空时调用 analysis 路由。

## 边界

该合同只读，不开放 Demo、Live、外部 Venue、订单提交、订单修改、自动重试、自动补救或
交易控件。它不改变 BacktestEngine 撮合、账户、持仓和既有统计语义。
