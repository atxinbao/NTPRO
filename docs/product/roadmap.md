# NTPRO Roadmap

Canonical repository path: `docs/product/roadmap.md`.

Date: 2026-08-05
Executor: Codex

## 当前正确北极星

NTPRO 当前只建设一个策略工作台：同一个不可变 `StrategyVersion` 可以分别运行
Backtest、Demo（代码中的 Sandbox）和真实 Live，并在一个入口中完成配置、运行、观察、
比较和复盘。

Live 是必须交付的产品能力，不是可选远期设想。v0.32.0 后端冻结基线保持有效。
当前 v0.33.0 维护版本仍禁止真实 Venue、真实订单和交易控件，因此“必达目标”和
“当前未开放”必须同时成立。

```text
Strategy
  └─ StrategyVersion（不可变）
       ├─ Backtest Run：历史数据 + BacktestEngine + 模拟执行
       ├─ Demo Run：实时/沙盒数据 + LiveNode + 模拟执行
       └─ Live Run：实时数据 + LiveNode + 真实适配器、账户与 Venue
```

三种模式共享策略逻辑、参数结构、订单语义、风险指标和证据格式。每个 Run 独立记录
`run_id`、`environment`、数据、配置、适配器、账户、Venue、权限、订单、成交、持仓、
风险和结果。Backtest 或 Demo 通过不能自动产生 Live 权限。

## 当前用户与产品入口

当前默认用户是策略研发和运行人员，而不是平台运维团队。用户只需要进入策略工作台：

- 管理策略与不可变版本；
- 创建和复现 Backtest；
- 启动并观察 Demo；
- 在明确准入后启动和观察真实 Live；
- 比较三种模式的收益、回撤、滑点、成交质量、风险事件和稳定性；
- 从任何结果追溯策略、版本、Run、数据、配置、账户和 Venue。

Supervisor、node、Axum、日志目录和进程控制属于技术支撑。系统状态可以作为辅助诊断
入口，但不主导产品导航。多机构、多节点和集中运维平台在策略三模式闭环完成后再独立
规划。

## 当前代码与能力边界

- `Environment` 已定义 Backtest、Sandbox、Live；
- `BacktestEngine` 已提供历史事件回放和模拟执行；
- `LiveNodeBuilder` 接受 Sandbox 与 Live，两种实时模式复用同一运行语义；
- 当前 `nautilus mvp serve` 只启动单 Supervisor + 单 Sandbox node；
- M0-M4 单节点 Demo MVP 已完成并冻结；
- 当前真实订单、外部 Venue、产品级 Live 终端和 Live 操作权限仍为 false。

代码存在不等于产品完成。Backtest 引擎、Sandbox 节点和 `Environment::Live` 枚举都
不能代替策略工作台、稳定产品合同、真实适配器或端到端 Live 验收。

## 前端产品交付线

SWB-001 已交付策略工作台页面框架，但当前实现仍是 Rust 源码内嵌的 HTML、CSS 与原生
JavaScript，只消费 `GET /api/mvp/v1/status`。它证明页面框架和只读状态桥接成立，不代表
各栏目已经产品化。

前端实现以 `docs/architecture/strategy_workbench_frontend_architecture.md` 为权威合同：

```text
FEA-001 前端架构文档
  -> FEF-001 React / TypeScript / Vite 工程底座
  -> S0-API-001 Strategy / StrategyVersion / Run 只读产品合同
  -> SWB-002 策略总览与 Run 详情纵向切片
  -> S1-S4 三模式产品闭环
```

`FEF-001` 与 `S0-API-001` 可以在合同确定后并行，`SWB-002` 必须同时依赖两者。不得先把
全部栏目建设成使用假数据的空页面；fixture 只能用于组件和浏览器测试，生产页面必须绑定
真实产品 API。Node.js 只用于前端开发与构建，生产运行时继续由 Rust/Axum 承担。

## 策略三模式 Roadmap

### S0：策略、版本与运行资源

- 建立 `Strategy`、不可变 `StrategyVersion` 和独立 `Run` 三层产品资源；
- 固定三模式共享的参数、数据需求、订单语义、风险指标和证据格式；
- 每个 Run 显式绑定 Environment、适配器、账户、Venue、配置和结果；
- 定义创建、排队、运行、停止、完成、失败和取消等稳定状态与错误合同。

退出条件：同一 StrategyVersion 可创建三类 Run，身份、状态、来源和错误合同稳定；
任何模式都不能修改已经冻结的版本。

### S1：Backtest 产品化

- 在策略工作台选择版本、数据集、时间范围、资金和模拟 Venue；
- 展示状态、交易明细、持仓、收益、回撤、风险、日志和来源；
- 支持多个版本与参数 Run 的确定性比较、复现和留证；
- 将已有 BacktestEngine 接入产品 API，不要求用户操作命令行。

退出条件：用户可以在页面完成创建、运行、查看、比较和复现 Backtest，并追溯到同一
StrategyVersion。

### S2：Demo 产品化

- 选择同一 StrategyVersion 创建 Demo Run，不复制策略代码；
- 接入已冻结的单 Supervisor + 单 Sandbox node；
- 展示实时数据、信号、订单意图、模拟成交、持仓、风险和技术健康；
- 在同一视图比较 Demo 与 Backtest 的行为和结果差异。

退出条件：Demo 闭环可由策略工作台完整操作和解释，真实订单继续关闭，任何 Demo
结果都不会自动开启 Live。

### S3：真实 Live 产品能力

- 在同一 LiveNode 与 StrategyVersion 语义上接入真实市场数据和执行适配器；
- 建设真实账户、Venue、凭证、连接状态、订单提交、撤改和成交回报；
- 建设持仓对账、风险门禁、最小权限、幂等、审计和人工停机；
- 验证断连、重复回报、部分成交、状态漂移、恢复和回滚场景；
- 在策略工作台提供受控 Live Run，而不是另建一套 Live 策略。

退出条件：真实账户与 Venue 在独立授权下完成受控端到端交易；异常和恢复场景
fail closed；Live 权限不会从 Backtest 或 Demo 自动继承。

### S4：三模式闭环与产品冻结

- 统一比较三种模式的收益、回撤、滑点、成交质量、风险和稳定性；
- 从任一结果追溯策略版本、数据、配置、账户、Venue 和审计证据；
- 完成浏览器、性能、故障、操作手册、发布和回滚验收；
- 固定产品声明、页面能力、API 合同和实际交易边界。

