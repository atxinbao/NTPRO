# NTPRO v0.15.0 Dashboard Mutation Preflight Panel

Date: 2026-06-22
Executor: Codex
Milestone: `v0.15.0`
Task: `V150-008`
Status: implementation evidence pending PR closeout

## Summary

`v0.15.0` adds a read-only Dashboard panel for the live-alpha mutation
preflight artifact chain. It surfaces local artifact state for owner review but
does not add trading controls.

Plain Chinese summary: 这是“看状态”的面板，不是“交易按钮”。交易员或 owner 可以看到
人工审批是否有效、请求预览是否已脱敏生成、dry-run adapter 是否只写本地工件、kill
switch runtime gate 是否只对 dry-run 打开，以及是否出现了任何真实请求或 Dashboard
控制痕迹。

## Displayed State

The Dashboard reads local artifacts and displays:

- readiness: `blocked`, `ready_for_owner_review`, or boundary violation;
- missing gates and risk reason;
- manual approval state, validity, one-time use, expiry, and issues;
- kill switch active state and runtime gate decision;
- redacted request preview metadata: method, target, endpoint class, query
  shape, signature preflight, and secret-redaction state;
- execution dry-run adapter state;
- order-state age and read-only proof summary;
- production mutation counters and forbidden-control flags;
- artifact paths for audit.

## Product Boundary

Allowed claim:

```text
Dashboard displays local live-alpha mutation preflight evidence for owner review.
```

Not allowed claim:

```text
Dashboard can submit production orders.
Dashboard can cancel, replace, amend, retry, reconnect, or correct production orders.
Dashboard can call production adapters.
Dashboard can use real funds or enable production trading.
```

## Boundary Enforcement

The panel degrades to boundary violation when any scoped artifact records:

```text
request_sent = true
dashboard_order_controls_enabled = true
production_order_submissions_attempted > 0
production_order_mutations_attempted > 0
production_orders_submitted > 0
production_adapter_called = true
production_adapter_instantiated = true
strategy_intent_reaches_production_adapter = true
network_attempted = true
real_orders_submitted = true
real_funds = true
production_trading_enabled = true
```

## Verification

```text
cargo test -p nautilus-cli live_alpha_v15_dashboard --lib
cargo test -p nautilus-cli dashboard --lib
scripts/ai/verify_fast.sh
git diff --check
```
