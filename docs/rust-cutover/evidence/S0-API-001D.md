# S0-API-001D - TypeScript 生成合同与跨层兼容门禁证据

Date: 2026-08-06
Executor: Codex
GitHub issue: #1261
Status: REVIEW_APPROVED_HOSTED_PENDING

## 交付证据

- 唯一权威源：`docs/product/api/ntpro_product_v1.openapi.json`；
- 标准生成器：`@hey-api/openapi-ts@0.99.0`；
- 生成内容：Strategy、StrategyVersion、Run、分页、错误类型，Fetch SDK 与 Zod schema；
- 只读客户端：六条 `GET /api/product/v1/*` 路由，同源 cookie 和 JSON Accept header；
- 漂移门禁：`npm run contract:check` 在临时目录重新生成、格式化并执行目录比较；
- 跨层夹具：七组 JSON 由真实 Rust/Axum 路由输出并由 Rust 测试锁定，随后由 Vitest
  与真实 Chrome 通过生成客户端消费；
- CI 绑定：现有 frontend gate 的 `npm run build` 包含合同漂移检查，不新增重复 workflow。

## 本地验证

- 生成结果可重复：通过；
- TypeScript typecheck：通过；
- 前端 Vitest：31/31 通过，其中跨层消费与 fail-closed 18 项、生成漂移负向自测 1 项；
- Playwright：4/4 通过，其中 E2E 专用静态 harness 由 Vite 加载生产 `productApi.ts`，真实
  Chrome 消费六条正常 Rust fixture 和一条稳定错误；该用例并行重复 5/5 通过；
- Vite production build：通过；
- npm audit：0 vulnerabilities；
- Rust Product API：32/32 通过；all-target/all-feature Clippy：通过；
- backend runtime risk inventory：29555 signals / 1224 files，production 保持 2702，
  self-test 8/8 通过；
- Rust 1.95.0 toolchain 与 Rustfmt：通过；
- CI classifier：21/21 通过；Rust-only：通过；
- MVP freeze：19 tasks / 5 phases / 19 boundaries / 13 frozen sources 通过。
- 独立 Verification & Release Gatekeeper：首轮两个 P2 已修复，复审 `REVIEW_APPROVED`。

## 负向验证

- 非法 schema、未知 enum、陈旧来源、任一开放交易边界拒绝消费；
- 未知响应字段拒绝消费，不允许 Zod 静默剥离后继续运行；
- `returned_count` 与数组长度不一致或超过 `limit`、空页继续分页、`has_more` 与
  `next_cursor` 不一致时拒绝消费；
- 非法错误 envelope 拒绝消费；合法错误保留 HTTP status、request ID、code、field 和
  retryable；网络错误与合同错误使用不同错误类型。

## 行为边界

- 仅新增生成合同和只读客户端，不接入业务页面；
- 不增加 mutation、Venue 连接、订单、自动重试、自动补救或交易控件；
- Live Run 仍只是只读配置/历史记录，所有执行能力继续显式为 false；
- v0.32.0 和 MVP 冻结基线不变。

## 行为与兼容性

- 行为影响：为后续 SWB-002 提供生成、可验证的只读浏览器 API 层；
- 公共 API：不新增或修改后端路由；
- 迁移说明：前端后续必须导入生成类型和 `productApi`，不得复制手写 DTO；
- 回滚：删除生成配置、生成目录、客户端、跨层 fixture/test，并恢复 package lock；
- 审查状态：独立审查已通过，hosted checks 待运行；禁止自动合并。