退出条件：一个策略版本可以在 Backtest、Demo、Live 中独立运行并统一复盘，用户能够
形成下一策略版本，三模式闭环可重复演示、审计和交付。

## 已冻结的技术基础

M0-M4 已交付并冻结以下基础能力：

- 稳定身份与四轴状态合同；
- 单 Supervisor + 单 Sandbox node 生命周期；
- 版本化只读 MVP 状态 API；
- 机构工作台与控制中心的历史最小只读页面；
- 角色边界、本地 start/stop、事件关联和人工恢复；
- 确定性回放、11 项故障矩阵、浏览器和性能验收。

### M4：MVP 验收与冻结（MVP-013 合并即完成）

MVP-013 / PR #1236 已完成并冻结 M4。No backend patch is scheduled. v0.32.0 继续由
`backend-freeze-governance` 管理；S0-S4 属于 `v0.33.0+` separately scoped 产品能力，
不改写冻结基线或自动继承真实交易权限。

当前公开发布面是 `ntpro-rust-only-v0.33.0`, the v0.33.0 Backend Maintenance release。
当前发布后治理轨道是 `backend-maintenance`。The next capability family is `v0.34.0+`；
新能力仍必须单独立项，不从维护版本继承交易权限。

这些能力是 S0-S4 的技术基础，不再作为当前产品北极星。历史任务、PR 和冻结证据继续
保留在 `docs/rust-cutover/` 和 `docs/product/mvp_freeze_manifest.json`，不得因为产品
路线调整而改写。

## Current Release Surface

Current published release:

```text
ntpro-rust-only-v0.25.0
ntpro-rust-only-v0.25.1
ntpro-rust-only-v0.26.0
ntpro-rust-only-v0.26.1
ntpro-rust-only-v0.27.0
ntpro-rust-only-v0.27.1
ntpro-rust-only-v0.28.0
ntpro-rust-only-v0.28.1
ntpro-rust-only-v0.29.0
ntpro-rust-only-v0.29.1
ntpro-rust-only-v0.30.0
ntpro-rust-only-v0.30.1
ntpro-rust-only-v0.31.0
ntpro-rust-only-v0.31.1
ntpro-rust-only-v0.32.0
ntpro-rust-only-v0.33.0
```

Current capability boundary:

```text
v0.33.0 Backend Maintenance
v0.32.0 Backend Production Closeout frozen baseline
v0.31.1 Release Governance Closeout Patch
v0.31.0 Controlled Backend Production Enablement Candidate Foundation
v0.30.0 Backend Production Go-Live Candidate Foundation
v0.29.1 Release Governance and v30 Start-Gate Hardening Patch
v0.29.0 Backend Production Readiness Foundation
v0.28.1 Backend Closure Governance Closeout Patch
v0.28.0 Backend Closure / Product Operations Runtime Finalization
v0.27.1 Product Operations Runtime Integration Closeout Patch
v0.27.0 Product Operations Runtime Integration Foundation baseline
v0.26.1 Product Hardening Foundation Closeout Patch
v0.26.0 Product Hardening Foundation baseline
capability class product operations runtime integration closeout patch only
v0.20.1 production order lifecycle hardening baseline
v0.21.0 unified read model foundation closeout baseline
v0.21.1 unified read model hardening baseline
v0.23.1 publication governance baseline
v0.24.1 execution algorithms and order-control hardening baseline
v0.25.0 monitoring / incident / disaster-recovery foundation baseline
account read model evidence required
position read model evidence required
order lifecycle read model evidence required
fill/execution read model evidence required
risk state projection evidence required
Trader Terminal read-only workbench evidence required
gated manual operation-entry contract required
runtime degradation and boundary tests required
read model golden trace coverage
strict release provenance required
default local/PR/release execution fail-closed
order intent and policy artifact evidence required
rate-limit and throttle preview evidence required
order slicing preview evidence required
cancel / replace / amend preview evidence required
retry / no-retry policy ledger evidence required
readback and audit evidence required
Dashboard / Workbench read-only order-control preview evidence required
monitoring observability contract evidence required
alert taxonomy and routing evidence required
incident lifecycle and acknowledgement evidence required
runbook audit evidence required
DR preview drill evidence required
Dashboard monitoring / incident / DR read-only surface evidence required
SLO freshness diagnostics evidence required
new submit capability not included
production order mutation not included
execution adapter send not included
adapter send not included
live exchange request not included
implicit retry not included
retry scheduler not included
automatic cancel not included
automatic remediation not included
retry / replace / amend / flatten not included
ungated submit / cancel / retry / replace / amend / flatten not included
no strategy-driven production execution
no listenKey lifecycle
no real-funds proof in CI
multi-account identity and read-model partitioning evidence
multi-strategy supervisor identity and isolation evidence
multi-venue node registry and lifecycle boundary evidence
orchestration/control-plane gating evidence
Dashboard / Workbench read-only observability evidence
production multi-node runtime implementation not included
product operations runtime integration foundation evidence
external identity / permission integration evidence
persistent operation audit storage evidence
deployment / upgrade / rollback orchestration evidence
long-run telemetry and SLO runtime evidence
Admin Workbench runtime state bridge evidence
runtime integration fail-closed evidence
v0.28.0 backend closure finalization evidence
v0.28.1 release governance closeout patch evidence
v0.29.0 backend production readiness foundation evidence
v0.29.1 release governance and v30 start-gate hardening evidence
v0.31.0 controlled backend production enablement candidate foundation evidence
v0.31.1 release governance closeout patch evidence
v0.32.0 backend production closeout evidence
v0.33.0 separately scoped backend maintenance evidence
no backend patch scheduled after v0.32.0; baseline-invalidity exception only
backend-freeze-governance post-baseline governance track
backend-maintenance active release track
v0.34.0+ capability tracks must be separately scoped and inherit no v0.32.0 or v0.33.0 trading controls
v0.30.0 backend production go-live candidate foundation evidence
no Dashboard order/approval/cancel/retry/submit/replace/amend/flatten controls
no Admin Workbench operation/trading controls
no Trader Terminal order-ticket/trading controls
no product-grade live trading terminal claim
```

## Published Capability Track: v0.33.0

