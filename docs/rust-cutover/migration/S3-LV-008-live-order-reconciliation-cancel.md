# S3-LV-008 Live 订单对账、人工撤单与部分成交恢复迁移说明

Date: 2026-08-13
Executor: Codex

## 行为变化

S3-LV-007 的一次性订单提交合同保持不变。执行订单状态升级为 v2，新增 Venue order ID、原始
数量、累计成交数量和剩余数量。策略工作台可显式发起订单对账，也可由机构负责人提交人工撤单
申请；撤单只有在操作员通过独立控制面确认同一不可变申请后才会进入 Runtime。

## 运维影响

- 显式启用 `NTPRO_S3_LIVE_ORDER_CONTROL=1` 后，运行中的已消费单笔订单才开放对账和人工
  撤单入口；默认部署继续关闭；
- 对账只需要机构负责人确认；撤单需要机构负责人和操作员分别确认同一 Run、admission、
  StrategyVersion、instrument、client order ID、订单状态哈希和有效期；
- Runtime 在处理控制请求前持久化 control attempt，仅在即将调用交易所撤单前另行持久化绑定
  请求哈希的 Venue attempt。已有 attempt 但没有结果时不会重试，而是发布
  `unknown_manual_review`；`cancel_attempted` 只表示可能已向交易所发送，不表示仅完成审批；
- 每份控制结果在本地发布前必须追加到外部单调审计锚点，并生成对应 result receipt；结果或
  回执缺失、锚点不是最新、订单状态倒退或身份漂移均 fail closed；
- 部分成交只更新累计成交量和剩余数量。撤单仅撤销剩余数量，不补单、不扩大数量、不改价；
- 升级不会自动对账、自动撤单或自动重试。回滚到 S3-LV-007 后新增控制入口不可用，既有单笔
  提交和订单状态仍可读取，但 v2 状态应保留供人工核对。

## API 兼容性

Live Run action 枚举增加 `reconcile_order`；新增机构负责人撤单申请和操作员确认端点；候选响应
增加 `execution_order_state_sha256`、`execution_control` 以及动态的人工对账/撤单边界。订单状态
schema 从 v1 升级到 v2，严格客户端必须从当前 OpenAPI 重新生成。新增能力不会从旧客户端、
Backtest、Demo、页面加载或 Runtime 启动自动触发。

改单、批量撤单、策略自动连续下单、自动重试、自动修复、自动恢复和补偿交易仍不开放。
