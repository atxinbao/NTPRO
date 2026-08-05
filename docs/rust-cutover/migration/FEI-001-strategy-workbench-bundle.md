# FEI-001 策略工作台生产 bundle 迁移说明

Date: 2026-08-06
Executor: Codex

策略工作台不再编译进 Rust 源码。部署或本地启动前必须显式生成 Vite production
bundle：

```bash
cd apps/strategy-workbench
npm ci
npm run build
cd ../..
```

随后通过默认路径 `apps/strategy-workbench/dist` 启动，或显式传入：

```bash
target/debug/nautilus mvp serve \
  --strategy-workbench-dist /absolute/path/to/strategy-workbench/dist \
  --config configs/nodes/btc-ema-shadow.toml
```

Node.js 只用于构建，不是生产进程。Rust/Axum 在监听端口和启动 node 前验证 React
入口、hash JS/CSS 以及入口引用的全部静态资产；缺失或不完整的 bundle 会拒绝启动。

HTTP API 合同、Supervisor/node 生命周期和交易边界没有变化。回滚 FEI-001 可恢复旧的
Rust 内嵌页面；不涉及数据库或运行状态迁移。
