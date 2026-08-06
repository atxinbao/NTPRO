# NTPRO 策略工作台前端

Date: 2026-08-06
Executor: Codex

本目录是策略工作台的 React、TypeScript 与 Vite 工程。当前已实现产品框架、总览、系统状态、
`GET /api/mvp/v1/status` 只读桥接，以及从后端 OpenAPI 生成的 Strategy、
StrategyVersion、Run 类型和只读客户端。业务页面尚未消费这些产品资源；Backtest、Demo
与 Live 产品能力继续按仓库 Roadmap 分阶段交付。

## 本地命令

```bash
npm ci
npm run dev
npm run contract:generate
npm run contract:check
npm run format:check
npm run lint
npm run typecheck
npm run test
npm run build
npm run test:e2e
npm run audit
```

开发地址：`http://127.0.0.1:4174/strategy-workbench/`。

默认将 `/api` 代理到 `http://127.0.0.1:3000`。连接其他本地 Axum 地址时设置
`NTPRO_API_ORIGIN`。Node.js 只用于开发和构建，不是生产运行时。

产品 API 的唯一权威合同是
`docs/product/api/ntpro_product_v1.openapi.json`。`contract:generate` 生成 TypeScript、Fetch
SDK 和 Zod schema；`contract:check` 在临时目录重复生成并比较，任何手工修改或合同漂移
都会使构建失败。

## Rust/Axum 生产运行

前端构建是显式步骤，Cargo 不会隐式执行 npm：

```bash
cd apps/strategy-workbench
npm ci
npm run build

cd ../..
cargo run -p nautilus-cli --bin nautilus -- dashboard serve \
  --registry target/ntpro-mvp/supervisor/registry.json \
  --strategy-workbench-dist apps/strategy-workbench/dist
```

生产运行只需要 `nautilus` Rust 进程和构建后的 `dist/` 目录，不需要 Node.js 进程。
`/strategy-workbench/*` 由 Axum 提供 SPA fallback，`/api/*` 保持独立的 Rust 路由语义。

## 边界

- `dist/`、`node_modules/`、coverage 和 Playwright 结果不提交；
- fixture 只用于自动测试，不得冒充生产数据；
- 合同、身份、来源或禁用边界异常时清空旧状态并 fail closed；
- Live 和未交付栏目保持不可操作；
- Rust 服务启动时验证 React 入口和 hash JS/CSS；缺失或错误的 bundle 直接拒绝启动。
