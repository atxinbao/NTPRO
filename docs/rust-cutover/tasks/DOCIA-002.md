# DOCIA-002 - project.html 策略三模式产品北极星

Date: 2026-08-05
Executor: Codex
GitHub issue: #1243
GitHub PR: pending
Risk: low
Owner role: Docs & Developer Experience Agent
Review role: Verification & Release Gatekeeper
Status: REVIEW_REQUIRED

## 目标

将 `project.html` 的产品主线收敛为一个量化策略的完整生命周期：同一个
`Strategy + StrategyVersion` 可以分别创建 Backtest、Demo（代码中的 Sandbox）和
Live Run，并在一个策略工作台中完成配置、运行、观察、比较和复盘。

Live 是必达产品能力，不是可选远期设想；但当前冻结基线仍禁止真实 Venue、真实订单和
交易控件，本任务不得把目标写成已实现能力。

## 范围

- 产品说明改为当前北极星、策略闭环、三模式合同、策略工作台和 S0-S4 推进路线；
- 同步 `docs/product/roadmap.md` 的权威产品路线，保留既有发布与冻结事实；
- 默认工作台保留左侧菜单、顶部上下文、主画布、右侧抽屉、底部活动区和状态栏；
- 左侧栏目围绕策略、Backtest、Demo、Live、运行、数据、风险和系统状态组织；
- 顶部固定 Strategy、StrategyVersion、Environment、账户与 Venue 上下文；
- 明确三种模式共享策略版本和交易语义，每个 Run 的环境、数据、账户、Venue、适配器、
  权限和结果保持独立；
- 从产品说明中删除旧的机构化双门户设计，不进入当前产品 Roadmap；
- 技术说明增加 `Environment`、`BacktestEngine` 和 `LiveNode` 的源码事实；
- 执行桌面/移动浏览器、hash、交互、打印、链接、文档治理和冻结基线验证。

## 非目标

- 不修改 Rust runtime、API、workflow、release 或冻结 manifest；
- 不启用 external Venue、真实账户、真实订单、submit/mutation、自动恢复或 Live 控件；
- 不建设机构租户、多人角色、多节点集群或平台控制中心；
- 不修改 v0.32.0 后端冻结文件。

## 验收

- 首页一句话说明“同一个策略从 Backtest 到 Demo，再进入真实 Live”；
- 产品默认可见内容不再以机构组织、平台运维和双门户为中心；
- 工作台完整呈现三种模式，共享 StrategyVersion 且 Run 身份独立；
- Live 明确为 S3 必达能力，同时当前状态明确为未开放；
- Backtest 或 Demo 结果不能被解释为 Live 自动授权；
- 1440x1000 与 390x844 无重叠、空白主内容或页面级横向溢出；
- 工作台右侧抽屉和底部四个活动页签可用；
- docs governance、MVP freeze、链接和 `git diff --check` 通过；
- PR 合并后状态更新为 `DONE_ON_MERGE` 并关闭 Issue #1243。
