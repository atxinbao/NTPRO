# NTPRO v0.11.0 Shadow Execution Intent Contract

Date: 2026-06-20
Executor: Codex
Status: PLANNED CONTRACT

## Summary

`v0.11.0` defines a local production shadow execution intent contract. A shadow
intent records what the strategy or operator would evaluate against read-only
production context, but the record is not an exchange order and cannot be sent
to an execution adapter in this release track.

Plain Chinese summary: 这份合同只说明“影子执行意图”该怎么记账。它可以把真实生产
行情或只读账户快照作为上下文引用，但实际下单必须是关闭的。`actual_submission`
必须是 `false`，`production_orders_submitted` 必须是 `0`。

## Artifact Location

The default v0.11 artifact path is:

```text
v0_11/shadow_execution_intent.jsonl
```

Each line is one JSON object using schema:

```text
schema_version=ntpro.v110_shadow_execution_intent.v1
```

## Required Fields

Every shadow intent must include:

```text
schema_version
run_id
intent_id
strategy_id
source_signal_id
symbol
venue
side
order_type
quantity
notional
time_in_force
market_context_ref
account_context_ref
risk_context_ref
created_at
mode=production_shadow
submission_allowed=false
actual_submission=false
submission_status=blocked_by_v110_shadow_execution_boundary
execution_adapter_called=false
order_endpoint_access_attempted=false
production_order_mutation_attempted=false
dashboard_order_controls_enabled=false
```

The artifact may include owner-visible context such as price bands, account
snapshot artifact paths, read-only balance summaries, or reason codes. It must
not include raw API keys, API secrets, signatures, signed queries, signed URLs,
exchange order ids, or venue order ids.

## Summary Counters

Any v0.11 summary that includes shadow execution intents must include:

```text
shadow_intents_created
production_orders_submitted=0
production_order_mutations_attempted=0
actual_submission_count=0
execution_adapter_calls=0
dashboard_order_controls_enabled=false
```

`shadow_intents_created` may be greater than zero. All mutation and submission
counters must remain zero.

## Allowed Flow

The allowed v0.11 flow is:

```text
read-only market/account context
  -> local signal or operator intent
  -> shadow_execution_intent.jsonl
  -> local risk/readiness decision
  -> local shadow portfolio snapshot
```

The flow must stop before execution adapter routing. Shadow intent generation
does not prove production order lifecycle parity and does not prove production
trading readiness.

## Forbidden Flow

The following are forbidden for this contract:

- sending a shadow intent to an exchange adapter;
- creating a production order request;
- calling `POST /api/v3/order`, `DELETE /api/v3/order`, or mutation-style
  production endpoints;
- creating, canceling, replacing, amending, retrying, or correcting production
  orders;
- persisting raw credentials, signatures, signed queries, or signed URLs;
- exposing Dashboard buttons or controls that can mutate production exchange
  state.

## Release Boundary

V110-004 may be used as evidence that NTPRO has a planned local artifact schema
for production shadow execution intent. It must not be used as evidence that
NTPRO can submit, cancel, amend, or reconcile real production orders.
