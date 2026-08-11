# S3-LV-003 Live 账户结果迁移说明

Date: 2026-08-12
Executor: Codex

## 合同升级

现有显式命令和请求体保持不变：

```text
POST /api/product/v1/strategies/{strategy_id}/versions/{version_id}/live-account/actions/refresh
Content-Type: application/json

{"action":"refresh"}
```

成功读取时，响应 schema 从 `ntpro.product_api.live_account_refresh.response.v1` 升级为
`ntpro.product_api.live_account_refresh.response.v2`，新增：

- `account_result`：账户类型和交易所返回的三个权限标志；
- `asset_balances`：非零资产的可用、锁定和总额；
- `funds_summary`：源条目数、非零条目数、省略零余额数和估值状态；
- `normalized_account_results_exposed` 与 `account_results_persisted` 边界。

这是同一 Product API v1 endpoint 内的响应 schema 升级。消费者必须使用重新生成的 SDK 和
严格客户端，不能继续按 refresh response v1 解析。

## 数值与估值

所有余额使用交易所返回的资产原生单位。Rust 使用 `Decimal` 校验并计算总额；前端严格
客户端使用整数系数和小数位对齐复核，不转为浮点数。不同资产没有价格与计价货币绑定时：

```text
valuation_status = unavailable_without_price_conversion
valuation_currency = null
portfolio_value = null
```

系统不会把 BTC、USDT 或其他资产数量直接相加成“总资产”。

生产响应在 JSON 解析前有 1 MiB 字节硬上限，并同时检查 HTTP 声明长度和实际流式读取
长度。余额字符串必须符合 OpenAPI 的规范十进制语法，不接受空白、前导零、缺失小数位、
负数、非有限值、超过 `Decimal` 可表示精度、静默舍入或 Decimal 加法溢出。

## 部署与权限

沿用 S3-LV-002 的两个环境凭证和五项运行门禁，不增加配置。页面加载仍不会调用生产接口，
只有机构用户点击“检查账户连接”时才执行一次读取；Operator 和未认证请求继续返回 403。

## 行为边界

- 原始响应、UID、commission、updateTime、凭证、签名和请求信息不暴露、不持久化；
- 零余额资产不进入响应；非法、重复、负数或超量结果失败；
- Live Run、订单 endpoint、submit/cancel/replace、fill reconciliation、自动重试和恢复仍关闭；
- Backtest/Demo 权限不会继承到 Live。

## 回滚

恢复 refresh response v1，删除 v2 schema、生成客户端字段和 Live 页面资产表即可。该回滚
不会修改外部账户或订单，因为本任务只有显式只读能力。
