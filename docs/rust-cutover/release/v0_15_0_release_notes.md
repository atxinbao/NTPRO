# NTPRO Rust-only v0.15.0 Release Notes

Date: 2026-06-22
Executor: Codex
Status: RELEASED
Tag: `ntpro-rust-only-v0.15.0`
Release name: `NTPRO Rust-only v0.15.0`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.15.0`

## Summary

`v0.15.0` is the Guarded Live Alpha Mutation Scope + Execution Dry-Run Harness
release. It defines a narrow production mutation research boundary and adds
local artifacts for redacted request preview, one-time manual approval, kill
switch runtime enforcement, dry-run execution isolation, incident evidence, and
Dashboard read-only preflight review.

Plain Chinese summary: v0.15.0 是“生产 mutation 范围 + 执行干跑链路”版本。它让
owner 可以在本地看到未来生产下单请求的脱敏预览、人工审批状态、kill switch、dry-run
adapter、事故回滚证据和 Dashboard 只读状态。但它仍然不发真实生产请求，不真实下单，
不撤单，不改单，不重试，不纠错，不用真实资金。

## Changed

- Defined the v0.15 guarded production mutation research scope.
- Classified production order mutation endpoints as default-denied preview
  candidates.
- Added a redacted production live-alpha order request dry-run preview builder.
- Added one-time manual approval lifecycle artifacts for request preview.
- Added kill-switch runtime enforcement before dry-run progression.
- Added execution adapter isolation artifacts proving strategy intent reaches
  only the local dry-run path.
- Added executable mutation dry-run golden traces.
- Added manual incident, rollback, and emergency-stop evidence contracts.
- Added a read-only Dashboard mutation preflight panel.
- Wired v0.15 aggregate release gates into local release verification, PR
  smoke, and the tag-triggered release workflow.
- Prepared v0.15 readiness and release-note material.

## Boundary

Included:

```text
Guarded Live Alpha Mutation Scope + Execution Dry-Run Harness
production mutation endpoint classification
redacted local order request preview
manual approval lifecycle for preview artifact creation
kill switch runtime gate
local dry-run execution adapter artifact
dry-run mutation golden trace replay
manual incident, rollback, and emergency-stop artifacts
read-only Dashboard mutation preflight panel
v0.15 release gate wiring
```

Not included:

```text
production order submission
production order-test submission
production cancel, replace, amend, retry, correction, or flatten
production HTTP request execution
production execution adapter implementation or calls
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
| #477 | V141-007 prerequisite | No | v0.14.1 hardening readiness/release notes, prerequisite closure only. |
| #478 | V150-000 | Boundary only | Defines guarded production mutation dry-run scope. |
| #479 | V150-001 | Yes, classifier only | Adds default-denied mutation endpoint preview classification. |
| #480 | V150-002 | Yes, preview only | Adds redacted request-preview artifacts with no send. |
| #481 | V150-003 | Yes, dry-run only | Adds execution adapter isolation artifact. |
| #482 | V150-004 | Yes, runtime gate only | Adds kill-switch runtime enforcement artifact. |
| #483 | V150-005 | Yes, approval artifact only | Adds one-time manual approval lifecycle for preview creation. |
| #484 | V150-006 | Yes, replay evidence only | Adds mutation dry-run golden traces. |
| #485 | V150-007 | Yes, manual evidence only | Adds incident, rollback, and emergency-stop artifact contracts. |
| #486 | V150-008 | Yes, read-only Dashboard only | Adds Dashboard mutation preflight status panel. |
| #487 | V150-009 | No | Adds v0.15 aggregate release gate wiring. |
| #488 | V150-010 | No | Prepares v0.15 readiness and release notes. |

## Validation

Readiness evidence is recorded in:

```text
docs/rust-cutover/evidence/V150-000.md
docs/rust-cutover/evidence/V150-001.md
docs/rust-cutover/evidence/V150-002.md
docs/rust-cutover/evidence/V150-003.md
docs/rust-cutover/evidence/V150-004.md
docs/rust-cutover/evidence/V150-005.md
docs/rust-cutover/evidence/V150-006.md
docs/rust-cutover/evidence/V150-007.md
docs/rust-cutover/evidence/V150-008.md
docs/rust-cutover/evidence/V150-009.md
docs/rust-cutover/evidence/V150-010.md
```

Required release validation for this package:

```text
scripts/ai/verify_release.sh release-surface-current-guard
scripts/ai/verify_release.sh v15-release-gates
scripts/ai/verify_fast.sh
git diff --check
```

## Release Status

This document is the formal GitHub Release note for:

```text
tag = ntpro-rust-only-v0.15.0
release name = NTPRO Rust-only v0.15.0
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.15.0
```

The release boundary must continue to preserve: dry-run request preview only,
manual owner approval for preview artifact creation only, local dry-run adapter
evidence only, no production request sending, no production order submission,
no production order mutation, no cancel/replace/amend/retry/correction, no
listenKey lifecycle, no signed WebSocket user stream runtime, no real funds, no
production trading, no automatic production remediation, and no Dashboard
order/cancel/replace/amend/retry/reconnect controls.
