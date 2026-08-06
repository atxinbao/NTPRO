# S0-API-001B - StrategyVersion 不可变只读 API

Date: 2026-08-06
Executor: Codex
GitHub issue: #1259
Risk: high
Owner role: Rust Product API Agent
Review role: Verification & Release Gatekeeper
Status: REVIEW_REQUIRED

## 依赖

- S0-API-001A #1258 已通过 PR #1262 合并；
- 父总控 S0-API-001 #1256 保持开放。

## 目标

在统一产品合同基座上交付 StrategyVersion 列表与详情只读端点。版本内容由节点 TOML
显式登记，MVP identity 保存内容 hash 锚点；读取时重新计算规范 SHA-256，并同时校验
Strategy 归属、版本身份、代码引用、参数 schema、数据要求、风险配置、来源与时间。

## 范围

- `GET /api/product/v1/strategies/{strategy_id}/versions`；
- `GET /api/product/v1/strategies/{strategy_id}/versions/{version_id}`；
- limit、cursor、sort、order、status 列表合同；
- 不可变版本 ID、内容 hash、代码引用、参数 schema、数据要求和风险配置；
- identity hash 锚点、稳定配置快照、来源时效与只读边界 fail closed；
- OpenAPI/schema compatibility、Rust 路由和真实浏览器测试。

## 非目标

- Run、Backtest、Demo 或 Live 产品 API；
- 版本创建、发布、修改、回滚或删除；
- 真实 Venue、订单发送、自动重试、补救或交易控件；
- v0.32.0 冻结发布文件。

## 允许路径

- `crates/cli/**`；
- `configs/nodes/btc-ema-shadow.toml`；
- `docs/product/api/**`；
- 本任务 task/evidence、MVP 冻结 manifest、风险清单与直接关联验证脚本。

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

- 已挂载 StrategyVersion 列表与详情两个受共享读取权限保护的 Axum 路由；
- 已将版本内容 hash 锚定到 MVP identity，并在每次读取时重算规范 hash；
- 已用工作区追加式版本注册表持久校验所有历史 StrategyVersion hash；版本切换后回切也
  不得漂移，内容变化必须使用从未登记的新版本号；
- 已使用 Draft 2020-12 meta-schema 校验参数 schema，并将 OpenAPI ID 长度与 Rust
  路径限制统一为两侧各 128 字符；
- 已对未知 Strategy/Version、归属错配、hash/代码/schema/风险漂移和非 GET 方法执行
  fail closed；
- 已扩展 OpenAPI 权威合同和真实 MVP/Axum/node/browser 验收；
- 当前停在 `REVIEW_REQUIRED`，等待独立审查和 hosted checks，不启用自动合并。
