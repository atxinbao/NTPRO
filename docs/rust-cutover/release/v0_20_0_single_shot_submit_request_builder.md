# NTPRO v0.20.0 Single-Shot Submit Request Builder

Date: 2026-06-29
Executor: Codex
Milestone: `v0.20.0`
Task: `V200-005`
Status: IMPLEMENTED LOCAL BUILDER

## Summary

V200-005 adds a deterministic Rust single-shot production submit request
builder in `crates/risk/src/v20_submit_request_builder.rs`. The builder only
emits a redacted request preview and request digest when V200-002 pre-submit
risk evidence is `allow`, V200-003 owner approval evidence is active, and
V200-004 signing material evidence is `ready`.

Plain Chinese summary: 这次实现 single-shot production order request builder。
它只从已通过 risk gate、owner approval 和 env gate 的 candidate 生成稳定 digest 和
redacted preview；不发网络请求、不真实提交、不 retry、不自动补全、不执行策略。

## Runtime Entry

```text
crate = nautilus-risk
module = nautilus_risk::v20_submit_request_builder
schema_version = ntpro.v200_single_shot_submit_request_builder.v1
contract_id = ntpro.v200_order_lifecycle_safety_contract.v1
builder = build_single_shot_submit_request(candidate, risk, approval, signing)
digest = submit_request_digest(candidate, risk, approval, signing)
```

## Required Evidence

```text
V200-002 risk decision = allow
V200-002 submit_builder_entry_allowed = true
V200-003 owner approval state = approved
V200-003 submit_consumption_allowed = true
V200-004 signing material decision = ready
V200-004 submit_builder_credential_ready = true
candidate fields match risk and approval evidence
```

## Supported Request Shape

```text
single order only
single account only
single venue only
side = buy | sell
order_type = limit
time_in_force = gtc
quantity > 0
price > 0
notional > 0
```

## Evidence Flags

```text
submit_request_built = true only when request digest and redacted preview are produced
network_attempted = false
production_order_submitted = false
retry_attempted = false
automatic_remediation_allowed = false
raw_secret_persisted = false
raw_signed_payload_persisted = false
dashboard_order_controls_enabled = false
```

## Stable Codes

```text
v200_submit_request_built
v200_submit_request_missing_risk_allow
v200_submit_request_missing_owner_approval
v200_submit_request_missing_signing_readiness
v200_submit_request_candidate_mismatch
v200_submit_request_unsupported_order_shape
```

## Coverage

The integration test `crates/risk/tests/v20_submit_request_builder.rs` covers:

```text
normal deterministic build
request digest stability
redacted preview serialization
missing risk allow rejection
missing owner approval rejection
missing signing readiness rejection
candidate/evidence mismatch rejection
unsupported order shape rejection
```

## Non-Goals

V200-005 does not submit orders, sign requests, persist raw payloads, call
adapters, retry, auto-complete fields, execute strategies, redact live exchange
responses, read back order state, cancel orders, update Dashboard controls, add
golden traces, or add release gates. Those remain assigned to later V200
issues.
