# v0.19.0 Cancel Executor Adapter Boundary

Date: 2026-06-27
Executor: Codex
Milestone: `v0.19.0`
Task: `V190-005`
Status: IMPLEMENTATION CONTRACT

## Summary

This document defines the v0.19.0 cancel executor adapter boundary for the
owner-approved single-shot actual cancel line. It adds a local/offline evidence
artifact that proves a future actual cancel command can only proceed through a
matched adapter capability declaration for one order, one venue, and one
attempt.

Plain Chinese summary: 这次定义真撤单前必须经过的 adapter 边界。大白话：
后续真的发撤单前，必须先证明 owner approval 已授权、adapter 明确支持这个 venue
的单订单撤单、只允许一次尝试，并且 request/response/readback/audit contract 都已记录。
本任务本身不联网、不发撤单、不加 Dashboard 按钮、不实现 retry 或 bulk cancel。

## CLI Surface

```text
nautilus live production-mutation-actual-cancel-executor-adapter-boundary \
  --run-id v190-cancel-executor-adapter-boundary \
  --owner-approval-lifecycle runs/v190/actual-cancel-owner-approval-lifecycle.json \
  --adapter-capability runs/v190/adapter-capability.json \
  --adapter-id binance_spot_cancel_adapter \
  --venue binance_spot \
  --order-id-type exchange_order_id \
  --expected-order-lineage-id lineage-v160-single-shot \
  --expected-symbol BTCUSDT \
  --expected-account-label prod-account-redacted \
  --output runs/v190/actual-cancel-executor-adapter-boundary.json \
  --allow-production-mutation-actual-cancel-executor-adapter-boundary \
  --confirm-adapter-capability \
  --confirm-request-response-readback-audit-contract \
  --confirm-one-order-one-venue-one-attempt \
  --confirm-fail-closed-unsupported-capability \
  --confirm-no-bulk-cancel \
  --confirm-no-retry \
  --confirm-no-automatic-cancel \
  --confirm-no-dashboard-execution \
  --confirm-no-network \
  --confirm-no-secret-persistence
```

## Artifact Contract

```text
schema_version = ntpro.v190_actual_cancel_executor_adapter_boundary.v1
artifact_type = actual_cancel_executor_adapter_boundary
status = adapter_boundary_ready | blocked_missing_gate | blocked_owner_approval_lifecycle | blocked_adapter_capability
adapter_boundary_scope = one_order_one_venue_one_attempt
actual_cancel_send_allowed_by_adapter_boundary = true only when all evidence matches
actual_cancel_send_allowed = false
cancel_attempted = false
cancel_requests_sent = 0
network_attempted = false
dashboard_cancel_controls_enabled = false
```

The artifact links:

```text
owner_approval_lifecycle_ref = V190-003 actual cancel owner approval lifecycle
adapter_capability_ref = adapter capability declaration fixture
adapter_id = required
venue = required
order_id_type = required
known_order_id = inherited from owner approval lifecycle
known_client_order_id = inherited from owner approval lifecycle
symbol = inherited from owner approval lifecycle
account_label = inherited from owner approval lifecycle
```

## Adapter Capability Declaration

The adapter capability input is a local fixture with this shape:

```text
schema_version = ntpro.v190_actual_cancel_adapter_capability.v1
artifact_type = actual_cancel_adapter_capability
adapter_id = required
actual_cancel_supported = true
supported_venues = [selected venue]
supported_order_id_types = [selected order id type]
bulk_cancel_supported = false
cancel_all_supported = false
retry_supported = false
automatic_cancel_supported = false
multi_venue_supported = false
```

## Request / Response / Readback / Audit Contract

```text
cancel_request_contract = single_order_cancel_request_v1
cancel_response_contract = single_order_cancel_response_metadata_v1
post_cancel_readback_contract = single_order_post_cancel_readback_required_v1
audit_contract = single_order_cancel_audit_event_required_v1
max_cancel_requests = 1
allowed_attempts = 1
allowed_order_count = 1
allowed_venue_count = 1
```

## Failure Taxonomy

The adapter boundary records the failure states that later V190 command and
readback tasks must consume:

```text
rejected
timeout
unknown
already_cancelled
venue_unavailable
transport_failure
```

Unsupported or forbidden adapter capabilities fail closed before any send:

```text
adapter_actual_cancel_unsupported
adapter_venue_unsupported
adapter_order_id_type_unsupported
adapter_bulk_cancel_supported_forbidden
adapter_cancel_all_supported_forbidden
adapter_retry_supported_forbidden
adapter_automatic_cancel_supported_forbidden
adapter_multi_venue_supported_forbidden
```

## Boundary

This boundary does not implement:

```text
actual cancel network send
production adapter integration
multi-venue adapter abstraction
automatic cancel
bulk cancel
cancel-all
retry / replace / amend / flatten / remediation
Dashboard cancel button
Dashboard owner approval button
production order submit lifecycle
credential persistence
```

## Validation

```text
cargo test -p nautilus-cli actual_cancel --lib
cargo test -p nautilus-cli parses_live_production_mutation_actual_cancel_executor_adapter_boundary_options --lib
cargo clippy -p nautilus-cli --all-targets -- -D warnings
scripts/ai/verify_fast.sh
git diff --check
```
