# NTPRO Rust-only v0.16.0 Release Notes

Date: 2026-06-23
Executor: Codex
Status: READY FOR OWNER RELEASE DECISION
Planned tag: `ntpro-rust-only-v0.16.0`

## Summary

`v0.16.0` prepares NTPRO for a minimum owner-approved production order mutation
candidate. This is not a general production trading release. The release scope
is a single tiny owner-approved `LIMIT` `GTC` production order candidate, with
explicit gates, redacted artifacts, readback evidence, audit evidence,
kill-switch enforcement, and no retry/remediation.

Plain Chinese summary: v0.16.0 是“最小生产下单候选”发布材料。大白话：它只围绕一笔
老板明确批准的小额 `LIMIT` `GTC` 订单候选做门禁和证据。默认仍然不联网、不下单；
它不是策略实盘，不是多单执行，不是 Dashboard 下单，也不包含撤单、改单、重试、
自动补救或 listenKey。

## Changed

- Added the v0.16 production mutation scope contract.
- Added owner-approved runtime gates for production mutation.
- Added production signing-material approval evidence.
- Added a single `LIMIT` `GTC` request builder.
- Added a guarded production HTTP send path behind explicit gates.
- Added production mutation response redaction requirements.
- Added post-submit order-state readback proof contract.
- Added kill-switch checks around the send boundary.
- Added production mutation audit trail artifacts.
- Added failure-mode and no-retry semantics.
- Added a read-only Dashboard production mutation evidence panel.
- Added v0.16 aggregate release gate wiring.
- Added readiness and release-note material for owner release decision.

## Boundary

Included:

```text
Minimum Owner-Approved Production Order Mutation Candidate
single owner-approved tiny LIMIT GTC production order candidate
default offline fail-closed posture
explicit owner approval
owner-gated signing material
guarded send path
redacted request/response evidence
post-submit readback evidence
kill-switch enforcement
audit trail
terminal failure/no-retry semantics
read-only Dashboard evidence
aggregate v0.16 release gate
```

Not included:

```text
strategy-driven production execution
multiple orders
MARKET orders
cancel, replace, amend, retry, correction, flatten, or remediation
Dashboard order controls
Dashboard credential input
multi-venue execution
multi-account execution
VWAP/POV/Iceberg execution algorithms
listenKey lifecycle
signed WebSocket user stream runtime
real-funds proof in CI
production trading platform claim
```

## Merged PR Accounting

| PR | Classification | Included in capability claim | Notes |
| --- | --- | --- | --- |
| #497 | V160-001 | Yes, boundary only | Defines the `Minimum Owner-Approved Production Order Mutation Candidate` scope. |
| #498 | V160-002 | Yes, gated candidate | Adds fail-closed runtime gates. |
| #499 | V160-003 | Yes, gated candidate | Adds production signing-material approval evidence. |
| #500 | V160-004 | Yes, gated candidate | Builds one redacted `LIMIT` `GTC` request object. |
| #501 | V160-005 | Yes, gated candidate | Adds guarded production HTTP send path behind explicit gates. |
| #502 | V160-006 | Yes, gated candidate | Adds response redaction contract. |
| #503 | V160-007 | Yes, gated candidate | Adds post-submit order-state readback proof contract. |
| #504 | V160-008 | Yes, gated candidate | Adds kill-switch checks around the send boundary. |
| #505 | V160-009 | Yes, gated candidate | Adds audit trail artifacts. |
| #506 | V160-010 | Yes, gated candidate | Defines terminal failure and no-retry semantics. |
| #507 | V160-011 | No Dashboard control expansion | Adds read-only Dashboard evidence panel. |
| #508 | V160-012 | No product-surface expansion | Adds aggregate release gates and hosted PR smoke wiring. |

## Validation

Readiness evidence is recorded in:

```text
docs/rust-cutover/evidence/V160-001.md
docs/rust-cutover/evidence/V160-002.md
docs/rust-cutover/evidence/V160-003.md
docs/rust-cutover/evidence/V160-004.md
docs/rust-cutover/evidence/V160-005.md
docs/rust-cutover/evidence/V160-006.md
docs/rust-cutover/evidence/V160-007.md
docs/rust-cutover/evidence/V160-008.md
docs/rust-cutover/evidence/V160-009.md
docs/rust-cutover/evidence/V160-010.md
docs/rust-cutover/evidence/V160-011.md
docs/rust-cutover/evidence/V160-012.md
docs/rust-cutover/evidence/V160-013.md
```

Required release validation for this package:

```text
scripts/ai/verify_release.sh v16-release-gates
scripts/ai/verify_release.sh release-surface-current-guard
scripts/ai/verify_fast.sh
git diff --check
hosted Rust Cutover Smoke
hosted security-audit
```

## Release Status

This document is prepared for a future formal release:

```text
planned tag = ntpro-rust-only-v0.16.0
release status = not yet tagged by this document
GitHub Release = not yet created by this document
```

The release boundary must continue to preserve: one owner-approved tiny
`LIMIT` `GTC` production order candidate only, default offline fail-closed
execution, no strategy-driven production execution, no multi-order execution,
no cancel/replace/amend/retry/correction/flatten, no automatic remediation, no
listenKey lifecycle, no real-funds claim, no production trading platform claim,
and no Dashboard order controls.
