# NTPRO

NTPRO 是面向系统化交易机构的 Rust 交易平台。项目采用“一个平台、两个门户、
一个共享控制平面”的产品结构：

- **机构工作台**：面向交易员、研究员和投资组合负责人，承接组合观察、交易监控、
  策略研发、回测、风险与报告。
- **平台控制中心**：面向平台管理员、技术运维和风控人员，承接组织权限、数据账户、
  模型部署、系统运行与审计。
- **共享控制平面**：统一策略状态、审批、部署、风险和审计事实，连接前台业务与后台
  运维，但不作为第三个独立产品。

完整产品定义和中文信息架构见 [`project.html`](project.html)。

## MVP 基线与下一步

PR #1198 已交付 `nautilus mvp serve`：一个 Supervisor 注册、启动并停止一个
本地沙盒 `ntpro-node`，同时提供本地 Dashboard。MVP-003 与 MVP-004 已完成身份
追溯和四轴状态合同；MVP-005 已交付双门户共享的只读状态 API。上述能力仍不代表
完整产品闭环已经完成。

下一步只补齐这条基线所需的可观察、可追溯和双门户只读产品闭环：

```text
一个 Supervisor（运行基线已交付）
  -> 一个 ntpro-node
  -> 一个策略实例
  -> 一个沙盒账户 / Venue
  -> 共享只读状态 API（已交付）
  -> 机构工作台（`/institution-workbench`，MVP-006 已合并）
  -> 控制中心（`/control-center`，由 MVP-007 完成只读绑定）
```

策略版本与回测结果必须能追溯到运行实例；Supervisor 负责节点生命周期，
机构工作台只展示账户、仓位、订单、成交、风险和策略状态，控制中心负责节点、
日志、指标和故障定位。`node_id`、`strategy_id` 和交易员工作空间保持独立。

当前阶段状态：M0 实现已交付并记录历史审查证据例外；M1 已交付；MVP-006 已合并，
机构工作台正在消费共享 API。MVP-007 提供控制中心的共享状态与本地运维投影关联；
issue #1213 关闭且对应 PR 合并后，M2 双门户消费退出条件正式满足。M3 继续开放，
后续补齐跨门户事件跳转、服务端角色边界和生命周期动作产品化。详细状态以
[`docs/product/roadmap.md`](docs/product/roadmap.md) 为准。

多节点生产编排、多账户/多 Venue 扩展、真实订单提交、订单变更和产品级实盘终端
均不属于本 MVP，必须在 MVP 验收后独立立项。

## 正式版本

`v0.33.0` 是正式发布的后端维护版本，建立在冻结的 `v0.32.0` 后端生产收尾
基线上。它提供性能基线、回归检测、错误边界和依赖治理，不代表前端产品完成，
也不授权产品级实盘交易。

GitHub Release：
<https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.33.0>

以下字段由发布治理脚本读取，必须保持稳定：

```text
Current source tag: ntpro-rust-only-v0.33.0
Current source-tree readiness: ntpro-rust-only-v0.33.0 release gate ready
Current capability: v0.33.0 Backend Maintenance
Capability class: separately scoped backend maintenance only
Backend patch: none scheduled; baseline-invalidity exception only
Post-baseline governance: backend-maintenance
Next capability family: v0.34.0+ separately scoped capability tracks
v0.34.0+ entry: separately scoped only; no capability inheritance from v0.32.0 or v0.33.0
Boundary: v0.33.0 is a Backend Maintenance release.
```

No backend patch is scheduled.

- 后端冻结治理：`backend-freeze-governance`
- 发布后维护轨道：`backend-maintenance`
- 冻结后的能力入口：`v0.33.0+` 必须独立立项
- The next capability family is `v0.34.0+`.

版本历史、治理变更和文档迁移见 [`changelog.html`](changelog.html)；完整发布证据见
[`docs/rust-cutover/release/`](docs/rust-cutover/release/)。

## 能力边界

仓库的正式产品面是纯 Rust：

- Rust workspace、CLI、示例和文档；
- Rust 与 Shell 验证、治理和发布工具；
- 本地回测、沙盒、只读运行状态和受约束的后端维护能力；
- 不包含 tracked Python helper、Python package、PyO3、Cython、wheel 或 PyPI
  发布路径。

以下能力没有从后端基线或维护版本继承：

- actual backend production go-live；
- production submit、order mutation 或 execution adapter send；
- live exchange request、隐式重试、自动恢复或自动补救；
- 策略驱动的生产执行和共享审批消费；
- Dashboard、Admin 或 Trader Terminal 的 submit、cancel、retry、replace、
  amend、flatten、remediation 和 order-ticket 控件；
- 产品级实盘终端、预构建二进制、Docker 镜像和多用户远程部署交付。

后端冻结政策见
[`backend_freeze_policy.md`](docs/rust-cutover/governance/backend_freeze_policy.md)，
后续能力入口规则见
[`v0_33_plus_intake_policy.md`](docs/rust-cutover/governance/v0_33_plus_intake_policy.md)。

## 快速开始

项目要求 Rust `1.95.0`：

```bash
rustup toolchain install 1.95.0
rustup override set 1.95.0
```

如果 Homebrew 工具链优先于 rustup，验证脚本会通过
`scripts/ai/toolchain_env.sh` 选择项目工具链。

查看 CLI：

```bash
cargo run -p nautilus-cli -- --help
cargo run -p nautilus-cli -- backtest --help
cargo run -p nautilus-cli -- sandbox --help
cargo run -p nautilus-cli -- live --help
```

本地安装：

```bash
cargo install --path crates/cli --bin nautilus --locked --force
nautilus --help
```

当前二进制名称是 `nautilus`，由 `nautilus-cli` package 提供。

## 验证

快速格式和工具链检查：

```bash
scripts/ai/verify_fast.sh
```

完整测试：

```bash
scripts/ai/verify_full.sh
```

发布与治理检查：

```bash
scripts/ai/verify_release.sh
scripts/ai/check_rust_only_runtime.sh
scripts/ai/check_cython_removed.sh
scripts/ai/run_golden_traces.sh
```

`verify_fast.sh` 不是发布证据；正式发布必须以 tagged commit、hosted release
gate、GitHub Release 和 source-controlled evidence 为准。

## 文档入口

根目录只保留四个项目级入口：

- [`AGENTS.md`](AGENTS.md)：代理执行规则和仓库边界；
- [`README.md`](README.md)：项目定位、正式版本、能力边界和使用入口；
- [`project.html`](project.html)：系统定义、目标用户和双门户信息架构；
- [`changelog.html`](changelog.html)：版本、治理和文档结构变更。

权威文档目录：

- [`docs/product/`](docs/product/)：产品路线和能力规划；
- [`docs/governance/`](docs/governance/)：持续治理政策；
- [`docs/rust-cutover/`](docs/rust-cutover/)：后端冻结、迁移、任务、发布与证据；
- [`.github/`](.github/)：贡献、安全、CLA、行为准则和协作模板；
- [`examples/rust/`](examples/rust/)：Rust 示例与使用说明。

核心治理文档：

- [`CONTRACT.md`](docs/rust-cutover/CONTRACT.md)
- [`DEFINITION_OF_DONE.md`](docs/rust-cutover/DEFINITION_OF_DONE.md)
- [`TASK_EXECUTION.md`](docs/rust-cutover/TASK_EXECUTION.md)
- [`AGENT_ROLES.md`](docs/rust-cutover/AGENT_ROLES.md)

## 许可证

本仓库继承 NautilusTrader 的许可证谱系。分发构建或发布产物前，请检查根目录
许可证文件和上游声明。
