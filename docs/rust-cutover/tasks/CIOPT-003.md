# CIOPT-003 - MVP 浏览器验收优雅退出超时竞态修复

Date: 2026-08-06
Executor: Codex
GitHub issue: #1264
GitHub PR: #1265
Risk: high
Owner role: Rust Product Surface Agent
Review role: Verification & Release Gatekeeper
Status: REVIEW_REQUIRED

## 目标

消除机构工作台浏览器验收外层退出等待与 MVP node 内层停止超时相等造成的竞态，同时
保持优雅退出、最终停止状态和禁用交易边界的验收强度不变。

## 范围

- 显式固定测试使用的 node 停止超时；
- 让测试进程退出等待严格大于 node 内层停止超时；
- Supervisor 对正常停止写入 `shutdown_escalated=false`，对 TERM/KILL escalation 写入
  `shutdown_escalated=true` 并返回错误；
- escalation 审计写入失败不得阻止 TERM/KILL，审计错误必须合并到最终错误；
- 增加超时关系和真实异常子进程的正向、负向自测；
- 保留超时失败、SIGKILL 兜底和完整停止证据检查；
- 更新两个 MVP 冻结源 hash 和风险盘点。

## 非目标

- 不修改 node、产品 API、页面或交易能力；
- 不放宽浏览器业务断言、交易边界或停止证据；
- 不修改其他门户验收脚本；
- 不修改 v0.32.0 冻结发布文件。

## 验收

- 外层等待严格大于 node 内层超时，等于、小于和非正数均被自测拒绝；
- 正常退出继续要求 `mvp.serve status=stopped`、`final_state=Stopped` 和禁用交易证据；
- Supervisor escalation 即使最终获得 Stopped artifact 也必须返回错误；
- 异常子进程探针必须实际进入 SIGKILL 路径并证明测试 fail closed；
- 异常探针必须在真实 MVP 启动前完成，探针自身始终清理；
- 机构工作台浏览器验收连续通过；
- MVP freeze、CI 分类、Rust-only 和 hosted Smoke 通过；
- 独立复审通过后手动合并，不启用 auto-merge。
