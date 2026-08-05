# DOCOPS-001 - NTPRO 系统运行与运营操作说明书

Date: 2026-08-05
Executor: Codex
GitHub issue: #1239
GitHub PR: PENDING
Risk: low
Owner role: Docs & Developer Experience Agent
Review role: Verification & Release Gatekeeper
Status: DONE_ON_MERGE

## 目标

将已冻结的单 Supervisor + 单沙盒节点 MVP 运行逻辑整理为一份面向交易员、运维、
产品负责人和首次接触项目人员的中文说明书，并提供可执行的启动、巡检、停止、故障处理
和人工恢复步骤。

## 范围

- 新增 `docs/product/ntpro_system_operations_manual.md`；
- 用非技术化语言解释 Supervisor、node、策略实例、双门户和 workspace；
- 记录当前准确拓扑、四轴状态、角色分工、日常运营、故障处置和未来演进边界；
- 在 README 增加说明书入口，并将 MVP-013 的陈旧执行中状态改为已完成冻结；
- 新增本任务与验证证据文档。

## 非目标

不修改 Rust 运行时、API、workflow、冻结 manifest、历史 release 或 GitHub Release；
不开放真实交易、外部 Venue、多节点、生产 IAM、自动 retry、remediation 或 recovery。

## 验收

- 说明书与 MVP freeze manifest、roadmap、发布/回滚说明和实际 CLI 参数一致；
- 当前能力和未来规划明确分离；
- README 不再把 MVP-013 标记为执行中；
- docs governance、MVP freeze、Rust toolchain、backend freeze 和 current governance 通过；
- diff 仅包含 Issue #1239 允许的四个文档路径；
- PR 合并后 `DONE_ON_MERGE` 生效并关闭 Issue #1239。
