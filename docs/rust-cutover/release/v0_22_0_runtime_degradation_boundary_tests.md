# v0.22.0 Runtime Degradation and Boundary Tests

Date: 2026-07-02
Executor: Codex
Task: `V220-006` / GitHub issue `#689`

## Summary

This document records the v0.22 Trader Terminal runtime degradation and
boundary test gate. The workbench remains read-only first and must degrade or
fail closed when required read-model evidence is missing, stale, mismatched, or
requests forbidden operation controls.

Plain Chinese summary: v0.22 Trader Terminal workbench 新增运行时降级和边界测试门禁。
缺证据、schema 不匹配、组件不可用、来源 stale、redaction breach 或 provenance
mismatch 时不能显示 healthy；任何 submit/cancel/retry/replace/amend/flatten/
order-ticket 控制标记出现时必须 fail closed。该门禁只验证只读工作台边界，不开放
真实操作。

## Runtime Degradation Cases

```text
missing read-model artifact -> degraded missing_artifact
schema mismatch -> fail_closed schema_mismatch
component unavailable -> degraded component_unavailable
stale source -> stale stale_artifact
redaction breach -> fail_closed
provenance mismatch -> fail_closed
```

## Forbidden Control Cases

```text
dashboard_submit_controls_enabled = fail_closed
dashboard_cancel_controls_enabled = fail_closed
dashboard_retry_controls_enabled = fail_closed
dashboard_replace_controls_enabled = fail_closed
dashboard_amend_controls_enabled = fail_closed
dashboard_flatten_controls_enabled = fail_closed
trader_terminal_order_ticket_enabled = fail_closed
manual_operation_submit_allowed = fail_closed
manual_operation_cancel_allowed = fail_closed
manual_operation_retry_allowed = fail_closed
manual_operation_replace_allowed = fail_closed
manual_operation_amend_allowed = fail_closed
manual_operation_flatten_allowed = fail_closed
```

## Display Claims

```text
read_only_first_boundary = required
foundation-boundary panel = required
read-only-boundary panel = required
gated-operation-boundary panel = required
product_grade_trading_terminal_claim = forbidden
product-grade live trading terminal claim = forbidden
```

## Release Gate

```text
local script = scripts/ai/verify_v22_runtime_boundary_tests.sh
verify_release stage = scripts/ai/verify_release.sh v22-runtime-boundary-tests
release-tag matrix stage = release-v22-runtime-boundary-tests
```

## Non-Goals

This gate does not implement real operation routes, submit/cancel/retry/
replace/amend/flatten controls, execution adapter calls, automatic remediation,
strategy-driven live trading, or product-grade live trading terminal claims.
