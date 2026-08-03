# MVP-008 双门户事件关联迁移说明

Date: 2026-08-04
Executor: Codex
Task: MVP-008 / GitHub issue #1217

## 新增公开本地路由

```text
GET /api/mvp/v1/event-correlation
```

该路由由现有 loopback Dashboard server 提供，只支持 GET。既有
`/api/mvp/v1/status` 和 `/api/mvp/v1/control-center` 的 schema、合同版本与行为不变。

## 消费迁移

机构工作台和控制中心在读取各自既有状态投影的同时读取事件关联投影。客户端必须：

- 校验 `ntpro.mvp_event_correlation_api.response.v1` 与
  `ntpro.mvp_event_correlation_api.v1`；
- 将事件 ID 与共享 identity contract、node 和 strategy instance 逐项比对；业务影响、
  技术根因与观察时间继续只从同一次共享状态响应读取，不能信任关联响应重复动态事实；
- 只信任合同中的两个固定相对路径，并自行对 `event_id` 做 URL 编码；
- 在 URL 已指定一个 `event_id` 时拒绝任何不一致关联，并拒绝任意多个同名参数；
- 在 HTTP、JSON、schema、身份、状态、链接或边界异常时清空旧关联数据并 fail closed。

没有消费跨门户关联的既有本地客户端无需迁移。新客户端不得把该投影解释为原始事件流
或 Event Store 查询结果。

## 能力影响

这是新增的只读状态观察关联，不授权服务端角色、Supervisor action、外部 Venue、交易、
重试或自动补救。原始 Event Store、source path、错误正文和凭证不进入响应。
