# NTPRO v0.24.0 Rate-Limit And Throttle Gate Preview

Date: 2026-07-04
Executor: Codex
Task: `V240-003` / GitHub issue `#746`
Milestone: `v0.24.0`

## Summary

This document defines the v0.24.0 rate-limit and throttle gate preview
contract. It produces deterministic read-only evidence for later Dashboard /
Workbench display. It does not perform production rate limiting, send network
requests, call execution adapters, or mutate orders.

Plain Chinese summary: 这是 v0.24.0 的 rate-limit / throttle preview gate 合约。
它只计算和记录 allowed_preview、throttled、blocked_missing_limit、
blocked_scope_mismatch 等只读结果；不会真实下单、不会连交易所、不会触发 adapter。

## Contract Identity

```text
schema_version = ntpro.v240_rate_limit_throttle_gate.v1
contract_id = ntpro.v240_rate_limit_throttle_gate_preview.v1
contract_status = preview_evidence_only_no_runtime_throttle_execution
start_gate_dependency = scripts/ai/verify_release.sh v24-order-intent-policy
golden_trace = tests/golden/v240_rate_limit_throttle_gate.jsonl
```

## Input Binding

```text
account_key_required = true
strategy_key_required = true
venue_node_key_required = true
isolation_scope_key_required = true
intent_id_required = true
policy_id_required = true
rate_limit_policy_id_required = true
throttle_policy_id_required = true
policy_provenance_id_required = true
window_started_at_required = true
evaluated_at_required = true
```

Every input must bind the same account, strategy, venue node, and isolation
scope across intent, execution policy, rate-limit policy, and throttle policy.

## Deterministic Decisions

```text
allowed_preview = within burst, rolling window, and venue-specific caps
throttled = burst, rolling window, or venue-specific cap exceeded
blocked_missing_limit = rate-limit or throttle policy missing
blocked_scope_mismatch = policy scope differs from intent scope
```

The gate is fail-closed. Unknown or incomplete policy data cannot implicitly
allow a preview.

## Required Coverage

```text
allowed_preview_case = execution.v240_rate_limit_throttle.allowed_preview.001
burst_exceeded_case = execution.v240_rate_limit_throttle.burst_exceeded.001
window_exceeded_case = execution.v240_rate_limit_throttle.window_exceeded.001
venue_cap_exceeded_case = execution.v240_rate_limit_throttle.venue_cap_exceeded.001
missing_limit_policy_case = execution.v240_rate_limit_throttle.missing_limit_policy.001
scope_mismatch_case = execution.v240_rate_limit_throttle.scope_mismatch.001
```

## Stable Codes

```text
v240_rate_limit_allowed_preview
v240_rate_limit_burst_exceeded
v240_rate_limit_window_exceeded
v240_rate_limit_venue_cap_exceeded
v240_rate_limit_missing_policy
v240_rate_limit_scope_mismatch
v240_rate_limit_forbidden_operation
```

## Read-Only Evidence Boundary

```text
dashboard_readonly_evidence = true
network_attempted = false
execution_adapter_call_allowed = false
live_exchange_request_allowed = false
production_order_mutation_allowed = false
new_submit_capability = false
dashboard_operation_controls_enabled = false
```

## Validation

Use:

```bash
scripts/ai/verify_release.sh v24-rate-limit-throttle-gate
```

The gate validates the v24 order-intent/policy prerequisite, the generic golden
trace envelope, release replay scope registration, deterministic decision
coverage, read-only evidence fields, and a negative selftest that rejects an
over-limit case incorrectly marked `allowed_preview`.
