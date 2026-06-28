# v0.19.0 Post-Cancel Readback Reconciliation

Date: 2026-06-28
Executor: Codex
Milestone: `v0.19.0`
Task: `V190-006`
Status: REVIEW_REQUIRED IMPLEMENTATION CONTRACT

## Summary

This document defines the v0.19.0 post-cancel readback reconciliation contract
for the owner-approved single-shot actual cancel line. After a real cancel
attempt is recorded, the attempt is not considered follow-up complete until a
redacted readback reconciliation artifact exists.

Plain Chinese summary: 这次补上真撤单后的回读对账。大白话：发过一次真实撤单后，不能只看
“请求发出”就说完成，必须再把交易所订单状态、成交状态、剩余数量、残余风险和本地 audit 状态写成
readback reconciliation 证据。unknown、timeout、partial fill、不一致状态都必须明确降级，不能误报已恢复。

## CLI Surface

```text
nautilus live production-mutation-actual-cancel-readback-reconciliation \
  --run-id v190-actual-cancel-readback-reconciliation \
  --actual-cancel-attempt runs/v190/actual-cancel-single-shot.json \
  --readback runs/v190/post-cancel-readback.json \
  --expected-order-lineage-id lineage-v160-single-shot \
  --expected-symbol BTCUSDT \
  --expected-account-label prod-account-redacted \
  --venue binance_spot \
  --output runs/v190/actual-cancel-readback-reconciliation.json \
  --allow-production-mutation-actual-cancel-readback-reconciliation \
  --confirm-actual-cancel-attempt-recorded \
  --confirm-readback-required \
  --confirm-readback-metadata-only \
  --confirm-order-status-reconciled \
  --confirm-execution-fill-status-reconciled \
  --confirm-remaining-quantity-reconciled \
  --confirm-risk-state-recorded \
  --confirm-local-audit-state-recorded \
  --confirm-dashboard-read-only-consumable \
  --confirm-no-raw-readback-persistence \
  --confirm-no-headers-persistence \
  --confirm-no-secret-persistence \
  --confirm-no-retry \
  --confirm-no-remediation \
  --confirm-no-second-cancel \
  --confirm-no-network \
  --confirm-dashboard-order-controls-disabled
```

## Artifact Contract

```text
schema_version = ntpro.v190_actual_cancel_readback_reconciliation.v1
artifact_type = actual_cancel_readback_reconciliation
status =
  ready_actual_cancel_readback_cancel_confirmed
  ready_actual_cancel_readback_already_cancelled
  ready_actual_cancel_readback_filled_before_cancel
  degraded_actual_cancel_readback_unknown
  degraded_actual_cancel_readback_timeout
  degraded_actual_cancel_readback_inconsistent
  blocked_missing_gate
  blocked_source_artifact
  blocked_forbidden_readback_marker
  blocked_readback_lineage
  blocked_unsupported_readback_state
readback_result = cancel_confirmed | already_cancelled | filled_before_cancel | unknown | timeout | inconsistent
dashboard_read_only_consumable = true when reconciliation evidence is valid
actual_cancel_followup_complete = true only for cancel_confirmed or already_cancelled
```

## Required Source

The command consumes a V190-004 actual cancel artifact and requires:

```text
schema_version = ntpro.v190_actual_cancel_single_shot.v1
artifact_type = actual_cancel_single_shot
status = actual_cancel_attempt_recorded
request_sent = true
cancel_attempted = true
cancel_requests_sent = 1
production_order_mutations_attempted = 1
readback_required = true
readback_requirement = post_cancel_readback_required_before_any_retry_or_followup
source issues = empty
missing env vars = empty
```

If the actual cancel artifact is only offline ready/no-send, or lacks the
readback requirement, reconciliation blocks with `blocked_source_artifact`.

## Readback Results

```text
cancel_confirmed = terminal cancelled state, no residual order risk
already_cancelled = terminal already-cancelled evidence, no second cancel
filled_before_cancel = terminal fill evidence, residual position review required
unknown = degraded/error, manual review and new-order block required
timeout = degraded/error, manual review and new-order block required
inconsistent = degraded/error; partial fill maps here with partial_fill_observed=true
```

The artifact aligns these surfaces for Dashboard read-only audit consumption:

```text
venue_state
order_status
execution_fill_status
remaining_quantity_state
residual_risk_state
local_audit_state
partial_fill_observed
manual_review_required
new_orders_blocked
risk_halted
```

## Boundary

This command does not implement:

```text
automatic second cancel
retry / replace / amend / flatten / remediation
position/account unified read model
v0.20 production order lifecycle
Dashboard cancel button
Dashboard owner approval button
network readback execution
raw readback persistence
credential persistence
```

## Validation

```text
cargo test -p nautilus-cli actual_cancel_readback_reconciliation --lib
cargo test -p nautilus-cli parses_live_production_mutation_actual_cancel_readback_reconciliation_options --lib
scripts/ai/verify_v19_post_cancel_readback_reconciliation.sh
rg -n "readback|reconciliation|partial fill|unknown|already cancelled" crates docs scripts
git diff --check
```
