# MVP-010A - M3 生命周期动作合并后状态收口

Date: 2026-08-04
Executor: Codex
GitHub issue: #1227
Risk: low
Owner role: Documentation & Governance Agent
Status: LOCAL VALIDATION PASSED

## 目标

在 MVP-010 / PR #1226 合并后，把任务、证据、README、产品 Roadmap 和本地项目页从
合并前状态收口为真实远端状态，正式关闭 M3，并把下一阶段切换到 M4 MVP 验收与冻结。

## 范围

- 登记 issue #1225、PR #1226、final head、merge SHA、独立审查和 hosted runs；
- 登记双门户 browser artifact 结果、digest 与令牌扫描；
- 将 MVP-010 task/evidence 标记为 DONE；
- 将 README、Roadmap 和 `project.html` 的 M3 状态标记为已完成；
- 明确控制中心只开放本地单节点 sandbox operator start/stop；
- 保持真实交易、多节点、生产 IAM、自动重试/补救和外部 Venue 能力关闭；
- 不修改运行时、API、workflow、Cargo 配置或 v0.32.0 冻结文件。

## 验收

```bash
scripts/ai/check_docs_examples_governance.sh
scripts/ai/check_rust_only_runtime.sh
scripts/ai/check_backend_freeze_baseline.sh
scripts/ai/verify_release.sh current-governance backend-freeze-baseline
git diff --check
```

本任务不发布版本、不新增产品能力。M3 关闭只表示双角色最小界面及其共享证据闭环已交付；
M4 仍需完成干净环境、故障注入、可重复演示与 MVP 冻结验收。
