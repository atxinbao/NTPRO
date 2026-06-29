# NTPRO v0.20.0 Order Lifecycle Golden Traces

Date: 2026-06-29
Executor: Codex
Milestone: `v0.20.0`
Task: `V200-011`
Status: IMPLEMENTED LOCAL GOLDEN TRACE COVERAGE

## Summary

V200-011 adds executable golden traces for the v0.20 production order
lifecycle. The trace set covers pre-submit blocking, accepted submit with
matched readback and closed audit, venue rejection, unknown response, readback
mismatch, and readback missing. Every case carries traceable candidate,
response, readback, failure, audit, Dashboard, and provenance references.

Plain Chinese summary: 这次为 v0.20 production order lifecycle 增加可执行
golden trace。覆盖 submit 前阻断、accepted + readback matched + audit closed、
venue rejected、unknown response、readback mismatch、readback missing。所有失败
路径都明确写 failure evidence 并停止，不会隐式二次提交、重试、改单、补单、平仓或
自动撤单。fixture 只包含脱敏引用和状态，不包含真实 credential 明文。

## Golden Trace File

```text
tests/golden/production_order_lifecycle_schema.jsonl
```

## Executable Harness

```text
crates/cli/tests/golden_trace_production_order_lifecycle.rs
scripts/ai/verify_v20_order_lifecycle_golden_traces.sh
scripts/ai/run_golden_traces.sh
docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json
```

## Covered Scenarios

```text
pre_submit_blocked_missing_approval
accepted_readback_matched_audit_closed
venue_rejected_failure_no_retry
unknown_response_failure_no_retry
readback_mismatch_failure_no_retry
readback_missing_failure_no_retry
```

## Required Outcome Coverage

```text
response_state = accepted, rejected, unknown, not_attempted
readback_state = matched, mismatch, missing, unknown, not_required
failure_category = none, approval_failed, venue_rejected, response_unknown, readback_mismatch, readback_missing
audit_state = audit_closed, audit_risk_visible
risk_visibility = clear, risk_visible
```

## No-Retry and Read-Only Boundary

Every V200-011 fixture keeps these boundaries explicit:

```text
no_implicit_retry = true
retry_allowed = false
retry_attempts = 0
retry_attempted = false
duplicate_submit_attempted = false
second_submit_attempted = false
replace_attempted = false
amend_attempted = false
flatten_attempted = false
automatic_cancel_attempted = false
automatic_remediation_allowed = false
dashboard_order_controls_enabled = false
dashboard_approval_controls_enabled = false
dashboard_cancel_controls_enabled = false
dashboard_retry_controls_enabled = false
```

## Credential and Raw Payload Boundary

The fixture stores no real credential plaintext and no raw venue payload:

```text
credential_plaintext_recorded = false
raw_response_recorded = false
raw_readback_body_recorded = false
credential_material_recorded = false
signature_material_recorded = false
token_value_recorded = false
signed_query_recorded = false
signed_url_recorded = false
network_replay_required = false
live_broker_required = false
```

## Validation Commands

```text
cargo fmt -p nautilus-cli = PASS
python3 scripts/ai/golden_trace_runner.py tests/golden/production_order_lifecycle_schema.jsonl --mode validate-only = PASS, 6 rows
cargo test -p nautilus-cli --test golden_trace_production_order_lifecycle = PASS, 1 passed
scripts/ai/verify_v20_order_lifecycle_golden_traces.sh = PASS
python3 scripts/ai/validate_golden_trace_release_scope.py --manifest docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json --trace-glob 'tests/golden/*.jsonl' = PASS, 51 cases
cargo clippy -p nautilus-cli --test golden_trace_production_order_lifecycle -- -D warnings = PASS
scripts/ai/run_golden_traces.sh = PASS
git diff --check = PASS
scripts/ai/verify_fast.sh = PASS
```

## Non-Goals

V200-011 does not add production submit execution, adapter network calls,
credential storage, Dashboard order controls, retry, replacement, amendment,
flattening, automatic cancel, automatic remediation, or a release publication.
It only adds executable local golden traces and fixture coverage for the
already-established V200-002 through V200-010 evidence chain.
