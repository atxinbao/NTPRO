# v0.16.0 Production Mutation Response Redaction Contract

Date: 2026-06-23
Executor: Codex
Scope: v0.16.0 minimum owner-approved production order mutation candidate

## Purpose

This document defines the release-facing response redaction contract for the
minimum owner-approved production mutation candidate.

Plain Chinese summary: 这份合同规定“真实生产下单响应回来后，NTPRO 只能保存哪些内容”。
大白话：可以保存 order id、client order id、订单状态、symbol、side、type、GTC、时间戳
形状这类订单元数据；不能保存 raw response、headers、API key、secret、signature、
signed query、signed URL、账户余额、fills 明细或任意 payload。

## Allowed Persisted Fields

The redacted artifact may persist only these response-derived fields:

```text
symbol
side
type
timeInForce
orderId
clientOrderId
status
transactTime shape
workingTime shape
```

Timestamp values are recorded as shape only:

```text
epoch_millis_present_redacted
missing
```

## Forbidden Persisted Fields

The redacted artifact must not persist:

```text
raw response body
HTTP response headers
API key
API secret
API key header value
signature
signed query
signed URL
request body
raw request body
account balances
fills / commission details
unrestricted payload
Dashboard order controls
retry/cancel/replace/amend/flatten evidence
listenKey lifecycle evidence
```

## Artifact Contract

```text
schema_version = ntpro.v160_production_mutation_response_redaction.v1
artifact_type = production_mutation_response_redaction
status = ready_response_redacted
response_redaction_ready = true
response_shape_validated = true
response_type = binance_order_response_redacted_metadata_v1
raw_exchange_response_recorded = false
response_body_recorded = false
response_headers_recorded = false
unrestricted_payload_recorded = false
account_balances_recorded = false
fills_recorded = false
api_key_value_recorded = false
api_secret_value_recorded = false
api_key_header_value_recorded = false
signature_recorded = false
signed_query_recorded = false
signed_url_recorded = false
response_redacted = true
```

## Gate Behavior

The command must block if owner confirmation flags are missing:

```text
status = blocked_missing_gate
response_redaction_ready = false
```

The command must block if the response input contains forbidden markers such as
headers, signature, balances, raw body, or payload:

```text
status = blocked_forbidden_response_marker
response_redaction_ready = false
```

## Non-Goals

This contract does not add:

```text
strategy-driven live trading
automatic production order submission
response body persistence
post-submit order-state readback
retry/cancel/replace/amend/flatten
Dashboard order controls
listenKey or WebSocket user stream lifecycle
multi-order execution
multi-venue or multi-account execution
```

## Verification

The release gate for this contract is:

```text
scripts/ai/verify_v16_response_redaction.sh
```

It must prove:

```text
ready synthetic response -> ready_response_redacted
forbidden synthetic response -> blocked_forbidden_response_marker
missing confirmations -> blocked_missing_gate
raw/headers/secrets/balances/payload are not persisted in ready artifacts
```
