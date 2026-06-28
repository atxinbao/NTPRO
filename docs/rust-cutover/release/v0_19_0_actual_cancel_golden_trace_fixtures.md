# v0.19.0 Actual Cancel Golden Trace Fixtures

Date: 2026-06-28
Executor: Codex
Milestone: `v0.19.0`
Task: `V190-009`
Status: REVIEW_REQUIRED IMPLEMENTATION CONTRACT

## Summary

This document defines the v0.19.0 golden trace fixture coverage for the
owner-approved single-shot actual cancel line. The trace is local/offline and
covers success, blocked pre-send, rejected, timeout, unknown,
already-cancelled, and partial-fill outcomes without live venue credentials or
broker connectivity.

Plain Chinese summary: 这次补的是 v0.19 真实撤单的 golden trace 和 fixture 覆盖。
大白话：成功、审批缺失、审批复用、risk 不匹配、adapter 不支持、被拒绝、超时、unknown、
already cancelled、partial fill 都有固定 trace。trace 只使用本地红acted references，
不连真实交易所，也不新增 retry、二次撤单、remediation 或 Dashboard 撤单按钮。

## Trace Contract

```text
trace = tests/golden/actual_cancel_schema.jsonl
schema_version = golden-trace-v1
category = execution
fixture_contract = ntpro.v190_actual_cancel_golden_fixture.v1
cases = 10
```

Every case records these references:

```text
request_ref
response_ref
readback_ref
audit_ref
provenance_ref
```

Blocked pre-send cases use explicit `not-sent`, `not-applicable`, or
`not-required` reference names so release gates can still prove that no
request, response, or readback evidence is missing accidentally.

## Covered Outcomes

```text
success -> cancel_confirmed
approval_missing -> blocked owner approval
approval_reused -> blocked single-use approval
risk_mismatch -> blocked risk/order lineage binding
adapter_unsupported -> blocked adapter actual-cancel capability
cancel_rejected -> rejected
timeout -> timeout
unknown -> unknown
already_cancelled -> recovered already-cancelled terminal state
partial_fill -> partial-success residual risk
```

`partial_fill` keeps `orig_qty`, `executed_qty`, and `remaining_qty` as decimal
strings. No quantity or price arithmetic path is changed by this task.

## Rust Harness

```text
cargo test -p nautilus-cli --test golden_trace_actual_cancel
```

The harness fails with the scenario name when:

```text
required scenario is missing
outcome or status changes
request/response/readback/audit/provenance ref is missing
input refs and expected refs diverge
retry/second-cancel/remediation/Dashboard cancel flags become true
partial-fill quantity fields stop being decimal strings
raw order ids, client order ids, API key markers, signatures, signed queries,
or signed URLs appear in the trace
```

## Standard Gate Integration

`scripts/ai/run_golden_traces.sh` now runs:

```text
cargo test -p nautilus-cli --test golden_trace_actual_cancel
```

The release replay scope manifest records all 10 V190-009 cases as
`executable_replay`. The only schema-only scoped row remains
`market_data.schema_smoke.001`.

## Boundary

This task does not implement:

```text
new production venue credentials
live broker connection
production order submit lifecycle expansion
actual cancel runtime behavior change
automatic retry
automatic remediation
second cancel
compensation trade
Dashboard cancel or approval controls
raw request/response/readback persistence
credential persistence
```

## Validation

```text
python3 scripts/ai/golden_trace_runner.py tests/golden/actual_cancel_schema.jsonl --mode validate-only
cargo test -p nautilus-cli --test golden_trace_actual_cancel
scripts/ai/verify_v19_actual_cancel_golden_traces.sh
scripts/ai/run_golden_traces.sh
python3 scripts/ai/validate_golden_trace_release_scope.py --manifest docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json --trace-glob 'tests/golden/*.jsonl'
scripts/ai/verify_release.sh v19-release-gates
git diff --check
```

`v19-release-gates` is intentionally unavailable until V190-010 wires the
release gate.
