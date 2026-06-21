# NTPRO Rust-only v0.12.0 Release Notes

Date: 2026-06-21
Executor: Codex
Status: RELEASED
Tag: `ntpro-rust-only-v0.12.0`
Release name: `NTPRO Rust-only v0.12.0`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.12.0`

## Summary

`v0.12.0` is the Production Online Read-Only + Persistent Shadow release. It
advances the v0.11 production read-only contract line by adding implemented
owner-gated production `GET` read-only paths, default fail-closed preflights,
and local persistent shadow runtime evidence.

Plain Chinese summary: v0.12.0 是“生产在线只读 + 持久 shadow”正式发布版本。它可以在
用户明确打开 gate 后尝试生产 Binance 只读 GET；默认发版 gate 只证明这些路径
fail-closed，不要求真实联网成功。真实在线成功证明属于 owner 可选运行证据。它还把本地
shadow portfolio、shadow strategy session、reconciliation 和 Dashboard 只读状态串起来。
它不是实盘交易版本，不下单、不撤单、不改单、不读生产订单状态、不碰真实资金。

## Changed

- Defined the v0.12 Production Online Read-Only + Persistent Shadow boundary.
- Added implemented owner-gated production public read-only `GET` path behavior.
- Added implemented owner-gated authenticated production account snapshot
  `GET` path behavior.
- Clarified that owner-run successful online proof is optional evidence, not a
  default CI or release-gate requirement.
- Added redacted response-shape validation for account snapshot evidence.
- Added a local shadow portfolio runtime artifact.
- Added a local persistent shadow strategy session event stream.
- Added local production read-only reconciliation classifications.
- Extended Dashboard with a v0.12 production shadow read-only panel.
- Added v0.12 offline release gates and manual-online fail-closed preflight.
- Prepared v0.12 readiness and release-note material.

## Boundary

Included:

```text
implemented owner-gated production public GET read-only path
implemented owner-gated authenticated production account snapshot GET path
optional owner-run successful online proof artifact contract
redacted account response-shape evidence
local shadow portfolio runtime artifacts
local persistent shadow strategy session artifacts
local read-only reconciliation classifications
Dashboard read-only production shadow status
offline v0.12 release gate bundle
manual-online fail-closed preflight gate
```

Not included:

```text
production order submission
production cancel, replace, amend, retry, or correction orders
production open-order or order-state reads
listenKey lifecycle access
strategy-driven production execution
automatic production remediation
production portfolio parity
exchange-confirmed shadow fills or positions
raw account response, raw balances, raw credentials, signatures, signed query, or signed URL persistence
real funds
production trading
Dashboard order/cancel/replace/amend/retry/reconnect controls
Dashboard credential input
```

## Validation

Readiness evidence is recorded in:

```text
docs/rust-cutover/evidence/V120-000.md
docs/rust-cutover/evidence/V120-001.md
docs/rust-cutover/evidence/V120-002.md
docs/rust-cutover/evidence/V120-003.md
docs/rust-cutover/evidence/V120-004.md
docs/rust-cutover/evidence/V120-005.md
docs/rust-cutover/evidence/V120-006.md
docs/rust-cutover/evidence/V120-007.md
docs/rust-cutover/evidence/V120-008.md
docs/rust-cutover/evidence/V120-009.md
```

Local validation for this release-note package:

```text
scripts/ai/verify_release.sh v12-offline-release-gates v12-manual-online-preflight = required PASS
scripts/ai/verify_fast.sh = required PASS
git diff --check = required PASS
```

## Release Status

This document is the formal GitHub Release note for:

```text
tag = ntpro-rust-only-v0.12.0
release name = NTPRO Rust-only v0.12.0
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.12.0
```

The release boundary must continue to preserve: Production Online Read-Only +
Persistent Shadow only, no production order submission, no production order
mutation, no production order-state reads, no listenKey lifecycle, no real
funds, no production trading, no automatic production remediation, and no
Dashboard order controls.

`v0.13.0` is the earliest possible Guarded Live Alpha candidate and requires a
separate owner-approved scope decision.
