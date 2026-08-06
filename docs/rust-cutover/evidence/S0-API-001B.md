# S0-API-001B - StrategyVersion 不可变只读 API 证据

Date: 2026-08-06
Executor: Codex
GitHub issue: #1259
Status: REVIEW_REQUIRED

## 交付证据

- 权威合同：`docs/product/api/ntpro_product_v1.openapi.json`，OpenAPI 3.1；
- 真实端点：StrategyVersion 列表与嵌套详情两个 GET 路由；
- 不可变内容：稳定 ID、Strategy 归属、版本号、SHA-256、代码引用、参数 schema、
  数据要求、风险配置、状态、创建时间和来源；
- hash 锚点：`MvpIdentitySet.strategy_version_content_hash` 捕获登记 hash，读取时与节点
  TOML 声明及规范内容重算结果三方比对；
- 权限边界：仅 institution/operator 共享读取角色可访问；
- 能力边界：策略/Run mutation、Venue、订单、重试、补救和交易控件继续显式为 false。

## 本地验证

- Product API：27/27 通过；
- Dashboard server：12/12 通过；
- MVP identity contract：8/8 通过；
- MVP status API：22/22 通过；
- CLI 全量库测试：604/604 通过；
- all-target/all-feature Clippy：通过；
- 真实 Vite build + `nautilus mvp serve` + `ntpro-node` + Axum + Chrome 浏览器 smoke：通过；
- OpenAPI JSON 解析与 Rust schema compatibility：通过；
- risk inventory：29519 signals / 1224 files，production 保持 2702，self-test 8/8 通过；
- MVP freeze：19 tasks / 5 phases / 19 boundaries / 13 frozen sources 通过；
- Rust-only 与 CI classifier 21/21：通过；
- `git diff --check`：通过。

## 负向验证

- 非法 limit/cursor/sort/order/status 与重复参数返回稳定 400；
- 未知 Strategy 和 Version 分别返回稳定 `strategy_not_found` / `strategy_version_not_found`；
- 非法 UTF-8 和非法版本资源 ID fail closed；
- code ref、参数 schema、风险边界、声明 hash 和 identity hash 锚点漂移 fail closed；
- 自行重算并替换声明 hash 不能绕过已发布 identity 锚点；
- HEAD/POST/PUT/PATCH/DELETE/OPTIONS/CONNECT/TRACE 均返回 405，`Allow: GET`；
- 所有响应继续携带 required-true read-only 与 required-false 交易边界。

## 独立审查修复

- 已将内容 hash 要求收窄到 StrategyVersion 端点；缺少新字段的旧 Strategy 工作区仍可
  读取策略列表与详情，版本端点继续 fail closed；
- 已新增工作区持久、追加式 `strategy_id -> version -> content_hash` 注册表；旧 identity
  首次迁移会回填历史锚点，`v1/A -> v2/B -> v1/C` 回切绕过会在触碰 Supervisor
  registry、identity 和 node 事件前 fail closed；
- 参数 schema 现在执行完整 Draft 2020-12 meta-schema 校验，未知类型等非法关键字值
  fail closed；
- OpenAPI `StrategyVersionId` 两侧各限制为 128 字符，与 Rust 路径校验完全一致；
- `Strategy.default_version_id` 直接复用 `StrategyVersionId` schema，Rust 总长度上限同步为 257；
- GitHub #1259 权威 Allowed paths 已补入 `configs/nodes/**`；
- 已增加旧工作区、重启不可变性、JSON Schema 和 OpenAPI 长度边界回归测试，并在暂存
  新模块后执行最终独立复审。

## 行为与兼容性

- 行为影响：新增两个只读 StrategyVersion 资源；MVP identity 在存在 `[strategy_version]`
  配置时增加内容 hash 锚点；
- 公共 API：在既有 `ntpro.product_api.v1` 中新增版本化 GET 路由和 schema；
- 迁移说明：旧 MVP identity JSON 缺少新字段时仍可反序列化，但 StrategyVersion 产品 API
  要求该字段存在且为有效 SHA-256；重新启动当前 MVP 会从节点配置自动生成；
- 回滚：删除两个版本路由、版本模块和 OpenAPI schema，移除节点 `[strategy_version]`
  段及 identity 可选 hash 字段，并恢复冻结 hash；
- 审查状态：本地验证完成后进入独立审查与 hosted checks，禁止自动合并。
