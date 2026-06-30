# v0.21.0 Order Lifecycle Read Model

Date: 2026-06-30
Executor: Codex
Task: `V210-004`
GitHub issue: `#655`
Status: COMPONENT CONTRACT

## Purpose

This document defines the v0.21 order lifecycle read-model component. It
projects v0.20 production order lifecycle evidence into the unified read model:
submit candidate, attempt ledger, response redaction, readback, cancel evidence,
and audit state.

Plain Chinese summary: 本任务只把 v0.20 的订单生命周期证据整理成只读 read model。
它能展示 submit/readback/cancel/audit 证据链，但不新增真实 submit/cancel，不自动重试，
不自动修复，也不新增 Dashboard 操作控件。

## Contract Surface

```text
component = components.orders
contract_version = ntpro.v210.unified_read_model.v1
component transform = ntpro.v210.order_lifecycle_read_model.v1
validator = scripts/ai/verify_v21_order_lifecycle_read_model.sh
release target = scripts/ai/verify_release.sh v21-order-lifecycle-read-model
golden trace = tests/golden/read_model_order_lifecycle_schema.jsonl
```

## Identity And Linkage

Every order lifecycle row must record:

```text
order_id
client_order_id
request_digest
attempt_id
approval_id
audit_ref
readback_ref
provenance_ref
```

Rules:

- `attempt_id` must link to exactly one ledger entry.
- Duplicate submit attempts fail closed.
- Missing ledger evidence fails closed.
- Request and response payloads must remain redacted references, not raw
  exchange payloads.

## Lifecycle States

The read model covers:

```text
submitted
accepted
rejected
unknown_response
readback_matched
readback_mismatch
cancel_preview
actual_cancel_evidence
audit_closed
audit_risk_visible
```

It may display these states as read-only evidence. It must not trigger retry,
cancel, replace, amend, flatten, or remediation.

## Fail-Closed Rules

The orders component must be `fail_closed` when any of these are true:

```text
unknown_order_response_no_retry
order_readback_mismatch
duplicate_submit_attempt
missing_attempt_ledger
missing_order_audit_ref
unredacted_order_payload
```

Fail-closed order snapshots must keep:

```text
health_status = fail_closed
components.orders.component_status = fail_closed
capability_boundary.production_order_submission_allowed = false
capability_boundary.production_order_mutation_allowed = false
capability_boundary.dashboard_order_controls_enabled = false
capability_boundary.retry_order_allowed = false
capability_boundary.automatic_order_remediation_allowed = false
capability_boundary.automatic_cancel_allowed = false
```

## Dashboard Boundary

Dashboard may display order lifecycle state, readback status, attempt/audit
references, and no-retry flags. Dashboard must not expose submit, cancel,
approval, retry, replace, amend, flatten, or remediation controls for this
read-model scope.

## Validation

Use:

```bash
scripts/ai/verify_release.sh v21-order-lifecycle-read-model
```
