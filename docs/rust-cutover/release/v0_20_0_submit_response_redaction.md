# NTPRO v0.20.0 Production Submit Response Redaction

Date: 2026-06-29
Executor: Codex
Milestone: `v0.20.0`
Task: `V200-007`
Status: IMPLEMENTED LOCAL RESPONSE REDACTION

## Summary

V200-007 adds production submit response redaction evidence in
`crates/risk/src/v20_submit_response_redaction.rs`. The redactor consumes
V200-006 submitted attempt evidence and a structured venue response summary,
then emits whitelist-only accepted, rejected, unknown, malformed, or blocked
evidence.

Plain Chinese summary: 这次实现 production submit response 的脱敏证据。它只保留
后续 readback/audit 需要的 order id、client order id、venue status、timestamp、
request digest 和 response digest；不会保存 raw response、header、credential、
signature、token、signed query 或 signed URL。redacted evidence 不是 readback 成功证明。

## Runtime Entry

```text
crate = nautilus-risk
module = nautilus_risk::v20_submit_response_redaction
schema_version = ntpro.v200_submit_response_redaction.v1
contract_id = ntpro.v200_order_lifecycle_safety_contract.v1
entry = redact_production_submit_response(request, submit_attempt)
```

## Required Input

```text
V200-006 submit attempt state = submitted
V200-006 production_submit_attempted = true
V200-006 submit_attempt_evidence_ready = true
request.lifecycle_id = submit_attempt.lifecycle_id
request.attempt_id = submit_attempt.attempt_id
request.request_digest = submit_attempt.request_digest
evidence_source = manual_structured | adapter_snapshot
source_provenance_id = present
manual_structured must not claim exchange truth
adapter_snapshot claiming exchange truth must be adapter_runtime_integrated
```

## Source and Provenance Labels

V201-005 adds explicit source labels so redacted response evidence cannot be
mistaken for adapter-integrated runtime truth.

```text
manual_structured = local structured evidence for the v0.20 foundation only
adapter_snapshot = adapter-sourced structured response snapshot with provenance
exchange_readback = not valid for submit response redaction
unknown = blocked
source_provenance_id = required
exchange_truth_claimed = false for manual_structured
adapter_runtime_integrated = false for v0.20 foundation evidence
foundation_only = true when adapter_runtime_integrated is false
```

## Retained Fields

```text
response_id
lifecycle_id
attempt_id
venue
http_status
venue_status
order_id
client_order_id
venue_timestamp_unix_ms
received_at_unix_ns
request_digest
response_digest
reject_code
reject_reason_code
malformed_reason_code
```

## Forbidden Evidence

```text
raw_exchange_response_recorded = false
response_headers_recorded = false
unrestricted_payload_recorded = false
credential_material_recorded = false
signature_material_recorded = false
token_value_recorded = false
signed_query_recorded = false
signed_url_recorded = false
dashboard_raw_response_enabled = false
dashboard_order_controls_enabled = false
readback_success_inferred = false
```

## Supported States

```text
accepted = venue accepted or acknowledged the submit response shape
rejected = venue rejected the submit response shape with stable reason codes
unknown = response was received but final venue state is ambiguous
malformed = response was incompatible and only diagnostic code was retained
blocked = submit attempt, lineage, request digest, response id, or sensitive-material check failed
```

## Stable Codes

```text
v200_submit_response_accepted
v200_submit_response_rejected
v200_submit_response_unknown
v200_submit_response_malformed
v200_submit_response_missing_submit_attempt
v200_submit_response_lifecycle_mismatch
v200_submit_response_request_digest_missing
v200_submit_response_request_digest_mismatch
v200_submit_response_id_missing
v200_submit_response_unknown_source
v200_submit_response_source_provenance_missing
v200_submit_response_source_claim_mismatch
v200_submit_response_sensitive_material_observed
```

## Coverage

The integration test `crates/risk/tests/v20_submit_response_redaction.rs`
covers:

```text
accepted response redaction and readback correlation fields
rejected response redaction with stable reason codes
unknown response with manual review required
malformed response with diagnostic-only evidence
request digest mismatch blocked
unknown source blocked
manual structured evidence claimed as exchange truth blocked
adapter response source missing provenance blocked
sensitive material observed without leaking marker text
missing submitted attempt evidence blocked
```

## Non-Goals

V200-007 does not perform readback, infer order success, call adapters, parse
raw exchange payloads, store raw response bodies, persist headers, expose
Dashboard raw response controls, retry, replace, amend, flatten, cancel, add
golden traces, or add release gates. Those remain assigned to later V200
issues. V201-005 hardening keeps this as foundation-only evidence unless an
explicit adapter source and provenance label says otherwise.
