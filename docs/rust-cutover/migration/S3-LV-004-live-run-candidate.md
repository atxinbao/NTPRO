# S3-LV-004 Live Run 候选迁移说明

Date: 2026-08-12
Executor: Codex

## 新增合同

本任务新增独立资源 `/api/product/v1/live-run-candidates`。它不是旧 `/runs` 中 Live 占位记录
的升级，也不复用 Demo Run 生命周期。创建请求必须显式绑定当前策略、版本、账户和 Venue：

```json
{
  "strategy_id": "ema_cross_btcusdt_v1",
  "strategy_version_id": "ema_cross_btcusdt_v1@v1",
  "environment": "live",
  "account_ref": "account://live/binance/primary",
  "venue_ref": "venue://live/BINANCE",
  "user_confirmed": true
}
```

动作端点仅接受：

```json
{"run_id":"live-<id>","action":"preflight","user_confirmed":true}
{"run_id":"live-<id>","action":"stop","user_confirmed":true}
```

## 状态语义

- `created`：候选和来源已冻结，尚未执行生产启动前检查；
- `preflight_ready`：账户连接与交易权限已验证，但真实 Runtime 仍未启动；
- `stopped`：机构用户已人工终止候选，不能恢复或再次执行动作。

消费者不得把 `preflight_ready` 显示为运行中。只有后续独立任务交付真实市场数据 Runtime 后，
才可以引入 `running` 状态。

## 部署配置

除 S3-LV-002 的生产账户只读门禁和凭证外，部署方还必须显式提供五项候选门禁：

```text
NTPRO_S3_LIVE_RUN_CANDIDATE_CREATE=1
NTPRO_S3_LIVE_RUN_OWNER_APPROVED=1
NTPRO_S3_LIVE_RUN_NO_ORDER_SEND=1
NTPRO_S3_LIVE_RUN_MANUAL_STOP=1
NTPRO_S3_LIVE_RUN_RISK_APPROVED=1
```

当前节点配置还必须保持 `risk.kill_switch_enabled=true` 和 `risk.kill_switch_active=false`。
该风险配置按严格 `[risk]` schema 规范化后以完整内容 SHA-256 写入候选来源；未知字段直接
拒绝，详情读取和 preflight 都重新比对当前内容。缺失任一配置或内容漂移时均 fail closed。
页面不会自动触发账户检查、创建或 preflight。

## 本地状态与异常恢复

- `artifacts/live-runs/<run_id>/state-<revision>.json` 是不可覆盖的候选状态 revision；
- `artifacts/live-runs/<run_id>/state-head.json` 原子固定当前最高 revision；
- `artifacts/live-run-state-commits/<run_id>.state.<revision>.json` 是独立的工作区 commit 链；
- `artifacts/.live-run-mutation.lock` 串行化同一工作区内的创建和动作，进程正常退出时自动删除；
- 若进程崩溃后锁文件仍存在，所有候选写入会返回冲突。运维必须先确认没有候选写进程，再人工
  删除锁文件；系统不会根据时间自动接管陈旧锁。

这些本地哈希链用于发现单一工件删除、篡改、单链回滚和不完整写入，不是 WORM。拥有整个
workspace 写权限的主体若同步恢复候选、commit 和 head 的完整旧快照，本任务无法检测。真实
Runtime 或订单能力启用前，必须另行接入外部不可回滚审计账本或单调 CAS 控制面。

## 订单边界

候选响应中的 submit、cancel、replace 和 fill reconciliation 必须全部为 `blocked`；
`runtime_started`、订单发送尝试和真实订单计数必须为 `false`。该版本不访问任何订单 endpoint，
不调用 execution adapter send，也不具备自动重试或恢复能力。

## 回滚

删除新增候选路由、`artifacts/live-runs/`、`artifacts/live-run-state-commits/` 与确认无写进程后的
`artifacts/.live-run-mutation.lock`，并移除前端候选区域和五项门禁即可。回滚不会修改交易所
账户或订单，因为本任务没有外部订单副作用。
