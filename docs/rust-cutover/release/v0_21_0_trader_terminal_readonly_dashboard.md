# v0.21.0 Trader Terminal Read-Only Dashboard Foundation

Date: 2026-07-01
Executor: Codex
Task: `V210-007`
GitHub issue: `#658`
Status: DASHBOARD FOUNDATION CONTRACT

## Purpose

This document defines the v0.21 Trader Terminal dashboard foundation as a
read-only projection of the unified account, position, order, fill, and risk
read model. It is a display contract only.

Plain Chinese summary: 本任务只建立 Trader Terminal 的只读 Dashboard
foundation。它展示 account、position、order、fill、risk 和 audit/provenance
diagnostics，但不提供下单、审批、撤单、重试、替换、改单、平仓等控件，也不宣称这是
产品级实盘终端。

## Contract Surface

```text
view = Trader Terminal read-only dashboard foundation
contract_version = ntpro.v210.unified_read_model.v1
view transform = ntpro.v210.trader_terminal_readonly_dashboard.v1
validator = scripts/ai/verify_v21_trader_terminal_readonly_dashboard.sh
release target = scripts/ai/verify_release.sh v21-trader-terminal-readonly-dashboard
golden trace = tests/golden/read_model_dashboard_schema.jsonl
```

## Display Scope

The dashboard foundation may display these read-only panels:

```text
accounts
positions
orders
fills
risk
audit_provenance_diagnostics
```

Required state flags:

```text
foundation_only = true
read_only = true
no_submit_controls = true
display_claim = read_only_foundation
product_grade_trading_terminal_claim = false
```

The view must keep source provenance, component freshness, component lineage,
redaction status, and blocking reasons visible enough for audit and review. A
missing input component must degrade the displayed evidence instead of
inventing a healthy state.

## Forbidden Controls

The dashboard foundation must not expose or enable:

```text
submit, approval, cancel, retry, replace, amend, flatten
```

The following boundary flags must remain false:

```text
new_submit_capability = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
dashboard_order_controls_enabled = false
dashboard_approval_controls_enabled = false
dashboard_cancel_controls_enabled = false
dashboard_retry_controls_enabled = false
dashboard_submit_controls_enabled = false
dashboard_replace_controls_enabled = false
dashboard_amend_controls_enabled = false
dashboard_flatten_controls_enabled = false
trader_terminal_order_ticket_enabled = false
trader_terminal_live_trading_claim = false
retry_replace_amend_flatten_allowed = false
product_grade_trading_terminal_claim = false
```

If a dashboard request attempts any forbidden control, the view contract must
fail closed with `blocked_forbidden_controls` and preserve every control flag
as false.

## Degraded Evidence

Missing evidence remains displayable only as degraded or unavailable state.
For example, missing fill evidence must keep accounts, positions, orders, and
risk visible while marking fills unavailable and audit/provenance diagnostics
degraded. It must not promote the terminal to healthy and must not enable any
control.

## Validation

Use:

```bash
scripts/ai/verify_release.sh v21-trader-terminal-readonly-dashboard
```
