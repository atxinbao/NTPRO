# NTPRO Rust-only v0.14.1 Release Notes

Date: 2026-06-22
Executor: Codex
Status: READY FOR OWNER RELEASE DECISION
Tag: `ntpro-rust-only-v0.14.1` not yet created
Release name: `NTPRO Rust-only v0.14.1` not yet published

## Summary

`v0.14.1` is a patch hardening candidate for `v0.14.0` Production Order-State
Read-Only + Live Alpha Dry-Run. It closes order-state evidence semantics,
dry-run wording, CLI copy, and Dashboard read-only status gaps without
expanding the production capability claim.

Plain Chinese summary: v0.14.1 是 v0.14.0 后面的补丁收口版本。它不是新交易能力，
只是把生产订单状态只读证据、live-alpha 干跑语义、CLI 帮助文案和 Dashboard 只读状态
讲清楚、验清楚、展示清楚。

## Changed

- Added a v0.14.1 owner-run order-state read-only evidence validator.
- Split order-state exchange truth from shadow and portfolio truth.
- Renamed live-alpha risk preflight decisions to
  `dry_run_approved` / `dry_run_rejected`.
- Added `execution_decision=blocked_no_production_mutation` to live-alpha
  risk preflight artifacts.
- Split empty `openOrders=[]` endpoint-shape validation from order lifecycle
  readiness.
- Clarified `nautilus live` help text so production read-only and dry-run proof
  commands are not described as sandbox-only.
- Added production order-state read-only proof fields to the Dashboard
  live-alpha readiness surface.
- Prepared v0.14.1 readiness and release-note material.

## Boundary

Included:

```text
owner-run production order-state read-only evidence validation
order-state exchange-truth field split
live-alpha dry-run risk decision wording
blocked_no_production_mutation execution decision
empty openOrders endpoint-shape semantics
CLI help boundary clarification
Dashboard order-state read-only evidence panel
```

Not included:

```text
production order submission
production cancel, replace, amend, retry, correction, or flatten
production order-test submission
execution adapter calls for live-alpha dry-run
default production network execution
listenKey creation, keepalive, or close lifecycle
signed WebSocket user stream runtime
strategy-driven production execution
automatic production remediation
real funds
production trading
Dashboard order/cancel/replace/amend/retry/reconnect controls
Dashboard credential input
raw production responses, raw credentials, signatures, signed queries, or signed URLs
```

## Merged PR Accounting

| PR | Classification | Included in capability claim | Notes |
| --- | --- | --- | --- |
| #471 | V141-001 | No production mutation | Adds owner-run order-state read-only evidence validation. |
| #472 | V141-002 | No production mutation | Splits order-state truth from shadow and portfolio truth. |
| #473 | V141-003 | No production mutation | Makes risk preflight decisions explicitly dry-run and execution-blocked. |
| #474 | V141-004 | No production mutation | Treats empty `openOrders=[]` as endpoint-shape evidence only. |
| #475 | V141-005 | No runtime capability | Clarifies CLI help text boundaries. |
| #476 | V141-006 | Read-only Dashboard status only | Displays order-state proof fields without order controls. |

## Validation

Readiness evidence is recorded in:

```text
docs/rust-cutover/evidence/V141-001.md
docs/rust-cutover/evidence/V141-002.md
docs/rust-cutover/evidence/V141-003.md
docs/rust-cutover/evidence/V141-004.md
docs/rust-cutover/evidence/V141-005.md
docs/rust-cutover/evidence/V141-006.md
docs/rust-cutover/evidence/V141-007.md
```

Required owner-side release validation before publication:

```text
scripts/ai/verify_release.sh release-surface-current-guard
scripts/ai/verify_release.sh v14-release-gates
scripts/ai/verify_v141_order_state_owner_evidence.sh
scripts/ai/verify_fast.sh
git diff --check
GitHub hosted Rust Cutover Release Gate for tag ntpro-rust-only-v0.14.1
```

## Release Status

This document is release-note material for a possible formal GitHub Release:

```text
tag = ntpro-rust-only-v0.14.1
release name = NTPRO Rust-only v0.14.1
publication status = not published by this document
```

The release boundary must continue to preserve: order-state read-only evidence
hardening only, live-alpha dry-run only, no production order submission, no
production order mutation, no cancel/replace/amend/retry/correction, no
listenKey lifecycle, no signed WebSocket user stream runtime, no real funds, no
production trading, no automatic production remediation, and no Dashboard
order/cancel/replace/amend/retry/reconnect controls.
