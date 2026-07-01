# v0.21.0 Unified Risk State Projection

Date: 2026-07-01
Executor: Codex
Task: `V210-006`
GitHub issue: `#657`
Status: COMPONENT CONTRACT

## Purpose

This document defines the v0.21 unified risk state projection. It aggregates
the account, position, order, and fill read-model component states into a
single read-only risk projection for Dashboard and release-gate evidence.

Plain Chinese summary: 本任务只把 account、position、order、fill 的 read model
证据汇总成统一 risk state。它能展示 healthy、risk_visible、manual_review、
halted、stale、mismatch 等状态和降级原因，但不触发自动交易、不自动修复、不自动
平仓，也不新增 Dashboard 操作控件。

## Contract Surface

```text
component = components.risk
contract_version = ntpro.v210.unified_read_model.v1
component transform = ntpro.v210.risk_state_projection.v1
validator = scripts/ai/verify_v21_risk_state_projection.sh
release target = scripts/ai/verify_release.sh v21-risk-state-projection
golden trace = tests/golden/read_model_risk_state_schema.jsonl
```

## Input Components

The risk projection consumes only read-only component summaries:

```text
account.component_status
positions.component_status
orders.component_status
fills.component_status
freshness.status for every input component
lineage.transform and input_refs for every input component
source_provenance.source_ref for every input component
blocking_reasons from every input component
```

Rules:

- The risk projection must not read raw exchange payloads, credentials,
  headers, signatures, unrestricted adapter payloads, or mutable execution
  objects.
- Missing source provenance, missing lineage, missing freshness, stale
  freshness, or component mismatch prevents `healthy`.
- `audit_closed` is allowed only when every critical input component is fresh,
  linked, redacted, and non-blocking.

## Risk States

Allowed read-only projection states:

```text
healthy
risk_visible
manual_review
halted
stale
mismatch
blocked
```

Priority order:

```text
halted > mismatch > stale > manual_review > risk_visible > healthy
```

Rules:

- `healthy` requires complete evidence and no input blockers.
- `risk_visible` means risk data may be displayed, but the snapshot is not an
  audit-closed healthy state.
- `manual_review` means the evidence is visible but requires human review
  outside the read model.
- `halted`, `stale`, and `mismatch` are fail-closed display states.
- None of these states may trigger submit, cancel, retry, replace, amend,
  flatten, execution algorithm, or automatic repair behavior.

## Fail-Closed Rules

The risk component must be `fail_closed` when any of these are true:

```text
stale_component_freshness
component_lineage_mismatch
missing_component_source_provenance
missing_component_lineage
missing_component_freshness
halted_by_risk_state
critical_evidence_missing
unredacted_risk_payload
```

Fail-closed risk snapshots must keep:

```text
health_status = fail_closed
components.risk.component_status = fail_closed
risk_state != healthy
lifecycle_status != audit_closed
capability_boundary.production_order_submission_allowed = false
capability_boundary.production_order_mutation_allowed = false
capability_boundary.dashboard_order_controls_enabled = false
capability_boundary.dashboard_risk_controls_enabled = false
capability_boundary.automatic_risk_action_allowed = false
capability_boundary.automatic_risk_repair_allowed = false
```

## Dashboard Boundary

Dashboard may display risk state, component rollups, blocking reasons, manual
review flags, and readonly evidence references. Dashboard must not expose risk
override, submit, cancel, retry, replace, amend, flatten, auto-halt, auto-repair,
or execution algorithm controls for this read-model scope.

## Validation

Use:

```bash
scripts/ai/verify_release.sh v21-risk-state-projection
```
