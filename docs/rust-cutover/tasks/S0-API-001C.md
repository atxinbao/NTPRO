# S0-API-001C - Run 只读 API 与三环境身份合同

Date: 2026-08-06
Executor: Codex
GitHub issue: #1260
Risk: high
Owner role: Rust Product API Agent
Review role: Verification & Release Gatekeeper
Status: REVIEW_REQUIRED

## 依赖

- S0-API-001A #1258 已通过 PR #1262 合并；
- S0-API-001B #1259 已通过 PR #1263 合并；
- 父总控 S0-API-001 #1256 保持开放。

## 目标

交付 Run 列表与详情只读端点。Run 是独立于 Strategy 和 StrategyVersion 的执行记录，
同一不可变版本可以被 Backtest、Sandbox 和 Live 三种环境分别引用；Live 记录仅表达配置
与历史事实，不产生外部连接、实盘准入或订单权限。

## 范围

- `GET /api/product/v1/runs`；
- `GET /api/product/v1/runs/{run_id}`；
- strategy、version、environment、lifecycle 过滤与稳定游标分页；
- 稳定身份、环境、数据/配置/适配器/账户/Venue 引用、生命周期、结果引用、风险引用、
  来源、错误和时间合同；
- 未知引用、归属错配、来源陈旧、非法环境、状态时间矛盾和禁用能力异常 fail closed；
- OpenAPI/schema compatibility、Rust 路由和真实浏览器验收。

## 非目标

- 创建、启动、停止或取消 Run；
- 详细结果、风险指标和比较 API；
- 真实 Venue、订单提交/撤改、自动重试、补救或交易控件；
- Backtest、Demo、Live 产品页面；
- v0.32.0 冻结发布文件。

## 验收

```bash
cargo test -p nautilus-cli --lib dashboard::product_api::tests
cargo test -p nautilus-cli --lib dashboard::server::tests
cargo test -p nautilus-cli --lib
cargo clippy -p nautilus-cli --all-targets --all-features -- -D warnings
node scripts/ai/test_strategy_workbench_browser.mjs
scripts/ci/test-ci-change-classifier.sh
scripts/ai/check_backend_runtime_risk_inventory.sh
scripts/ai/test_backend_runtime_risk_inventory.sh
scripts/ai/check_rust_only_runtime.sh
scripts/ai/check_mvp_freeze_baseline.sh
git diff --check
```

高风险产品合同变更必须完成独立审查与 hosted checks 后手动合并，不启用自动合并。

## 实施结果

- 已新增 Run 列表与详情两个受共享读取权限保护的 Axum GET 路由；
- 已从稳定节点配置快照投影 Backtest、Sandbox、Live 三类 Run，并将它们绑定到同一
  StrategyVersion；
- Backtest ID/结果、Sandbox instance/account/Venue、config/risk 引用均与当前 identity
  和配置来源严格匹配；未知引用与归属错配 fail closed；
- Live 仅允许 `created + pending result + blocked risk`，data/adapter/account/Venue 必须为
  disabled/unconfigured 引用，7 项执行能力必须显式为 false；
- 已实现四类过滤、排序、稳定游标、稳定错误 envelope 和 OpenAPI 3.1 schema；
- 已覆盖非法环境、重复 ID、生命周期/时间矛盾、来源陈旧、能力漂移、权限、404 与
  非 GET 405；
- 当前停在 `REVIEW_REQUIRED`，等待独立审查和 hosted checks，不启用自动合并。
