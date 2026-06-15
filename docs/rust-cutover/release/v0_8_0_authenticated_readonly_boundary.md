# NTPRO v0.8.0 Authenticated Read-Only Boundary

Date: 2026-06-16
Executor: Codex
Milestone: v0.8.0 authenticated Binance testnet read-only proof
Status: DESIGN GATE

## Purpose

`v0.8.0` may advance NTPRO from the v0.7 public Binance testnet read-only proof
to an authenticated Binance testnet read-only proof. The release must remain
read-only, testnet-only, manual-online-only, and fail-closed.

Plain Chinese summary: v0.8.0 可以证明“带 testnet API key 的只读账号接口能被
手动 opt-in 调通”，但不能下单、撤单、改单、查生产账号、连接生产 Binance，也不能把
任何 secret 写进 artifact、日志、stdout、stderr、文档或 PR body。

## Product Claim

```text
current capability = authenticated Binance testnet read-only proof
default mode = offline / CI-safe
manual online mode = explicit owner opt-in only
production Binance connectivity = not included
order submission = not included
account mutation = not included
real funds = not included
production trading = not included
```

## Execution Gates

All authenticated probes must fail closed unless every gate is true:

```text
NTPRO_V08_MANUAL_ONLINE=1
NTPRO_ALLOW_TESTNET_NETWORK=1
--allow-testnet-network is passed
config environment = testnet
order_submission = disabled
real_orders_submitted = false
required credential env vars are present
credential values are never persisted or printed
endpoint is explicitly allowlisted
```

Default local, PR, CI, and release-gate runs must not require network access or
real credential values. They may validate config, artifact schema, redaction
rules, and dry-run/manual-gate behavior.

## Endpoint Allowlist

The initial v0.8.0 authenticated online proof may use exactly one Binance
testnet endpoint:

| Method | Path | Purpose | Allowed output |
| --- | --- | --- | --- |
| `GET` | `/api/v3/account` | Authenticated account read-only proof on Binance testnet | status classification, HTTP status class, latency, redaction summary, schema summary |

Constraints:

- The endpoint host must be Binance testnet, not production Binance.
- The request must be signed in memory only.
- The signature must never be recorded.
- The raw authorization header must never be recorded.
- The raw response body must never be recorded.
- Balance values, asset names, UID, commission fields, permissions, and any
  account-identifying fields must not be recorded in artifacts.

## Endpoint Denylist

Any endpoint that submits, cancels, replaces, amends, tests, streams, or mutates
orders/account state is out of scope.

Explicitly denied examples:

| Method | Path / class | Reason |
| --- | --- | --- |
| `POST` | `/api/v3/order` | Order submission. |
| `DELETE` | `/api/v3/order` | Order cancellation. |
| `PUT` / `PATCH` | any order endpoint | Order/account mutation surface. |
| `POST` | `/api/v3/order/test` | Order validation path still exercises order-submit surface. |
| `POST` / `PUT` / `DELETE` | listenKey or user-data-stream lifecycle endpoints | User stream lifecycle mutation / session state. |
| any method | production Binance REST/WebSocket endpoint | Production connectivity is outside v0.8.0. |
| any method | margin, futures, transfer, withdrawal, deposit, or account settings endpoints | Account mutation or expanded account surface. |

If an endpoint is not explicitly allowlisted, it is denied.

## Secret Handling Contract

Credentials are env-only:

```text
NTPRO_BINANCE_TESTNET_API_KEY
NTPRO_BINANCE_TESTNET_API_SECRET
```

Allowed secret-related artifact fields:

```text
credential_source = env
api_key_env_var = NTPRO_BINANCE_TESTNET_API_KEY
api_secret_env_var = NTPRO_BINANCE_TESTNET_API_SECRET
api_key_present = true/false
api_secret_present = true/false
secrets_redacted = true
values_recorded = false
```

Forbidden everywhere, including artifacts, stdout, stderr, logs, docs, and PR
bodies:

```text
raw API key value
raw API secret value
raw signature
raw signed query string
raw Authorization / X-MBX-APIKEY header
raw account response body
balances, asset names, account UID, commission details, permissions
```

## Artifact Schema

The authenticated read-only probe artifact may record only bounded metadata:

```json
{
  "schema_version": "v0.8-auth-readonly-probe.v1",
  "environment": "testnet",
  "mode": "authenticated-read-only",
  "manual_online": true,
  "network_allowed": true,
  "endpoint_allowlisted": true,
  "endpoint_method": "GET",
  "endpoint_path": "/api/v3/account",
  "endpoint_class": "binance-testnet-authenticated-account-readonly",
  "endpoint_url_redacted": "https://testnet.binance.vision/api/v3/account",
  "http_status_class": "2xx|4xx|5xx|network_error",
  "latency_ms": 0,
  "credential_source": "env",
  "api_key_env_var": "NTPRO_BINANCE_TESTNET_API_KEY",
  "api_secret_env_var": "NTPRO_BINANCE_TESTNET_API_SECRET",
  "api_key_present": false,
  "api_secret_present": false,
  "secrets_redacted": true,
  "values_recorded": false,
  "raw_response_recorded": false,
  "orders_submitted": false,
  "account_mutation_attempted": false,
  "production_endpoint_used": false
}
```

The artifact must not include account balances, account identifiers, asset
symbols, raw response payloads, signatures, credential values, or raw headers.

## Dashboard Boundary

Dashboard may read and display completed artifacts only. It must not:

- start authenticated probes;
- load credential values;
- render credential values;
- render raw account responses;
- render balances or account-identifying fields;
- open network connections.

Allowed Dashboard copy must use `read-only artifact` language and must preserve
`manual-online-only`, `testnet-only`, `no orders`, `no real funds`, and
`no production trading`.

## Release Gate Requirements

Before a v0.8.0 release decision:

- offline gate validates schema and fail-closed behavior without network or
  credential values;
- manual online gate is opt-in only and not required for default CI;
- synthetic secret leak scanner checks artifacts/logs/stdout/stderr fixtures;
- release notes and readiness report keep authenticated read-only scope
  explicit;
- no PR body or evidence file includes real credential values.

## Out of Scope

```text
order submission
order cancel/replace/amend
listenKey/user-data-stream lifecycle mutation
account mutation
production Binance connectivity
production trading
real funds
Dashboard-started probes
prebuilt binary or Docker delivery
```

## Rollback Plan

Revert the V080-000 boundary document and evidence commit. No runtime behavior
depends on this design gate.
