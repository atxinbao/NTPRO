# MVP-005 共享只读状态 API 迁移说明

Date: 2026-08-02
Executor: Codex

## 新增接口

本变更新增本地 loopback Dashboard 接口：

```text
GET /api/mvp/v1/status
```

该接口只在现有 Dashboard HTTP server 上提供，不新增监听地址或远程服务。现有
`/api/snapshot`、`/api/nodes/*` 和 v0.28 只读接口保持不变。

## 消费规则

- 机构工作台与控制中心都使用 `ntpro.mvp_shared_status_api.v1`，不得分别读取原始
  registry、node status、metrics 或 Event Store；
- `identity` 是跨门户对象关联基础，`node_id` 与 `strategy_instance_id` 仍是不同身份；
- `status` 的四个轴必须分别解释，不能把 HTTP 200 或 `runtime=running` 当作
  `technical_health=healthy`；
- `business.availability=missing` 表示 Unified Read Model 尚未交付，不是空账户或
  零仓位；
- `business.availability=stale|error|identity_mismatch` 时不得继续使用旧业务摘要作
  实时决策；
- `trading_readiness.status` 在当前 MVP 中始终为 `blocked`。

## 兼容性

这是新增的版本化只读 HTTP API，不改变现有 CLI 参数或 Rust 公共库 API。调用方应
固定检查 `schema_version` 和 `contract_version`，并完整处理 409、500、503 错误信封。

## 能力边界

本接口不暴露原始 Event Store、秘密字段、原始 Venue payload、订单票据或交易命令。
非 GET 方法返回 405；订单提交、订单变更、真实 Venue、自动重试和自动补救能力均为
false。
