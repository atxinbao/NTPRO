# NTPRO v0.11.0 Production Public Read-Only Probe Contract

Date: 2026-06-19
Executor: Codex
Status: PLANNED CONTRACT

## Summary

`v0.11.0` introduces a local production public read-only probe contract. The
contract classifies Binance Spot production public endpoints and writes
auditable artifacts, but V110-002 does not open a network connection.

Plain Chinese summary: 这一步只是把“哪些生产公开 endpoint 可以只读”写成 CLI
和 JSON 证据。它默认离线，不用 API key，不读账户，不提交订单，不撤单，也不能说明
NTPRO 已经可以做真实生产交易。

## Supported Public Endpoints

| Endpoint | Method | Class | Credentials | Mutation |
| --- | --- | --- | --- | --- |
| `https://api.binance.com/api/v3/time` | `GET` | `production_public_read_only` | No | No |
| `https://api.binance.com/api/v3/exchangeInfo` | `GET` | `production_public_read_only` | No | No |

Any signed, account, order, cancel, replace, amend, retry, correction, or
unknown production endpoint remains out of scope for this task.

## CLI Contract

```bash
nautilus live production-public-read-probe \
  --endpoint server-time \
  --output /tmp/public-read-probe.json
```

Default behavior is closed:

```text
status=blocked_missing_gate
network_attempted=false
credentials_used=false
production_order_submission_attempted=false
production_order_mutation_attempted=false
dashboard_order_controls_enabled=false
```

Offline ready-contract mode requires both CLI gates and environment gates:

```bash
NTPRO_ALLOW_PRODUCTION_PUBLIC_READ=1 \
NTPRO_CONFIRM_PRODUCTION_PUBLIC_READ_ONLY=1 \
NTPRO_CONFIRM_NO_PRODUCTION_ORDER_MUTATION=1 \
nautilus live production-public-read-probe \
  --endpoint exchange-info \
  --allow-production-public-read \
  --confirm-read-only \
  --confirm-no-order-mutation \
  --output /tmp/public-read-probe.json
```

This still records `network_attempted=false`. `--manual-online` is deliberately
blocked in V110-002 because actual online production reads require a later task
and separate evidence.

## Artifact Contract

The artifact schema is `ntpro.v110_production_public_read_probe.v1`.

Required boundary fields:

```text
endpoint_class = production_public_read_only
requires_api_key = false
requires_signature = false
read_allowed = false unless all gates are present and manual_online=false
mutation_allowed = false
online_execution_supported = false
network_attempted = false
credentials_used = false
account_mutation_attempted = false
production_order_submission_attempted = false
production_order_mutation_attempted = false
dashboard_order_controls_enabled = false
```

## Release Boundary

V110-002 may be used as evidence that the production public read-only path is
classified and fail-closed. It must not be used as evidence of:

- successful online production reads;
- authenticated account reads;
- production order lifecycle readiness;
- real-funds trading readiness.
