# MVP-008A - M3 事件关联合并后状态收口

Date: 2026-08-04
Executor: Codex
GitHub issue: #1219
Risk: low
Owner role: Documentation & Governance Agent
Status: LOCAL VALIDATION PASSED

## 目标

在 MVP-008 / PR #1218 合并后，把任务、证据、README、产品 Roadmap 和本地项目页从
PR 阶段状态收口为真实远端状态，并明确 M3 剩余范围。

## 范围

- 登记 issue #1217、PR #1218、merge SHA、最终 head、独立审查和 hosted runs；
- 登记两个门户 browser artifact 的名称与 digest；
- 将 MVP-008 task/evidence 标记为 DONE；
- 将 README、Roadmap 和本地 `project.html` 中跨门户事件跳转标记为已完成；
- M3 保持开放，剩余服务端角色边界和生命周期动作产品化继续独立立项；
- 不修改运行时、API、workflow、Cargo 配置或 v0.32.0 冻结文件。

## 验收

```bash
scripts/ai/check_docs_examples_governance.sh
scripts/ai/check_rust_only_runtime.sh
scripts/ai/check_backend_freeze_baseline.sh
scripts/ai/verify_release.sh current-governance backend-freeze-baseline
git diff --check
```

本任务不发布新版本，不新增 Supervisor action、交易、重试、自动补救或外部 Venue
能力，也不关闭 M3。
