# S3-LV-006 生产行情 Runtime 迁移说明

Date: 2026-08-12
Executor: Codex

## API 变化

`POST /api/product/v1/live-run-candidates/{run_id}/actions` 新增
`action=start_market_data`。Live Run 生命周期新增 `starting`、`market_data_running`、
`stopping` 和 `failed`；候选响应新增行情连接、Supervisor node、进程状态和归一化错误字段。

消费方必须重新生成 Product API 客户端，并按生命周期渲染状态。只有
`market_data_running` 可以同时声明 `runtime_started=true`、`market_data_connected=true`；
任何工件、进程或执行边界漂移都必须 fail closed。

## 部署变化

生产行情节点只读取以下环境凭证：

```text
NTPRO_BINANCE_LIVE_API_KEY
NTPRO_BINANCE_LIVE_API_SECRET
```

Live Run 仍要求 S3-LV-004 的五项显式候选门禁和 S3-LV-005 的外部审计锚点配置。配置工件
只保存环境变量名称，不保存凭证值。Supervisor 使用候选 Run ID 作为 node ID，并保存不可变
Run ownership 和终态锚点。

## 行为边界

本次只注册 Binance Spot 生产行情客户端，不注册执行客户端。订单 endpoint、submit、cancel、
replace、fill reconciliation、自动 retry/remediation/recovery 和交易控件全部保持关闭。
页面加载不会自动启动、停止或重连 Runtime。`automatic_reconnect_allowed=false` 约束的是
NTPRO Supervisor 不自动重启 Runtime；每个心跳仍检查真实数据客户端状态，空客户端或断连
会优雅停止节点并由 Product API 记录失败，而不会继续投影为在线。

## 回滚

先通过 Product API 人工停止活动 Live Run，确认 Supervisor 进程退出和 `stopped` 外部锚点后，
再回滚应用二进制与前端 bundle。不得删除审计回执、Supervisor ownership 或历史候选工件；
不得用开放订单能力作为行情启动失败的降级路径。
