# NTPRO Rust-only v0.16.1 Release Notes

Date: 2026-06-24
Executor: Codex
Status: READY FOR OWNER RELEASE DECISION
Planned tag: `ntpro-rust-only-v0.16.1`

## Summary

`v0.16.1` is a patch/hardening release for the v0.16 minimum owner-approved
production order mutation candidate. It does not expand production trading
capability. It closes evidence and wording gaps around the single
owner-approved tiny `LIMIT` `GTC` production order candidate.

Plain Chinese summary: v0.16.1 是 v0.16.0 的补强版，不是新实盘能力版本。大白话：
它只把“一笔 owner 批准的小额 LIMIT GTC 生产订单候选”的证据补硬：有没有真的尝试、
有没有 HTTP ack、kill switch 有没有二次读取、响应脱敏是不是来自真实结果、价格是否安全、
owner-run 有没有执行、CLI 和 classifier 是否说清楚边界。

## Changed

- Clarified guarded-send counters for attempts, HTTP sends, exchange ack,
  confirmed submission, and platform-trading semantics.
- Added a real post-send kill-switch second read after the HTTP boundary.
- Bound response-redaction evidence to the actual guarded-send result when
  owner-run evidence exists.
- Added non-marketable price safety checks before request-builder send
  consideration.
- Added a formal owner-run production mutation evidence slot:
  `owner-run-not-executed` vs `owner-run-executed-classified`.
- Aligned CLI help and endpoint classifier wording with the v0.16/v0.16.1
  owner-gated mutation-candidate boundary.
- Added v0.16.1 readiness and release-note accounting.

## Boundary

Included:

```text
patch/hardening only
capability expansion = false
single owner-approved tiny LIMIT GTC production order candidate boundary
default offline fail-closed posture
guarded-send evidence clarification
post-send kill-switch read evidence
response-redaction source binding
non-marketable price safety evidence
owner-run evidence slot
CLI/classifier wording alignment
v0.16.1 readiness and release notes
```

Not included:

```text
strategy-driven production execution
multiple production orders
batch production orders
MARKET orders
cancel, replace, amend, retry, correction, flatten, or remediation
Dashboard order controls
Dashboard credential input
multi-account execution
multi-venue execution
VWAP/POV/Iceberg execution algorithms
listenKey lifecycle
real-funds proof in CI
production trading platform claim
```

## Merged PR Accounting

| PR | Classification | Included in capability claim | Notes |
| --- | --- | --- | --- |
| #511 | V161-001 | No expansion | Clarifies guarded-send counters and confirmed submission semantics. |
| #512 | V161-002 | No expansion | Adds post-send kill-switch second-read evidence. |
| #513 | V161-003 | No expansion | Binds response redaction to actual guarded-send source when applicable. |
| #514 | V161-004 | No expansion | Adds non-marketable price safety preflight evidence. |
| #515 | V161-005 | No expansion | Adds owner-run evidence slot; current outcome is `owner-run-not-executed`. |
| #516 | V161-006 | No expansion | Aligns CLI and classifier wording with owner-gated mutation-candidate boundary. |

## Validation

Readiness evidence is recorded in:

```text
docs/rust-cutover/evidence/V161-001.md
docs/rust-cutover/evidence/V161-002.md
docs/rust-cutover/evidence/V161-003.md
docs/rust-cutover/evidence/V161-004.md
docs/rust-cutover/evidence/V161-owner-run-production-mutation-candidate.md
docs/rust-cutover/evidence/V161-006.md
docs/rust-cutover/evidence/V161-007.md
```

Required release validation for this package:

```text
scripts/ai/verify_release.sh v16-release-gates
scripts/ai/verify_release.sh release-surface-current-guard
scripts/ai/verify_fast.sh
git diff --check
hosted Rust Cutover Smoke
```

## Release Status

This document is prepared for a future formal release:

```text
planned tag = ntpro-rust-only-v0.16.1
release status = not yet tagged by this document
GitHub Release = not yet created by this document
latest formal release = ntpro-rust-only-v0.16.0
```

The release boundary must continue to preserve: one owner-approved tiny
`LIMIT` `GTC` production order candidate only, default offline fail-closed
execution, no strategy-driven production execution, no multi-order execution,
no cancel/replace/amend/retry/correction/flatten, no automatic remediation, no
listenKey lifecycle, no real-funds claim, no production trading platform claim,
and no Dashboard order controls.
