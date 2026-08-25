# NTPRO

NTPRO 是一个 Rust 策略工作台。当前目标不是先建设完整机构平台，而是让量化研发和
运行人员在本地完成一个策略的配置、Backtest、Demo、比较和复盘。

当前产品边界是：

```text
一个用户 + 一个策略 + 一个交易品种 + 一个账户 + 一台机器
  -> 一个 Supervisor + 一个 ntpro-node
  -> 同一个不可变 StrategyVersion
  -> Backtest：当前产品闭环
  -> Demo / Sandbox：当前产品闭环
  -> Live：独立受控能力，不从 Backtest 或 Demo 自动继承
```

用户只需要使用策略工作台，不需要理解 Supervisor、node、Axum 或 Rust 内部实现。
系统状态、日志和节点控制是辅助诊断能力，不是当前产品主导航。完整中文产品说明见
[`project.html`](project.html)，权威开发顺序见
[`docs/product/roadmap.md`](docs/product/roadmap.md)。

## 当前目标

`NTPRO Usable Product v1` 先完成一条可重复交付的用户流程：

1. 查看当前内置策略和不可变版本；
2. 无需命令行创建并查看 Backtest；
3. 启动、观察和停止同一版本的 Demo；
4. 比较 Backtest 与 Demo 的交易、收益、回撤和风险；
5. 重启服务后仍能读取历史结果，并能在页面理解失败原因。

当前 Backtest 使用内置确定性数据，适合验证产品流程和确定性，不代表真实市场研究或
收益证明。下一阶段将接入本地历史数据目录和用户可选择的数据范围。Live 已有独立准入
与受控技术路径，但真实资金端到端验收仍需 owner 单独授权，不是 Usable Product v1 的
完成前提。

多机构、多账户、多节点、集中运维平台、策略市场和远程多用户部署全部后置。已冻结的
M0-M4 单节点技术基线继续作为底层资产，历史合同和证据保留在
[`docs/rust-cutover/`](docs/rust-cutover/)；机器可校验范围见
[`docs/product/mvp_freeze_manifest.json`](docs/product/mvp_freeze_manifest.json)。

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
