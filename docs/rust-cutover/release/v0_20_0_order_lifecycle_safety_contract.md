# NTPRO v0.20.0 Production Order Lifecycle Safety Contract

Date: 2026-06-29
Executor: Codex
Milestone: `v0.20.0`
Task: `V200-001`
Status: PLANNED CONTRACT

## Summary

`v0.20.0` defines a safety contract for an owner-approved production order
lifecycle foundation. The contract fixes the lifecycle states, transition
rules, immutable fields, evidence fields, and failure semantics before later
V200 tasks implement risk gates, owner approval, signing gates, submit
candidates, readback, cancel follow-up, audit, fixtures, or release gates.

Plain Chinese summary: 这份文档先把 v0.20 的生产订单生命周期“安全协议”写死，不写代码。
大白话：一笔订单必须先是 draft，再过风控、人工批准、签名材料 env gate，之后才允许后续任务
构造单次 submit candidate。任何失败都必须写 evidence，不能静默失败；不能自动 retry、不能批量、
不能策略自动下单，也不能让 Dashboard 出现下单或批准按钮。

## Contract Identity

```text
contract_id = ntpro.v200_order_lifecycle_safety_contract.v1
capability = Owner-Approved Production Order Lifecycle Foundation
scope_source = docs/rust-cutover/scope/v0_20_0_owner_approved_production_order_lifecycle_foundation.md
status = planned_contract
runtime_behavior_changed = false
```

## Lifecycle States

| State | Meaning | Required entry evidence | Allowed exit |
| --- | --- | --- | --- |
| `draft` | Non-executable lifecycle candidate. | Candidate metadata only; no approval, signing, or send. | `risk_checked`, `rejected`, `audit_closed` |
| `risk_checked` | Pre-submit risk gate accepted the immutable order intent. | Risk gate artifact, order intent hash, scope boundary check. | `owner_approved`, `rejected`, `audit_closed` |
| `owner_approved` | Manual owner approval exists and is unexpired, single-use, and matched to the risk-checked intent. | Owner approval artifact, expiry, approval hash, owner identity label. | `signing_material_ready`, `approval_expired`, `approval_rejected`, `audit_closed` |
| `signing_material_ready` | Env-only signing material gates are present for the approved intent. | Env gate names, credential-source labels only, no secret values. | `submit_candidate_built`, `rejected`, `audit_closed` |
| `submit_candidate_built` | Single-shot submit request candidate exists with redacted request evidence. | Request builder artifact, redacted request summary, adapter boundary. | `submit_attempted`, `rejected`, `audit_closed` |
| `submit_attempted` | One guarded production submit attempt was attempted or recorded. | Attempt artifact, approval consumed before attempt, send count. | `readback_verified`, `submit_rejected`, `readback_mismatch`, `audit_closed` |
| `readback_verified` | Post-submit readback matched the submitted order identity and expected state. | Readback artifact, redacted order identifiers, match result. | `cancel_requested`, `audit_closed` |
| `cancel_requested` | Owner-approved follow-up cancel was requested through the scoped cancel boundary. | Cancel approval/ref, v19 actual-cancel or V200 cancel evidence, no automatic cancel. | `cancel_verified`, `cancel_failed`, `audit_closed` |
| `cancel_verified` | Cancel readback verified terminal cancellation state. | Cancel readback artifact, redacted order identifiers, match result. | `audit_closed` |
| `submit_rejected` | Submit path rejected before send or exchange rejection was recorded after one attempt. | Failure evidence, no retry marker, consumed approval state when applicable. | `audit_closed` |
| `approval_expired` | Owner approval expired before signing or submit attempt. | Expiry evidence, no send marker. | `audit_closed` |
| `approval_rejected` | Owner approval is missing, rejected, mismatched, or reused. | Approval failure evidence, no send marker. | `audit_closed` |
| `readback_mismatch` | Readback did not match the submit artifact or required state. | Mismatch evidence, no retry marker. | `audit_closed` |
| `cancel_failed` | Cancel follow-up failed or readback did not verify cancellation. | Failure evidence, no retry marker. | `audit_closed` |
| `rejected` | Generic pre-submit rejection before any production send. | Rejection reason and gate evidence. | `audit_closed` |
| `audit_closed` | Terminal evidence package is complete. | Audit artifact, validation summary, rollback/follow-up status. | none |

`audit_closed` is terminal. A new production lifecycle attempt must create a new
`lifecycle_id`; it must not mutate or reuse a closed lifecycle.

## Allowed Transitions

```text
draft -> risk_checked
draft -> rejected
draft -> audit_closed
risk_checked -> owner_approved
risk_checked -> rejected
risk_checked -> audit_closed
owner_approved -> signing_material_ready
owner_approved -> approval_expired
owner_approved -> approval_rejected
owner_approved -> audit_closed
signing_material_ready -> submit_candidate_built
signing_material_ready -> rejected
signing_material_ready -> audit_closed
submit_candidate_built -> submit_attempted
submit_candidate_built -> rejected
submit_candidate_built -> audit_closed
submit_attempted -> readback_verified
submit_attempted -> submit_rejected
submit_attempted -> readback_mismatch
submit_attempted -> audit_closed
readback_verified -> cancel_requested
readback_verified -> audit_closed
cancel_requested -> cancel_verified
cancel_requested -> cancel_failed
cancel_requested -> audit_closed
cancel_verified -> audit_closed
submit_rejected -> audit_closed
approval_expired -> audit_closed
approval_rejected -> audit_closed
readback_mismatch -> audit_closed
cancel_failed -> audit_closed
rejected -> audit_closed
```

## Forbidden Transitions

