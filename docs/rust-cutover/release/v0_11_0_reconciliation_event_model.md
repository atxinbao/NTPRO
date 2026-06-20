# NTPRO v0.11.0 Production Read-Only Reconciliation Event Model

Date: 2026-06-20
Executor: Codex
Status: PLANNED CONTRACT

## Summary

`v0.11.0` defines a local reconciliation event model for production read-only
and shadow evidence. The model records observations and mismatches, then routes
them to local evidence, degraded status, risk halt, or manual remediation. It
does not give NTPRO authority to mutate production exchange state.

Plain Chinese summary: 这份合同只定义“看到不一致时怎么记录”。v0.11 可以写事件、
标记 degraded、触发本地 risk halt、提示人工处理；不能自动下纠错单，不能自动撤单、
改单或补单。

## Artifact Location

The default v0.11 artifact path is:

```text
v0_11/reconciliation_events.jsonl
```

Each line is one reconciliation event using schema:

```text
schema_version=ntpro.v110_reconciliation_event.v1
```

## Required Fields

Each event must include:

```text
schema_version
run_id
event_id
event_type
severity
observed_at
source_ref
shadow_state_ref
account_snapshot_ref
order_lifecycle_ref
recommended_action
automatic_correction_orders_submitted=0
production_orders_submitted=0
production_order_mutations_attempted=0
cancel_replace_amend_attempted=false
dashboard_order_controls_enabled=false
```

## Event Types

Allowed event types:

```text
event_type=observed_account_state
event_type=observed_order_state
event_type=shadow_mismatch
event_type=orphan_observation
event_type=degraded_status
event_type=risk_halt
event_type=manual_remediation_required
```

## Severity

Allowed severity values:

```text
severity=info
severity=warning
severity=degraded
severity=halt
```

`severity=halt` means local shadow progression must stop until an owner reviews
the evidence. It does not authorize exchange mutation.

## Allowed Actions

Reconciliation events may lead only to local actions:

```text
recommended_action=record_only
recommended_action=mark_degraded
recommended_action=halt_shadow_flow
recommended_action=manual_review_required
recommended_action=manual_remediation_required
```

Manual remediation means an owner must inspect evidence outside automated
v0.11 execution. It does not mean the CLI or Dashboard may create a production
order.

## Forbidden Actions

The model forbids automated production mutation actions:

```text
recommended_action=submit_correction_order
recommended_action=cancel_production_order
recommended_action=replace_production_order
recommended_action=amend_production_order
recommended_action=retry_production_order
recommended_action=auto_flatten_position
```

Those actions may only be considered in a later guarded live release with
separate owner approval and production mutation evidence.

## Summary Counters

Any summary that includes reconciliation events must include:

```text
reconciliation_events_created
orphan_observations_created
shadow_mismatches_created
risk_halts_created
manual_remediation_events_created
automatic_correction_orders_submitted=0
production_orders_submitted=0
production_order_mutations_attempted=0
cancel_replace_amend_attempted=false
dashboard_order_controls_enabled=false
```

## Release Boundary

V110-007 may be used as evidence that NTPRO has a planned local event model for
read-only/shadow reconciliation evidence. It must not be used as evidence of
automatic production reconciliation, correction orders, cancel/replace/amend
flows, or real-funds trading readiness.
