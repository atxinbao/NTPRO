# MVP-004 四轴状态合同迁移说明

Date: 2026-08-02
Executor: Codex

## 新增工件

`nautilus mvp serve` 现在在身份合同旁生成：

```text
<workspace>/mvp/status_contract.json
```

其 schema version 为 `ntpro.mvp_status_contract.v1`。这是新增的只读工件，不修改
既有 Supervisor registry、node status、node metrics 或 Dashboard HTTP 合同。
运行期间该工件按节点心跳间隔原子刷新；干净停止后写入最终停止状态。
新鲜度最大年龄为节点心跳间隔的三倍，且不小于 2 秒。status 与 metrics 时间戳
缺失、非法、超时或代际不一致时，技术健康不得为 `healthy`。启动时会先清除同一
工作区的旧身份与状态合同，避免旧证据被本次运行继承。

## 消费规则

- `research` 只表示回测结果引用已绑定，不表示研究通过或盈利；
- `runtime` 表示 Supervisor 进程和节点生命周期；
- `technical_health` 必须由 status、metrics 和错误共同确认；
- 身份合同缺失、不可读、非法或与当前运行身份不一致时，研究状态和技术健康报错；
- `trading_readiness` 在当前只读沙盒 MVP 中始终为 `blocked`；
- Unified Read Model 只有在路径是可读 JSON 对象时才标记为 available；
- HTTP 成功和进程存活均不能单独解释为技术健康；
- 干净停止对应 `not_running`，不是 `unhealthy`。

## 能力边界

本变更不新增 HTTP API、前端、写操作、真实 Venue 连接、订单提交、订单变更、自动
重试或自动补救。无需迁移既有调用方。
