# NTPRO Rust-only v0.14.0 Release Notes

Date: 2026-06-22
Executor: Codex
Status: RELEASED
Tag: `ntpro-rust-only-v0.14.0`
Release name: `NTPRO Rust-only v0.14.0`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.14.0`

## Summary

`v0.14.0` is the Production Order-State Read-Only + Live Alpha Dry-Run release.
It adds owner-gated production order-state GET proof scope and local live-alpha
dry-run evidence while preserving the no-production-mutation boundary.

Plain Chinese summary: v0.14.0 是“只读订单状态 + live-alpha 干跑证据”版本。它可以
让 owner 手动开 gate 做生产订单状态只读 GET 证明，也能在本地记录 live-alpha
dry-run、风控预检、reconciliation 和 Dashboard 只读状态。但它仍然不是实盘下单版本。

## Changed

- Defined the v0.14 production order-state read-only boundary.
- Added owner-gated production order-state GET proof with default offline
  fail-closed preflight.
- Added supervisor-managed shadow runtime lifecycle evidence.
- Added live-alpha dry-run order gate artifacts.
- Added live-alpha local risk preflight artifacts.
- Added executable live-alpha reconciliation golden traces.
- Added a read-only Dashboard live-alpha dry-run panel.
- Wired the v0.14 aggregate release gate into local release verification, PR
  smoke, and release-tag workflow.
- Prepared v0.14 readiness and release-note material.

## Boundary

Included:

```text
production order-state read-only boundary
owner-gated production order-state GET proof
local supervisor shadow runtime evidence
live-alpha dry-run order gate
live-alpha local risk preflight
live-alpha reconciliation golden traces
Dashboard live-alpha dry-run read-only panel
v0.14 release gate wiring
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
| #459 | V131-007 prerequisite | No | v0.13.1 readiness/release notes, prerequisite closure only. |
| #460 | V140-000 | Boundary only | Defines owner-gated production order-state read-only scope. |
| #461 | V140-001 | Yes, read-only only | Adds production order-state GET proof with default offline preflight. |
| #462 | V140-002 | Yes, local shadow runtime only | Adds supervisor-managed shadow runtime evidence. |
| #463 | V140-003 | Yes, dry-run only | Adds live-alpha dry-run order gate with no submission. |
| #464 | V140-004 | Yes, local preflight only | Adds live-alpha risk preflight without execution. |
| #465 | V140-005 | Yes, replay evidence only | Adds live-alpha reconciliation golden traces. |
| #466 | V140-006 | Yes, read-only Dashboard only | Adds live-alpha dry-run Dashboard status panel. |
| #467 | V140-007 | No | Adds v0.14 release gate wiring. |
| #468 | V140-008 | No | Prepares v0.14 readiness and release notes. |

## Validation

Readiness evidence is recorded in:

```text
docs/rust-cutover/evidence/V140-000.md
docs/rust-cutover/evidence/V140-001.md
docs/rust-cutover/evidence/V140-002.md
docs/rust-cutover/evidence/V140-003.md
docs/rust-cutover/evidence/V140-004.md
docs/rust-cutover/evidence/V140-005.md
docs/rust-cutover/evidence/V140-006.md
docs/rust-cutover/evidence/V140-007.md
docs/rust-cutover/evidence/V140-008.md
```

Required release validation for this package:

```text
scripts/ai/verify_release.sh release-surface-current-guard
scripts/ai/verify_release.sh v14-release-gates
scripts/ai/verify_fast.sh
git diff --check
```

## Release Status

This document is the formal GitHub Release note for:

```text
tag = ntpro-rust-only-v0.14.0
release name = NTPRO Rust-only v0.14.0
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.14.0
```

The release boundary must continue to preserve: owner-gated read-only
order-state proof only, live-alpha dry-run only, no production order
submission, no production order mutation, no cancel/replace/amend/retry/
correction, no listenKey lifecycle, no signed WebSocket user stream runtime, no
real funds, no production trading, no automatic production remediation, and no
Dashboard order/cancel/replace/amend/retry/reconnect controls.