`v0.33.0` is the current Backend Maintenance release surface. It records a
reproducible performance baseline, hosted regression workflow, behavior-
preserving CLI module decomposition, checked runtime error boundaries,
dependency and feature cleanup, one measured rate-limiter optimization, and
gate-before-publish release governance.

`v0.33.0` is maintenance-only. It does not reopen the backend mainline, rewrite
the frozen v0.32.0 package, or include actual backend go-live, frontend
completion, product-grade live terminal readiness, submit/mutation, adapter
send, live exchange request, retry/remediation/recovery, or Dashboard/Admin/
Trader Terminal trading controls.

## Published Capability Track: v0.32.0

`v0.32.0` is the frozen Backend Production Closeout baseline. It records
v0.31.1 dependency proof, backend closeout boundary, scoped owner/operator
approval, freeze/change-window lifecycle, risk/audit/go-no-go gates,
config/venue/credential/environment provenance, canary/rollback/DR,
telemetry/SLO/alert/incident gates, backend enablement state read model and
read-only admin bridge, fail-closed negative tests, release gates, strict
provenance, publication evidence, and source-controlled closeout contract.

`v0.32.0` explicitly does not include actual backend production go-live,
frontend product completion, product-grade live trading terminal readiness,
new production submit capability, production order mutation, execution adapter
send, live exchange request, retry scheduler, automatic remediation/recovery,
strategy-driven production execution, shared approval consumption, or
Dashboard/Admin/Trader Terminal operation/trading controls. v0.33.0 entered
only through separate scope and inherited no v0.32.0 trading controls.

## Published Capability Track: v0.31.0

`v0.31.0` is the prior Controlled Backend Production Enablement Candidate
Foundation release surface. It records v0.30.1 dependency proof, explicit
scoped enablement approval, operator approval/freeze/change-window lifecycle,
risk/audit/go-no-go gates, canary/rollback/DR boundaries, production config and
venue readiness provenance, telemetry/SLO/incident gates, backend enablement
state read model and read-only admin bridge, forbidden production execution
negative tests, release gates, strict provenance, and v32 backend closeout
handoff.

## Published Capability Track: v0.30.0

`v0.30.0` is the prior Backend Production Go-Live Candidate Foundation
release surface. It records v0.29.1 dependency proof, backend go-live candidate
boundary, deployment plan, runtime enablement boundary, operator approval and
freeze lifecycle, canary preflight, rollback/DR boundary, config/venue
readiness, telemetry/SLO and incident freeze gate, audit retention/export,
go/no-go runbook, release gates, strict provenance, and v31 production
enablement handoff.

`v0.30.0` explicitly does not include actual backend production go-live,
frontend product completion, product-grade live trading terminal readiness,
new production submit capability, production order mutation, execution adapter
send, live exchange request, retry scheduler, automatic remediation/recovery,
strategy-driven production execution, shared approval consumption, or
Dashboard/Admin/Trader Terminal operation/trading controls.

## Published Capability Track: v0.29.1

`v0.29.1` is the prior Release Governance and v30 Start-Gate Hardening Patch
release surface. It records v0.29.0 publication closeout, v29
publish-after-gate current binding, stale V290 evidence cleanup,
post-publication closeout gate hardening, v30 start-gate hardening, v29.1
release gates, strict provenance, and published v0.29.1 release evidence.

`v0.29.1` explicitly does not include backend go-live, frontend product
completion, product-grade live trading terminal readiness, new production
submit capability, production order mutation, execution adapter send, live
exchange request, retry scheduler, automatic remediation/recovery,
strategy-driven production execution, shared approval consumption, or
Dashboard/Admin/Trader Terminal operation/trading controls.

## Published Capability Track: v0.29.0

`v0.29.0` is the published Backend Production Readiness Foundation release
surface. It records the v0.28.1 dependency proof, backend readiness boundary
contract, persistent audit storage readiness, telemetry/SLO readiness,
permission source readiness, read-only backend API readiness, deployment
config/runbook readiness, monitoring/alert/incident readiness,
canary/rollback/DR preflight readiness, fail-closed hardening, v29 release
gates, strict provenance, and the v30 go-live candidate handoff.

`v0.29.0` explicitly does not include backend go-live, frontend product
completion, product-grade live trading terminal readiness, new production
submit capability, production order mutation, execution adapter send, live
exchange request, retry scheduler, automatic remediation/recovery,
strategy-driven production execution, shared approval consumption, or
Dashboard/Admin/Trader Terminal operation/trading controls.

`v0.28.0` publishes the Backend Closure / Product Operations Runtime
Finalization release over the v0.27.1 closeout line. It closes v0.27.1
dependency proof, backend closure boundary classification, identity/permission,
persistent audit storage, deployment orchestration, telemetry/SLO ingestion,
Admin Workbench backend state, Trader Terminal backend API handoff,
fail-closed hardening, release gates, and strict provenance. It does not create
a product-grade live trading terminal, ungated operation controls, production
order mutation, adapter send, retry scheduler, shared approval consumption, or
automatic execution/remediation.

## Published Capability Track: v0.28.1

`v0.28.1` is the prior Backend Closure Governance Closeout Patch release
surface. It records v0.28.0 release closeout, stale evidence cleanup, release
body hash normalization, runtime-closed terminology hardening,
release-publish-after-gate current-release binding, v28.1 release gates, strict
provenance, and the v29 intake dependency target. It is not a production
execution runtime implementation.

`v0.28.1` explicitly does not include frontend product completion,
product-grade live trading terminal readiness, new production submit capability,
production order mutation, execution adapter send, live exchange request, retry
scheduler, automatic remediation/recovery, strategy-driven production
execution, shared approval consumption, or Dashboard/Admin/Trader Terminal
operation/trading controls.

## Published Capability Track: v0.28.0

`v0.28.0` is the prior Backend Closure / Product Operations Runtime
Finalization release. It preserves the no-submit and no trading-control
boundary while closing source-controlled backend evidence and release
governance for the v28 line. It is not a production execution runtime
implementation.

`v0.28.0` explicitly does not include frontend product completion,
product-grade live trading terminal readiness, new production submit capability,
production order mutation, execution adapter send, live exchange request, retry
scheduler, automatic remediation/recovery, strategy-driven production
execution, shared approval consumption, or Dashboard/Admin/Trader Terminal
operation/trading controls.

## Published Capability Track: v0.27.1

