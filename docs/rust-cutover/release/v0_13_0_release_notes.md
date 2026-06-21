# NTPRO Rust-only v0.13.0 Release Notes

Date: 2026-06-21
Executor: Codex
Status: RELEASED
Tag: `ntpro-rust-only-v0.13.0`
Release name: `NTPRO Rust-only v0.13.0`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.13.0`

## Summary

`v0.13.0` is the Guarded Live Alpha Preflight release. It adds the evidence
needed before any future live-alpha execution decision, while keeping default
local, PR, CI, and release-gate execution offline/fail-closed.

Plain Chinese summary: v0.13.0 不是实盘交易版本。它只是把 live alpha 之前必须
先证明的东西补齐：本地 shadow preflight session、owner-gated 只读 proof pack、
kill switch 审批证据、Dashboard 控制边界、Decimal 金额边界，以及发版门禁证明
没有生产下单/撤改/订单状态读取/listenKey/Dashboard 下单按钮。

## Changed

- Defined the v0.13 Guarded Live Alpha Preflight scope.
- Added a local shadow preflight session with heartbeat, stop-file, and
  stale-data evidence.
- Added an owner-run production online read-only proof-pack wrapper. The default
  gate path remains offline/fail-closed.
- Added a kill-switch dry-run/manual approval artifact contract.
- Defined the trader/ops Dashboard boundary and kept production order controls
  disabled.
- Enforced Decimal/string-only amount preflight boundaries and rejected
  non-plain decimal strings.
- Added the v13 no-production-mutation PR/release gate.
- Absorbed the post-v0.12.1 release-publication guard fix from PR #440 as a
  source-tree hardening delta; it does not expand the v0.13 capability claim.

## Boundary

Included:

```text
Guarded Live Alpha Preflight scope decision
local shadow preflight session evidence
owner-gated production online read-only proof-pack wrapper
kill-switch dry-run/manual approval artifact
trader/ops Dashboard read-only/control boundary evidence
Decimal/string-only amount preflight evidence
no-production-mutation PR and release gate
release-publication guard hardening absorbed from PR #440
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
| #440 | Source-tree hardening delta | No | Fixes v0.12.1 release-publication guard after the v0.12.1 tag. Absorbed into the v0.13 source tree as release hygiene only. |
| #441 | V130-001 | Yes | Defines Guarded Live Alpha Preflight scope. |
| #442 | V130-002 | Yes | Adds local shadow preflight session loop and evidence. |
| #443 | V130-003 | Yes | Adds owner-run online read-only proof-pack wrapper. |
| #444 | V130-004 | Yes | Adds kill-switch dry-run/manual approval artifact. |
| #445 | V130-005 | Yes | Defines Dashboard trader/ops control boundary. |
| #446 | V130-006 | Yes | Enforces Decimal/string amount boundary. |
| #447 | V130-007 | Yes | Wires no-production-mutation PR/release gate. |

## Validation

Readiness evidence is recorded in:

```text
docs/rust-cutover/evidence/V130-001.md
docs/rust-cutover/evidence/V130-002.md
docs/rust-cutover/evidence/V130-003.md
docs/rust-cutover/evidence/V130-004.md
docs/rust-cutover/evidence/V130-005.md
docs/rust-cutover/evidence/V130-006.md
docs/rust-cutover/evidence/V130-007.md
```

Required release validation:

```text
scripts/ai/verify_release.sh release-surface-current-guard
scripts/ai/verify_release.sh v13-no-production-mutation-gate
scripts/ai/verify_release.sh v12-offline-release-gates
scripts/ai/verify_release.sh v12-manual-online-preflight
scripts/ai/verify_fast.sh
git diff --check
GitHub hosted Rust Cutover Release Gate for tag ntpro-rust-only-v0.13.0
```

## Release Status

This document is the formal GitHub Release note for:

```text
tag = ntpro-rust-only-v0.13.0
release name = NTPRO Rust-only v0.13.0
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.13.0
```

The release boundary must continue to preserve: Guarded Live Alpha Preflight
only, no production order submission, no production order mutation, no
production order-state reads, no listenKey lifecycle, no signed WebSocket user
stream runtime, no real funds, no production trading, no automatic production
remediation, no production portfolio parity, no risk/execution-grade
live-alpha money math, and no Dashboard order/cancel/replace/amend/retry/
reconnect controls.
