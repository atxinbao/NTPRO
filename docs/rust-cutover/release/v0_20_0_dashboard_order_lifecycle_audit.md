# NTPRO v0.20.0 Dashboard Order Lifecycle Audit Read-Only View

Date: 2026-06-29
Executor: Codex
Milestone: `v0.20.0`
Task: `V200-010`
Status: IMPLEMENTED LOCAL DASHBOARD READ-ONLY AUDIT VIEW

## Summary

V200-010 adds a read-only Dashboard view for the v0.20 production order
lifecycle evidence chain. The view consumes local evidence files for guarded
submit candidate, redacted submit response, post-submit readback
reconciliation, failure/no-retry evidence, and audit closeout. It displays the
submit, readback, failure, and audit state without adding Dashboard execution
routes or controls.

Plain Chinese summary: 这次实现 Dashboard 订单生命周期审计只读视图。它只读取
本地 evidence/read model，展示 submit、response、readback、failure、audit
closeout 和只读边界。页面不会出现下单、审批、撤单、重试、改单、补单、平仓按钮，
也不会调用任何生产执行接口。unknown、missing、mismatch 会显示为风险可见，不会
被当成成功。

## Dashboard Source Artifacts

```text
v0_20/guarded_submit_candidate.json
v0_20/submit_response_redaction.json
v0_20/submit_readback_reconciliation.json
v0_20/failure_no_retry_evidence.json
v0_20/order_lifecycle_audit_closeout.json
```

## Required Schemas

```text
ntpro.v200_guarded_single_shot_submit_candidate.v1
ntpro.v200_submit_response_redaction.v1
ntpro.v200_submit_readback_reconciliation.v1
ntpro.v200_failure_no_retry_evidence.v1
ntpro.v200_order_lifecycle_audit_closeout.v1
```

## Displayed State

The Dashboard snapshot adds `production_order_lifecycle_audit` entries with:

```text
node_id
health
readiness_status
audit_state
risk_visibility
diagnostic
missing_artifacts
schema_diagnostics
provenance_diagnostics
stale_artifacts
lifecycle_id
attempt_id
submit_attempt_state
owner_approval_state_before_attempt
owner_approval_state_after_attempt
production_submit_attempted
readback_required
response_state
venue_status
venue_order_id
client_order_id
readback_state
mismatch_fields
readback_consistent
readback_missing
readback_failed
failure_category
next_allowed_action
no_implicit_retry
unknown_state_visible
audit_closed
dashboard_audit_consumable
release_gate_consumable
read-only boundary flags
artifact paths
```

## Readiness Semantics

```text
production_order_lifecycle_audit_audit_closed = all required artifacts present, schema/provenance/stale checks clean, submit/readback/audit evidence complete, no risk visible
production_order_lifecycle_audit_risk_visible = evidence complete, but unknown/missing/mismatch/cancel/audit risk remains visible
production_order_lifecycle_audit_incomplete = required artifact or readiness evidence is missing
production_order_lifecycle_audit_boundary_violation = schema/provenance/stale or forbidden control/action evidence is present
```

## Forbidden Dashboard Behavior

The Dashboard view must not add or expose:

```text
submit/order execution buttons
owner approval buttons
retry/replace/amend/flatten buttons
cancel buttons
automatic cancel/remediation controls
raw response or credential display
production order execution routes
approval execution routes
cancel execution routes
```

## Coverage

The target test filter `production_order_lifecycle_audit` covers:

```text
complete artifacts populate the read-only Dashboard view
readback mismatch is risk_visible, not successful
unknown response is risk_visible, not successful
forbidden Dashboard/action flags degrade to boundary_violation
missing evidence never appears healthy
renderer contains no execution routes, action attributes, fetch calls, or buttons
```

## Non-Goals

V200-010 does not implement production submit, owner approval execution,
cancel execution, retry, replacement, amendment, flattening, automatic
remediation, golden traces, or release gates. It is a Dashboard read-only
consumer for evidence produced by V200-006 through V200-009 and a future audit
closeout producer.
