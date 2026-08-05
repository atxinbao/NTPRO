# S0-API-001A - 产品合同基座与 Strategy 只读 API

Date: 2026-08-06
Executor: Codex
GitHub issue: #1258
Risk: high
Owner role: Rust Product API Agent
Review role: Verification & Release Gatekeeper
Status: REVIEW_REQUIRED

## 依赖

- FEF-001 #1253 已关闭；
- FEI-001 #1255 已通过 PR #1257 合并；
- 父总控 S0-API-001 #1256 保持开放。

## 目标

以当前 MVP 身份合同和节点 TOML 产品元数据为真实来源，建立 OpenAPI 3.1 权威合同，
并交付 Strategy 列表与详情两个只读 Rust/Axum 端点。

## 范围

- `GET /api/product/v1/strategies`；
- `GET /api/product/v1/strategies/{strategy_id}`；
- 稳定 request ID、错误码、中文摘要与 retryable；
- cursor、limit、sort、order、lifecycle、owner 查询合同；
- identity、配置、来源时效和 required-false 边界 fail closed；
- OpenAPI/schema compatibility、Rust 路由和负向测试。

## 非目标

- StrategyVersion、Run 或前端业务页；
- 任何创建、修改、启动、停止或交易动作；
- 真实 Venue、订单发送、重试或自动补救；
- v0.32.0 冻结发布文件。

## 允许路径

- `crates/cli/src/dashboard/**`、`crates/cli/src/dashboard.rs`、`crates/cli/src/mvp_contract.rs`；
- `crates/cli/src/supervisor.rs` 仅限补齐未配置 kill-switch 时五个 required-false/zero 指标；
- `configs/nodes/btc-ema-shadow.toml`、`docs/product/api/**`；
- 本任务 task/evidence、冻结清单、风险清单与直接关联验证脚本；
- `Cargo.toml` / `Cargo.lock` 中仅限测试依赖登记。

## 禁止路径

- `docs/rust-cutover/release/v0_32_0_*`；
- adapter、订单发送、风险语义和交易执行实现；
- StrategyVersion、Run 和前端业务页面实现。

## 验收

```bash
cargo test -p nautilus-cli --lib dashboard::product_api::tests
cargo test -p nautilus-cli --lib dashboard::server::tests
cargo clippy -p nautilus-cli --all-targets --all-features -- -D warnings
scripts/ci/test-ci-change-classifier.sh
scripts/ai/check_rust_only_runtime.sh
scripts/ai/check_mvp_freeze_baseline.sh
git diff --check
```

高风险产品合同变更必须完成独立审查与 hosted checks 后手动合并，不启用 auto-merge。

## 实施结果

- 已建立 `docs/product/api/ntpro_product_v1.openapi.json` 权威合同；
- 已挂载 Strategy 列表与详情两个受共享读取权限保护的 Axum 路由；
- 已对查询、身份、产品元数据、来源时效、只读边界和非 GET 方法执行 fail-closed 验证；
- 已从 MVP status contract 读取 heartbeat 派生时效阈值，并要求 metrics 明确提供禁用边界；
- 已通过真实 `nautilus mvp serve`、`ntpro-node` 与 React 生产 bundle 浏览器验收；
- 当前停在 `REVIEW_REQUIRED`，等待独立审查和 hosted checks，不启用自动合并。
