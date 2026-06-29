# NTPRO v0.20.0 Failure And No-Retry Evidence Model

Date: 2026-06-29
Executor: Codex
Milestone: `v0.20.0`
Task: `V200-009`
Status: IMPLEMENTED LOCAL FAILURE EVIDENCE MODEL

## Summary

V200-009 adds local failure and no-retry evidence in
`crates/risk/src/v20_failure_no_retry.rs`. The model records terminal failure
evidence for submit-before, submit, response, post-submit readback, cancel
follow-up, and audit completion surfaces. Every supported category emits a
stable code, human-readable reason, source evidence pointer, and next allowed
action while keeping implicit retry and remediation disabled.

Plain Chinese summary: 这次实现 failure / no-retry evidence model。它把
blocked、validation_failed、approval_failed、credential_unavailable、
submit_failed、venue_rejected、response_unknown、readback_missing、
readback_mismatch、cancel_required、audit_incomplete 都记录成稳定 evidence。
失败后只写证据并停止；不会自动重试、撤单、改单、补单、平仓，也不会启用
Dashboard order controls。operator/owner 能看到下一步允许动作，但 v0.20 不会
静默重试或自动补救。

## Runtime Entry

```text
crate = nautilus-risk
module = nautilus_risk::v20_failure_no_retry
schema_version = ntpro.v200_failure_no_retry_evidence.v1
contract_id = ntpro.v200_order_lifecycle_safety_contract.v1
entry = build_failure_no_retry_evidence(request)
```

## Required Input

```text
failure_id = present
lifecycle_id = present
source_evidence_ready = true
source_evidence.source_id = present
source_evidence.schema_version = present
source_evidence.lifecycle_id = request.lifecycle_id
source_evidence.attempt_id = request.attempt_id
reason = human-readable and non-empty
```

## Categories

```text
blocked
validation_failed
approval_failed
credential_unavailable
submit_failed
venue_rejected
response_unknown
readback_missing
readback_mismatch
cancel_required
audit_incomplete
```

## Stable Codes

```text
v200_failure_blocked
v200_failure_validation_failed
v200_failure_approval_failed
v200_failure_credential_unavailable
v200_failure_submit_failed
v200_failure_venue_rejected
v200_failure_response_unknown
v200_failure_readback_missing
v200_failure_readback_mismatch
v200_failure_cancel_required
v200_failure_audit_incomplete
v200_failure_id_missing
v200_failure_lifecycle_id_missing
v200_failure_reason_missing
v200_failure_source_evidence_missing
v200_failure_source_lineage_mismatch
```

## Next Allowed Actions

```text
blocked -> no_action_until_evidence_ready
validation_failed -> fix_input_and_rebuild
approval_failed -> request_owner_approval
credential_unavailable -> provide_signing_material
submit_failed -> write_submit_failure_evidence
venue_rejected -> audit_venue_rejection
response_unknown -> manual_review_unknown_response
readback_missing -> prepare_cancel_or_audit
readback_mismatch -> prepare_cancel_or_audit
cancel_required -> prepare_owner_approved_cancel
audit_incomplete -> complete_audit
```

## Required No-Retry Semantics

```text
terminal_action = write_evidence_and_stop
evidence_written = true for valid source-backed failures
stop_after_evidence = true for valid source-backed failures
dashboard_audit_consumable = true for valid source-backed failures
release_gate_consumable = true for valid source-backed failures
unknown_state_visible = true for blocked, response_unknown, audit_incomplete
no_implicit_retry = true
retry_allowed = false
retry_attempted = false
retry_attempts = 0
max_retry_attempts = 0
replace_attempted = false
amend_attempted = false
flatten_attempted = false
automatic_cancel_attempted = false
automatic_remediation_allowed = false
strategy_continuation_allowed = false
dashboard_order_controls_enabled = false
```

## Coverage

The integration test `crates/risk/tests/v20_failure_no_retry.rs` covers:

```text
all supported failure categories emit stable code and next allowed action
response_unknown and audit_incomplete keep unknown state visible
missing source evidence blocks dashboard/release consumption
source lineage mismatch blocks dashboard/release consumption
missing human-readable reason blocks dashboard/release consumption
every failure keeps retry/replace/amend/flatten/remediation/dashboard controls disabled
```

## Non-Goals

V200-009 does not query a live adapter, submit orders, cancel orders, retry,
replace, amend, flatten, perform automatic remediation, continue a strategy
after failure evidence, add Dashboard order controls, add golden traces, or add
release gates. Those remain assigned to later V200 issues.