`v0.27.1` is the published Product Operations Runtime Integration closeout
patch before v0.28.0. It preserves the read-only Workbench boundary and hardens
release governance/provenance for the v0.27.0 foundation. It is not a
production execution runtime implementation.

`v0.27.1` explicitly does not include product-grade live trading terminal
readiness, new production submit capability, production order mutation,
execution adapter send, live exchange request, retry scheduler,
automatic remediation/recovery, strategy-driven production execution, shared
approval consumption, or Dashboard/Admin operation/trading controls.

## Published Capability Track: v0.27.0

`v0.27.0` is the published Product Operations Runtime Integration Foundation.
It preserves the read-only Workbench boundary and adds source-provenanced
runtime integration evidence for identity, audit, orchestration, telemetry/SLO,
Admin Workbench state, fail-closed hardening, release gates, and strict
provenance. It is not a production execution runtime implementation.

`v0.26.0` publishes the product hardening foundation over the monitoring /
incident / disaster-recovery foundation line. It closes product hardening
boundary evidence, operator permission evidence, operation audit evidence,
deployment provenance, upgrade/rollback runbook evidence, SLO/runbook stability
evidence, read-only admin Dashboard evidence, release gates, and strict
provenance. It does not create a product-grade live trading terminal, ungated
operation controls, production order mutation, adapter send, retry scheduler,
shared approval consumption, or automatic execution/remediation.

## Published Capability Track: v0.26.1

`v0.26.1` is the published Product Hardening Foundation closeout patch that
precedes v0.27.0.
It preserves the read-only Workbench boundary and adds v0.26.0 closeout
evidence, final scope integration, stale evidence cleanup, post-publication
strict gate proof, v26.1 release gates, v27 intake hard-block evidence, and
strict provenance. It is not a production execution runtime implementation.

`v0.26.1` explicitly does not include product-grade live trading terminal
readiness, new production submit capability, production order mutation,
execution adapter send, live exchange request, retry scheduler,
automatic remediation/recovery, strategy-driven production execution, shared
approval consumption, or Dashboard operation/trading controls.

## Published Capability Track: v0.26.0

`v0.26.0` is the published Product Hardening Foundation baseline. It preserves
the read-only Workbench boundary and adds release-gated product hardening
evidence for permissions, audit, deployment provenance, upgrade/rollback,
stability/SLO, Dashboard admin display, release gates, and strict provenance.
It is not a production execution runtime implementation.

`v0.26.0` explicitly does not include product-grade live trading terminal
readiness, new production submit capability, production order mutation,
execution adapter send, live exchange request, retry scheduler,
automatic remediation/recovery, strategy-driven production execution, shared
approval consumption, or Dashboard operation/trading controls.

## Published Capability Track: v0.25.1

`v0.25.1` is the published monitoring / incident / disaster-recovery foundation
hardening patch that precedes v0.26.0. It preserves the read-only Workbench
boundary and adds v0.25.0 closeout evidence, corrective release-scope linkage,
stale pre-tag cleanup, Dashboard source reference integrity, post-release gate
split, release gates, and strict provenance. It is a monitoring governance
hardening surface, not a production execution runtime implementation.

`v0.25.1` explicitly does not include product-grade live trading terminal
readiness, new production submit capability, production order mutation,
execution adapter send, live exchange request, retry scheduler,
automatic remediation/recovery, strategy-driven production execution, shared
approval consumption, or Dashboard operation/trading controls.

## Published Capability Track: v0.25.0

`v0.25.0` is the published monitoring / incident / disaster-recovery foundation
line. It preserves the read-only Workbench boundary and adds
monitoring observability contracts, alert taxonomy/routing evidence, incident
lifecycle and acknowledgement evidence, runbook/audit evidence, DR preview
drill evidence, read-only Dashboard monitoring surface, SLO/freshness
diagnostics, release gates, and strict provenance. It is a monitoring and
governance release surface, not a production execution runtime implementation.

`v0.25.0` explicitly does not include product-grade live trading terminal
readiness, new production submit capability, production order mutation,
execution adapter send, live exchange request, retry scheduler,
automatic remediation/recovery, strategy-driven production execution, shared
approval consumption, or Dashboard operation/trading controls.

## Published Capability Track: v0.24.1

`v0.24.1` is the published preview/evidence order-control foundation
hardening line that precedes v0.25.0. It preserves the read-only Workbench boundary and the v0.24.0
foundation while adding release-blocked governance and Dashboard evidence
hardening for release closeout, provenance, schema classification, artifact
ingestion, fixture references, release gates, and strict provenance. It is an
evidence/replay-only release surface, not a production order-control runtime
implementation.

`v0.24.1` explicitly does not include product-grade live trading terminal
readiness, complete executable order-control runtime coverage, new production
submit capability, production order mutation, execution adapter send, live
exchange request, retry scheduler, ungated operation controls, automatic
cancel/remediation, strategy-driven production execution, shared approval
consumption, or Dashboard operation controls.

## Published Hardening Patch: v0.7.1

`v0.7.1` is the published hardening patch for the `v0.7.0` surface. It does
not expand the capability claim, does not add order submission, and keeps the
default local/CI path offline.

Completed hardening work:

- wire v0.7 default offline and manual-online preflight scripts into
  `verify_release.sh`, PR smoke, and hosted release gate;
- align Roadmap, readiness, and release-facing wording for the v0.7.1
  hardening release;
- normalize the v0.7 HTTP connectivity probe artifact path/schema contract;
- validate Binance `/api/v3/time` response shape before claiming HTTP
  connectivity success;
- split manual-online classification from manual-online connectivity proof;
- prepare v0.7.1 readiness notes and final gate evidence.

v0.7.1 explicitly does not include:

- real Binance testnet order submission;
- authenticated Binance testnet account access;
- real account reconciliation;
- production Binance connectivity;
- production trading;
- remote or multi-user Dashboard operation;
- prebuilt binary or Docker release delivery.

## Published Wording/Evidence Patch: v0.7.2

`v0.7.2` is the published wording and evidence patch for the `v0.7.1`
hardening surface. It does not expand the capability claim, does not add
authenticated account access, and keeps the default local/CI path offline.

Completed release-surface cleanup:

- finalize v0.7.2 release notes as published;
- finalize v0.7.2 readiness report as released/PASS;
- record formal tag, GitHub Release URL, hosted release gates, and publication
  flags;
