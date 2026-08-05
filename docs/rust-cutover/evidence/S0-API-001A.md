# S0-API-001A - 产品合同基座与 Strategy 只读 API 证据

Date: 2026-08-06
Executor: Codex
GitHub issue: #1258
Status: REVIEW_REQUIRED

## 交付证据

- 权威合同：`docs/product/api/ntpro_product_v1.openapi.json`，OpenAPI 3.1；
- 真实端点：`GET /api/product/v1/strategies`、`GET /api/product/v1/strategies/{strategy_id}`；
- 数据来源：MVP identity/status contract、Supervisor registry 与节点 TOML 显式产品元数据；
- 权限边界：仅 institution/operator 共享读取角色可访问，无凭据返回 403；
- 能力边界：产品合同只读，策略/Run mutation、Venue、订单、重试、补救、交易控件全部显式为 false；
- 失败语义：非法查询、缺失元数据、身份漂移、来源陈旧和边界漂移均 fail closed。

## 本地验证

- `cargo test -p nautilus-cli --lib dashboard::product_api::tests`：19/19 通过；
- `cargo test -p nautilus-cli --lib dashboard::server::tests`：12/12 通过；
- `cargo test -p nautilus-cli --lib`：593/593 通过；
- `cargo clippy -p nautilus-cli --all-targets --all-features -- -D warnings`：通过；
- `npm run build && npm run test && npm run lint && npm run format:check && npm run audit`：构建通过、12/12 测试通过、无 lint/格式问题、0 个高危漏洞；
- `scripts/ai/test_strategy_workbench_browser.mjs`：真实 MVP/Axum/node/React 浏览器 smoke 通过，覆盖产品列表、详情、错误、权限和 required-false 边界；
- `scripts/ai/test_mvp_acceptance.mjs`：通过，2 次确定性运行、单节点启动/停止和角色边界成立；
- `scripts/ai/test_mvp_fault_matrix.mjs`：11/11 通过；
- `scripts/ci/test-ci-change-classifier.sh`：21/21 通过；
- `scripts/ai/check_backend_runtime_risk_inventory.sh`：通过，29466 signals / 1223 files；
- `scripts/ai/test_backend_runtime_risk_inventory.sh`：8/8 通过；
- `scripts/ai/check_rust_only_runtime.sh`：通过；
- `scripts/ai/check_mvp_freeze_baseline.sh`：通过，19 tasks / 13 frozen sources；
- `cargo audit`：0 个漏洞，2 个仓库既有 allowed warnings；
- `cargo deny check`：advisories、bans、licenses、sources 均通过，保留仓库既有维护告警；
- `git diff --check`：通过。

独立审查提出的配置时效宽限、非法 UTF-8、完整身份漂移、权限/方法错误信封、
cursor 预校验、路径提取拒绝、稳定配置快照、运行时边界证据、缺失 registry
重试语义、registry 损坏分类、artifact 状态/磁盘一致性、405 `Allow: GET` 和
OpenAPI/CI 绑定问题均已修复；registry 记录身份、runtime artifact 时效与嵌套
kill-switch 交易边界也已纳入 fail-closed 验证，并增加对应负向测试。
终审进一步要求 metrics 的五个禁用值必须显式为 `Available(false/0)`，并要求 runtime
时效使用 MVP heartbeat 派生阈值；两项均已修复，缺失/损坏字段继续 fail closed。
最终独立复审补充指出 runtime artifact containment 与 dot-segment StrategyId 两项地址/来源
一致性缺口；现已要求 registry、node root、status/metrics 子路径全部匹配 MVP 固定布局，并
拒绝 `.`/`..` 完整路径段，同时保留普通标识符中的句点。
复审随后确认运行中缺少 status/metrics 仍可能被“可选工件”分支接受，并指出 stale 摘要会
把心跳过期误写成配置变化；现已只允许 stopped/not-started 节点缺少可选工件，其他进程状态
全部 fail closed，同时将 stale 摘要改为来源中性提示。

补充说明：额外执行的 `cargo audit --deny warnings` 因仓库既有的
`RUSTSEC-2026-0173` 与 yanked `spin 0.9.8` 维护告警退出；两者均为本次变更前已存在的
传递依赖，正式仓库命令 `cargo audit` 与 `cargo deny check` 通过。本任务未新增漏洞。

## 行为与兼容性

- 行为影响：新增两个只读产品资源；未配置 kill-switch artifact 时，Supervisor metrics
  将五个禁用边界从 unknown 收紧为显式 false/zero；控制中心和 v28 路由语义不变；
- 公共 API：新增版本化 `v1` API，响应由 OpenAPI 权威合同约束；
- 迁移说明：新增 API，无旧调用方需要迁移；metrics 消费方可继续接受原 schema，并优先读取显式禁用值；
- 回滚：删除产品路由与模块、OpenAPI 文件和节点 `[strategy]` 产品元数据，并恢复
  Supervisor 未配置 kill-switch 指标与冻结 hash；不触碰 v0.32.0 冻结发布文件；
- 审查状态：本地验证完成，当前 `REVIEW_REQUIRED`，等待独立审查和 hosted checks 后手动合并。
