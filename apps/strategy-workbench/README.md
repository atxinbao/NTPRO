# NTPRO 策略工作台前端

Date: 2026-08-05
Executor: Codex

本目录是策略工作台的 React、TypeScript 与 Vite 工程。当前只实现产品框架、总览、系统状态
和 `GET /api/mvp/v1/status` 只读桥接；Strategy、StrategyVersion、Run、Backtest、Demo
与 Live 产品能力按仓库 Roadmap 分阶段交付。

## 本地命令

```bash
npm ci
npm run dev
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

## 边界

- `dist/`、`node_modules/`、coverage 和 Playwright 结果不提交；
- fixture 只用于自动测试，不得冒充生产数据；
- 合同、身份、来源或禁用边界异常时清空旧状态并 fail closed；
- Live 和未交付栏目保持不可操作；
- Axum 资产接入与旧页面迁移由后续 FEI-001 独立完成。
