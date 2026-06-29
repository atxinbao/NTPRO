# NTPRO v0.20.0 Owner Approval Lifecycle

Date: 2026-06-29
Executor: Codex
Milestone: `v0.20.0`
Task: `V200-003`
Status: IMPLEMENTED LOCAL LIFECYCLE

## Summary

V200-003 adds a typed Rust owner approval lifecycle in
`crates/risk/src/v20_owner_approval.rs`. The lifecycle binds one owner approval
request to one production submit candidate through deterministic digest,
scope, owner, expiry, nonce, environment, and release provenance fields. It
returns auditable approved, rejected, expired, revoked, or consumed evidence and
can export only active approval evidence into the V200-002 pre-submit risk gate
shape.

Plain Chinese summary: 这次实现 production submit 的 owner approval 生命周期。
approval request 会固定 digest、订单 scope、owner、过期时间、nonce、环境和 release
provenance；只有匹配且未过期、未撤销、未消费的 approval 才能给后续 submit builder
使用。没有 Dashboard approval button，没有真实下单，没有 adapter 调用。

## Runtime Entry

```text
crate = nautilus-risk
module = nautilus_risk::v20_owner_approval
schema_version = ntpro.v200_owner_approval_lifecycle_event.v1
contract_id = ntpro.v200_order_lifecycle_safety_contract.v1
digest = owner_approval_digest(scope, nonce, environment, release_provenance)
evaluation = evaluate_owner_approval(record, candidate, evaluated_at_unix_ns)
consumption = consume_owner_approval(evidence, consumed_at_unix_ns)
pre_submit_bridge = OwnerApprovalEvidence::as_pre_submit_approval()
```

## Required Fields

```text
approval_id
request_id
lifecycle_id
owner_label
account_label
instrument_id
venue
side
quantity
price
notional
order_type
time_in_force
order_intent_hash
nonce
environment
release_tag
release_commit
release_gate
strict_provenance
expires_at_unix_ns
owner decision
revoked_at_unix_ns optional
consumed_at_unix_ns optional
```

## States And Rejection Rules

```text
approved = digest, scope, environment, provenance, owner decision, expiry, revocation, and consumed markers are valid
rejected = owner rejected, digest mismatch, scope mismatch, environment mismatch, provenance mismatch, missing owner, or missing nonce
expired = evaluated after expires_at_unix_ns
revoked = revoked_at_unix_ns is present and effective
consumed = already consumed or consumed by this lifecycle before submit attempt
```

The owner approval evidence always keeps:

```text
single_use = true
approval_reusable = false
approval_execution_authorized_after_attempt = false
dashboard_approval_controls_enabled = false
dashboard_order_controls_enabled = false
retry_attempted = false
automatic_remediation_allowed = false
```

## Stable Codes

```text
v200_owner_approval_allowed
v200_owner_approval_request_digest_mismatch
v200_owner_approval_candidate_digest_mismatch
v200_owner_approval_scope_mismatch
v200_owner_approval_environment_mismatch
v200_owner_approval_release_provenance_mismatch
v200_owner_approval_owner_missing
v200_owner_approval_nonce_missing
v200_owner_approval_rejected
v200_owner_approval_expired
v200_owner_approval_revoked
v200_owner_approval_already_consumed
v200_owner_approval_consumed
v200_owner_approval_consumption_requires_approved_evidence
```

## Coverage

The integration test `crates/risk/tests/v20_owner_approval.rs` covers:

```text
normal approved path
pre-submit gate bridge
single-use consume path
second consume rejection
expired approval rejection
revoked approval rejection
owner rejected decision
already consumed record
request digest mismatch
cross-account/candidate digest mismatch
cross-environment reuse rejection
```

## Non-Goals

V200-003 does not add Dashboard approval controls, persistent approval storage,
signing material gates, submit request construction, adapter submit calls,
response redaction, readback, cancel, golden traces, or release gates. Those
remain assigned to later V200 issues.