- keep the no orders / no real funds / no production trading boundary explicit.

v0.7.2 explicitly does not include:

- real Binance testnet order submission;
- authenticated Binance testnet account access;
- real account reconciliation;
- production Binance connectivity;
- production trading;
- Dashboard-started network probes.

## Published Capability Track: v0.8.0

`v0.8.0` is the published capability track after the v0.7.2 wording and
evidence patch. It advances the boundary to authenticated Binance testnet
read-only proof.

The only intended boundary change is:

```text
public read-only testnet proof -> authenticated read-only testnet proof
```

Required constraints:

- no order submission;
- no account mutation;
- no real account trading;
- no production trading claim;
- Dashboard remains read-only and must not start probes;
- secrets are never written to artifacts, stdout, logs, docs, or PR bodies;
- default CI remains offline;
- manual online verification is opt-in only.

Authenticated read-only access must fail closed unless all of these are true:

- `--allow-testnet-network` is passed;
- `NTPRO_ALLOW_TESTNET_NETWORK=1` is set;
- config environment is `testnet`;
- `order_submission = disabled`;
- `real_orders_submitted = false`.
- required credential env vars are present;
- credential values are never persisted or printed.

The v0.8.0 proof must stay read-only. It must not place, cancel, amend, or query
through any endpoint that mutates account state.

Completed release closure:

- formal tag: `ntpro-rust-only-v0.8.0`;
- GitHub Release:
  `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.8.0`;
- hosted workflow-dispatch and tag-triggered release gates passed;
- closure evidence recorded in `docs/rust-cutover/evidence/V080-009.md`.

## Safety Patch Track: v0.8.1

`v0.8.1` is a safety and release-surface closure patch for the v0.8.0 line. It
must not add order submission, account mutation, production Binance
connectivity, real funds, production trading, or Dashboard-started network
probes.

The v0.8.1 patch scope is:

- align README and ROADMAP with the published v0.8.0 release surface;
- enforce `NTPRO_V08_MANUAL_ONLINE=1` inside the Rust authenticated runtime
  gate, not only in Bash verification scripts;
- expose authenticated read-only proof status in summary / manifest output;
- tighten authenticated response-shape naming and validation;
- publish v0.8.1 readiness and release notes as a safety/closure patch.

## Published Capability Track: v0.9.0

`v0.9.0` is the published local deterministic Strategy Runtime batch foundation
release. It proves that `ntpro-node` can load a local strategy session, consume
a bounded fixture/mock market input batch, write signal and shadow order-intent
artifacts, write shadow risk decisions, expose supervisor status, and render
read-only Dashboard state.

Completed release closure:

- formal tag: `ntpro-rust-only-v0.9.0`;
- GitHub Release:
  `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.9.0`;
- hosted workflow-dispatch and tag-triggered release gates passed;
- closure evidence recorded in `docs/rust-cutover/evidence/V090-014.md`.

`v0.9.0` explicitly does not include:

- persistent long-running Strategy Runtime semantics;
- Binance testnet order submission;
- order cancel/replace/amend;
- production order submission;
- real funds;
- production trading;
- Dashboard order controls;
- strategy-driven live execution through an exchange adapter.

## Historical Hardening Patch Track: v0.9.1

`v0.9.1` is Strategy Runtime Semantics & Audit Hardening for the published
v0.9.0 line. It must not add Binance testnet order submission or production
trading capability. Its scope is to make node/session/market/risk/heartbeat and
artifact audit semantics true before later Binance sandbox order proof work.

The v0.9.1 patch scope is:

- align README and ROADMAP with the published v0.9.0 release surface;
- mark v0.9 readiness/boundary wording as released rather than planning;
- unify StrategyNode config validation between CLI and node runtime;
- make StrategySession lifecycle semantics persistent until stop/pause/risk
  halt, instead of stopping before the node waits for shutdown;
- align node/session/market status transitions;
- keep heartbeat counters monotonic and sourced from one runtime snapshot;
- split kill-switch enabled/active semantics in config and artifacts;
- add Strategy Session manifest and artifact integrity audit;
- surface artifact/status conflicts as degraded in Supervisor and Dashboard;
- add integration, heartbeat, shutdown, and restart smoke coverage;
- add v0.9.1 release notes and readiness material after the hardening tasks;
- document that v0.10.0 is the Binance spot sandbox order proof release track.

## Published Capability Track: v0.10.0

`v0.10.0` is the published Binance spot sandbox order proof release. It proves
one owner-gated Spot Demo Mode LIMIT GTC submit/cancel lifecycle with redacted
artifacts, terminal reconciliation, production order counters fixed at zero,
and read-only Dashboard evidence display.

Completed release closure:

- formal tag: `ntpro-rust-only-v0.10.0`;
- GitHub Release:
  `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.10.0`;
- tag-triggered release gate passed;
- release notes and readiness report recorded under
  `docs/rust-cutover/release/v0_10_0_release_notes.md` and
  `docs/rust-cutover/release/v0_10_0_readiness_report.md`.

`v0.10.0` explicitly does not include:

- production Binance connectivity;
- production order submission, cancel, replace, amend, or live order
  management;
- automatic online order mutation;
- real funds;
- production trading;
- Dashboard order controls;
- production account reconciliation.

## Published Capability Track: v0.11.0

`v0.11.0` is the published Production Read-Only Contract + Offline Shadow
Portfolio release. It defines production endpoint classification, read-only
public/account snapshot contracts, offline fail-closed read artifacts, local
shadow execution/portfolio evidence, local shadow/read-only lifecycle and
reconciliation models, and read-only Dashboard production shadow status.

Completed release closure:

- formal tag: `ntpro-rust-only-v0.11.0`;
- GitHub Release:
  `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.11.0`;
- release notes and readiness report recorded under
  `docs/rust-cutover/release/v0_11_0_release_notes.md` and
  `docs/rust-cutover/release/v0_11_0_readiness_report.md`.

`v0.11.0` explicitly does not include:

- production order submission;
- production cancel, replace, amend, retry, or correction orders;
- successful online production public/account reads;
- production network-read runtime as completed capability;
- real funds;
- production trading;
- automatic production reconciliation or remediation;
- production order lifecycle parity;
- Dashboard order, cancel, replace, amend, or retry controls.

