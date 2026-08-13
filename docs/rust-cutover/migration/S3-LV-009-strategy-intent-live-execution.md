# S3-LV-009 策略意图绑定的 Live 执行迁移说明

Date: 2026-08-14
Executor: Codex

## 行为变化

Live 单笔订单准入现在必须引用一个已冻结 Demo Run 的最新策略意图。标的、方向和数量由意图
决定，不能由客户端改写；LIMIT 价格和最大名义金额仍由用户填写，并经过三角色审批和风险上限。

## API 兼容性

- Live execution request 新增 `source_demo_run_id` 和 `strategy_intent_id`；
- Live candidate 新增 `strategy_intent` 和 `strategy_intent_sha256`；
- approval/admission schema 从 v1 升级为 v2；
- production execution node config 从 v1 升级为 v2；
- execution order state 从 v2 升级为 v3，并新增 Demo Run、intent ID 和 intent hash。

严格客户端必须从当前 OpenAPI 重新生成。旧审批、准入、Runtime 配置和 v2 订单状态不能作为
新执行链的有效 authority；升级不会自动创建或发送任何订单。

## 回滚

回滚到 S3-LV-008 会移除策略意图绑定入口。已生成的 v3 订单状态和策略意图工件必须保留用于
人工审计，不得降级解释为旧 v2 authority。