```text
draft -> owner_approved
draft -> submit_candidate_built
draft -> submit_attempted
risk_checked -> submit_candidate_built
risk_checked -> submit_attempted
owner_approved -> submit_attempted
signing_material_ready -> submit_attempted
submit_attempted -> submit_attempted
submit_rejected -> submit_attempted
readback_mismatch -> submit_attempted
cancel_failed -> cancel_requested
audit_closed -> any state
any state -> retry
any state -> replace
any state -> amend
any state -> flatten
any state -> automatic_remediation
any state -> dashboard_execution
```

## Immutable Fields

These fields are immutable once the lifecycle reaches `risk_checked`:

```text
lifecycle_id
run_id
venue
account_label
symbol
side
order_type
time_in_force
quantity_or_notional_cap
owner_approval_id
risk_gate_id
adapter_boundary_id
expected_order_intent_hash
manual_online_scope
```

Later states may add evidence references, redacted identifiers, readback
results, cancel results, failure reasons, and audit closeout status. They must
not rewrite the immutable order intent.

## Required Event Evidence Fields

Every lifecycle transition event must include:

```text
schema_version = ntpro.v200_order_lifecycle_event.v1
contract_id = ntpro.v200_order_lifecycle_safety_contract.v1
lifecycle_id
transition_id
previous_state
next_state
reason
created_at
actor = owner | cli | runtime_gate | adapter_boundary | readback | dashboard_readonly
owner_approval_required = true
pre_submit_risk_gate_required = true
manual_online_gate_required = true
single_order_required = true
single_venue_required = true
single_account_required = true
single_attempt_required = true
readback_required = true
audit_artifact_required = true
retry_attempted = false
replace_attempted = false
amend_attempted = false
flatten_attempted = false
automatic_order_placement_allowed = false
automatic_remediation_allowed = false
bulk_order_allowed = false
dashboard_order_controls_enabled = false
dashboard_approval_controls_enabled = false
raw_secret_persistence_allowed = false
raw_exchange_response_persistence_allowed = false
```

## Submit Boundary

`submit_attempted` is allowed only when all of these are true:

```text
previous_state = submit_candidate_built
owner_approval_state = active
owner_approval_consumed_before_attempt = true
pre_submit_risk_gate_state = passed
signing_material_gate_state = ready
adapter_boundary_state = ready
order_type = LIMIT unless later explicit scope decision approves otherwise
submit_attempt_count = 1
production_order_submission_allowed = owner_approved_single_shot_only
```

The artifact must record:

```text
production_order_submissions_attempted = 0 or 1
production_orders_submitted = 0 or 1
approval_consumed_before_send = true when request_sent = true
approval_execution_authorized_after_attempt = false
readback_required = true after any request_sent = true
```

## Readback Boundary

Readback must never silently succeed. A readback result must classify as:

```text
readback_verified
readback_mismatch
readback_unavailable
readback_timeout
readback_redacted_id_mismatch
```

Any non-verified readback goes to `readback_mismatch` or `audit_closed` with
failure evidence. It must not trigger retry, replace, amend, flatten, or
automatic remediation.

## Cancel Boundary

`cancel_requested` is optional and must be owner-approved. It may use the
existing v19 actual-cancel boundary or later V200 cancel evidence, but it must
preserve:

```text
automatic_cancel_allowed = false
bulk_cancel_allowed = false
second_cancel_allowed = false
retry_attempted = false
dashboard_cancel_controls_enabled = false
cancel_attempt_count <= 1
```

Cancel failure must transition to `cancel_failed` or `audit_closed` with
failure evidence. It must not silently retry or create a second cancel.

## Failure Semantics

Every failed transition must produce a failure evidence event with:

```text
failure_kind =
  risk_rejected
  approval_missing
  approval_expired
  approval_reused
  approval_mismatch
  signing_material_gate_missing
  adapter_boundary_rejected
  submit_rejected
  readback_mismatch
  readback_timeout
  cancel_rejected
  cancel_timeout
  cancel_readback_mismatch
  audit_closeout_failed
state_after_failure
request_sent = true or false
retry_attempted = false
automatic_remediation_allowed = false
operator_follow_up_required = true or false
```

Silent failure is forbidden. If a command cannot create evidence, it must fail
closed and return a non-zero status in the implementation task that owns the
command.

## Dashboard Boundary

Dashboard may consume lifecycle evidence in read-only mode only:

```text
dashboard_lifecycle_audit_view_allowed = true
dashboard_order_button_allowed = false
dashboard_approval_button_allowed = false
dashboard_cancel_button_allowed = false
dashboard_credential_input_allowed = false
dashboard_execution_allowed = false
```

## Normal And Boundary Coverage

Later V200 tests or fixtures must cover:

```text
normal path: draft -> risk_checked -> owner_approved -> signing_material_ready -> submit_candidate_built -> submit_attempted -> readback_verified -> audit_closed
cancel path: readback_verified -> cancel_requested -> cancel_verified -> audit_closed
risk rejection: draft -> risk_checked -> rejected -> audit_closed
approval expired: owner_approved -> approval_expired -> audit_closed
approval reused or mismatched: owner_approved -> approval_rejected -> audit_closed
readback mismatch: submit_attempted -> readback_mismatch -> audit_closed
cancel failure: cancel_requested -> cancel_failed -> audit_closed
forbidden retry: any failure state must keep retry_attempted = false
```

## Non-Goals

This contract does not implement state machine code, adapter calls, signing,
HTTP sends, order submission, readback, cancel, Dashboard UI, release gates, or
golden traces. It is the acceptance entry contract for V200-002 through
V200-012.