## Historical Hardening Patch Track: v0.11.1

`v0.11.1` is the historical release-surface hardening patch material for the
published v0.11.0 line. It prepared readiness and release-note material for
owner release decision, but it did not become the current public source release
line after v0.12.0 was published.

The v0.11.1 patch scope is:

- align v0.11 wording to contract/offline reality;
- add a central endpoint classifier API and deny tests;
- add a production shadow manifest contract;
- harden Dashboard production shadow artifact health;
- wire the public read probe into v11 offline release gates;
- clarify that `/api/v3/openOrders` and order-state reads are out of scope;
- normalize `read_allowed` artifact semantics with explicit
  `contract_ready` and `online_read_allowed` fields;
- prepare v0.11.1 readiness and release-note material.

`v0.11.1` explicitly does not include:

- production online read runtime;
- successful online production public/account reads;
- production open-order or order-state reads;
- production order submission or mutation;
- real funds;
- production trading;
- automatic production remediation;
- Dashboard order controls.

## Published Capability Track: v0.12.0

`v0.12.0` is the published Production Online Read-Only + Persistent Shadow
release. It includes owner-gated production `GET` read-only proof paths and
persistent local shadow artifact evidence.

The v0.12.0 release scope is:

- owner-gated production public read-only online proof;
- owner-gated authenticated production account snapshot read-only proof;
- redacted production account response-shape evidence;
- local shadow portfolio runtime artifact;
- bounded local shadow strategy session event artifact;
- local read-only reconciliation classifications;
- Dashboard v0.12 production shadow read-only panel;
- v0.12 offline release gates and manual-online fail-closed preflight.

`v0.12.0` explicitly does not include:

- production order submission;
- production cancel, replace, amend, retry, or correction orders;
- production open-order or order-state reads;
- listenKey lifecycle access;
- strategy-driven production execution;
- automatic production remediation;
- production portfolio parity;
- real funds;
- production trading;
- Dashboard order controls.

## Published Capability Track: v0.13.0

`v0.13.0` is the published Guarded Live Alpha Preflight line. It keeps the
release default offline/fail-closed and adds preflight evidence only before any
future live-alpha execution decision.

`v0.13.0` includes:

- v0.13 Guarded Live Alpha Preflight scope decision;
- bounded local shadow preflight loop heartbeat/stop/stale-data evidence;
- owner-gated production online read-only proof-pack wrapper;
- kill-switch dry-run/manual approval artifact;
- trader/ops Dashboard read-only/control boundary evidence;
- Decimal/string-only amount preflight evidence;
- no-production-mutation PR and release gate;
- readiness report and release notes.

`v0.13.0` explicitly does not include:

- production order submission;
- production cancel, replace, amend, retry, or correction orders;
- production open-order or order-state reads;
- listenKey lifecycle access;
- signed WebSocket user stream runtime;
- strategy-driven production execution;
- automatic production remediation;
- production portfolio parity;
- live-alpha risk/execution-grade money math;
- real funds;
- production trading;
- Dashboard order controls.

`v0.12.1` remains the published hardening baseline for the v0.12 production
read-only/shadow-only line. `v0.13.0` remains the Guarded Live Alpha Preflight
line, `v0.14.0` remains the Production Order-State Read-Only + Live Alpha
Dry-Run line, and `v0.15.0` remains the Guarded Live Alpha Mutation Scope +
Execution Dry-Run Harness line. These are superseded by the v0.16.0
owner-approved single-order candidate line for current public release-surface
wording.

## Published Capability Track: v0.15.0

`v0.15.0` is the published Guarded Live Alpha Mutation Scope + Execution
Dry-Run Harness line. It keeps release defaults offline/fail-closed and adds
production mutation endpoint classification, redacted local request-preview
artifacts, manual approval, kill-switch runtime gating, dry-run execution
adapter evidence, incident/rollback evidence, and Dashboard read-only mutation
preflight only.

`v0.15.0` includes:

- v0.15 production mutation scope decision;
- production mutation endpoint classifier gate;
- redacted live order request dry-run preview;
- manual approval lifecycle artifact;
- kill switch runtime gate;
- execution adapter isolation artifact;
- dry-run mutation golden traces;
- manual incident, rollback, and emergency-stop artifacts;
- Dashboard live-alpha mutation preflight read-only panel;
- v0.15 release gates;
- readiness report and release notes.

`v0.15.0` explicitly does not include:

- production request sending;
- production order submission;
- production cancel, replace, amend, retry, or correction orders;
- production order-test submission;
- production execution adapter calls;
- default production network execution;
- listenKey lifecycle access;
- signed WebSocket user stream runtime;
- strategy-driven production execution;
- automatic production remediation;
- real funds;
- production trading;
- Dashboard order controls.

## Published Capability Track: v0.16.0

`v0.16.0` is the published Minimum Owner-Approved Production Order Mutation
Candidate line. It keeps release defaults offline/fail-closed and adds the
smallest production mutation candidate boundary: one owner-approved tiny
`LIMIT` `GTC` production order candidate with explicit gates, redacted
evidence, readback, audit, kill-switch enforcement, and terminal no-retry
failure semantics.

`v0.16.0` includes:

- v0.16 production mutation scope contract;
- owner-approved runtime gates;
- production signing-material approval evidence;
- single `LIMIT` `GTC` request builder;
- guarded production HTTP send path behind explicit gates;
- production mutation response redaction;
- post-submit order-state readback proof contract;
- kill-switch checks around the send boundary;
- production mutation audit trail;
- failure-mode and no-retry semantics;
- Dashboard production mutation evidence read-only panel;
- v0.16 release gates;
- readiness report and release notes.

`v0.16.0` explicitly does not include:

- strategy-driven production execution;
- multiple production orders;
- batch production orders;
- `MARKET` orders;
- production cancel, replace, amend, retry, correction, or flatten;
- automatic production remediation;
- listenKey lifecycle access;
- signed WebSocket user stream runtime;
- multi-account or multi-venue execution;
- VWAP/POV/Iceberg execution algorithms;
- real-funds proof in CI;
- production trading platform claim;
- Dashboard order controls.

## Published Capability Track: v0.17.0

`v0.17.0` is the published Production Reconciliation And Orphan Recovery
Evidence line. It preserves the v0.16 single owner-approved production mutation
candidate boundary and adds local/offline evidence for reconciliation,
orphan-risk detection, restart recovery, read-only Dashboard visibility, and
incident classification.

