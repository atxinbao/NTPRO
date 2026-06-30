# NTPRO v0.20.0 Pre-Submit Risk Gate

Date: 2026-06-29
Executor: Codex
Milestone: `v0.20.0`
Task: `V200-002`
Status: IMPLEMENTED LOCAL GATE

## Summary

V200-002 adds a typed Rust pre-submit risk gate in
`crates/risk/src/v20_pre_submit_gate.rs`. The gate evaluates one production
order intent before it may enter a later submit builder. It returns auditable
allow, deny, or blocked evidence with stable codes and keeps all production
submit, adapter calls, retry, remediation, and Dashboard controls out of scope.

Plain Chinese summary: 这次实现的是“提交前风控门”。它只判断一笔生产订单意图是否
允许进入后续 submit builder，并输出 evidence；它不真实下单、不连交易所、不自动修复、
不 retry，也不打开 Dashboard 下单按钮。

## Runtime Entry

```text
crate = nautilus-risk
module = nautilus_risk::v20_pre_submit_gate
schema_version = ntpro.v200_pre_submit_risk_gate_decision.v1
contract_id = ntpro.v200_order_lifecycle_safety_contract.v1
function = evaluate_pre_submit_risk_gate(request, policy, evaluated_at_unix_ns)
```

## Required Checks

The gate fails closed before submit-builder entry when any required field,
allowlist entry, limit, approval, environment, or release provenance condition
is missing or invalid.

```text
account_label
instrument_id
venue
side
quantity
price
notional
order_type
time_in_force
environment
order_intent_hash
owner approval id / intent hash / expiry / single-use state
strict release provenance tag / commit / gate
unrecognized fields
```

The default supported production shape is a single allowlisted account, venue,
instrument, side, LIMIT order type, GTC time-in-force, positive quantity,
positive price, capped notional, matching production environment, unexpired
single-use owner approval, and strict v20 runtime release provenance. The
v0.19.1 closeout is prerequisite evidence only; it is not accepted as current
runtime submit provenance.

## Decisions

```text
allow   = all checks pass; submit builder entry evidence may be consumed later
deny    = order intent or approval is invalid; no submit builder entry
blocked = environment or release provenance precondition is incomplete; no submit builder entry
```

All non-allow decisions keep:

```text
production_order_submission_allowed = false
submit_builder_entry_allowed = false
retry_attempted = false
replace_attempted = false
amend_attempted = false
flatten_attempted = false
automatic_remediation_allowed = false
dashboard_order_controls_enabled = false
```

## Stable Codes

The Rust enum `PreSubmitRiskCode` provides stable evidence codes for every
implemented reason. Required V200-002 examples include:

```text
v200_pre_submit_allowed
v200_pre_submit_unknown_field
v200_pre_submit_account_missing
v200_pre_submit_account_unknown
v200_pre_submit_instrument_missing
v200_pre_submit_instrument_unknown
v200_pre_submit_venue_missing
v200_pre_submit_venue_unknown
v200_pre_submit_side_missing
v200_pre_submit_side_unsupported
v200_pre_submit_quantity_missing
v200_pre_submit_quantity_not_positive
v200_pre_submit_quantity_limit_exceeded
v200_pre_submit_price_missing
v200_pre_submit_price_not_positive
v200_pre_submit_price_limit_exceeded
v200_pre_submit_notional_missing
v200_pre_submit_notional_not_positive
v200_pre_submit_notional_limit_exceeded
v200_pre_submit_order_type_missing
v200_pre_submit_order_type_unsupported
v200_pre_submit_time_in_force_missing
v200_pre_submit_time_in_force_unsupported
v200_pre_submit_environment_missing
v200_pre_submit_environment_mismatch
v200_pre_submit_intent_hash_missing
v200_pre_submit_approval_missing
v200_pre_submit_approval_id_missing
v200_pre_submit_approval_expired
v200_pre_submit_approval_intent_mismatch
v200_pre_submit_approval_not_single_use
v200_pre_submit_approval_already_consumed
v200_pre_submit_provenance_missing
v200_pre_submit_provenance_tag_missing
v200_pre_submit_provenance_commit_missing
v200_pre_submit_provenance_gate_mismatch
v200_pre_submit_provenance_not_strict
```

## Coverage

The integration test `crates/risk/tests/v20_pre_submit_gate.rs` covers:

```text
allow path
unknown account denial
missing field denial
notional limit denial
expired approval denial
missing approval denial
environment mismatch blocked
missing release provenance blocked
unrecognized field denial and evidence serialization
unknown JSON shape rejection before evaluation
```

## Non-Goals

V200-002 does not implement owner approval lifecycle storage, signing material
gates, submit request construction, adapter submit calls, response redaction,
post-submit readback, cancel follow-up, Dashboard audit UI, golden traces, or
release gates. Those remain assigned to V200-003 through V200-012.
