# MVP-006 机构工作台迁移说明

Date: 2026-08-03
Executor: Codex

## 新增产品入口

Dashboard loopback server 新增：

```text
http://<loopback-bind>/institution-workbench
```

CLI 启动输出新增 `institution_workbench_url`，现有 `dashboard_url` 保持不变。

## 消费变化

- 机构业务用户使用 `/institution-workbench`；
- 运维现有 `/dashboard` 暂时保持原样，待 MVP-007 完成控制中心接入；
- 机构工作台只消费 `/api/mvp/v1/status`，不得改回 `/api/snapshot` 或原始工件；
- 共享 API 不可用或边界异常时页面清空旧事实并阻断。
- 机构工作台 HTML、静态资源和共享状态 API 均为严格 GET-only；HEAD 和其他非 GET
  方法现在显式返回 405。

## 兼容性

这是新增本地只读页面和 CLI 输出字段，不改变 CLI 参数、共享 API schema 或 Rust 公共
库 API。现有 Dashboard URL 保持兼容，不需要数据迁移。`/api/mvp/v1/status` 的 HEAD
请求由隐式 GET 行为收紧为 405；GET 响应保持不变。
