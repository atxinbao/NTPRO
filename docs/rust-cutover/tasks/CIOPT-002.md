# CIOPT-002 - docs-only fast lane Hosted 验证

Date: 2026-08-05
Executor: Codex
GitHub issue: #1247
Risk: low
Owner role: Docs & Developer Experience Agent
Review role: Verification & Release Gatekeeper
Status: READY_FOR_PR

## 目标

在 CIOPT-001 合并后，以真实 docs-only Pull Request 验证 GitHub Actions fast lane，
记录 required `smoke` 和 security-audit 的 job 数、耗时与 skipped 状态。

## 范围

- 新增本任务和验证 evidence；
- 首轮 Hosted checks 完成后回写 run ID、分类、耗时和 job 状态；
- 复核 branch protection required context 仍为 `smoke`；
- 通过后关闭 CIOPT-001 #1245 与本任务 #1247。

## 非目标

- 不修改 workflow、CI 脚本、Rust、Cargo、产品页面或配置；
- 不修改 MVP 冻结源、release 文件或交易权限；
- 不重新运行真实 MVP 进程、故障矩阵或门户浏览器验收。

## 验收

- docs-only 分类的 heavy Rust、MVP acceptance、fault matrix 和门户 browser 全部为 false；
- required `smoke` 成功且目标小于 120 秒；
- security-audit 仅执行 `changes`，五个专项 jobs 全部 skipped；
- 文档治理、冻结基线与 `git diff --check` 通过。
