# NTPRO v0.11.0 Authenticated Account Snapshot Contract

Date: 2026-06-20
Executor: Codex
Status: PLANNED CONTRACT

## Summary

`v0.11.0` adds a local authenticated production read-only account snapshot
contract. The contract records how a future owner-gated `GET /api/v3/account`
read must be classified and redacted, but V110-003 does not open a network
connection and does not compute or persist a signature.

Plain Chinese summary: 这一步只是把“生产认证只读账户快照”做成 CLI 和 JSON 证据。
它默认离线，只记录 env 变量名和凭证是否存在，不保存 API key、secret、signature、
signed query 或 signed URL。它不能说明 NTPRO 已经可以做生产交易。

## Supported Endpoint Contract

| Endpoint | Method | Class | Credentials | Mutation |
| --- | --- | --- | --- | --- |
| `https://api.binance.com/api/v3/account` | `GET` | `production_authenticated_read_only` | Env-only, redacted | No |

Order endpoints such as `/api/v3/openOrders`, `/api/v3/order`, cancel, replace,
amend, retry, correction, or unknown production paths remain out of scope for
V110-003.

## CLI Contract

```bash
nautilus live production-account-snapshot-contract \
  --output /tmp/account-snapshot-contract.json
```

Default behavior is closed:

```text
status=blocked_missing_gate
network_attempted=false
account_read_attempted=false
account_mutation_attempted=false
order_endpoint_access_attempted=false
production_order_submission_attempted=false
production_order_mutation_attempted=false
dashboard_order_controls_enabled=false
```

Offline ready-contract mode requires CLI gates, environment gates, and env-only
credential presence:

```bash
NTPRO_ALLOW_PRODUCTION_AUTHENTICATED_READ=1 \
NTPRO_OWNER_APPROVED_PRODUCTION_READ_ONLY=1 \
NTPRO_CONFIRM_PRODUCTION_ACCOUNT_NO_ORDER_MUTATION=1 \
NTPRO_CONFIRM_NO_SECRET_PERSISTENCE=1 \
BINANCE_PRODUCTION_READONLY_API_KEY=... \
BINANCE_PRODUCTION_READONLY_API_SECRET=... \
nautilus live production-account-snapshot-contract \
  --allow-production-authenticated-read \
  --confirm-owner-approved-read-only \
  --confirm-no-order-mutation \
  --confirm-no-secret-persistence \
  --output /tmp/account-snapshot-contract.json
```

The artifact must record presence booleans only. Credential values, raw
signatures, signed queries, and signed URLs must not be persisted.

## Artifact Contract

The artifact schema is `ntpro.v110_authenticated_account_snapshot_contract.v1`.

Required boundary fields:

```text
endpoint_class = production_authenticated_read_only
requires_api_key = true
requires_signature = true
env_credentials_only = true
api_key_value_recorded = false
api_secret_value_recorded = false
signature_recorded = false
signed_query_recorded = false
signed_url_recorded = false
network_attempted = false
account_read_attempted = false
account_mutation_attempted = false
order_endpoint_access_attempted = false
production_order_submission_attempted = false
production_order_mutation_attempted = false
dashboard_order_controls_enabled = false
```

## Release Boundary

V110-003 may be used as evidence that the authenticated production read-only
account snapshot path is classified, redacted, and fail-closed. It must not be
used as evidence of:

- successful online production account reads;
- open-order reads;
- production order lifecycle readiness;
- real-funds trading readiness.
