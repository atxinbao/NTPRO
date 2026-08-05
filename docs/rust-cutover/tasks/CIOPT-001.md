# CIOPT-001 - GitHub Actions path-aware fast lane

Date: 2026-08-05
Executor: Codex
GitHub issue: #1245
Risk: critical
Owner role: Rust Product Surface Agent
Review role: Verification & Release Gatekeeper
Status: READY_FOR_PR

## 目标

缩短日常 Pull Request 的 GitHub Actions 反馈时间，同时保持 Rust Cutover Smoke 的
`smoke` 必需检查、真实 Rust/MVP 变更的完整验收、每周全量安全扫描、Backend
Performance 基线和 Release Gate 不降级。

## 范围

- 把 Rust Smoke 的路径分类从 workflow 内联逻辑迁移到可直接测试的共享分类器；
- 普通 `project.html`、`README.md` 和产品 Roadmap 文案不再触发真实 MVP 进程、故障矩阵
  与双门户浏览器验收；
- MVP 冻结清单、MVP-012/013、冻结源、验收脚本和 smoke workflow 变化仍触发完整验收；
- security-audit 按 workflow 风险与 Rust 依赖风险分别启动专项扫描；
- schedule、workflow_dispatch 和无法可靠计算 diff 的事件继续 fail closed 全量扫描；
- 为日常 jobs 增加超时，并输出路径分类摘要；
- 审计 Backend Performance、Release Gate 与 Release Publish，确认不以削弱证据换取提速。

## 非目标

- 不修改 Rust runtime、产品 API、策略逻辑、交易权限或冻结 manifest；
- 不修改 `smoke` job 名称或 branch protection 必需检查；
- 不缩减 Backend Performance 六类工作负载；
- 不缩减 release tag 的 30-stage gate；
- 不启用 auto-merge。

## 验收

- 分类器正向、负向和强制全量场景自测通过；
- docs-only 的 Rust/MVP/fault/browser 分类均为 false；
- runtime、Cargo、MVP freeze 和 workflow 变化仍进入对应重检查；
- docs-only security workflow 仅运行快速分类，依赖或 workflow 变化运行对应专项扫描；
- schedule 与手动安全审计始终全量运行；
- workflow YAML、Shell 语法、zizmor 和现有治理 gate 通过；
- Hosted `smoke` 与安全检查通过并获得独立 review；
- 合并后用真实 docs-only PR 证明 `smoke` 目标小于 120 秒。
