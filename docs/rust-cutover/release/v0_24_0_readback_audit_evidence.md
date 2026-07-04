# NTPRO v0.24.0 Readback and Audit Evidence

Date: 2026-07-04
Executor: Codex
Task: `V240-007` / GitHub issue `#750`
Milestone: `v0.24.0`

## Summary

This document defines the v0.24.0 readback and audit evidence contract for
order-control preview artifacts. It binds preview inputs, decision outputs,
policy refs, risk refs, readback refs, audit refs, provenance refs, and scope
keys into one redacted read-only closeout record. It does not claim exchange
truth, read live order state, call execution adapters, send network requests,
or mutate orders.

Plain Chinese summary: 这是 v0.24.0 order-control preview 的 readback/audit
证据合约。它只把预览输入、决策输出、policy、risk、readback、audit、
provenance 和 scope key 绑定成可审计记录；Dashboard 只能消费脱敏只读证据；
不会宣称交易所真实状态、不会读真实订单状态、不会调用 adapter、不会修改订单。

## Contract Identity

```text
schema_version = ntpro.v240_order_control_readback_audit.v1
contract_id = ntpro.v240_order_control_readback_audit_evidence.v1
contract_status = preview_readback_audit_only_no_exchange_truth
start_gate_dependency = scripts/ai/verify_release.sh v24-retry-policy-ledger
golden_trace = tests/golden/v240_readback_audit_evidence.jsonl
```

## Evidence Binding

```text
preview_input_ref_required = true
decision_output_ref_required = true
policy_ref_required = true
risk_ref_required = true
readback_ref_required = true
audit_ref_required = true
provenance_ref_required = true
account_key_required = true
strategy_key_required = true
venue_node_key_required = true
isolation_scope_key_required = true
source_commit_current_required = true
dashboard_redacted_ref_required = true
```

Every closeout row must bind the preview input, decision output, policy, risk,
readback, audit, provenance, account, strategy, venue node, and isolation
scope. Dashboard consumption is limited to the redacted audit reference.

## Audit Closeout States

```text
ready_preview = readback and audit evidence complete
blocked = scope mismatch blocks preview closeout
degraded_unavailable = readback source unavailable, redacted audit retained, no exchange truth claim
fail_closed = missing readback, missing audit, missing provenance, stale source, or redaction breach
```

The contract is fail-closed. Missing, stale, unredacted, or cross-scope data
cannot produce a ready preview.

## Required Coverage

```text
ready_preview_case = execution.v240_readback_audit.ready_preview.001
missing_readback_case = execution.v240_readback_audit.missing_readback.001
missing_audit_case = execution.v240_readback_audit.missing_audit.001
missing_provenance_case = execution.v240_readback_audit.missing_provenance.001
stale_source_case = execution.v240_readback_audit.stale_source.001
redaction_breach_case = execution.v240_readback_audit.redaction_breach.001
cross_scope_mismatch_case = execution.v240_readback_audit.cross_scope_mismatch.001
degraded_unavailable_case = execution.v240_readback_audit.degraded_unavailable.001
```

## Stable Codes

```text
v240_readback_audit_ready
v240_readback_missing
v240_audit_missing
v240_provenance_missing
v240_source_stale
v240_redaction_breach
v240_scope_mismatch
v240_readback_degraded_unavailable
```

## Read-Only Evidence Boundary

```text
dashboard_readonly_evidence = true
redacted_audit_only = true
dashboard_can_consume_redacted_audit = true
exchange_truth_claimed = false
network_attempted = false
execution_adapter_call_allowed = false
live_exchange_request_allowed = false
production_order_mutation_allowed = false
new_submit_capability = false
real_order_state_read_expanded = false
dashboard_operation_controls_enabled = false
signed_request_present = false
secret_material_present = false
raw_readback_body_present = false
```

## Validation

Use:

```bash
scripts/ai/verify_release.sh v24-readback-audit-evidence
```

The gate validates the v24 retry policy prerequisite, the generic golden trace
envelope, release replay scope registration, all readback/audit closeout
states, fail-closed evidence for missing/stale/unredacted inputs, and a
negative selftest that rejects a redaction breach incorrectly marked ready.
