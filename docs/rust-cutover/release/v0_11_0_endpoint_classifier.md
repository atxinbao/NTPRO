# NTPRO v0.11.0 Endpoint Classifier Design

Date: 2026-06-19
Executor: Codex
Status: PLANNED DESIGN

## Summary

v0.11.0 needs a central endpoint classifier so sandbox proof, production
read-only probes, authenticated account snapshots, and forbidden mutation
surfaces cannot be confused.

Plain Chinese summary: v0.11 要先把 endpoint 分清楚。`demo-api.binance.com`
和 `testnet.binance.vision` 是 sandbox/testnet；`api.binance.com` 可以只读，但
只有明确 gate 后才能读；任何生产下单、撤单、改单 endpoint 都必须判定为 forbidden。

## Endpoint Classes

| Class | Examples | v0.11 behavior |
| --- | --- | --- |
| `sandbox_spot_demo` | `https://demo-api.binance.com` | Allowed only for sandbox proof family and explicit sandbox gates. |
| `sandbox_spot_test_network` | `https://testnet.binance.vision` | Allowed only for sandbox/testnet proof family and explicit sandbox gates. |
| `production_public_read_only` | `https://api.binance.com/api/v3/time`, `https://api.binance.com/api/v3/exchangeInfo` | Allowed only behind v0.11 public read-only gates. |
| `production_authenticated_read_only` | `GET https://api.binance.com/api/v3/account`, `GET /api/v3/openOrders` | Allowed only behind owner-approved authenticated read-only gates and redaction. |
| `production_mutation_forbidden` | `POST /api/v3/order`, `DELETE /api/v3/order`, replace/amend style endpoints | Forbidden in v0.11. |
| `websocket_public_read_only` | Binance public stream endpoints for market data | Allowed only as read-only stream inputs when a task explicitly scopes it. |
| `websocket_user_read_only` | User data stream / account update stream | Deferred unless a task explicitly approves read-only authenticated stream evidence. |
| `unknown_forbidden` | Any unclassified host, scheme, method, or path | Forbidden by default. |

## Method And Path Rules

The classifier must consider all of these fields:

```text
scheme
host
port
method
path
query_shape
requires_signature
requires_api_key
endpoint_family
```

Rules:

- `GET` does not automatically mean safe; signed account reads still require
  owner-approved authenticated read-only gates.
- Any method that mutates exchange state is forbidden on production hosts in
  v0.11.
- Sandbox endpoints must not be used as evidence of production readiness.
- Production read-only endpoints must not be used as evidence of order mutation
  readiness.
- Unknown hosts, unknown paths, non-HTTPS REST URLs, and ambiguous endpoint
  families fail closed.

## Required Classifier Output

The future classifier artifact should use this shape:

```json
{
  "input_url_redacted": "https://api.binance.com/api/v3/time",
  "method": "GET",
  "host_class": "production",
  "endpoint_class": "production_public_read_only",
  "requires_signature": false,
  "requires_api_key": false,
  "mutation_allowed": false,
  "read_allowed": true,
  "owner_gate_required": false,
  "dashboard_order_controls_allowed": false,
  "decision": "allow_read_only",
  "reason": "public production read-only endpoint"
}
```

For forbidden mutation endpoints:

```json
{
  "method": "POST",
  "host_class": "production",
  "endpoint_class": "production_mutation_forbidden",
  "mutation_allowed": false,
  "read_allowed": false,
  "owner_gate_required": true,
  "dashboard_order_controls_allowed": false,
  "decision": "deny",
  "reason": "production order mutation is out of scope for v0.11"
}
```

## Boundary With v0.10

v0.10.0 proved one owner-gated Binance Spot Demo Mode submit/cancel lifecycle.
That remains a sandbox proof. The classifier must never promote
`sandbox_spot_demo` or `sandbox_spot_test_network` evidence into production
trading readiness.

v0.11.0 may add production reads only. It must keep:

```text
production_order_submission_allowed = false
production_order_mutation_allowed = false
dashboard_order_controls_allowed = false
```

## Gate Requirements

Before any future v0.11 implementation is accepted:

- classifier tests must cover every endpoint class in this document;
- production mutation endpoints must have explicit deny tests;
- artifacts must redact credentials, signatures, and signed queries;
- production read-only probes must record `mutation_allowed=false`;
- Dashboard status must display classifier output as read-only evidence only.
