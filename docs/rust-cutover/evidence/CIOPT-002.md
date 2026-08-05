# CIOPT-002 - docs-only fast lane Hosted 验证证据

Date: 2026-08-05
Executor: Codex
GitHub issue: #1247
GitHub PR: #1248
Status: FINAL_HOSTED_VALIDATION_PENDING

## 变更范围

本任务只新增 `CIOPT-002` 的 task 与 evidence Markdown。没有修改 workflow、脚本、Rust、
Cargo、产品页面、MVP 冻结源或 release 文件。

## 预期分类

- `heavy_rust=false`；
- `release_verify=false`；
- `institution_workbench=false`；
- `control_center=false`；
- `mvp_acceptance=false`；
- `mvp_fault_matrix=false`；
- `mvp_final_acceptance=false`；
- `security_workflow=false`；
- `security_dependencies=false`。

## Hosted 证据

首轮 docs-only Hosted 验证：

- Rust Cutover Smoke run `30999228447`：success；job 从 `10:53:28Z` 到
  `10:55:32Z`，实际 124 秒；
- heavy Rust、release verify、CLI tests、MVP acceptance、fault matrix、MVP freeze
  candidate、两个门户 contract/browser 与 final summary 全部 skipped；
- 固定治理 gate、golden trace schema、Rust-only 与 Cython removal 均通过；
- security-audit run `30999228351`：`changes` success / 23 秒；zizmor、cargo-audit、
  cargo-deny、cargo-vet、osv-scanner 全部 skipped；
- 九类分类结果符合预期，安全 workflow 从原来的 6 个实际 jobs 收敛为 1 个实际 job。

首轮 smoke 比 120 秒目标多 4 秒。步骤计时显示 `Release surface current guard` 占 67
秒，原因是首次为新 workflow 缓存键编译 `ntpro-governance`；真实 MVP、故障矩阵和浏览器
验收均未运行。当前 evidence-only 提交将触发第二轮 docs-only run，用于验证热缓存时长。

## 行为影响

无产品行为、公共 API、交易权限或发布影响，无迁移说明。

## 回滚

删除本任务文档即可；不涉及代码、数据、部署或 release 回滚。
