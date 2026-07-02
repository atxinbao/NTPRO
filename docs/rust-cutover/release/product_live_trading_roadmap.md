# NTPRO Product Live Trading Roadmap

Date: 2026-06-30
Executor: Codex
Status: ROADMAP BASELINE

## Purpose

This document is the planning baseline for the NTPRO path from production order
evidence to a product-grade live trading system.

Plain Chinese summary: 本文档是后续版本推进的基准口径。`v0.20.0` 到
`v0.21.0` 只能把项目推进到“生产订单生命周期证据 + 统一账户/持仓/订单/成交/风控
read model”。即使 `v0.21.0` 完成，也不能把系统描述为交易员日常可用的产品级实盘
终端。

## Current Positioning

```text
v0.20.0
Owner-approved production order lifecycle foundation
= 最小实盘闭环基础证据，但仍偏底层/证据链

v0.20.1
v20 发布收口 + provenance 修复 + single-shot attempt ledger
= 修补 v0.20 的可信度和防重复提交硬约束

v0.21.0
统一 read model：账户 / 持仓 / 订单 / 成交 / 风控状态
= 交易员视角的数据底座
```

After `v0.21.0`, the system may be described as:

```text
production order lifecycle evidence + unified live-trading read model
```

It must not be described as:

```text
product-grade live trading terminal
trader daily-use production system
unattended production execution platform
multi-account / multi-venue production trading platform
```

## Version Roadmap

### v0.20.1 - Production Order Lifecycle Hardening Patch

Goal: harden the credibility and duplicate-submit constraints of the v0.20
foundation before any read-model expansion.

Scope:

- v0.20.0 release closeout and publication evidence backfill.
- V20 provenance hardening across tests, fixtures, and golden traces.
- Durable single-shot attempt ledger and atomic owner approval consumption.
- Pre-submit notional consistency hardening.
- Adapter source and readback provenance labeling.
- Dashboard diagnostics hardening for foundation boundaries.
- v0.20.1 release gates and dependency proof.

GitHub milestone:

```text
v0.20.1 = #644-#650
```

### v0.21.0 - Unified Read Model Foundation

Goal: build the trader-facing data foundation without adding new submit
capability.

Scope:

- Unified account / position / order / fill / risk read model contract.
- Account snapshot read model.
- Position read model and risk projection inputs.
- Order lifecycle read model from submit, readback, cancel, and audit evidence.
- Fill and execution read model with dedupe and reconciliation.
- Unified risk state projection.
- Trader Terminal read-only foundation dashboard.
- v0.21 golden traces, release gates, and strict provenance.

GitHub milestone:

```text
v0.21.0 = #651-#659
unblocked by v0.20.1 #644-#650 closeout and ntpro-rust-only-v0.20.1 publication
```

Non-goals:

```text
new submit capability = not included
Dashboard order controls = not included
retry / replace / amend / flatten = not included
strategy-driven live trading = not included
product-grade trading terminal claim = not included
```

### v0.22.0 - Trader Terminal Workbench

Goal: turn the unified read model into a trader workbench surface while keeping
real operations gated.

Scope:

- Trader Terminal read-only shell with navigation, status summary, and
  artifact/provenance drawer over the v0.21.1 canonical read model.
- Account and position workbench panels for account status, balance, position
  side, quantity, notional, precision, freshness, provenance, redaction, and
  lineage diagnostics.
- Order and fill workbench panels for order lifecycle, attempt ledger,
  readback, audit, fill/execution identity, dedupe, partial fill, linkage,
  reconciliation, risk input, provenance, lineage, and schema-only truth
  diagnostics.
- Risk, alerts, audit, and provenance drill-down panels for risk priority,
  missing/stale/schema/redaction/forbidden-control alert severity, audit
  evidence completeness, release provenance, artifact digest, and lineage
  diagnostics.
- Manual operation entry design.
- Owner approval, risk gate, and audit gate integration for any future real
  operation.
- Operator-facing diagnostics and evidence drill-down.

Boundary:

```text
operation entry design = included
read-only workbench shell = included
account/position workbench panels = included
order/fill workbench panels = included
ungated real operation buttons = not included
automatic execution = not included
funds transfer/account mutation/auto flatten/position repair controls = not included
order/fill repair/reconciliation repair/execution algorithm controls = not included
risk/alert/audit/provenance automatic action controls = not included
```

### v0.23.0 - Multi-Account / Multi-Strategy / Multi-Venue Node Isolation

Goal: make live-trading state usable in normal production topology without
mixing accounts, strategies, venues, or permissions.

Scope:

- Account isolation.
- Strategy isolation.
- Venue isolation.
- Node-level identity, permission, and evidence partitioning.
- Cross-node read-model aggregation with explicit provenance.

Boundary:

```text
cross-account implicit operation = not included
cross-venue implicit operation = not included
shared approval consumption across isolated nodes = not included
```

### v0.24.0 - Execution Algorithms And Order Control

Goal: introduce controlled execution and order-control capability only after
the read model and isolation layers are established.

Scope candidates:

- Limit/market boundary decisions.
- Rate limits.
- Order slicing.
- Cancel/replace/amend flow.
- Explicit retry strategy.
- Execution policy evidence and release gates.

Required boundary:

```text
all execution algorithms must be explicitly risk-gated
all retry/replace/amend behavior must be owner-approved or policy-approved
all order-control behavior must be auditable
```

### v0.25.0 - Production Monitoring, Alerting, Incident Handling, And Disaster Recovery

Goal: make production operation observable, recoverable, and accountable.

Scope:

- Production monitoring.
- Alert routing.
- Incident state and operator acknowledgement.
- Disaster recovery and restart evidence.
- Runbooks and recovery drills.
- Post-incident audit artifacts.

Boundary:

```text
system can place or manage orders = insufficient
system must be observable, recoverable, and auditable
```

### v0.26.0+ - Product-Grade Hardening

Goal: mature the system from capability-complete to product-grade.

Scope:

- Permission model.
- Operator audit trail.
- SLOs and production readiness indicators.
- Deployment, upgrade, rollback, and migration runbooks.
- Long-running stability evidence.
- Product support and operational acceptance evidence.

## Dependency Rules

```text
v0.21.0 start gate was satisfied on 2026-06-30 by the v0.20.1 closeout and release evidence.
v0.22.0 start gate was satisfied on 2026-07-01 by the v0.21.1 closeout and release evidence.
V220 work must still follow the V220-000 through V220-007 issue order.
v0.23.0 must remain blocked until the Trader Terminal workbench has a stable read-only/operator boundary.
v0.24.0 must remain blocked until account/strategy/venue isolation exists.
v0.25.0 must remain blocked until order control semantics are explicit and gated.
v0.26.0+ must not be used to excuse missing monitoring, alerting, audit, or rollback evidence in earlier versions.
```

Every future milestone must encode dependency relationships in three places:

```text
milestone description
issue body
issue comments
```

## Product Claim Guardrails

Before `v0.25.0`, do not claim product-grade live trading. Before V220 release
gates close, do not claim Trader Terminal readiness beyond the scoped workbench
line. Before `v0.21.0`, do not claim a trader-facing unified data foundation.

Allowed staged claim after `v0.21.0`:

```text
NTPRO has production order lifecycle evidence and a unified live-trading read
model foundation.
```

Disallowed staged claim after `v0.21.0`:

```text
NTPRO is ready for trader daily-use product-grade live trading.
```
