# NTPRO v0.24.0 Order Intent And Execution Policy Model

Date: 2026-07-04
Executor: Codex
Task: `V240-002` / GitHub issue `#745`
Milestone: `v0.24.0`

## Summary

This document defines the v0.24.0 order intent and execution policy artifact
model. It is a preview-only model for audit and replay. It does not submit
orders, call execution adapters, generate signed payloads, send live exchange
requests, or expose Dashboard live order controls.

Plain Chinese summary: 这是 v0.24.0 的 order intent / execution policy artifact
合约。每个 intent 必须绑定 account、strategy、venue node 和 isolation scope，并且必须
带 policy、owner approval、risk、audit 和 source provenance。缺字段、scope 不匹配或
出现真实操作请求时全部 fail closed。

## Contract Identity

```text
schema_version = ntpro.v240_order_intent_policy_model.v1
contract_id = ntpro.v240_order_intent_execution_policy_model.v1
contract_status = artifact_model_only_no_runtime_adapter_call
start_gate_dependency = scripts/ai/verify_release.sh v24-order-control-contract
golden_trace = tests/golden/v240_order_intent_execution_policy.jsonl
```

## Order Intent Schema

```text
order_intent_schema_version = ntpro.v240_order_intent.v1
intent_id_required = true
instrument_required = true
side_required = true
quantity_required = true
order_type_required = true
time_in_force_required = true
account_key_required = true
strategy_key_required = true
venue_node_key_required = true
isolation_scope_key_required = true
policy_id_required = true
policy_provenance_id_required = true
owner_approval_id_required = true
risk_decision_id_required = true
audit_trace_id_required = true
source_provenance_id_required = true
created_at_required = true
```

The order intent is the local preview record of what a strategy wants to do. It
is not an exchange order, adapter request, signed payload, or production trading
request.

## Execution Policy Schema

```text
execution_policy_schema_version = ntpro.v240_execution_policy.v1
policy_id_required = true
policy_provenance_id_required = true
allowed_order_types_required = true
rate_limit_policy_required = true
throttle_policy_required = true
slicing_policy_required = true
retry_policy_required = true
risk_gate_required = true
audit_gate_required = true
owner_approval_required = true
policy_approval_required = true
account_key_required = true
strategy_key_required = true
venue_node_key_required = true
isolation_scope_key_required = true
redaction_boundary_required = true
```

The policy must bind to the same `account_key`, `strategy_key`,
`venue_node_key`, and `isolation_scope_key` as the intent. Cross-scope reuse is
invalid.

## Fail-Closed Decisions

```text
missing_identity = fail_closed
missing_policy_provenance = fail_closed
missing_owner_approval = fail_closed
missing_risk_decision = fail_closed
missing_audit_trace = fail_closed
policy_scope_mismatch = fail_closed
forbidden_operation = fail_closed
secret_or_signed_payload_present = fail_closed
```

## Stable Codes

```text
v240_order_intent_policy_ready
v240_order_intent_missing_account_key
v240_order_intent_missing_strategy_key
v240_order_intent_missing_venue_node_key
v240_order_intent_missing_isolation_scope_key
v240_order_intent_missing_policy_provenance
v240_order_intent_missing_owner_approval
v240_order_intent_missing_risk_decision
v240_order_intent_missing_audit_trace
v240_execution_policy_scope_mismatch
v240_order_intent_forbidden_operation
v240_order_intent_secret_boundary_violation
```

## Redaction Boundary

```text
api_key_value_recorded = false
api_secret_value_recorded = false
raw_credential_recorded = false
signature_recorded = false
signed_payload_recorded = false
signed_query_recorded = false
signed_url_recorded = false
raw_request_body_recorded = false
raw_exchange_response_recorded = false
execution_adapter_call_allowed = false
live_exchange_request_allowed = false
production_order_mutation_allowed = false
new_submit_capability = false
dashboard_operation_controls_enabled = false
```

## Golden Trace Coverage

```text
valid_intent = execution.v240_order_intent_policy.valid_intent.001
missing_scope = execution.v240_order_intent_policy.missing_scope.001
policy_mismatch = execution.v240_order_intent_policy.policy_mismatch.001
forbidden_operation = execution.v240_order_intent_policy.forbidden_operation.001
```

The V240-002 trace is schema-scoped and contract-only for this task. It records
expected preview decisions and fail-closed outcomes without claiming an adapter
or runtime execution replay.

## Validation

Use:

```bash
scripts/ai/verify_release.sh v24-order-intent-policy
```

The gate validates the v24 order-control contract prerequisite, the generic
golden trace envelope, all required schema markers, fail-closed outcomes, and a
negative selftest that rejects a forbidden submit request.
