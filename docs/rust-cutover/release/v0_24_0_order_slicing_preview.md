# NTPRO v0.24.0 Order Slicing Preview Foundation

Date: 2026-07-04
Executor: Codex
Task: `V240-004` / GitHub issue `#747`
Milestone: `v0.24.0`

## Summary

This document defines the v0.24.0 order slicing preview contract. It produces
deterministic read-only child-order plan evidence for later audit, readback,
and Dashboard / Workbench display. It does not schedule child orders, send
network requests, call execution adapters, or mutate orders.

Plain Chinese summary: 这是 v0.24.0 的 order slicing preview 合约。它只根据
parent intent 和 slicing policy 计算 child-order 预览计划、notional、rounding
和 risk refs；不会真实拆单、不会下单、不会连接交易所、不会触发 adapter。

## Contract Identity

```text
schema_version = ntpro.v240_order_slicing_preview.v1
contract_id = ntpro.v240_order_slicing_preview_foundation.v1
contract_status = preview_evidence_only_no_child_order_submission
start_gate_dependency = scripts/ai/verify_release.sh v24-rate-limit-throttle-gate
golden_trace = tests/golden/v240_order_slicing_preview.jsonl
```

## Input Binding

```text
parent_intent_id_required = true
execution_policy_id_required = true
slicing_policy_id_required = true
account_key_required = true
strategy_key_required = true
venue_node_key_required = true
isolation_scope_key_required = true
policy_scope_key_required = true
max_child_size_required = true
min_child_interval_ms_required = true
quantity_precision_required = true
price_precision_required = true
rounding_mode_required = true
risk_policy_refs_required = true
```

Every input must bind the same account, strategy, venue node, and isolation
scope across parent intent, execution policy, slicing policy, and risk policy
references.

## Deterministic Decisions

```text
preview_ready = deterministic child-order preview plan produced
blocked_invalid_size = parent quantity or max child size is invalid
blocked_precision_mismatch = parent quantity, child quantity, price, or notional precision is invalid
blocked_scope_mismatch = slicing policy scope differs from parent intent scope
blocked_missing_policy = slicing policy or risk policy refs missing
blocked_forbidden_order_combo = market/limit combination is forbidden by policy
```

The preview is fail-closed. Unknown or incomplete policy data cannot implicitly
allow a child plan.

## Required Coverage

```text
valid_plan_case = execution.v240_order_slicing.valid_plan.001
invalid_size_case = execution.v240_order_slicing.invalid_size.001
precision_mismatch_case = execution.v240_order_slicing.precision_mismatch.001
scope_mismatch_case = execution.v240_order_slicing.scope_mismatch.001
policy_missing_case = execution.v240_order_slicing.policy_missing.001
forbidden_market_limit_combo_case = execution.v240_order_slicing.forbidden_market_limit_combo.001
```

## Stable Codes

```text
v240_order_slicing_preview_ready
v240_order_slicing_invalid_size
v240_order_slicing_precision_mismatch
v240_order_slicing_scope_mismatch
v240_order_slicing_missing_policy
v240_order_slicing_forbidden_order_combo
v240_order_slicing_forbidden_operation
```

## Child Plan Evidence

```text
child_plan_preview_only = true
child_quantity_sum_equals_parent = true
child_quantity_lte_max_child_size = true
min_interval_enforced = true
notional_totals_required = true
rounding_evidence_required = true
risk_policy_refs_required = true
```

Child rows may carry preview identifiers, quantities, notionals, interval
offsets, and risk references only. They must not carry signed requests, raw
exchange payloads, adapter request bodies, exchange order identifiers, or
production routing handles.

## Read-Only Evidence Boundary

```text
dashboard_readonly_evidence = true
network_attempted = false
execution_adapter_call_allowed = false
live_exchange_request_allowed = false
production_order_mutation_allowed = false
new_submit_capability = false
child_order_submission_allowed = false
child_order_scheduler_enabled = false
dashboard_operation_controls_enabled = false
signed_request_present = false
```

## Validation

Use:

```bash
scripts/ai/verify_release.sh v24-order-slicing-preview
```

The gate validates the v24 rate-limit/throttle prerequisite, the generic golden
trace envelope, release replay scope registration, deterministic child-plan
coverage, read-only evidence fields, and a negative selftest that rejects a
child quantity larger than `max_child_size`.
