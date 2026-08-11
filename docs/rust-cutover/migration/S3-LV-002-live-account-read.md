# S3-LV-002 生产账户只读接入迁移说明

Date: 2026-08-11
Executor: Codex

## 新增产品命令

策略工作台 Product API 新增显式命令：

```text
POST /api/product/v1/strategies/{strategy_id}/versions/{version_id}/live-account/actions/refresh
Content-Type: application/json

{"action":"refresh"}
```

该命令只允许机构用户读取 Binance Spot `/api/v3/account`。Operator cookie 和未认证请求返回
403；GET 和其他方法返回 `405 Allow: POST`；未知字段、缺失字段或其他 action 返回产品错误
合同。

## 部署门禁

源配置声明账户只读能力后，部署侧仍必须同时配置两个环境凭证和五项一次性运行门禁：

```text
NTPRO_BINANCE_LIVE_API_KEY
NTPRO_BINANCE_LIVE_API_SECRET
NTPRO_ALLOW_PRODUCTION_AUTHENTICATED_READ=1
NTPRO_OWNER_APPROVED_PRODUCTION_READ_ONLY=1
NTPRO_CONFIRM_PRODUCTION_ACCOUNT_NO_ORDER_MUTATION=1
NTPRO_CONFIRM_NO_SECRET_PERSISTENCE=1
NTPRO_V12_MANUAL_ONLINE=1
```

任一条件缺失时返回 `blocked`，且不调用生产 HTTP。页面加载不会自动执行该命令，失败后也
不会自动重试。认证 HTTP 禁止跟随任何 3xx 重定向，响应重定向时直接返回失败，API Key、
签名查询和 Referer 不会发送到第二个 origin。

## 数据边界

响应只包含连接结果、HTTP 状态、延迟、响应形状是否通过，以及余额/权限条目数量等摘要。
凭证值、签名、签名查询、签名 URL、原始账户响应、资产名称、余额和权限值均不进入产品
响应或日志。

## 行为影响

- Live 页面新增“检查账户连接”显式按钮；
- 默认开发和 CI 环境保持 `blocked`，并证明没有生产网络访问；
- Live Run、订单 endpoint、提交、撤单、改单、重试、补救和恢复继续关闭；
- Backtest 和 Demo 权限不会继承到 Live。

## 回滚

删除刷新路由、OpenAPI schema、生成客户端和页面检查面板，并把源配置的生产网络与账户读取
能力恢复为 false。回滚不涉及订单或外部账户状态，因为本任务不具备变更能力。
