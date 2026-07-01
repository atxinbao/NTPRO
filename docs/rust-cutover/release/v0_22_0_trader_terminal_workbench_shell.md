# v0.22.0 Trader Terminal Workbench Shell

Date: 2026-07-01
Executor: Codex
Task: `V220-001` / GitHub issue `#684`

## Summary

This document defines the first v0.22 Trader Terminal workbench surface. The
workbench is a read-only shell over the existing v0.21.1 canonical Unified Read
Model runtime bridge. It adds navigation, status summary, component panel
slots, and an artifact/provenance drawer for later V220 account, position,
order, fill, risk, audit, and gated-operation tasks.

Plain Chinese summary: 本任务只建立 Trader Terminal workbench 的只读外壳。
它读取已有的 v0.21.1 canonical read model runtime，不新增下单、撤单、重试、
改价、改单、平仓或自动执行入口；缺少 read model 工件时必须显示 degraded，
不能显示 healthy，也不能宣称产品级实盘交易终端。

## Runtime Contract

```text
surface = Trader Terminal Workbench shell
source artifact = v0_21/unified_read_model_snapshot.json
source snapshot field = read_model_runtime
fallback without artifact = degraded shell
submit/cancel/retry/replace/amend/flatten buttons = not included
product-grade live trading terminal claim = false
```

## Shell Layout

```text
mount = trader-terminal-workbench
summary = Workbench status, read-model node count, read-only boundary, diagnostic
navigation = account, positions, orders, fills, risk, audit/provenance
foundation boundary = v0.21.1 canonical read model artifact and contract
read-only boundary = operation controls disabled
gated-operation boundary = owner approval required before future manual entry
provenance drawer = artifact path, snapshot id, source ref, redaction, blockers
```

## Boundary

The shell is intentionally display-only. Later V220 tasks may fill panel data
and design gated manual entry, but this task does not authorize real operation
controls or product-grade live trading terminal readiness.

```text
account/position workbench panels = reserved for V220-002
order/fill workbench panels = reserved for V220-003
risk/alert/audit drill-down = reserved for V220-004
manual operation entry contract = reserved for V220-005
runtime degradation and boundary tests = reserved for V220-006
release gates = reserved for V220-007
```

## Validation Surface

The implementation is validated through local Rust dashboard tests and a JS
syntax smoke. The tests assert that the workbench shell is present, consumes
`read_model_runtime`, exposes degraded fallback markers, and keeps operation
control entrypoints absent.

