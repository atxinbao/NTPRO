# NTPRO v0.20.0 Guarded Single-Shot Submit Candidate

Date: 2026-06-29
Executor: Codex
Milestone: `v0.20.0`
Task: `V200-006`
Status: IMPLEMENTED LOCAL SUBMIT CANDIDATE GATE

## Summary

V200-006 adds a guarded single-shot submit candidate gate in
`crates/risk/src/v20_submit_candidate.rs`. The gate consumes V200-002 risk
allow evidence, V200-003 active owner approval evidence, V200-004 signing
material readiness evidence, and V200-005 request-builder evidence before it
can record a submit attempt.

Plain Chinese summary: 这次实现 guarded single-shot submit candidate gate。它会
先确认 risk allow、owner approval、env signing readiness、request digest、durable
attempt ledger 和 release provenance 都匹配；preview 和 dry-run 只生成证据，不提交；
真实 submit 模式还必须有 manual online gate，并且会以可审计的原子语义消费 owner
approval，保证一份 approval 和一个 request digest 只对应一次 submit attempt。

## Runtime Entry

```text
crate = nautilus-risk
module = nautilus_risk::v20_submit_candidate
schema_version = ntpro.v200_guarded_single_shot_submit_candidate.v1
contract_id = ntpro.v200_order_lifecycle_safety_contract.v1
entry = evaluate_guarded_single_shot_submit_candidate(request, risk, approval, signing, builder, evaluated_at_unix_ns)
ledger_schema_version = ntpro.v200_submit_attempt_ledger.v1
```

## Required Evidence

```text
V200-002 risk decision = allow
V200-002 production_order_submission_allowed = true
V200-002 submit_builder_entry_allowed = true
V200-002 release_provenance_valid = true
V200-003 owner approval state = approved
V200-003 submit_consumption_allowed = true
V200-004 signing material decision = ready
V200-004 submit_builder_credential_ready = true
V200-005 request builder decision = built
V200-005 request digest = expected request digest
durable attempt ledger = present, trusted, current, same lifecycle, same v20 provenance
durable attempt ledger duplicate request digest = absent
durable attempt ledger consumed approval id = absent
```

## Supported States

```text
blocked = any prerequisite evidence, digest, lifecycle, manual gate, or duplicate check fails
preview = all prerequisite evidence matches, but no submit side effect is recorded
dry_run = all prerequisite evidence matches, but no submit side effect is recorded
submitted = manual online gate is present, digest is unique, and owner approval is consumed
```

## Single-Shot Constraints

```text
single_attempt_required = true
single_order_required = true
single_venue_required = true
single_account_required = true
manual_online_gate_required = true
owner_approval_required = true
request_digest_required = true
release_provenance_required = true
attempt_ledger_required = true
attempt_ledger_trusted = true
atomic_approval_consumption_required = true
duplicate request digest = blocked
approval consumed before submit = blocked
approval consumed in durable ledger = blocked
ledger stale / untrusted / lineage mismatch / provenance mismatch = blocked
```

## Submit Evidence Flags

```text
submit_attempt_evidence_ready = true for preview, dry_run, and submitted evidence
production_submit_attempted = true only for submitted evidence
adapter_submit_handoff_allowed = true only for submitted evidence
readback_required = true only after submitted evidence
audit_artifact_required = true
retry_attempted = false
replace_attempted = false
amend_attempted = false
flatten_attempted = false
bulk_submit_attempted = false
automatic_remediation_allowed = false
dashboard_order_controls_enabled = false
raw_secret_persisted = false
raw_signed_payload_persisted = false
raw_exchange_response_persisted = false
```

## Stable Codes

```text
v200_guarded_submit_preview_ready
v200_guarded_submit_dry_run_ready
v200_guarded_submit_submitted
v200_guarded_submit_candidate_id_missing
v200_guarded_submit_attempt_id_missing
v200_guarded_submit_lifecycle_mismatch
v200_guarded_submit_missing_risk_allow
v200_guarded_submit_missing_owner_approval
v200_guarded_submit_missing_signing_readiness
v200_guarded_submit_missing_request_build
v200_guarded_submit_evidence_mismatch
v200_guarded_submit_missing_release_provenance
v200_guarded_submit_request_digest_missing
v200_guarded_submit_request_digest_mismatch
v200_guarded_submit_attempt_ledger_missing
v200_guarded_submit_attempt_ledger_untrusted
v200_guarded_submit_attempt_ledger_lineage_mismatch
v200_guarded_submit_attempt_ledger_provenance_mismatch
v200_guarded_submit_approval_already_consumed
v200_guarded_submit_manual_gate_missing
v200_guarded_submit_duplicate_rejected
```

## Coverage

The integration test `crates/risk/tests/v20_submit_candidate.rs` covers:

```text
preview evidence without approval consumption
dry-run evidence without submit side effects
single-shot submitted evidence
approval consumed on submitted evidence
missing manual gate blocked
missing risk allow blocked
consumed approval blocked
duplicate request digest rejected
request digest mismatch blocked
missing signing readiness blocked
missing durable ledger blocked
stale durable ledger blocked
ledger lineage mismatch blocked
ledger provenance mismatch blocked
approval already consumed by durable ledger blocked
```

## Non-Goals

V200-006 does not implement adapter networking, response redaction, exchange
response persistence, order readback, reconciliation, failure classification,
Dashboard UI, golden traces, release gates, retry, replace, amend, flatten, or
bulk submit. Those remain assigned to later V200 issues.
