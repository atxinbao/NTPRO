# MVP-007 控制中心共享状态入口迁移说明

Date: 2026-08-03
Executor: Codex
Task: MVP-007 / GitHub issue #1213

## 新增公开本地路由

```text
GET /control-center
GET /assets/control-center.css
GET /assets/control-center.js
GET /api/mvp/v1/control-center
```

这些路由由本地 loopback Dashboard server 提供，严格拒绝 HEAD 和其他非 GET 方法。
既有 `/dashboard`、`/institution-workbench` 和 API 路由不变，不需要客户端迁移。

## 消费合同

控制中心同时读取：

- `GET /api/mvp/v1/status`：身份、四轴状态、业务影响、来源与关闭边界；
- `GET /api/mvp/v1/control-center`：最小化的本地节点、组件、日志、指标、告警、能力
  缺口和 registry provenance。

两份响应必须在 node identity、registry provenance、生命周期和沙盒边界上互相一致；
不一致时页面清空旧数据并阻断。专用运维投影不包含完整 Dashboard snapshot 的 controls、
订单/生产操作证据、账户引用或原始错误文本，页面也不调用 `/api/nodes/*/actions/*`。

## 能力影响

这是新增的只读产品入口，不授权真实交易、节点动作、外部 Venue、订单提交/变更、重试
或自动补救。需要生命周期动作的旧本地 Dashboard 用户继续使用既有受控接口；该能力
不会被自动带入控制中心。
