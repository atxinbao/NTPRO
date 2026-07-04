# NTPRO v0.24.0 Retry No-Retry Policy Ledger

Date: 2026-07-04
Executor: Codex
Task: `V240-006` / GitHub issue `#749`
Milestone: `v0.24.0`

## Summary

This document defines the v0.24.0 retry / no-retry policy ledger contract. It
records deterministic read-only retry eligibility decisions and no-retry
terminal decisions. It does not schedule retries, send network requests, call
execution adapters, or mutate orders.

Plain Chinese summary: 这是 v0.24.0 的 retry / no-retry policy ledger 合约。
它只记录 retry 是否被 policy/owner/audit/scope/prior attempt 明确允许，或是否
属于 no-retry terminal；不会自动重试真实订单、不会调用 adapter、不会改订单状态。

## Contract Identity

```text
schema_version = ntpro.v240_retry_no_retry_ledger.v1
contract_id = ntpro.v240_retry_no_retry_policy_ledger.v1
contract_status = preview_evidence_only_no_runtime_retry_scheduler
start_gate_dependency = scripts/ai/verify_release.sh v24-cancel-replace-amend-preview
golden_trace = tests/golden/v240_retry_policy_ledger.jsonl
```

## Ledger Binding

```text
retry_intent_digest_required = true
prior_attempt_ref_required = true
retry_policy_id_required = true
policy_approval_id_required = true
owner_approval_id_required = true
audit_ref_required = true
account_key_required = true
strategy_key_required = true
venue_node_key_required = true
isolation_scope_key_required = true
attempt_sequence_required = true
retry_reason_required = true
retry_category_required = true
```

Every ledger row must bind retry intent, prior attempt, retry policy, approval,
audit ref, account, strategy, venue node, and isolation scope. Attempt sequence
must be monotonic within the same isolation scope.

## Deterministic Decisions

```text
retry_preview_allowed = transport error or timeout explicitly allowed by policy
no_retry_terminal = business rejection or risk rejection is terminal
blocked_duplicate_retry = retry intent digest already consumed
blocked_missing_prior_attempt = prior attempt ref missing
blocked_unknown_state_retry = unknown state retry is not explicitly allowed
blocked_policy_mismatch = retry policy scope differs from attempt scope
```

The ledger is fail-closed. Unknown, missing, duplicate, stale, or cross-scope
data cannot implicitly allow retry.

## Required Coverage

```text
transport_retry_allowed_case = execution.v240_retry_policy.transport_retry_allowed.001
timeout_retry_allowed_case = execution.v240_retry_policy.timeout_retry_allowed.001
business_rejection_terminal_case = execution.v240_retry_policy.business_rejection_terminal.001
risk_rejection_terminal_case = execution.v240_retry_policy.risk_rejection_terminal.001
duplicate_retry_case = execution.v240_retry_policy.duplicate_retry.001
missing_prior_attempt_case = execution.v240_retry_policy.missing_prior_attempt.001
unknown_state_blocked_case = execution.v240_retry_policy.unknown_state_blocked.001
policy_mismatch_case = execution.v240_retry_policy.policy_mismatch.001
```

## Stable Codes

```text
v240_retry_policy_transport_retry_allowed
v240_retry_policy_timeout_retry_allowed
v240_retry_policy_business_rejection_terminal
v240_retry_policy_risk_rejection_terminal
v240_retry_policy_duplicate_retry
v240_retry_policy_missing_prior_attempt
v240_retry_policy_unknown_state_blocked
v240_retry_policy_scope_mismatch
```

## Read-Only Evidence Boundary

```text
dashboard_readonly_evidence = true
network_attempted = false
execution_adapter_call_allowed = false
live_exchange_request_allowed = false
production_order_mutation_allowed = false
new_submit_capability = false
retry_scheduler_enabled = false
implicit_retry_allowed = false
dashboard_operation_controls_enabled = false
signed_request_present = false
```

## Validation

Use:

```bash
scripts/ai/verify_release.sh v24-retry-policy-ledger
```

The gate validates the v24 cancel/replace/amend prerequisite, the generic
golden trace envelope, release replay scope registration, retry/no-retry
coverage, no implicit retry boundary, and a negative selftest that rejects an
unknown-state retry incorrectly marked allowed.
