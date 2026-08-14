# S3-LV-010 Live 风险预算与仓位 sizing 准入迁移说明

Date: 2026-08-14
Executor: Codex

## 行为变化

Live preflight 现在冻结脱敏账户可用资金和配置中的 Binance Spot tick、step、数量与最小名义
金额规则。三角色审批前生成确定性 sizing 决策，最终数量只能小于或等于策略意图数量，并按
step size 向下取整。证据缺失、过期、资金不足或规则漂移均拒绝准入。

## API 兼容性

- Live candidate 新增 `sizing_decision` 和 `sizing_decision_sha256`；
- execution order state 从 v3 升级为 v4，并绑定 sizing hash；
- production execution node config 新增 sizing hash 和 `source_quantity`；
- 严格客户端必须从当前 OpenAPI 重新生成。

旧 v3 订单状态和缺少 sizing authority 的 Runtime 配置不能启动新的真实执行链。升级不会自动
创建、扩大或发送订单。

## 回滚

回滚到 S3-LV-009 会移除 sizing 准入。已产生的 v4 订单状态与 sizing 工件必须保留用于人工
审计，不得降级解释为旧 v3 authority。
