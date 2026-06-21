# NTPRO v0.13.0 Dashboard Trader/Ops Control Boundary

Date: 2026-06-21
Executor: Codex
Milestone: `v0.13.0`
Task: `V130-005`
Status: boundary contract

## Summary

V130-005 defines the Dashboard control boundary for the Guarded Live Alpha
Preflight line. The Dashboard remains a local read-model and local supervisor
control surface. It does not become a production trading terminal in v0.13.

Plain Chinese summary: v0.13 的 Dashboard 还是“看状态、看证据、管本地节点”的
控制台，不是实盘下单终端。交易员可以看到状态和风险/执行/只读证明；运维可以启动、
停止、暂停、恢复本地 node，并记录本地沙盒重连不支持。生产下单、撤单、改单、重试、
纠错、密钥输入、listenKey、真实资金交易按钮都没有。

## Role Boundary

| Role | Allowed in v0.13 | Not allowed in v0.13 |
| --- | --- | --- |
| Trader | Read status, shadow evidence, proof-pack summaries, risk/execution indicators, and no-mutation evidence. | Submit, cancel, replace, amend, retry, correct, flatten, or otherwise mutate production orders. |
| Ops | Run local supervisor lifecycle controls and inspect local node artifacts. | Enter production credentials, open listenKey/user-stream runtime, reconnect production venues, or enable order controls. |

## Allowed Dashboard Controls

The Dashboard may expose only local supervisor controls:

```text
start
stop
pause
resume
reconnect_data
reconnect_execution
```

The two reconnect controls remain local sandbox unsupported-status records in
v0.13. They do not connect to production venues, do not touch credentials, and
do not recover production order state.

## Required Boundary Markers

```text
dashboard_surface=local_read_model
trader_role=read_only_status_and_evidence
ops_role=local_supervisor_lifecycle_only
allowed_local_controls=start,stop,pause,resume,reconnect_data,reconnect_execution
dashboard_order_controls_enabled=false
dashboard_credential_entry_enabled=false
production_order_submission_allowed=false
production_order_mutation_allowed=false
production_reconnect_allowed=false
listen_key_lifecycle_allowed=false
real_orders_submitted=false
production_trading_enabled=false
```

## Forbidden Dashboard Routes

The Dashboard must not expose HTTP control routes for:

```text
submit
submit_order
cancel
cancel_order
replace
replace_order
amend
amend_order
retry
retry_order
correct
correct_order
flatten
flatten_position
credential_entry
listen_key
```

## Implementation Evidence

- Static Dashboard tests assert the local supervisor allowlist stays explicit
  and forbidden order-control action names stay absent from the shell and JS.
- HTTP tests assert forbidden production order-control routes return 404.
- `scripts/ai/verify_v13_dashboard_control_boundary.sh` runs the targeted
  tests and scans this release contract plus the v0.13 scope decision.

## Non-Claims

This document does not claim production order mutation readiness. It does not
authorize any Dashboard, supervisor, node, strategy, risk, execution, or adapter
surface to submit, cancel, replace, amend, retry, correct, flatten, reconnect,
or otherwise mutate production exchange orders.
