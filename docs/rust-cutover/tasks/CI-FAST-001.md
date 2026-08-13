# CI-FAST-001 - Pull Request 验证并行化

Date: 2026-08-13
Executor: Codex
GitHub issue: #1304
Risk: medium
Owner role: Verification & Release Gatekeeper
Review role: Rust Product Surface Agent
Status: REVIEW_REQUIRED

## 目标

缩短日常 Pull Request 的 GitHub Actions 等待时间，同时保留格式、治理、Clippy、Rust
单元测试和变更范围对应的产品验收。

## 范围

- Backend Performance 只保留每周定时和手动运行，不再由普通 PR 触发；
- 任意 `crates/**` 或 Cargo 文件不再自动触发冻结 MVP 的进程与故障验收；
- 冻结清单中的 13 个源文件及明确的 MVP 脚本、配置仍触发完整 MVP 验收；
- required smoke 拆为 change classification、core、Rust lint、Rust tests，并行后聚合；
- 删除 Clippy 前重复的 workspace `cargo check`；
- 最终 required check 名称继续为 `smoke`。

## 非目标

- 不修改 runtime、策略、订单、API、数据格式或交易权限；
- 不降低 release tag gate、security audit 或冻结基线；
- 不删除 Backend Performance 工作负载和回归合同；
- 不修改 branch protection。

## 验收

- classifier 正负向、冻结源和 workflow 结构自测通过；
- 五个 workflow YAML 和相关 shell 语法通过；
- 普通 PR 不再启动 Backend Performance；
- Rust lint、Rust tests 与 core 可并行执行；
- 任一必要 lane 失败、取消、错误跳过或分类无效时，最终 `smoke` 必须失败；
- Hosted checks 通过并完成独立审查后再合并。

## 回滚

回退本任务提交即可恢复串行 smoke 和 PR 性能矩阵，不涉及产品、数据或发布回滚。
