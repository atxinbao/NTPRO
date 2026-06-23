# v0.16.0 Production Mutation Audit Trail Contract

Date: 2026-06-23
Executor: Codex
Scope: v0.16.0 minimum owner-approved production order mutation candidate
Task: V160-009

## Purpose

The audit trail is the redacted closeout artifact for a v0.16 production
mutation candidate. It connects the owner approval, request-builder, guarded
send, response redaction, order-state readback, kill-switch evidence, and
failure state into one auditable record.

Plain Chinese summary: 审计链路不是交易按钮。大白话：它是一张“这次候选单证据是否
齐全”的收口单，给 owner、reviewer 和后续 Dashboard 只读面板查看。

## Source Artifacts

The audit trail must read these artifacts:

```text
ntpro.v160_production_mutation_request_builder.v1
ntpro.v160_production_mutation_guarded_send.v1
ntpro.v160_production_mutation_response_redaction.v1
ntpro.v160_production_mutation_order_state_readback.v1
ntpro.v160_production_mutation_runtime_gate.v1
ntpro.v160_production_mutation_signing_approval.v1
ntpro.v150_live_alpha_kill_switch_runtime_gate.v1
```

## Ready Contract

Ready audit trail output must satisfy:

```text
schema_version = ntpro.v160_production_mutation_audit_trail.v1
status = ready_redacted_audit_trail
audit_trail_ready = true
preview_hash = fnv1a64:<hash>
source_artifact_issues = []
missing_cli_flags = []
failure_state = none_recorded
request_sent = false by default/offline
network_attempted = false by default/offline
retry_attempted = false
cancel_attempted = false
replace_attempted = false
amend_attempted = false
flatten_attempted = false
dashboard_order_controls_enabled = false
```

The request-builder source runtime gate remains fail-closed as
`blocked_explicit_send_gate`. The guarded-send artifact is the source of the
send-path and kill-switch around-send evidence.

## Redaction Contract

The audit trail must never persist:

```text
api key value
api secret value
API key header value
signature value
signed query
signed URL
raw request body
raw exchange response
response body
response headers
account balances
unrestricted payload
```

## Non-Goals

V160-009 does not add:

```text
new production send behavior
strategy-driven production trading
retry/cancel/replace/amend/flatten
Dashboard order controls
listenKey lifecycle
multi-order execution
multi-venue or multi-account execution
```
