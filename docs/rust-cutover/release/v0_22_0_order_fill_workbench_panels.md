# v0.22.0 Order And Fill Workbench Panels

Date: 2026-07-02
Executor: Codex
Task: `V220-003` / GitHub issue `#686`

## Summary

This document records the v0.22 Trader Terminal order and fill panel surface.
The panels are read-only projections over the existing v0.21.1 canonical
Unified Read Model runtime bridge. They expose order lifecycle, attempt ledger,
readback, audit, fill/execution identity, dedupe, partial fill, missing
linkage, reconciliation, risk input, provenance, and lineage diagnostics
without adding operation controls.

Plain Chinese summary: 本任务只增加 Trader Terminal workbench 的订单和成交只读
面板。数据来自已有 v0.21.1 canonical read model runtime；unknown response、
readback mismatch、duplicate attempt、missing ledger、partial fill、duplicate fill
和 missing linkage 会显示 degraded 或 fail_closed。schema-only evidence 不会被
标成 runtime exchange truth。本任务不新增任何下单、撤单、重试、改价、改单、平仓、
fill repair、reconciliation repair 或 execution algorithm 入口。

## Runtime Contract

```text
surface = Trader Terminal order and fill panels
source artifact = v0_21/unified_read_model_snapshot.json
source snapshot field = read_model_runtime.orders / read_model_runtime.fills
fallback without artifact = degraded shell with unknown panel values
order fail closed = unknown response, readback mismatch, duplicate attempt, missing ledger
fill degraded/fail closed = partial fill, duplicate fill, missing linkage, stale/ambiguous source
schema-only truth claim = fail_closed/error diagnostic
submit/cancel/retry/replace/amend/flatten/fill-repair/reconciliation-repair/execution-algorithm controls = false or absent
```

## Panel Layout

```text
order panel = status, freshness, lifecycle, client order id, request digest, attempt id, approval id, readback, audit, ledger present, duplicate attempt, no retry, diagnostics, lineage, source provenance, exchange-truth flags
fill panel = status, freshness, fill id, execution id, order id, client order id, linkage, reconciliation, quantity, cumulative quantity, remaining quantity, price, precision, partial/duplicate flags, risk input, diagnostics, lineage, source provenance, exchange-truth flags
provenance drill-down = source_type, source_ref, exchange_truth, adapter_runtime_integrated, component lineage, component diagnostics
```

## Boundary

The panels are display-only. They do not authorize order entry, order mutation,
fill repair, reconciliation repair, execution algorithms, or product-grade live
trading readiness.

```text
submit/cancel/retry/replace/amend/flatten controls = not included
order permission controls = not included
fill repair controls = not included
reconciliation repair controls = not included
execution algorithm controls = not included
risk/alert/audit drill-down = reserved for V220-004
manual operation entry contract = reserved for V220-005
runtime degradation and boundary tests = reserved for V220-006
release gates = reserved for V220-007
```

## Validation Surface

The implementation is validated through local Rust dashboard tests and a JS
syntax smoke. The tests assert matched order and reconciled fill projection,
unknown response, readback mismatch, duplicate attempt, missing ledger, partial
fill, duplicate fill, missing linkage, schema-only truth fail-closed behavior,
and absent/false order/fill operation controls.
