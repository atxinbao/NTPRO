# NTPRO Rust-only v0.12.1 Release Notes

Date: 2026-06-21
Executor: Codex
Status: READY FOR OWNER RELEASE DECISION
Candidate tag: `ntpro-rust-only-v0.12.1`

## Summary

`v0.12.1` is a hardening patch for the published `v0.12.0` Production Online
Read-Only + Persistent Shadow release. It closes publication-surface, gate,
owner-run evidence, artifact-semantics, WebSocket boundary, shadow-session
wording, and Decimal/string notional preflight gaps without expanding the
product capability.

Plain Chinese summary: v0.12.1 是 v0.12.0 的补丁，不是新实盘交易能力。它把发布面、
门禁、owner 手动在线证据、JSON 字段语义、WebSocket 用户流、shadow session 和
notional 金额证据写清楚。它仍然不下单、不撤单、不改单、不读生产订单状态、不创建
listenKey、不碰真实资金，也不开放 Dashboard 下单按钮。

## Changed

- Closed the v0.12.0 release-publication surface so README, ROADMAP, versioning,
  release notes, readiness, tag, and GitHub Release wording align.
- Hardened `release-surface-current-guard` against stale current-version
  wording.
- Added a GitHub Release publication guard for release workflows.
- Clarified owner-run production online proof as optional evidence, while
  default release gates remain fail-closed and offline.
- Normalized v0.12 artifact fields:

```text
endpoint_read_allowed
offline_contract_ready
read_allowed
contract_ready
online_read_allowed
```

- Denied/deferred signed WebSocket user stream classification until listenKey
  lifecycle exists.
- Clarified that v0.12 shadow strategy session output is a bounded local JSONL
  event artifact, not a long-running runtime, daemon, stale-data monitor, or
  production execution process.
- Added Decimal/string notional preflight to the v0.12 shadow portfolio runtime
  artifact:

```text
f64_aggregation_used=false
live_alpha_money_math_ready=false
risk_or_execution_grade=false
```

## Boundary

Included:

```text
release-publication surface hardening
release guard hardening
GitHub Release publication guard
owner-run online proof wording and artifact contract clarity
v0.12 read artifact field-semantics clarity
signed WebSocket user stream deferred/denied classification
bounded shadow strategy session wording
Decimal/string shadow notional preflight evidence
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
live-alpha risk/execution-grade money math
exchange-confirmed shadow fills or positions
real funds
production trading
Dashboard order/cancel/replace/amend controls
Dashboard credential input
```

## Validation

Readiness evidence is recorded in:

```text
PR #430 / Shrimp closeout for V121-001
docs/rust-cutover/evidence/V121-002.md
docs/rust-cutover/evidence/V121-003.md
docs/rust-cutover/evidence/V121-004.md
docs/rust-cutover/evidence/V121-005.md
docs/rust-cutover/evidence/V121-006.md
docs/rust-cutover/evidence/V121-007.md
docs/rust-cutover/evidence/V121-008.md
docs/rust-cutover/evidence/V121-009.md
```

Local validation for this release-note package:

```text
scripts/ai/verify_release.sh release-surface-current-guard = PASS
scripts/ai/verify_release.sh v12-offline-release-gates = PASS
scripts/ai/verify_release.sh v12-manual-online-preflight = PASS
scripts/ai/verify_fast.sh = PASS
git diff --check = PASS
```

## Release Status

This document is release-note material for a possible owner-approved
`ntpro-rust-only-v0.12.1` GitHub Release. This PR does not create the tag and
does not publish the GitHub Release.

The release boundary must continue to preserve: Production Online Read-Only +
Persistent Shadow hardening only, no production order submission, no production
order mutation, no production order-state reads, no listenKey lifecycle, no
signed WebSocket user stream runtime, no real funds, no production trading, no
automatic production remediation, no production portfolio parity, no
live-alpha money-math readiness, and no Dashboard order controls.
