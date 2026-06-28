# v0.19.0 Dashboard Actual Cancel Audit View

Date: 2026-06-28
Executor: Codex
Milestone: `v0.19.0`
Task: `V190-008`
Status: REVIEW_REQUIRED IMPLEMENTATION CONTRACT

## Summary

This document defines the v0.19.0 Dashboard read-only audit view for the
owner-approved single-shot actual cancel line. The view consumes existing local
evidence artifacts and displays approval, risk gate, cancel attempt, venue
response, readback, outcome, and audit references without adding any executable
Dashboard operation.

Plain Chinese summary: 这次给 Dashboard 增加真撤单审计的只读面板。页面只读本地证据，
展示审批、风险门禁、撤单尝试、venue 响应、readback、最终结果和 audit references。缺证据、
schema 不匹配、provenance 不匹配、unknown readback 或边界违规都会降级；不会新增撤单按钮、
审批按钮、重试按钮、批量操作或 Dashboard 写操作。

## Consumed Artifacts

```text
v0_18/cancel_risk_gate.json
v0_19/actual_cancel_owner_approval_lifecycle.json
v0_19/actual_cancel_single_shot.json
v0_19/actual_cancel_readback_reconciliation.json
v0_19/actual_cancel_failure_evidence.json
```

## Snapshot Contract

```text
field = production_actual_cancel_audit
node_id
health
readiness_status
audit_state = ready | recovered | degraded | failed | unknown
diagnostic
missing_artifacts
schema_diagnostics
provenance_diagnostics
stale_artifacts
```

The view is healthy only when all required evidence is present, schemas match,
provenance checks pass, artifacts are not stale, Dashboard/control boundary
flags stay disabled, and the outcome is `ready` or `recovered`.

## Display Contract

```text
approval = owner approval lifecycle status, approval state, lifecycle validity, execution authorization
risk gate = status, result, readiness
cancel attempt = command readiness, single-shot allowance, request sent, cancel attempt, request id
venue response = redacted status/source/code/error metadata
readback = readback result, reconciliation status, readback state, venue state, terminal/unknown indicators
outcome = cancel outcome, outcome category, recovered/degraded/failed/partial-success flags
audit references = request/response/readback/audit refs recorded flag and source/lineage issue summaries
```

## Diagnostics

```text
production_actual_cancel_audit_missing_evidence
production_actual_cancel_audit_schema_mismatch
production_actual_cancel_audit_provenance_mismatch
production_actual_cancel_audit_stale_evidence
production_actual_cancel_audit_unknown_readback
production_actual_cancel_audit_boundary_violation
production_actual_cancel_audit_source_issue_blocked
```

Missing evidence, schema mismatch, provenance mismatch, stale evidence, unknown
readback, and boundary violations must not render as healthy/recovered.

## Forbidden Dashboard Surface

```text
Dashboard cancel button = forbidden
Dashboard owner approval button = forbidden
Dashboard retry button = forbidden
Dashboard bulk action = forbidden
Dashboard order/cancel route = forbidden
Dashboard write operation = forbidden
trader terminal = out of scope
multi-account / multi-strategy aggregation = out of scope
```

The view may display evidence fields such as `cancel_attempted`,
`cancel_requests_sent`, and endpoint-attempt flags from the already-recorded
actual-cancel artifact. Those fields are audit evidence, not controls.

## Validation

```text
cargo test -p nautilus-cli production_actual_cancel_audit --lib
cargo test -p nautilus-cli production_cancel_recovery --lib
scripts/ai/verify_v19_dashboard_actual_cancel_audit_view.sh
rg -n "cancel button|approve button|retry|bulk|actual_cancel|read-only|readonly" crates docs scripts
git diff --check
```

## Rollback

Revert the V190-008 PR. This removes the Dashboard read-only view and its local
tests/scripts/docs without touching actual-cancel command execution, adapter
behavior, release tags, GitHub Releases, credentials, or exchange state.
