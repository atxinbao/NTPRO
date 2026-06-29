# NTPRO v0.20.0 Post-submit Readback Reconciliation

Date: 2026-06-29
Executor: Codex
Milestone: `v0.20.0`
Task: `V200-008`
Status: IMPLEMENTED LOCAL READBACK RECONCILIATION

## Summary

V200-008 adds local post-submit readback reconciliation evidence in
`crates/risk/src/v20_submit_readback_reconciliation.rs`. The reconciler
compares a local submit expectation, V200-007 redacted submit response evidence,
and a redacted venue order readback snapshot. It classifies whether the order
is matched, mismatched, missing, ambiguous, failed to read, or blocked before a
valid readback comparison can start.

Plain Chinese summary: 这次实现 production submit 之后的 readback reconciliation。
它把本地 submit expectation、V200-007 的脱敏 response evidence、以及 venue
readback snapshot 对齐，确认订单是否存在、字段是否一致、是否需要进入风险证据
链和后续 cancel/audit。它只产出 readback/audit evidence，不会自动撤单、重试、
改单、补单、平仓，也不会打开 Dashboard order controls。

## Runtime Entry

```text
crate = nautilus-risk
module = nautilus_risk::v20_submit_readback_reconciliation
schema_version = ntpro.v200_submit_readback_reconciliation.v1
contract_id = ntpro.v200_order_lifecycle_safety_contract.v1
entry = reconcile_post_submit_readback(expectation, response, readback)
```

## Required Input

```text
V200-007 redacted_evidence_ready = true
V200-007 response_digest = present
expectation.lifecycle_id = response.lifecycle_id
expectation.attempt_id = response.attempt_id
expectation.request_digest = response.request_digest
readback.readback_id = present
```

## Compared Fields

```text
account_label
instrument_id
venue
side
quantity
price
client_order_id
venue_order_id
venue_status
venue_timestamp_unix_ms
```

## Supported States

```text
matched = venue readback exists and all compared fields match local evidence
mismatched = venue readback exists but one or more compared fields differ
missing = venue readback completed and did not find the submitted order
ambiguous = venue readback returned an ambiguous order state
readback_failed = venue readback failed before a comparable snapshot was available
blocked = response evidence, lineage, or readback id was invalid
```

## Stable Codes

```text
v200_submit_readback_matched
v200_submit_readback_mismatched
v200_submit_readback_missing
v200_submit_readback_ambiguous
v200_submit_readback_failed
v200_submit_readback_missing_response_evidence
v200_submit_readback_lineage_mismatch
v200_submit_readback_id_missing
```

## Risk Evidence and Audit Output

```text
risk_evidence_required = true for mismatched, missing, ambiguous, readback_failed
cancel_or_audit_input_ready = true for non-blocked states
dashboard_read_only_consumable = true for non-blocked states
raw_readback_body_recorded = false
response_headers_recorded = false
automatic_cancel_attempted = false
automatic_remediation_allowed = false
retry_attempted = false
replace_attempted = false
amend_attempted = false
flatten_attempted = false
dashboard_order_controls_enabled = false
```

## Coverage

The integration test
`crates/risk/tests/v20_submit_readback_reconciliation.rs` covers:

```text
matched readback for read-only audit
mismatched quantity and venue status fields
missing order readback
ambiguous readback
venue read failure
lineage mismatch blocked before readback evidence is accepted
```

## Non-Goals

V200-008 does not query a live adapter, perform network I/O, submit orders,
cancel orders, infer success from a submit response alone, retry, replace,
amend, flatten, store raw readback bodies, persist headers, expose Dashboard
order controls, add golden traces, or add release gates. Those remain assigned
to later V200 issues.
