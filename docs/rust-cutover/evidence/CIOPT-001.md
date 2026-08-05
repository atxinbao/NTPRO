# CIOPT-001 - GitHub Actions 整体优化证据

Date: 2026-08-05
Executor: Codex
GitHub issue: #1245
GitHub PR: #1246
Status: LOCAL_VALIDATION_PASSED_REVIEW_REQUIRED

## 修改前基线

- 最近 80 次 workflow runs 中，Rust Cutover Smoke 平均约 398 秒；
- docs-only PR #1244 的成功 run `30993920564` 用时 7 分 23 秒；
- 该 run 中 `MVP clean deterministic acceptance` 单步用时 5 分 21 秒；
- 原分类规则把 `project.html`、`README.md` 和普通产品 Roadmap 视为 MVP 最终冻结变更，
  强制启动构建、故障矩阵和两个产品门户浏览器 smoke；
- security-audit 对所有 PR 固定启动 6 个 jobs：changes、zizmor、cargo-audit、
  cargo-deny、cargo-vet 和 osv-scanner。

## 实现

- 新增 `scripts/ci/classify-ci-changes.sh`，集中输出 Rust、release、MVP、门户、workflow
  安全和依赖安全九类布尔结果；
- 新增 `scripts/ci/test-ci-change-classifier.sh`，覆盖 docs-only、runtime、freeze、Cargo、
  workflow、security workflow 和强制全量七种场景；
- Rust Smoke 保留单一 `smoke` job，只把内联分类替换为共享分类器；
- 从 MVP 最终验收触发面删除普通 `project.html`、`README.md` 和
  `docs/product/roadmap.md`，但保留所有日常治理检查；
- security-audit 的 `changes` job 负责事件 diff；zizmor 仅响应 workflow 风险，四类依赖
  扫描仅响应 Cargo/供应链风险；定时、手动和无法确定 diff 时强制两类全开；
- Rust Smoke 超时为 45 分钟，security 分类为 5 分钟、zizmor 为 10 分钟、依赖扫描为
  15 分钟；concurrency 与 cancel-in-progress 保持原样。

## 未修改的重门禁

- Backend Performance 仍保留六类 workload、同 runner baseline/candidate 对比与 60 分钟
  超时；
- Rust Cutover Release Gate 仍保留 30 个 release stages、90 分钟超时和 tag/手动触发；
- Release Publish 仍为手动、绑定成功 release gate 后发布；
- branch protection 的必需检查名称仍为 `smoke`。

## 本地验证

- `bash -n scripts/ci/classify-ci-changes.sh scripts/ci/security-audit-gate.sh scripts/ci/test-ci-change-classifier.sh`：PASS；
- `scripts/ci/test-ci-change-classifier.sh`：PASS，`cases=7`；
- `security-audit-gate.sh` 的真实 PR commit diff 与 schedule 事件模拟：PASS，PR 和定时
  场景均按预期输出两类安全检查；
- 五个 workflow YAML 解析：PASS；
- `scripts/ai/verify_fast.sh`：PASS，Rust 与 Cargo 均固定为 `1.95.0`，fmt 通过；
- `scripts/ai/check_docs_examples_governance.sh`：PASS，134 个 Markdown、315 个本地链接；
- `scripts/ai/check_mvp_freeze_baseline.sh`：PASS，19 个关闭边界、13 个冻结源；
- `scripts/ai/check_backend_freeze_baseline.sh`：PASS，v0.32.0 的 27 个关闭边界和 4 个
  冻结源哈希未变化；
- `scripts/ai/check_zero_python_closeout.sh`、`check_backend_hygiene.sh`、
  `check_rust_only_runtime.sh`、控制面与历史发布退役 gate：PASS；
- `scripts/ai/verify_release.sh current-governance backend-freeze-baseline v33-maintenance-release`：
  PASS；
- live branch protection：`strict=true`，唯一 required context 仍为 `smoke`；
- `git diff --check`：PASS；
- Hosted workflow 与独立 review：等待 PR。

## 行为影响

仅改变 GitHub Actions 对变更范围的调度，不修改产品运行时、API、交易语义、真实 Live
权限或发布边界。分类失败时安全审计继续 fail closed。

## 回滚

回退本任务提交即可恢复原 workflow 内联分类和每个 PR 全量安全扫描；不涉及数据、schema、
部署或 release 回滚。
