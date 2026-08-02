# MVP-005A - M0/M2 交付状态与审查证据收口证据

Date: 2026-08-02
Executor: Codex
GitHub issue: #1209
Status: REVIEW_APPROVED_FOR_MERGE

## GitHub 事实基线

```text
MVP-003: issue #1203 closed; PR #1204 merged; hosted checks 14/14 success;
         GitHub independent review records 0
MVP-004: issue #1205 closed; PR #1206 merged; hosted checks 14/14 success;
         REVIEW_APPROVED record posted 11 seconds after merge
MVP-005: issue #1207 closed; PR #1208 merged; hosted checks 14/14 success;
         REVIEW_APPROVED and FINAL_HEAD_REVIEW_APPROVED recorded before merge
```

## 治理判断

- M0 的身份追溯和四轴状态技术退出条件已经满足；
- MVP-003 缺少合并前独立审查记录，MVP-004 无法由 GitHub 时间线证明审批记录在合并
  前存在；两项作为历史治理例外记录，不虚构、不回写远端历史；
- M2 的共享只读 API 已交付，但当前浏览器仍消费 `/api/server` 和
  `/api/snapshot`，没有消费 `/api/mvp/v1/status`；
- M2 阶段退出必须等待 MVP-006 机构工作台和 MVP-007 控制中心都完成共享 API 绑定；
- M3 尚未开始。

## 审计面与发布面

本次状态审计以 MVP-005 merge `6bc2a7246534` 加 issue #1209 工作树为代码面；当前
正式发布基线 v0.33.0 指向 commit `0ef6fd9d8577`。MVP-003/004/005 均在 v0.33.0
之后合并，不属于 v0.33.0 release 内容，项目页面必须将两者分开标注。

## 变更边界

本任务仅校正文档和治理状态，不改变运行时行为、公开 API、交易语义或 v0.32.0 冻结
发布面。所有提交、变更、外部 Venue、重试、自动补救和真实订单能力继续保持 false。

## 验证结果

```text
scripts/ai/check_docs_examples_governance.sh
PASS: markdown_files=127, local_links=307, image_links=20,
      integration_pages=15, python_fences_classified=203

scripts/ai/check_backend_freeze_baseline.sh
PASS: v0.32.0 tag/SHA, 27 boundaries, 4 source hashes, 20 negative cases

scripts/ai/check_rust_only_runtime.sh
PASS

git diff --check
PASS

Playwright real-browser check
PASS: 1440x1000 desktop and 390x844 narrow viewport
```

首次直接运行治理脚本时 shell 默认 `rustc 1.87.0`，低于项目固定的 Rust 1.95.0；
通过 `source scripts/ai/toolchain_env.sh` 切换到 `rustc 1.95.0` 后，完整验证通过。

## 独立审查证据

```text
Reviewer: 019fc29d-d53a-7152-a07f-4ea12736bee6
Initial findings: P1=0, P2=1, P3=0
Repair: split current audit code surface from v0.33.0 published baseline
Rereview findings: P1=0, P2=0, P3=0
Decision: REVIEW_APPROVED
```

初审指出 `project.html` 把 GitHub main 与 v0.33.0 写成同一代码基线，可能误导读者
认为 MVP-003/004/005 已包含在 v0.33.0。修复后，页面分别标识 MVP-005 merge 加
issue #1209 的审计代码面，以及 commit `0ef6fd9d8577` 的 v0.33.0 正式发布基线，
并明确当前三个 MVP 不属于该 release。
