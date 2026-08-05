# DOCIA-002 - project.html 策略三模式产品北极星证据

Date: 2026-08-05
Executor: Codex
GitHub issue: #1243
GitHub PR: #1244
Status: REVIEW_REQUIRED

## 代码事实

- `crates/common/src/enums.rs` 定义 `Environment::Backtest`、`Sandbox` 和 `Live`；
- `crates/backtest/src/engine.rs` 的 `BacktestEngine` 负责历史事件回放与模拟执行；
- `crates/live/src/builder.rs` 的 `LiveNodeBuilder` 接受 `Sandbox | Live`，拒绝
  `Backtest`，因此 Demo 和 Live 共用实时运行语义；
- `crates/cli/src/live/node_runtime.rs` 当前仅构建 `Environment::Sandbox`，并固定
  `external_venue_connection=false`、`real_orders_submitted=false`；
- `docs/product/mvp_freeze_manifest.json` 将当前冻结能力限定为 sandbox，并明确真实订单
  未提交；
- `docs/rust-cutover/TASK_EXECUTION.md` 禁止任务自动继承 submit、mutation、adapter send、
  live exchange、retry、remediation、recovery 或 trading-control 能力。

## 产品决定

- NTPRO 当前北极星是一个策略工作台，不是先建设机构租户或平台运维产品；
- 同一个不可变 StrategyVersion 可分别创建 Backtest、Demo 和 Live Run；
- 三种模式共享策略逻辑、参数结构、订单语义、风险指标和证据格式；
- 每个 Run 独立记录 Environment、数据、配置、账户、Venue、适配器、权限和结果；
- Live 是 S3 必达产品能力，但当前 UI 必须显示未开放，不能渲染真实交易控件；
- 旧机构化双门户设计已从产品说明删除，不再作为当前产品 Roadmap 或可展开背景。
- `docs/product/roadmap.md` 已同步为 S0-S4 策略三模式路线，后半部发布与冻结事实保持
  原样。

## 外部布局参考

- Figma Make 文档用于确认固定左侧菜单、顶部上下文、主画布、右侧详情、底部活动区和
  状态栏的布局关系；
- 本任务只吸收布局关系，继续沿用 `project.html` 已确认的浅灰文档底色和炭黑墨绿工作台
  视觉语言，不复制参考页面的资产管理内容。

## 验证

- `git diff --check`：PASS；
- `scripts/ai/check_docs_examples_governance.sh`：PASS，134 个 Markdown 文件、315 个
  本地链接和 20 个图片链接通过；
- `scripts/ai/check_mvp_freeze_baseline.sh`：PASS，19 个关闭边界、13 个冻结源、2 个
  浏览器视口和 6 组性能工作负载保持有效；
- `scripts/ai/check_backend_freeze_baseline.sh`：PASS，v0.32.0 tag/SHA、27 个关闭边界和
  4 个冻结源哈希保持有效；
- 桌面 `1440x1000`：PASS，页面宽度等于视口宽度，策略工作台、右侧抽屉和三模式状态
  可见；
- 移动 `390x844`：PASS，页面宽度等于视口宽度，策略工作台宽 354px、左边距 18px，
  无页面级横向溢出；
- 底部活动区：PASS，持仓、活动、成交、日志四个页签可切换并刷新表格；
- 右侧抽屉：PASS，收起后 `drawer-open=false`、`aria-expanded=false`；
- URL hash：PASS，直接打开 `#frontend-ia` 显示产品说明，直接打开 `#backend` 自动切换
  技术说明并隐藏产品章节；
- 旧机构化双门户入口：PASS，源码和浏览器正文均不存在；
- 打印模式：PASS，10 个产品/技术主章节全部可见；
- 23 个 `project.html` 本地链接：PASS，HTTP 状态全部为 200；
- 浏览器控制台：PASS，error 和 warning 均为 0；
- hosted checks 和 PR merge：由 GitHub 提供。
