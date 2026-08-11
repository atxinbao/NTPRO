# S3-LV-001 Live 独立准入迁移说明

Date: 2026-08-11
Executor: Codex

## 新增产品合同

策略工作台 Product API 新增只读资源：

```text
GET /api/product/v1/strategies/{strategy_id}/versions/{version_id}/live-admission
```

响应提供当前策略版本的生产 Venue 目标、账户引用、凭证存在性、订单生命周期状态、阻断原因
和独立 Live 边界。机构用户和运维用户都可读取；非 GET 方法返回 `405 Allow: GET`。

## 配置迁移

节点配置新增 `[live_admission]`，固定首个产品目标为 Binance Spot。部署侧可以配置：

```text
NTPRO_BINANCE_LIVE_API_KEY
NTPRO_BINANCE_LIVE_API_SECRET
```

API 只返回 `present` 或 `missing`，不会输出凭证值。仅配置凭证不会产生网络、账户或交易权限。

## 行为影响

- 工作台 Live 导航从禁用占位变为只读准入页面；
- owner approval、生产网络、authenticated read、Live Run、submit/cancel/replace/reconciliation
  和自动恢复仍为 false；
- Backtest 和 Demo 权限不会继承到 Live；
- 无下单、撤单、改单、平仓或自动重试控件。

## 回滚

删除 Live 准入路由、配置区块、OpenAPI schema、生成客户端与 Live 页面，并恢复 Live 导航为
禁用占位。回滚不涉及订单、账户状态或外部 Venue，因为本任务没有执行这些动作。
