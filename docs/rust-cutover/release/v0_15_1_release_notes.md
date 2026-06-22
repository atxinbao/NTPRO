# NTPRO Rust-only v0.15.1 Release Notes

Date: 2026-06-23
Executor: Codex
Status: READY FOR OWNER RELEASE DECISION
Planned tag: `ntpro-rust-only-v0.15.1`

## Summary

`v0.15.1` is a patch/hardening release for the v0.15 guarded live-alpha mutation
scope and execution dry-run harness. It does not expand trading capability. It
closes release-evidence gaps and tightens local dry-run contracts before any
future v0.16 production mutation work is considered.

Plain Chinese summary: v0.15.1 是补强版，不是实盘交易版。它把 v0.15.0 的人工审批、
签名材料、订单类型、endpoint、dry-run adapter 边界和 release gate 证据补齐、收紧。
它仍然不发真实生产请求，不真实下单，不撤单，不改单，不重试，不纠错，不调用生产
adapter，不用真实资金。

## Changed

- Recorded formal v0.15.0 tag-triggered hosted release-gate evidence.
- Made manual approval one-time by consuming it after successful request preview.
- Defaulted request-preview signing material to synthetic local material.
- Required explicit owner gates before production signing material can be read
  for memory-only dry-run preview.
- Tightened the production live-alpha dry-run order gate to `LIMIT` only.
- Denied production `POST /api/v3/order/test` request preview for now.
- Added an explicit execution boundary:
  `StrategyIntent -> RiskDecision -> ExecutionCommand -> DryRunExecutionAdapter`.
- Added v0.15.1 aggregate release gate wiring and release accounting.
- Updated the transitive `quinn-proto` lockfile entry from `0.11.14` to
  `0.11.15` to remediate `RUSTSEC-2026-0185` in hosted release/security gates.
- Aligned cargo-vet policy/exemption entries for the `quinn-proto` update and
  stale first-party crate policy checks.

## Boundary

Included:

```text
patch/hardening
capability expansion = false
formal v0.15.0 release gate evidence accounting
manual approval consume artifact
synthetic signing-material default
owner-gated production signing-material dry-run preview
LIMIT-only dry-run order gate
production order-test request-preview denial
execution dry-run adapter boundary contract
v0.15.1 aggregate release gates
security-audit lockfile remediation for transitive quinn-proto RUSTSEC-2026-0185
cargo-vet policy/exemption alignment for release security gates
```

Not included:

```text
production order submission
production order-test submission
production cancel, replace, amend, retry, correction, or flatten
production HTTP request execution
production execution adapter implementation
production execution adapter instantiation
production execution adapter calls
default production network execution
listenKey creation, keepalive, or close lifecycle
signed WebSocket user stream runtime
strategy-driven production execution
automatic production remediation
real funds
production trading
Dashboard order controls
Dashboard credential input
raw production responses, raw credentials, signatures, signed queries, or signed URLs
```

## Merged PR Accounting

| PR | Classification | Included in capability claim | Notes |
| --- | --- | --- | --- |
| #490 | V151-001 | No | Records formal v0.15.0 tag-triggered release gate evidence. |
| #491 | V151-002 | No | Hardens one-time manual approval by consuming it after request preview. |
| #492 | V151-003 | No | Defaults request-preview signing material to synthetic and gates production signing-material reads. |
| #493 | V151-004 | No | Narrows dry-run order gate to `LIMIT` only. |
| #494 | V151-005 | No | Denies production `/api/v3/order/test` request preview. |
| #495 | V151-006 | No | Defines explicit dry-run execution adapter boundary. |
| #496 | V151-007 | No | Adds aggregate v0.15.1 release gates, readiness, release notes, transitive `quinn-proto` patch lockfile remediation, and cargo-vet gate alignment. |

## Validation

Readiness evidence is recorded in:

```text
docs/rust-cutover/evidence/V151-001.md
docs/rust-cutover/evidence/V151-002.md
docs/rust-cutover/evidence/V151-003.md
docs/rust-cutover/evidence/V151-004.md
docs/rust-cutover/evidence/V151-005.md
docs/rust-cutover/evidence/V151-006.md
docs/rust-cutover/evidence/V151-007.md
```

Required release validation for this package:

```text
scripts/ai/verify_release.sh v151-release-gates
scripts/ai/verify_release.sh release-surface-current-guard
scripts/ai/verify_fast.sh
/Users/mac/.cargo/bin/cargo-audit audit
cargo vet --locked --no-minimize-exemptions
git diff --check
hosted security-audit
```

## Release Status

This document is prepared for a future formal release:

```text
planned tag = ntpro-rust-only-v0.15.1
release status = not yet tagged by this document
GitHub Release = not yet created by this document
```

The release boundary must continue to preserve: dry-run request preview only,
one-time manual owner approval, synthetic signing material by default, local
dry-run adapter evidence only, no production request sending, no production
order submission, no production order-test submission, no production order
mutation, no cancel/replace/amend/retry/correction, no listenKey lifecycle, no
signed WebSocket user stream runtime, no real funds, no production trading, no
automatic production remediation, and no Dashboard order controls.