`v0.17.0` includes:

- local production order ledger;
- redacted exchange readback mapper;
- local-vs-exchange reconciliation classifier;
- orphan order risk detector;
- restart recovery evidence;
- owner-approved cancel recovery boundary documentation without cancel
  execution;
- read-only Dashboard reconciliation and orphan-risk evidence panel;
- failure and incident semantics;
- v0.17 aggregate release gates;
- readiness report and release notes.

`v0.17.0` explicitly does not include:

- network readback execution;
- new production order submission;
- additional production order mutation;
- actual cancel send;
- automatic cancel;
- automatic orphan cleanup;
- retry, replace, amend, correction, flatten, or remediation;
- Dashboard order controls;
- Dashboard cancel controls;
- Dashboard credential input;
- multi-account production execution;
- multi-venue production execution;
- VWAP/POV/Iceberg execution algorithms;
- listenKey creation, keepalive, or close lifecycle;
- real-funds proof in CI;
- general production trading platform claim.

## Published Capability Track: v0.21.1

`v0.21.1` is the published Unified Read Model Foundation Hardening Patch line.
It preserves the v0.21.0 no-submit/no-Dashboard-controls boundary while adding
release gates, strict provenance, and v0.22.0 dependency proof for the V211
hardening chain.

`v0.21.1` includes:

- unified read model contract and JSON schema;
- account snapshot read model;
- position read model and risk projection inputs;
- order lifecycle read model;
- fill/execution read model with dedupe and reconciliation;
- unified risk state projection;
- Trader Terminal read-only Dashboard foundation;
- health status semantic hardening;
- executable read-model replay for critical cases;
- JSON Schema boundary tightening;
- Trader Terminal read-only runtime bridge evidence;
- v0.21.1 release gates and strict provenance.

`v0.21.1` explicitly does not include:

- product-grade live trading terminal;
- Trader Terminal workbench;
- new production submit capability;
- production order mutation;
- implicit retry;
- automatic cancel;
- automatic remediation;
- retry, replace, amend, correction, flatten, or remediation;
- Dashboard order controls;
- Dashboard approval controls;
- Dashboard cancel controls;
- Dashboard retry controls;
- Dashboard submit/replace/amend/flatten/order-ticket controls;
- strategy-driven production execution;
- multi-account production execution;
- multi-venue production execution;
- listenKey creation, keepalive, or close lifecycle;
- real-funds proof in CI;
- general production trading platform claim.

## Published Capability Track: v0.22.0

`v0.22.0` is the published Trader Terminal Workbench line. It uses the
v0.21.1 canonical Unified Read Model runtime bridge and presents the scoped
workbench as read-only first.

`v0.22.0` includes:

- v0.22 scope decision and v0.21.1 dependency gate;
- Trader Terminal read-only workbench shell and navigation;
- account and position workbench panels;
- order and fill workbench panels;
- risk, alerts, audit, and provenance drill-down panels;
- gated manual operation-entry contract;
- runtime degradation and boundary tests;
- v0.22 release gates and strict provenance.

`v0.22.0` explicitly does not include:

- product-grade live trading terminal;
- new production submit capability;
- production order mutation;
- ungated submit, cancel, retry, replace, amend, or flatten;
- implicit retry;
- automatic cancel;
- automatic remediation;
- Dashboard order controls;
- Dashboard approval controls;
- Dashboard cancel controls;
- Dashboard retry controls;
- Dashboard submit/replace/amend/flatten/order-ticket controls;
- strategy-driven production execution;
- multi-account production execution;
- multi-venue production execution;
- listenKey creation, keepalive, or close lifecycle;
- real-funds proof in CI;
- general production trading platform claim.

## Published Hardening Patch: v0.20.1

`v0.20.1` is the published Production Order Lifecycle Release Closeout &
Provenance Hardening Patch. It preserves the v0.20.0 owner-approved foundation
boundary while closing V201 release evidence before v0.21.0 read-model work
started.

`v0.20.1` includes:

- v0.20.0 release closeout and publication evidence backfill;
- V20 provenance hardening across tests, fixtures, and golden traces;
- durable single-shot attempt ledger and atomic approval consumption;
- pre-submit notional consistency hardening;
- adapter source and readback provenance labeling;
- Dashboard diagnostics hardening for foundation boundaries;
- v0.20.1 release gates and dependency proof.

`v0.20.1` explicitly does not include:

- product-grade live trading terminal;
- new production submit capability;
- implicit retry;
- automatic cancel;
- automatic remediation;
- bulk order execution;
- retry, replace, amend, correction, flatten, or remediation;
- Dashboard order controls;
- Dashboard approval controls;
- Dashboard cancel controls;
- Dashboard retry controls;
- strategy-driven production execution;
- multi-account production execution;
- multi-venue production execution;
- listenKey creation, keepalive, or close lifecycle;
- real-funds proof in CI;
- general production trading platform claim.

`v0.20.0` remains the base Owner-Approved Production Order Lifecycle Foundation
release for this hardening patch. `v0.21.0` absorbed the next scoped read-model
foundation line without inheriting submit expansion from v0.20.1.

## Published Capability Track: v0.19.0

`v0.19.0` is the published Owner-Approved Single-Shot Actual Cancel line. It
preserves the v0.18 release/provenance hardening boundary while adding a single
manual actual cancel attempt under explicit owner approval, risk, adapter,
readback, failure, Dashboard audit, and golden trace gates.

`v0.19.0` includes:

- owner-approved single-shot actual cancel;
- one approval, one order, one venue, one execution attempt;
- risk gate evidence;
- adapter boundary and capability evidence;
- post-cancel readback reconciliation;
- failure and partial-success evidence;
- read-only Dashboard actual cancel audit;
- actual cancel golden traces;
- v0.19 aggregate release gates;
- released readiness report and release notes.

`v0.19.0` explicitly does not include:

- production order submit lifecycle;
- automatic cancel;
- bulk cancel;
- retry, replace, amend, correction, flatten, or remediation;
- second cancel;
- compensation trade;
- Dashboard order controls;
- Dashboard cancel controls;
- Dashboard credential input;
- strategy-driven production execution;
- multi-account production execution;
- multi-venue production execution;
- listenKey creation, keepalive, or close lifecycle;
- real-funds proof in CI;
- general production trading platform claim.

