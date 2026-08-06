# S0-API-001D - TypeScript 生成合同与跨层兼容门禁

Date: 2026-08-06
Executor: Codex
GitHub issue: #1261
Risk: high
Owner role: Frontend Contract Agent
Review role: Verification & Release Gatekeeper
Status: REVIEW_APPROVED_HOSTED_PENDING

## 依赖

- S0-API-001A #1258 已通过 PR #1262 合并；
- S0-API-001B #1259 已通过 PR #1263 合并；
- S0-API-001C #1260 已通过 PR #1266 合并；
- 父总控 S0-API-001 #1256 保持开放。

## 目标

从后端权威 OpenAPI 3.1 合同生成策略工作台 TypeScript DTO、Fetch SDK 和 Zod schema，
提供六条只读 GET 路由的类型化客户端，并建立 Rust 路由响应到浏览器消费的可重复兼容门禁。

## 范围

- 使用 `@hey-api/openapi-ts` 从权威 OpenAPI 生成类型、Fetch SDK 和 Zod schema；
- 提供 Strategy、StrategyVersion、Run 列表与详情客户端；
- 同源 cookie、`Accept: application/json`、稳定错误和 request ID 透传；
- clean checkout 可重复生成并拒绝生成结果漂移；
- 七组真实 Rust/Axum JSON fixture 由 Rust 测试锁定并由 Vitest 消费；
- 非法 payload、未知 enum、未知字段、分页错误、陈旧来源和开放交易边界 fail closed；
- 复用现有 frontend/Smoke 分类器和 workflow，不新增独立 workflow。

## 非目标

- 不实现策略工作台业务页面；
- 不新增写 API、运行控制或交易动作；
- 不连接真实 Venue；
- 不修改 v0.32.0 冻结发布文件或 MVP 冻结源。

## 验收

```bash
cd apps/strategy-workbench
npm ci
npm run audit
npm run format:check
npm run lint
npm run typecheck
npm run test
npm run build
npm run test:e2e

cd ../..
cargo test -p nautilus-cli --lib dashboard::product_api::tests
cargo clippy -p nautilus-cli --all-targets --all-features -- -D warnings
scripts/ci/test-ci-change-classifier.sh
scripts/ai/check_backend_runtime_risk_inventory.sh
scripts/ai/test_backend_runtime_risk_inventory.sh
scripts/ai/check_rust_only_runtime.sh
scripts/ai/check_mvp_freeze_baseline.sh
git diff --check
```

高风险跨层合同必须完成独立审查与 hosted checks 后手动合并，不启用自动合并。

## 实施结果

- OpenAPI 生成配置、锁定依赖、生成目录和临时目录漂移检查已建立；
- 六条只读 GET 客户端不维护手写同构 DTO，所有返回类型来自生成代码；
- 运行时使用生成 Zod schema，并额外拒绝未知字段和分页语义不一致；
- Rust 路由测试负责生成并锁定正常、错误 fixture；Vitest 和真实 Chrome 均通过生成客户端
  构造 Fetch Request 并消费全部六条正常路由和稳定错误；
- 独立 Verification & Release Gatekeeper 已输出 `REVIEW_APPROVED`；当前等待 hosted checks，
  不启用自动合并。
