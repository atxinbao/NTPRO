# NTPRO v0.15.0 Manual Approval Lifecycle

Date: 2026-06-22
Executor: Codex
Milestone: `v0.15.0`
Task: `V150-005`
Status: implementation evidence pending PR closeout

## Summary

`v0.15.0` introduces a local manual approval lifecycle artifact for production
live-alpha request preview. This does not enable production trading. It only
lets an owner-approved, scoped, short-lived, one-time approval advance the local
dry-run request preview builder.

Plain Chinese summary: 这不是“可以实盘下单”。它只是要求在生成实盘下单请求预览前，
必须先有一张人工审批单，而且这张审批单只能用一次、很快过期、只能对应一个明确的
run/策略/交易对/金额。

## Product Boundary

Allowed claim:

```text
manual approval lifecycle = local dry-run request preview gate
approval state = pending | approved | expired | revoked | used
approval use = one-time
approval scope = request preview only
production request sent = false
production order mutation = false
Dashboard order controls = disabled
```

Not allowed claim:

```text
approval authorizes production order submission
approval authorizes production cancel/replace/amend
approval authorizes execution adapter calls
approval opens network access
approval enables Dashboard order buttons
approval is a durable server-side approval service
```

## Lifecycle Contract

The approval artifact must bind these fields:

| Field | Requirement |
| --- | --- |
| `run_id` | Must match the request preview run. |
| `strategy_id` | Must match the request preview strategy. |
| `symbol` | Must match the request preview symbol. |
| `notional` | Must match the request preview notional. |
| `expires_at_unix_ms` | Must be present and not expired at lifecycle evaluation time. |
| `approval_state` | Must be `approved` to allow preview creation. |
| `manual_approval_id` | Required for non-pending states. |
| `approved_by` | Required for non-pending states. |

Only this path can continue:

```text
approval_state = approved
approval_lifecycle_valid = true
manual_approval_recorded = true
one_time_approval = true
dry_run_request_preview_only = true
approval_expired = false
approval_revoked = false
approval_used = false
```

All other paths remain blocked:

```text
pending -> blocked_manual_approval_lifecycle
expired -> blocked_manual_approval_lifecycle
revoked -> blocked_manual_approval_lifecycle
used -> blocked_manual_approval_lifecycle
field mismatch -> blocked_manual_approval_lifecycle
forbidden mutation evidence -> blocked_manual_approval_lifecycle
```

## CLI Surface

New lifecycle command:

```text
nautilus live production-live-alpha-manual-approval-lifecycle \
  --run-id v150-request-preview \
  --strategy-id ema_cross_btcusdt_v1 \
  --symbol BTCUSDT \
  --notional 10.00 \
  --approval-state approved \
  --manual-approval-id owner-approval-v150-005 \
  --approved-by owner \
  --now-unix-ms 1718400000000 \
  --expires-at-unix-ms 1718400060000 \
  --output manual-approval-lifecycle.json \
  --confirm-dry-run-request-preview-only \
  --confirm-one-time-approval \
  --confirm-no-production-mutation \
  --confirm-dashboard-order-controls-disabled
```

Request preview now requires the approval lifecycle artifact:

```text
nautilus live production-live-alpha-order-request-preview \
  --run-id v150-request-preview \
  --order-gate live-alpha-order-gate.json \
  --manual-approval-lifecycle manual-approval-lifecycle.json \
  ...
```

## Evidence Invariants

Every lifecycle and request-preview path must preserve:

```text
request_sent = false
network_attempted = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
production_order_submissions_attempted = 0
production_order_mutations_attempted = 0
production_orders_submitted = 0
execution_adapter_called = false
production_adapter_called = false
listen_key_lifecycle_attempted = false
dashboard_order_controls_enabled = false
real_funds = false
production_trading_enabled = false
```

## Verification

Task-level verification:

```text
cargo test -p nautilus-cli production_live_alpha_manual_approval_lifecycle --lib
cargo test -p nautilus-cli production_live_alpha_order_request_preview --lib
scripts/ai/verify_v15_manual_approval_lifecycle.sh
```

Regression verification:

```text
scripts/ai/verify_v15_live_order_request_dry_run_builder.sh
scripts/ai/verify_v15_execution_adapter_isolation.sh
scripts/ai/verify_v15_kill_switch_runtime_enforcement.sh
```
