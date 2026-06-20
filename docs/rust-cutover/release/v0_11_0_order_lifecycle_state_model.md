# NTPRO v0.11.0 Shadow/Read-Only Order Lifecycle State Model

Date: 2026-06-20
Executor: Codex
Status: PLANNED CONTRACT

## Summary

`v0.11.0` defines a local order lifecycle state model for shadow/read-only
evidence. The model lets NTPRO record how a local shadow execution intent moved
through validation, preflight, rejection, observation, or halt states. It is not
a production exchange order lifecycle.

Plain Chinese summary: 这份状态机只服务于本地 shadow/read-only 证据。`ShadowSubmitted`
不是“已经向交易所提交订单”，而是“已经写入本地 shadow ledger”。v0.11 仍然禁止生产
下单、撤单、改单和 Dashboard 下单控制。

## Artifact Location

The default v0.11 artifact path is:

```text
v0_11/order_lifecycle_state.jsonl
```

Each line is one lifecycle event using schema:

```text
schema_version=ntpro.v110_order_lifecycle_state.v1
```

## Required Fields

Each lifecycle event must include:

```text
schema_version
run_id
lifecycle_event_id
intent_id
previous_state
next_state
reason
created_at
actual_submission=false
production_orders_submitted=0
production_order_mutations_attempted=0
exchange_order_id_recorded=false
venue_order_id_recorded=false
dashboard_order_controls_enabled=false
```

The event may include references to read-only account snapshots, shadow
execution intents, shadow portfolio snapshots, and local reconciliation events.
It must not include exchange order identifiers because v0.11 does not submit
production orders.

## Allowed States

| State | Meaning |
| --- | --- |
| `Created` | A local shadow intent or observation record was created. |
| `Validated` | Local schema and boundary checks passed. |
| `PreflightPassed` | Local read-only/shadow preflight passed. |
| `ShadowSubmitted` | The intent was written to the local shadow ledger only. |
| `Rejected` | Local risk/readiness gate rejected the intent. |
| `ObservedOnly` | Read-only production state was observed without mutation. |
| `Orphaned` | A read-only observation has no matching local shadow intent. |
| `Halted` | A mismatch or ambiguity stopped shadow progression. |

## Allowed Transitions

```text
Created -> Validated
Validated -> PreflightPassed
Validated -> Rejected
PreflightPassed -> ShadowSubmitted
PreflightPassed -> Rejected
ObservedOnly -> Orphaned
Orphaned -> Halted
ShadowSubmitted -> Halted
Rejected -> Halted
```

`ShadowSubmitted` is a local-only state. It must never imply an exchange order
submit, cancel, replace, amend, retry, correction, or fill event.

## Forbidden Transitions

The model forbids transitions that imply production mutation:

```text
ShadowSubmitted -> ExchangeSubmitted
ShadowSubmitted -> ExchangeAccepted
ShadowSubmitted -> Filled
ShadowSubmitted -> CancelRequested
ShadowSubmitted -> Cancelled
ShadowSubmitted -> Replaced
ShadowSubmitted -> Corrected
ObservedOnly -> CorrectionOrderSubmitted
Orphaned -> CorrectionOrderSubmitted
```

Those states may only be introduced by a later guarded live release with
separate owner approval and evidence.

## Summary Counters

Any summary that includes lifecycle events must include:

```text
lifecycle_events_created
shadow_lifecycle_events_created
observed_only_events_created
orphaned_events_created
halted_events_created
actual_submission_count=0
production_orders_submitted=0
production_order_mutations_attempted=0
dashboard_order_controls_enabled=false
```

## Release Boundary

V110-006 may be used as evidence that NTPRO has a planned local state model for
shadow/read-only order lifecycle evidence. It must not be used as evidence of
production order lifecycle parity, live fills, exchange order submission, or
real-funds trading readiness.
