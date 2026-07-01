# v0.22.0 Account And Position Workbench Panels

Date: 2026-07-01
Executor: Codex
Task: `V220-002` / GitHub issue `#685`

## Summary

This document records the v0.22 Trader Terminal account and position panel
surface. The panels are read-only projections over the existing v0.21.1
canonical Unified Read Model runtime bridge. They expose account and position
state, freshness, provenance, redaction, and lineage diagnostics without
adding operation controls.

Plain Chinese summary: 本任务只增加 Trader Terminal workbench 的账户和持仓
只读面板。账户与持仓数据来自已有 v0.21.1 canonical read model runtime；缺少
provenance、数据 stale 或 account_id 不一致时必须显示 degraded/fail_closed。
它不新增资金划转、账户配置修改、自动平仓、持仓修复或任何真实交易入口。

## Runtime Contract

```text
surface = Trader Terminal account and position panels
source artifact = v0_21/unified_read_model_snapshot.json
source snapshot field = read_model_runtime.account / read_model_runtime.positions
fallback without artifact = degraded shell with unknown panel values
missing account/position provenance = fail_closed/error diagnostic
account-position mismatch = fail_closed/error diagnostic
funds transfer / account config mutation / auto flatten / position repair = false or absent
```

## Panel Layout

```text
account panel = account status, freshness, risk state, equity, available balance, balance entry count, source provenance, redaction
position panel = position status, freshness, account, long/short/flat side, quantity, notional, precision, lineage, source provenance, redaction
provenance drill-down = source_type, source_ref, redaction state, positions lineage
```

## Boundary

The panels are display-only. They do not authorize account mutation or position
repair behavior and do not introduce product-grade live trading readiness.

```text
funds transfer controls = not included
account configuration mutation controls = not included
auto flatten controls = not included
position repair controls = not included
order/fill workbench panels = reserved for V220-003
risk/alert/audit drill-down = reserved for V220-004
manual operation entry contract = reserved for V220-005
runtime degradation and boundary tests = reserved for V220-006
release gates = reserved for V220-007
```

## Validation Surface

The implementation is validated through local Rust dashboard tests and a JS
syntax smoke. The tests assert fresh data projection, stale degradation, missing
provenance fail-closed diagnostics, account-position mismatch fail-closed
diagnostics, and absent/false account and position operation controls.
