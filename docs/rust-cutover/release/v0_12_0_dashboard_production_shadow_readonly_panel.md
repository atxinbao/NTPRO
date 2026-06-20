# v0.12.0 Dashboard Production Shadow Read-Only Panel

Date: 2026-06-21
Executor: Codex
Task: V120-007

## Positioning

V120-007 makes the local Dashboard consume v0.12 production read-only and
persistent shadow artifacts. It is a visibility surface only.

Plain Chinese summary: 这个面板是“看状态”的，不是“做交易”的。用户可以看到生产只读
探测有没有证据、账户快照是否可用、shadow portfolio/session/reconciliation 当前是什么
状态，以及系统是否保持 risk halt。Dashboard 不会读取密钥，不会发网络请求，不会下单。

## Inputs

When present, Dashboard prefers:

```text
artifact_root/v0_12/production_public_online_read_probe.json
artifact_root/v0_12/production_account_snapshot_redacted.json
artifact_root/v0_12/production_readonly_response_shape.json
artifact_root/v0_12/shadow_portfolio_runtime.json
artifact_root/v0_12/shadow_strategy_session.jsonl
artifact_root/v0_12/reconciliation_events.jsonl
```

If no v0.12 artifact exists, Dashboard keeps the existing v0.11 production
shadow read-model path.

## Dashboard Fields

The panel exposes:

- artifact version;
- public read-only probe status and endpoint class;
- account snapshot status;
- response-shape status and validation boolean;
- shadow portfolio runtime status, exposure status, and PnL status;
- shadow strategy session state and heartbeat count;
- reconciliation status, classification, and recommended local-only action;
- risk-halt, manual-review, and new-order-blocked status;
- production boundary counters.

## Boundary

The panel must keep these values visible and non-mutating:

```text
production_order_submissions_attempted=0
production_orders_submitted=0
production_order_mutations_attempted=0
production_order_state_reads_attempted=0
listen_key_lifecycle_attempted=0
automatic_correction_orders_submitted=0
dashboard_order_controls_enabled=false
real_orders_submitted=false
values_are_exchange_truth=false
```

## Non-Claims

This release note does not claim:

- production trading readiness;
- production order submission;
- production order-state read support;
- exchange-confirmed portfolio parity;
- automatic remediation;
- Dashboard order controls.
