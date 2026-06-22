# NTPRO v0.14.1 Order-State Owner Evidence Contract

Date: 2026-06-22
Executor: Codex
Milestone: `v0.14.1`
Task: `V141-001`
Status: HARDENING CONTRACT

## Summary

`v0.14.1` may harden the v0.14 owner-gated production order-state read-only
proof by defining how owner-run evidence is validated after the owner manually
runs the proof command.

Plain Chinese summary: v0.14.1 这里做的是“证据收口”：如果 owner 手动跑了生产订单状态只读
GET，就要能用脚本验收证据有没有脱敏、有没有误触下单/改单/撤单、有没有把失败误说成真实交易所状态。
它不是实盘下单版本。

## Product Claim

Allowed claim:

```text
capability = owner-run production order-state read-only evidence validation
default execution posture = offline fixture validation
production mutation = none
Dashboard order controls = disabled
real funds trading = not included
```

Not allowed claim:

```text
production order submission
production order cancel
production order replace
production order amend
production order retry
production order correction
listenKey creation
listenKey keepalive
listenKey close
signed WebSocket user stream runtime
automatic remediation
production trading
Dashboard order controls
```

## Owner Evidence Shape

Required evidence:

| Endpoint | Required | Meaning |
| --- | --- | --- |
| `GET /api/v3/openOrders` | Yes | Proves the owner-run read-only path can classify open-order state evidence. |
| `GET /api/v3/order` | Optional | Proves a known single-order lookup when the owner supplies `orderId` or `origClientOrderId`. |

Successful owner-run artifacts may mark:

```text
values_are_exchange_truth = true
status = online_order_state_read_ok
```

For `GET /api/v3/openOrders`, an empty array is valid endpoint-shape evidence
only. It must not be promoted into order lifecycle readiness:

```text
endpoint_shape_validated = true
order_entries_observed = 0
non_empty_order_state_observed = false
order_lifecycle_readiness = false
```

Only non-empty validated order-state evidence may mark:

```text
order_entries_observed >= 1
non_empty_order_state_observed = true
order_lifecycle_readiness = true
```

Classified failures must not mark exchange truth:

```text
values_are_exchange_truth = false
status = online_order_state_read_failed
error_code = stable classified error
```

## Manifest

The validator writes:

```text
schema_version = ntpro.v141_order_state_owner_evidence_manifest.v1
status = offline_fixture_contract_ok | owner_run_order_state_evidence_ok | owner_run_classified_failure
open_orders_evidence_required = true
single_order_evidence_optional = true
production_order_submission_attempted = false
production_order_mutation_attempted = false
cancel_replace_amend_attempted = false
listen_key_lifecycle_attempted = false
dashboard_order_controls_enabled = false
automatic_remediation_attempted = false
real_orders_submitted = false
production_trading_enabled = false
secrets_redacted = true
```

## Non-Goals

This contract does not submit orders, mutate orders, create listenKeys, accept
Dashboard credentials, start production trading, or change trading semantics.
