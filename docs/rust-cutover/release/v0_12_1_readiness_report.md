# NTPRO v0.12.1 Production Read-Only Evidence & Release Surface Hardening Readiness Report

Date: 2026-06-21
Executor: Codex
Milestone: `ntpro-rust-only-v0.12.1`
Status: PASS - RELEASED

## Summary

`v0.12.1` is a patch/hardening queue for the published `v0.12.0` Production
Online Read-Only + Persistent Shadow line. It closes release-publication,
release-surface, owner-run proof wording, artifact-field semantics, signed
WebSocket scope, bounded shadow session wording, and Decimal/string notional
preflight gaps found after v0.12.0.

Plain Chinese summary: v0.12.1 不是新交易能力版本。它只是把 v0.12.0 发布后发现的
公开文案、GitHub Release 校验、owner 手动在线证据、artifact 字段语义、WebSocket
用户流边界、shadow session 形态和 notional 金额证据补严。它不下生产订单，不撤单、
不改单、不读生产订单状态，不创建 listenKey，不碰真实资金，也不把 Dashboard 变成下单面板。

## Product Claim

```text
capability = v0.12.0 Production Online Read-Only + Persistent Shadow hardening
current published release before v0.12.1 = ntpro-rust-only-v0.12.0
release tag = ntpro-rust-only-v0.12.1
release name = NTPRO Rust-only v0.12.1
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.12.1
default execution posture = offline fail-closed
production public online read path = implemented owner-gated GET-only path, fail-closed by default
production authenticated account snapshot path = implemented owner-gated GET-only path, fail-closed by default
successful online production read proof = optional owner-run evidence only
release publication guard = verifies formal GitHub Release publication state
shadow portfolio runtime = local artifact only
shadow notional preflight = Decimal/string evidence only, not risk/execution-grade money math
shadow strategy session = bounded local JSONL event artifact only, not a long-running runtime
signed WebSocket user stream = denied/deferred because listenKey lifecycle is not included
production open-order/order-state reads = not included
production order submission = not included
production order mutation = not included
listenKey lifecycle = not included
real funds = not included
production trading = not included
Dashboard order controls = not included
formal tag = ntpro-rust-only-v0.12.1
formal GitHub Release = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.12.1
```

## Included

```text
v0.12.0 release-publication surface closure
release surface guard hardening against stale current-version wording
GitHub Release publication guard for release workflows
owner-run online proof wording and artifact contract normalization
read_allowed / contract_ready / online_read_allowed field semantics normalization
signed WebSocket user stream denied/deferred until listenKey lifecycle exists
bounded shadow strategy session wording as local event artifact only
Decimal/string shadow notional preflight evidence
v0.12.1 readiness and release-note material
```

## Not Included

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
raw account response, raw balances, raw credentials, signatures, signed query, or signed URL persistence
real funds
production trading
Dashboard order/cancel/replace/amend/retry/reconnect controls
Dashboard credential input
```

## Task Accounting

| Task | Status | Evidence | PR | Notes |
| --- | --- | --- | --- | --- |
| V121-001 | PASS | PR #430 / Shrimp closeout | #430 | Finalizes v0.12.0 release-publication surface and current-release wording. |
| V121-002 | PASS | `docs/rust-cutover/evidence/V121-002.md` | #431 | Hardens release surface guard against stale current-version wording. |
| V121-003 | PASS | `docs/rust-cutover/evidence/V121-003.md` | #432 | Adds GitHub Release publication guard. |
| V121-004 | PASS | `docs/rust-cutover/evidence/V121-004.md` | #433 | Normalizes owner-run online proof wording and artifact contract. |
| V121-005 | PASS | `docs/rust-cutover/evidence/V121-005.md` | #434 | Normalizes `read_allowed`, `contract_ready`, and `online_read_allowed` artifact semantics. |
| V121-006 | PASS | `docs/rust-cutover/evidence/V121-006.md` | #435 | Denies/defers signed WebSocket user stream until listenKey lifecycle exists. |
| V121-007 | PASS | `docs/rust-cutover/evidence/V121-007.md` | #436 | Clarifies shadow strategy session as bounded local event artifact, not a long-running runtime. |
| V121-008 | PASS | `docs/rust-cutover/evidence/V121-008.md` | #437 | Adds Decimal/string shadow notional preflight and blocks f64 aggregation from live-alpha money-math claims. |
| V121-009 | PASS | `docs/rust-cutover/evidence/V121-009.md` | #438 | Prepares v0.12.1 readiness report and release notes. |

## Gate Evidence

Required local validation for the v0.12.1 readiness material:

```bash
scripts/ai/verify_release.sh release-surface-current-guard
scripts/ai/verify_release.sh v12-offline-release-gates
scripts/ai/verify_release.sh v12-manual-online-preflight
scripts/ai/verify_fast.sh
git diff --check
```

The v0.12 offline release gate verifies:

```text
production public read-only probe preflight
authenticated account snapshot preflight
redacted response-shape validation
local shadow portfolio runtime
shadow notional preflight with f64_aggregation_used=false
bounded local shadow strategy session event artifact
local read-only reconciliation
Dashboard v0.12 production shadow read-only panel
release boundary markers
network_attempted=false by default
production_orders_submitted=0
production_order_mutations_attempted=0
production_order_state_reads_attempted=0
listen_key_lifecycle_attempted=0
dashboard_order_controls_enabled=false
```

## Release Closure Status

The V121 task queue is complete after V121-009, and the owner release decision
has moved the release package to the formal publication path:

```text
formal tag = ntpro-rust-only-v0.12.1
formal release name = NTPRO Rust-only v0.12.1
formal GitHub Release = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.12.1
```

## Final Verdict

The v0.12.1 release package is the formal publication package for
`ntpro-rust-only-v0.12.1`.

Do not describe this readiness PASS as production order submission readiness,
production order mutation readiness, production order-state read readiness,
listenKey lifecycle readiness, real-funds readiness, production trading
readiness, automatic production remediation readiness, production portfolio
parity readiness, live-alpha money-math readiness, or Dashboard order-control
readiness.
