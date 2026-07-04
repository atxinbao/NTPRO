# NTPRO v0.24.0 Cancel Replace Amend Preview Contract

Date: 2026-07-04
Executor: Codex
Task: `V240-005` / GitHub issue `#748`
Milestone: `v0.24.0`

## Summary

This document defines the v0.24.0 cancel / replace / amend preview contract. It
records read-only operation plans for existing orders, with lineage, approval,
risk, and audit gates attached. It does not send network requests, call
execution adapters, mutate orders, or enable flatten.

Plain Chinese summary: 这是 v0.24.0 的 cancel / replace / amend preview 合约。
它只生成可审计的撤改单预案，要求 original order lineage、approval、risk gate 和
audit gate 齐全；不会发送真实请求、不会触发 adapter、不会修改订单状态、不会开放
flatten。

## Contract Identity

```text
schema_version = ntpro.v240_cancel_replace_amend_preview.v1
contract_id = ntpro.v240_cancel_replace_amend_preview_contract.v1
contract_status = preview_evidence_only_no_cancel_replace_amend_send
start_gate_dependency = scripts/ai/verify_release.sh v24-order-slicing-preview
golden_trace = tests/golden/v240_cancel_replace_amend_preview.jsonl
```

## Schema Binding

```text
cancel_intent_schema_version = ntpro.v240_cancel_intent.v1
replace_intent_schema_version = ntpro.v240_replace_intent.v1
amend_intent_schema_version = ntpro.v240_amend_intent.v1
original_order_lineage_required = true
owner_approval_id_required = true
policy_approval_id_required = true
risk_gate_id_required = true
audit_gate_id_required = true
account_key_required = true
strategy_key_required = true
venue_node_key_required = true
isolation_scope_key_required = true
approval_expires_at_required = true
evaluated_at_required = true
field_change_audit_required = true
```

Every preview must bind the same account, strategy, venue node, and isolation
scope across original order lineage, operation intent, policy approval, risk
gate, and audit gate.

## Deterministic Decisions

```text
cancel_preview_ready = cancel preview plan produced
replace_preview_ready = replace preview plan produced with audited field changes
amend_preview_ready = amend preview plan produced with audited field changes
blocked_missing_lineage = original order lineage missing
blocked_scope_mismatch = original order lineage scope differs from intent scope
blocked_expired_approval = owner or policy approval expired
blocked_forbidden_operation = forbidden replace, amend, or flatten requested
```

The preview is fail-closed. Unknown lineage, stale approvals, incomplete audit
data, or prohibited operation claims cannot implicitly allow a send plan.

## Required Coverage

```text
cancel_preview_case = execution.v240_cancel_replace_amend.cancel_preview.001
replace_preview_case = execution.v240_cancel_replace_amend.replace_preview.001
amend_preview_case = execution.v240_cancel_replace_amend.amend_preview.001
missing_lineage_case = execution.v240_cancel_replace_amend.missing_lineage.001
scope_mismatch_case = execution.v240_cancel_replace_amend.scope_mismatch.001
expired_approval_case = execution.v240_cancel_replace_amend.expired_approval.001
forbidden_operation_case = execution.v240_cancel_replace_amend.forbidden_operation.001
```

## Stable Codes

```text
v240_cancel_preview_ready
v240_replace_preview_ready
v240_amend_preview_ready
v240_cancel_replace_amend_missing_lineage
v240_cancel_replace_amend_scope_mismatch
v240_cancel_replace_amend_expired_approval
v240_cancel_replace_amend_forbidden_operation
```

## Read-Only Evidence Boundary

```text
dashboard_readonly_evidence = true
network_attempted = false
execution_adapter_call_allowed = false
live_exchange_request_allowed = false
production_order_mutation_allowed = false
new_submit_capability = false
cancel_replace_amend_send_allowed = false
flatten_allowed = false
dashboard_operation_controls_enabled = false
signed_request_present = false
```

## Validation

Use:

```bash
scripts/ai/verify_release.sh v24-cancel-replace-amend-preview
```

The gate validates the v24 order slicing prerequisite, the generic golden trace
envelope, release replay scope registration, operation coverage, audited field
changes for replace/amend, no-send evidence fields, and a negative selftest
that rejects a preview containing a signed request marker.
