# NTPRO Rust-only v0.13.1 Release Notes

Date: 2026-06-22
Executor: Codex
Status: READY FOR OWNER RELEASE DECISION
Tag: `ntpro-rust-only-v0.13.1` not yet created
Release name: `NTPRO Rust-only v0.13.1` not yet published

## Summary

`v0.13.1` is a patch hardening candidate for `v0.13.0` Guarded Live Alpha
Preflight. It closes post-release evidence and wording gaps without expanding
the production capability claim.

Plain Chinese summary: v0.13.1 是 v0.13.0 后面的补丁收口版本。它不是新交易能力，
只是让证据、文档、Dashboard 只读状态和未来 money/price/quantity 合同更清楚。

## Changed

- Recorded v0.13.0 release closure evidence and release index linkage.
- Clarified owner-run online read-only proof-pack wording so wrapper
  implementation is not confused with included owner-run online success
  evidence.
- Normalized v0.13 shadow wording to bounded local shadow preflight loop.
- Propagated kill-switch dry-run/manual approval artifact status into
  read-only Supervisor and Dashboard status surfaces.
- Added a Money/Price/Quantity contract draft for future live-alpha preflight.
- Added a read-only Dashboard preflight readiness panel.
- Prepared v0.13.1 readiness and release-note material.

## Boundary

Included:

```text
release closure evidence
proof-pack wording hardening
bounded local shadow preflight wording
kill-switch dry-run read-only status propagation
Money/Price/Quantity contract draft
Dashboard preflight readiness read-only panel
```

Not included:

```text
production order submission
production cancel, replace, amend, retry, or correction orders
production open-order or order-state reads
listenKey creation, keepalive, or close lifecycle
signed WebSocket user stream runtime
strategy-driven production execution
automatic production remediation
production portfolio parity
risk/execution-grade live-alpha money math
exchange-confirmed shadow fills or positions
raw account response, raw balances, raw credentials, signatures, signed query, or signed URL persistence
real funds
production trading
Dashboard order/cancel/replace/amend/retry/reconnect controls
Dashboard credential input
```

## Merged PR Accounting

| PR | Classification | Included in capability claim | Notes |
| --- | --- | --- | --- |
| #452 | V131-001 | No | Records v0.13.0 formal release closure evidence. |
| #453 | V131-001 | No | Adds release index evidence linkage. |
| #454 | V131-002 | No | Clarifies owner-run read-only proof-pack evidence semantics. |
| #455 | V131-003 | No | Clarifies bounded local shadow preflight wording. |
| #456 | V131-004 | No | Adds read-only kill-switch dry-run status propagation. |
| #457 | V131-005 | No | Adds Money/Price/Quantity future contract draft. |
| #458 | V131-006 | No | Adds read-only Dashboard preflight readiness panel. |

## Validation

Readiness evidence is recorded in:

```text
docs/rust-cutover/evidence/V131-001.md
docs/rust-cutover/evidence/V131-002.md
docs/rust-cutover/evidence/V131-003.md
docs/rust-cutover/evidence/V131-004.md
docs/rust-cutover/evidence/V131-005.md
docs/rust-cutover/evidence/V131-006.md
docs/rust-cutover/evidence/V131-007.md
```

Required owner-side release validation before publication:

```text
scripts/ai/verify_release.sh release-surface-current-guard
scripts/ai/verify_release.sh v13-no-production-mutation-gate
scripts/ai/verify_fast.sh
git diff --check
GitHub hosted Rust Cutover Release Gate for tag ntpro-rust-only-v0.13.1
```

## Release Status

This document is release-note material for a possible formal GitHub Release:

```text
tag = ntpro-rust-only-v0.13.1
release name = NTPRO Rust-only v0.13.1
publication status = not published by this document
```

The release boundary must continue to preserve: Guarded Live Alpha Preflight
hardening only, no production order submission, no production order mutation,
no production order-state reads, no listenKey lifecycle, no signed WebSocket
user stream runtime, no real funds, no production trading, no automatic
production remediation, no production portfolio parity, no risk/execution-grade
live-alpha money math, and no Dashboard order/cancel/replace/amend/retry/
reconnect controls.
