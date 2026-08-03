# MVP-007A - M2 双门户消费合并后状态收口

Date: 2026-08-03
Executor: Codex
GitHub issue: #1215
Risk: low
Owner role: Documentation & Governance Agent
Status: REVIEW_APPROVED / HOSTED_CHECKS_PENDING

## 目标

在 MVP-007 / PR #1214 合并后，把任务、证据、README、Roadmap 和项目页从 PR 阶段
状态收口为真实远端状态，并正式关闭 M2 双门户消费退出条件。

## 范围

- 登记 PR #1214、merge SHA、hosted run 和双门户浏览器工件；
- 将 MVP-007 task/evidence 标记为 DONE；
- 将 M2 标记为已完成，同时保持 M3 开放；
- 不修改运行时、API、workflow 或 v0.32.0 冻结文件。

## 验收

```bash
scripts/ai/check_docs_examples_governance.sh
scripts/ai/check_rust_only_runtime.sh
scripts/ai/check_backend_freeze_baseline.sh
scripts/ai/verify_release.sh current-governance backend-freeze-baseline
git diff --check
```

本任务仅治理合并后状态，不发布新版本，不改变任何交易或 Supervisor 能力边界。