`v0.19.1` is the Actual Cancel Release Closeout & Provenance Hardening patch
for this actual-cancel-only line. `v0.20.0` is the next capability track for
Owner-Approved Production Order Lifecycle Foundation.

## Published Capability Track: v0.18.0

`v0.18.0` is the published Owner-Approved Cancel Recovery Preview line. It
preserves the v0.17 reconciliation and orphan-risk evidence boundary while
adding preview-only cancel recovery contracts, owner approval lifecycle
evidence, failure/rollback evidence, release gates, and Dashboard diagnostics.

`v0.18.0` includes:

- cancel recovery intent contract;
- owner approval lifecycle evidence;
- preview request/response artifact contracts;
- post-cancel readback contract;
- failure and partial-success semantics;
- rollback evidence;
- read-only Dashboard cancel recovery diagnostics;
- v0.18 aggregate release gates;
- readiness report and release notes.

`v0.18.0` explicitly does not include:

- actual cancel send;
- automatic cancel;
- automatic remediation;
- retry, replace, amend, correction, flatten, or remediation;
- Dashboard order controls;
- Dashboard cancel controls;
- Dashboard credential input;
- strategy-driven production execution;
- multi-account production execution;
- multi-venue production execution;
- listenKey creation, keepalive, or close lifecycle;
- real-funds proof in CI;
- general production trading platform claim.

`v0.18.1` is the release surface and provenance hardening patch for this
preview-only line. `v0.19.0` is the published successor capability track for
owner-approved single-shot actual cancel.

## Corrected Capability Sequence: v0.9.0 through v0.16.0

The previous idea of making `v0.9.0` a Binance testnet order lifecycle proof is
superseded. `v0.9.0` is now published as Strategy Runtime Foundation, and
`v0.10.0` is now published as the Binance spot sandbox order proof release.

Corrected sequence:

```text
v0.9.0  = local deterministic Strategy Runtime batch foundation
v0.9.1  = Strategy Runtime Semantics & Audit Hardening
v0.10.0 = Binance Spot Sandbox Order Proof
v0.11.0 = Production Read-Only Contract + Offline Shadow Portfolio
v0.12.0 = Production Online Read-Only + Persistent Shadow
v0.12.1 = Production Read-Only Evidence & Release Surface Hardening
v0.13.0 = Guarded Live Alpha Preflight only
v0.14.0 = Production Order-State Read-Only + Live Alpha Dry-Run
v0.15.0 = Guarded Live Alpha Mutation Scope + Execution Dry-Run Harness
v0.16.0 = Minimum Owner-Approved Production Order Mutation Candidate
```

`v0.9.0` is the published batch foundation track. It makes `ntpro-node` load a
strategy session config, run a bounded built-in demo strategy against
fixture/mock market input, write signal/order-intent/risk decision/audit
artifacts, expose supervisor status, and show read-only Dashboard state.

`v0.9.1` is the hardening track that must make the runtime semantics honest:
node running implies session state is coherent, market exhaustion is not labeled
running, heartbeat counters do not regress, artifact gaps are visible, and
release gates verify the Supervisor/Dashboard path.

`v0.9.0` explicitly does not include:

- Binance testnet order submission;
- order cancel/replace/amend;
- production order submission;
- real funds;
- production trading;
- Dashboard order controls;
- strategy-driven live execution through an exchange adapter.

`v0.10.0` is the published track where Binance spot sandbox order proof was
completed behind explicit owner gates and with its own risk, redaction, and
lifecycle evidence.

`v0.11.0` is the published Production Read-Only Contract + Offline Shadow
Portfolio track. It is contract/offline-shadow only and must not claim
successful online production reads, submit, cancel, replace, amend, or
automatically correct production orders.

`v0.12.0` is the Production Online Read-Only + Persistent Shadow release. It
must not be described as production trading readiness, production order
submission readiness, real-funds readiness, production portfolio parity, or
Dashboard order-control readiness.

`v0.12.1` is the hardening-only patch track for v0.12 release evidence and
release surface. It must not be described as a live-alpha, production trading,
production order mutation, listenKey lifecycle, signed WebSocket user stream
runtime, real-funds, production portfolio parity, or Dashboard order-control
release.

`v0.13.0` is the published Guarded Live Alpha Preflight line. The V130-001
scope decision limits it to preflight evidence only before any live command or
production trading capability is claimed. `v0.14.0` is the published follow-up
read-only/dry-run line. `v0.15.0` is the published mutation-scope/request-preview
dry-run line. `v0.16.0` is the published owner-approved single-order candidate
line. `v0.17.0` is the published reconciliation and orphan-risk evidence line.
`v0.18.0` is the published owner-approved cancel recovery preview line.
`v0.18.1` is the release surface and provenance hardening patch. `v0.19.0` is
the published owner-approved single-shot actual cancel line. `v0.19.1` is the
actual cancel release closeout and provenance hardening patch. `v0.20.0` is the
next capability track for Owner-Approved Production Order Lifecycle Foundation.

## Product Surface Direction

Supported product surfaces:

- Rust workspace crates;
- Rust CLI commands and command contracts;
- Rust examples and documentation;
- Rust release verification scripts;
- Dashboard read-only local artifact surfaces;
- local Python helper scripts under `scripts/` only, used for repository
  control or release evidence.

Unsupported product surfaces:

- Python package installation;
- Python import/API usage;
- PyO3 bindings;
- Cython build or runtime paths;
- Python wheels, PyPI publication, or mixed Rust/Python packaging;
- Cap'n Proto serialization;
- production exchange trading claims without dedicated release evidence.

## Release Gate Direction

Before any next public release, these must agree:

- Shrimp task queue state;
- task evidence under `docs/rust-cutover/evidence/`;
- readiness report;
- release notes;
- README release surface;
- local verification commands;
- hosted GitHub release or PR checks when used as release evidence.

No release may describe a dry-run, fixture, mock, sandbox, or read-only probe as
production trading readiness.

## Versioning

NTPRO has multiple version identifiers with different meanings:

- release tags such as `ntpro-rust-only-v0.6.0`;
- Cargo workspace package version such as `0.58.0`;
- badge metadata in `version.json`.

The release tag is the product milestone identity. Cargo and badge metadata are
not proof of the current NTPRO release surface. See `docs/versioning.md`.
