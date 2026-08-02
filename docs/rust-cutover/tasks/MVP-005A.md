# MVP-005A - M0/M2 交付状态与审查证据收口

Date: 2026-08-02
Executor: Codex
GitHub issue: #1209
Risk: medium
Owner role: Control & Scope Agent
Review role: Verification & Release Gatekeeper
Status: REVIEW_APPROVED_FOR_MERGE

## 目标

以 GitHub 远端事实为准，统一 README、产品 Roadmap、项目页面和 MVP-003/004/005
证据状态，消除已合并任务仍显示 `REVIEW_REQUIRED`、共享 API 已交付却仍写“待建”
以及 M2 退出条件可能被提前关闭的问题。

## 范围

- 标记 M0 技术实现已交付，并登记 MVP-003/004 的历史审查证据例外；
- 标记 MVP-005 已审查、hosted checks 成功且已合并；
- 标记 M2 共享 API 已交付，但双门户消费退出条件保持开放；
- 明确 MVP-006 与 MVP-007 是关闭 M2 的共同依赖；
- 同步 README、`docs/product/roadmap.md` 和 `project.html`。

## 非目标

- 不修改 Rust 运行时、API schema 或路由；
- 不实现机构工作台或控制中心；
- 不修改 v0.32.0 冻结发布文件；
- 不补写不存在的历史审批，不授权真实交易能力。

## 验收

```bash
git diff --check
scripts/ai/check_docs_examples_governance.sh
scripts/ai/check_backend_freeze_baseline.sh
scripts/ai/check_rust_only_runtime.sh
```

`project.html` 还须通过真实浏览器桌面与窄屏检查，确认文本无溢出、重叠或不可读。

## 审查状态

独立 Verification & Release Gatekeeper 初审发现 1 个 P2：`project.html` 混写当前
审计代码面与 v0.33.0 发布基线。修复后复审 P1/P2/P3 均为 0，结论
`REVIEW_APPROVED`。不得启用自动合并，仍须等待 final head 确认和 hosted checks。
