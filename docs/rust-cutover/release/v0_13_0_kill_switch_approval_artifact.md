# NTPRO v0.13.0 Kill-switch Dry-run and Manual Approval Artifact

Date: 2026-06-21
Executor: Codex
Milestone: `v0.13.0`
Status: LOCAL PREFLIGHT CONTRACT

## Summary

`v0.13.0` records kill-switch and manual-approval evidence as local artifacts
only. The artifact is a preflight audit record, not permission for production
order mutation.

Plain Chinese summary: 这份 artifact 是“以后进入 live alpha 前要看的审批和关停
证据”，不是“系统已经能实盘下单”。它会记录 kill switch dry-run、人工批准状态和
no-mutation 字段，但不会联网，不会下单，不会撤单/改单，也不会给 Dashboard 加交易
按钮。

## Command

```bash
nautilus live production-kill-switch-approval-artifact \
  --run-id v130-live-alpha-preflight \
  --session-id v130-live-alpha-session \
  --strategy-id ema_cross_btcusdt_v1 \
  --output kill_switch_approval_artifact.json \
  --kill-switch-active true \
  --approval-state approved \
  --manual-approval-id owner-approval-v130-004 \
  --approved-by owner \
  --confirm-dry-run-only \
  --confirm-no-production-mutation \
  --confirm-dashboard-order-controls-disabled
```

## Artifact Contract

Required markers:

```text
schema_version = ntpro.v130_kill_switch_approval_artifact.v1
artifact_type = kill_switch_dry_run_manual_approval
kill_switch_enabled = true
kill_switch_dry_run = true
manual_approval_required = true
approval_artifact_only = true
owner_approval_required_before_any_mutation = true
production_order_submission_allowed = false
production_order_mutation_allowed = false
production_order_state_reads_allowed = false
listen_key_lifecycle_allowed = false
dashboard_order_controls_enabled = false
network_attempted = false
values_are_exchange_truth = false
```

## Approval Semantics

The artifact supports:

```text
approval_state = pending
approval_state = approved
approval_state = rejected
```

`approval_state=approved` requires `manual_approval_id` and `approved_by`, but
it still does not allow production order mutation in v0.13.0. It only records
that an owner approval artifact exists for later release review.

## Explicit Non-Goals

This artifact does not authorize production order submission, production order
mutation, production order-state reads, listenKey lifecycle, production
WebSocket user streams, automatic remediation, real funds, production trading,
or Dashboard order controls.

## Verification

```text
cargo test -p nautilus-cli production_kill_switch_approval_artifact --lib
scripts/ai/verify_v13_kill_switch_approval_artifact.sh
scripts/ai/verify_fast.sh
git diff --check
```
