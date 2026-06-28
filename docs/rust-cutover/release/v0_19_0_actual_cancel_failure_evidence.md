# v0.19.0 Actual Cancel Failure And Partial-Success Evidence

Date: 2026-06-28
Executor: Codex
Milestone: `v0.19.0`
Task: `V190-007`
Status: REVIEW_REQUIRED IMPLEMENTATION CONTRACT

## Summary

This document defines the v0.19.0 failure and partial-success evidence model
for the owner-approved single-shot actual cancel line. It consumes V190-006
readback reconciliation plus request, response, readback, and audit references,
then emits a Dashboard/release-gate consumable outcome artifact.

Plain Chinese summary: 这次补上真撤单后的失败和部分成功证据模型。大白话：真实撤单后，
无论结果是被拒绝、超时、未知、部分成交、已撤单、venue 不可用还是 adapter 失败，都必须写成结构化
evidence。unknown 不会被标成 recovered，partial fill 会明确显示 residual risk，命令本身不重试、
不补偿交易、不二次撤单、不联网。

## CLI Surface

```text
nautilus live production-mutation-actual-cancel-failure-evidence \
  --run-id v190-actual-cancel-failure-evidence \
  --readback-reconciliation runs/v190/actual-cancel-readback-reconciliation.json \
  --request-ref runs/v190/actual-cancel-request-ref.json \
  --response-ref runs/v190/actual-cancel-response-ref.json \
  --readback-ref runs/v190/actual-cancel-readback-ref.json \
  --audit-ref runs/v190/actual-cancel-audit-ref.json \
  --expected-order-lineage-id lineage-v160-single-shot \
  --expected-symbol BTCUSDT \
  --expected-account-label prod-account-redacted \
  --venue binance_spot \
  --output runs/v190/actual-cancel-failure-evidence.json \
  --allow-production-mutation-actual-cancel-failure-evidence \
  --confirm-request-ref-recorded \
  --confirm-response-ref-recorded \
  --confirm-readback-ref-recorded \
  --confirm-audit-ref-recorded \
  --confirm-failure-outcomes-classified \
  --confirm-operator-action-model \
  --confirm-unknown-not-recovered \
  --confirm-partial-fill-residual-risk \
  --confirm-dashboard-release-gate-consumable \
  --confirm-no-retry \
  --confirm-no-remediation \
  --confirm-no-compensation-trade \
  --confirm-no-network \
  --confirm-dashboard-order-controls-disabled \
  --confirm-no-secret-persistence
```

## Artifact Contract

```text
schema_version = ntpro.v190_actual_cancel_failure_evidence.v1
artifact_type = actual_cancel_failure_evidence
status =
  ready_actual_cancel_failure_recovered_cancel_confirmed
  ready_actual_cancel_failure_recovered_already_cancelled
  ready_actual_cancel_failure_rejected
  ready_actual_cancel_failure_timeout
  ready_actual_cancel_failure_unknown
  ready_actual_cancel_partial_success_partial_fill
  ready_actual_cancel_partial_success_filled_before_cancel
  ready_actual_cancel_failure_venue_unavailable
  ready_actual_cancel_failure_adapter_failure
  ready_actual_cancel_failure_inconsistent
  blocked_missing_gate
  blocked_source_artifact
  blocked_lineage
cancel_outcome =
  cancel_confirmed
  already_cancelled
  rejected
  timeout
  unknown
  partial_fill
  filled_before_cancel
  venue_unavailable
  adapter_failure
  inconsistent
outcome_category = recovered | failed | partial_success
```

## Required References

```text
readback_reconciliation = V190-006 actual_cancel_readback_reconciliation
request_ref = redacted request reference
response_ref = redacted response reference
readback_ref = redacted readback reference
audit_ref = local audit reference
request_response_readback_audit_refs_recorded = true
```

The evidence blocks when any reference is missing, not JSON, lacks
`artifact_type`/`status`, contains forbidden raw markers, or when the V190-006
reconciliation artifact is not ready/Dashboard consumable.

## Outcome Model

```text
cancel_confirmed = recovered, no operator action required
already_cancelled = recovered idempotent terminal state, no operator action required
rejected = failed, operator reviews rejection and exchange state
timeout = failed, operator confirms exchange state after timeout
unknown = failed, operator confirms unknown exchange state; never recovered
partial_fill = partial_success, residual risk visible and manual review required
filled_before_cancel = partial_success, filled position review required
venue_unavailable = failed, operator confirms venue state before follow-up
adapter_failure = failed, operator reviews adapter failure before follow-up
inconsistent = failed, operator reconciles inconsistent exchange state
```

## Dashboard / Release Gate Fields

```text
evidence_ready
failure_evidence_ready
dashboard_read_only_consumable
release_gate_consumable
operator_action
operator_action_required
recovered
degraded
failed
partial_success
residual_risk_visible
unknown_not_recovered
partial_fill_residual_risk_visible
request_response_readback_audit_refs_recorded
```

## Boundary

This command does not implement:

```text
automatic compensation trade
automatic retry
automatic remediation
automatic second cancel
network readback execution
Dashboard cancel button
Dashboard owner approval button
multi-account aggregation view
raw request/response/readback persistence
credential persistence
```

## Validation

```text
cargo test -p nautilus-cli actual_cancel_failure_evidence --lib
cargo test -p nautilus-cli parses_live_production_mutation_actual_cancel_failure_evidence_options --lib
scripts/ai/verify_v19_actual_cancel_failure_evidence.sh
rg -n "failure evidence|partial-success|partial fill|venue_unavailable|adapter_failure|unknown_not_recovered" crates docs scripts
git diff --check
```
