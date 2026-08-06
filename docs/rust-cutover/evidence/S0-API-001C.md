# S0-API-001C - Run 只读 API 与三环境身份合同证据

Date: 2026-08-06
Executor: Codex
GitHub issue: #1260
Status: REVIEW_REQUIRED

## 交付证据

- 权威合同：`docs/product/api/ntpro_product_v1.openapi.json`，OpenAPI 3.1；
- 真实端点：`GET /api/product/v1/runs` 与 `GET /api/product/v1/runs/{run_id}`；
- 三环境记录：Backtest completed、Sandbox running、Live created；
- 稳定绑定：同一 StrategyVersion；Backtest identity/result/数据/适配器/账户/Venue、
  Sandbox instance/数据/适配器/账户/Venue、配置与风险来源均与当前受验证来源精确一致；
- Live 边界：pending result、blocked risk、disabled/unconfigured 引用、7 项执行能力
  required false；
- 权限边界：仅 institution/operator 共享读取角色可访问；
- 响应边界：策略/Run mutation、Venue、订单、重试、补救和交易控件继续显式为 false。

## 本地验证

- Product API：31/31 通过；
- Dashboard server：12/12 通过；
- MVP identity contract：8/8 通过；
- CLI 全量库测试：610/610 通过；
- all-target/all-feature Clippy：通过；
- Vite build、format、lint、typecheck：通过；
- Vitest：12/12 通过；Playwright 前端 E2E：3/3 通过；
- 真实 `nautilus mvp serve + ntpro-node + Axum + Chrome` 浏览器验收：通过；
- OpenAPI JSON 与 Rust schema compatibility：通过；
- risk inventory：29548 signals / 1224 files，production 保持 2702，self-test 8/8 通过；
- MVP freeze：19 tasks / 5 phases / 19 boundaries / 13 frozen sources 通过；
- backend freeze、Rust-only、CI classifier 21/21、docs/examples governance：通过；
- `git diff --check`：通过。

## 负向验证

- 非法 limit/cursor/sort/order、重复参数和非法 strategy/version/environment/lifecycle
  返回稳定 400；
- 未知 Run 返回 `run_not_found`，非法 UTF-8 ID 返回统一产品查询错误；
- 重复 Run ID、StrategyVersion 归属错配、未知 Backtest/Sandbox 引用、非唯一 Live
  禁用引用、Run 早于策略版本、时间矛盾和非法生命周期 fail closed；
- 任一 Run 执行能力为 true 均返回 `product_boundary_violation`；
- Live 不是 `created + pending + blocked` 时 fail closed；
- 无读取 cookie 返回 403；HEAD/POST/PUT/PATCH/DELETE/OPTIONS/CONNECT/TRACE 返回
  405 和 `Allow: GET`；
- 来源缺失、失效、过期和 runtime 边界异常继续由共享产品源校验阻断。

## 行为边界

- 仅新增只读 Run 资源；
- 不创建或控制 Run；
- Live 记录不代表真实连接、实盘准入或订单权限；
- 所有 mutation、Venue、订单、重试、补救和交易控件边界继续显式关闭。

## 行为与兼容性

- 行为影响：新增两个只读 Run 资源与节点配置中的三环境 Run manifest；
- 公共 API：在既有 `ntpro.product_api.v1` 中新增版本化 GET 路由和 schema；
- 迁移说明：旧节点配置仍可读取 Strategy/StrategyVersion，但 Run 端点要求显式
  `[[product_runs]]`；当前权威节点配置已迁移；
- 回滚：删除两个 Run 路由、Run 模块、OpenAPI schema 与配置段，并恢复冻结 hash；
- 审查状态：本地验证完成后进入独立审查与 hosted checks，禁止自动合并。
