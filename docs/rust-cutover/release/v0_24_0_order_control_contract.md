# NTPRO v0.24.0 Execution And Order-Control Contract

Date: 2026-07-04
Executor: Codex
Task: `V240-001` / GitHub issue `#744`
Milestone: `v0.24.0`

## Summary

This document defines the v0.24.0 execution algorithms and order-control
contract. It is a contract-only boundary for later gated implementation tasks.
It does not call execution adapters, submit orders, cancel orders, replace
orders, amend orders, flatten positions, or expose live Dashboard controls.

Plain Chinese summary: 这是 v0.24.0 的 execution / order-control 合约，不是 runtime
实现。所有真实操作默认禁用；后续 #745-#751 只能在 owner、policy、risk、audit 和
isolation scope 证据都满足时逐步实现 preview/foundation。

## Contract Identity

```text
schema_version = ntpro.v240_order_control_contract.v1
contract_id = ntpro.v240_execution_order_control_contract.v1
contract_status = contract_only_no_runtime_adapter_call
start_gate_dependency = scripts/ai/verify_release.sh v24-intake-gate
```

## Order Type Boundary

```text
limit_order_boundary = preview_contract_only
market_order_boundary = preview_contract_only
limit_requires_price = true
market_requires_slippage_policy = true
market_requires_notional_cap = true
market_requires_liquidity_context = true
time_in_force_policy_required = true
order_quantity_precision_policy_required = true
price_precision_policy_required = true
```

Limit and market order support begins as explicit policy and evidence
contracts. No v0.24.0 task may infer live submit readiness from this document.

## Execution Control Gates

```text
rate_limit_gate = required_before_any_runtime_operation
throttle_gate = required_before_any_runtime_operation
order_slicing_gate = required_before_child_order_preview
cancel_replace_amend_gate = preview_contract_only
retry_policy = default_no_retry
implicit_retry_allowed = false
automatic_cancel_allowed = false
automatic_replace_allowed = false
automatic_amend_allowed = false
automatic_flatten_allowed = false
```

Rate-limit, throttle, slicing, cancel, replace, amend, and retry behavior must
be introduced by their dedicated V240 tasks and must remain preview-only until
a later release gate explicitly allows a narrower runtime action.

## Minimum Approval Conditions

```text
owner_approval_required = true
policy_approval_required = true
risk_gate_required = true
audit_gate_required = true
owner_approval_id_required = true
execution_policy_id_required = true
risk_decision_id_required = true
audit_trace_id_required = true
source_provenance_id_required = true
release_gate_reference_required = true
```

All preview artifacts must preserve enough lineage for later readback and audit
evidence. Missing or mismatched approval, policy, risk, audit, or provenance
inputs block the preview.

## Isolation Scope Binding

```text
account_key_required = true
strategy_key_required = true
venue_node_key_required = true
isolation_scope_key_required = true
missing_account_key = fail_closed
missing_strategy_key = fail_closed
missing_venue_node_key = fail_closed
missing_isolation_scope_key = fail_closed
cross_account_operation = fail_closed
cross_strategy_operation = fail_closed
cross_venue_operation = fail_closed
cross_node_operation = fail_closed
shared_approval_consumption_allowed = false
```

The same `account_key`, `strategy_key`, `venue_node_key`, and
`isolation_scope_key` must bind the order intent, policy decision, risk
decision, preview plan, and audit artifact. Cross-scope reuse is invalid.

## Supported Contract States

```text
disabled = default state for real operations
blocked = missing dependency, identity, policy, risk, owner approval, audit, or provenance evidence
preview_ready = all required evidence matches and no runtime side effect is requested
policy_blocked = execution policy denies or is missing
risk_blocked = risk gate denies or is missing
audit_blocked = audit trace cannot be produced
identity_blocked = account/strategy/venue/isolation binding missing or mismatched
```

## Stable Codes

```text
v240_order_control_contract_ready
v240_order_control_missing_account_key
v240_order_control_missing_strategy_key
v240_order_control_missing_venue_node_key
v240_order_control_missing_isolation_scope_key
v240_order_control_cross_scope_mismatch
v240_order_control_missing_owner_approval
v240_order_control_missing_policy_approval
v240_order_control_missing_risk_gate
v240_order_control_missing_audit_gate
v240_order_control_submit_disabled
v240_order_control_adapter_call_disabled
v240_order_control_dashboard_live_control_disabled
v240_order_control_retry_disabled_without_ledger
```

## Dashboard And Workbench Boundary

```text
dashboard_workbench_boundary = read_only_preview
dashboard_preview_may_show = policy_state,risk_state,audit_state,blocked_reason,preview_plan
dashboard_preview_must_not_send = submit,cancel,retry,replace,amend,flatten
live_order_control_button_enabled = false
dashboard_operation_controls_enabled = false
dashboard_order_controls_enabled = false
dashboard_submit_controls_enabled = false
dashboard_cancel_controls_enabled = false
dashboard_retry_controls_enabled = false
dashboard_replace_controls_enabled = false
dashboard_amend_controls_enabled = false
dashboard_flatten_controls_enabled = false
trader_terminal_order_ticket_enabled = false
```

## Forbidden Claims

```text
new_submit_capability = false
production_order_mutation_allowed = false
execution_adapter_call_allowed = false
live_exchange_request_allowed = false
real_order_side_effect_allowed = false
product_grade_trading_terminal_claim = false
```

## Follow-Up Task Mapping

```text
V240-002 = order intent and execution policy model
V240-003 = rate-limit and throttle gate preview
V240-004 = order slicing preview foundation
V240-005 = cancel replace amend preview contract
V240-006 = retry no-retry policy ledger
V240-007 = readback and audit evidence
V240-008 = Dashboard Workbench read-only order-control preview
```

## Validation

Use:

```bash
scripts/ai/verify_release.sh v24-order-control-contract
```

The gate fails closed if the contract omits required identity bindings, approval
conditions, read-only Dashboard boundary, or forbidden operation flags. It also
contains a negative selftest that rejects a true submit-capability marker.
