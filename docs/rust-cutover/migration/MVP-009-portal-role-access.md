# MVP-009 双门户本地角色访问迁移说明

Date: 2026-08-04
Executor: Codex

## 变化

`nautilus dashboard serve` 和 `nautilus mvp serve` 的动态页面与运行数据 API 不再允许
无会话直接访问。Dashboard 启动日志会提供三条 bootstrap URL：

```text
dashboard_url=http://127.0.0.1:<port>/dashboard?access_token=<operator-token>
institution_workbench_url=http://127.0.0.1:<port>/institution-workbench?access_token=<institution-token>
control_center_url=http://127.0.0.1:<port>/control-center?access_token=<operator-token>
```

首次打开角色 URL 时，服务端验证 token、写入角色专属 HttpOnly 会话 Cookie，并 303
重定向到不含 `access_token` 的地址。不要把完整 bootstrap URL 写入 issue、PR、截图或
长期日志；浏览器 smoke 上传日志前会把 token 替换为 `[REDACTED]`。

## 兼容影响

以前直接调用 `/api/snapshot`、`/api/nodes/*` 或 MVP API 的本地脚本会收到 403。脚本
需要先从对应 bootstrap URL 建立会话，再在后续同源请求中携带该 Cookie。静态 CSS 和
JavaScript 路径继续公开，但不包含运行数据。

跨门户事件链接不携带 token。用户只有已经建立目标角色会话时才能完成跳转；否则目标
门户返回 403。这避免把运维访问能力嵌入交易员链接。

## 能力边界

这是 loopback 单机 MVP 的进程级角色访问合同，不是生产身份系统，也不证明组织、租户、
SSO、用户目录或机构权限治理已经完成。它不新增 Supervisor action，不开放真实交易、
订单提交、撤改、adapter send、外部 Venue、重试或自动补救。

## 回滚

回滚 MVP-009 提交会恢复无会话的本地 HTTP 行为。运行状态、注册表、身份合同、四轴状态
合同和 v0.32.0 冻结基线均不需要迁移。
