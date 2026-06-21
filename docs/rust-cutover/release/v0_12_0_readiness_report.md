# NTPRO v0.12.0 Production Online Read-Only + Persistent Shadow Readiness Report

Date: 2026-06-21
Executor: Codex
Milestone: `ntpro-rust-only-v0.12.0`
Status: PASS - RELEASED

## Summary

`v0.12.0` is the Production Online Read-Only + Persistent Shadow release line
after the published `v0.11.0` Production Read-Only Contract + Offline Shadow
Portfolio milestone. It advances the source tree from offline contracts into
implemented owner-gated production `GET` read-only paths and local persistent
shadow artifact evidence.

Plain Chinese summary: v0.12.0 不是实盘交易版本。它把生产环境只读 GET 的
owner-gated 路径、本地 fail-closed 预检和 persistent shadow 证据打通。默认发版 gate
不要求真实联网成功证明；真实生产只读成功证明只允许 owner 手动运行并留下可选证据。
它不下生产订单，不撤单、不改单、不碰真实资金，Dashboard 也没有下单按钮。

## Product Claim

```text
capability = Production Online Read-Only + Persistent Shadow
current published release before v0.12.0 = ntpro-rust-only-v0.11.0
release tag = ntpro-rust-only-v0.12.0
release name = NTPRO Rust-only v0.12.0
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.12.0
default execution posture = offline fail-closed
production public online read = implemented owner-gated GET-only path
production authenticated account snapshot online read = implemented owner-gated GET-only path
default release-gate online proof = not required, fail-closed preflight only
owner-run successful online proof = optional evidence artifact only
production read-only response shape = redacted and bounded evidence only
shadow portfolio runtime = local artifact only
shadow notional preflight = Decimal/string evidence only, not risk/execution-grade money math
shadow strategy session = bounded local JSONL event artifact only, not a long-running runtime
production read-only reconciliation = local classification only
Dashboard production shadow status = read-only display only
production order submission = not included
production order mutation = not included
production order-state reads = not included
listenKey lifecycle = not included
signed WebSocket user stream = deferred because listenKey lifecycle is not included
real funds = not included
production trading = not included
Dashboard order controls = not included
```

## Included

```text
v0.12 production online read-only boundary
implemented owner-gated production public GET read-only path
implemented owner-gated authenticated production account snapshot GET path
optional owner-run successful online proof artifact contract
redacted account response-shape validation
local shadow portfolio runtime artifact
Decimal/string notional preflight inside the local shadow portfolio runtime artifact
bounded local shadow strategy session JSONL event artifact
local production read-only reconciliation classifications
Dashboard v0.12 production shadow read-only panel
offline release gate bundle
manual-online fail-closed preflight gate
readiness and release-note material
```

## Not Included

```text
production order submission
production cancel, replace, amend, retry, or correction orders
production open-order or order-state reads
listenKey creation, keepalive, or close lifecycle
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

| Task | Status | Evidence | Notes |
| --- | --- | --- | --- |
| V120-000 | PASS | `docs/rust-cutover/evidence/V120-000.md` | Defines the Production Online Read-Only + Persistent Shadow boundary. |
| V120-001 | PASS | `docs/rust-cutover/evidence/V120-001.md` | Adds implemented owner-gated production public read-only GET path behavior. |
| V120-002 | PASS | `docs/rust-cutover/evidence/V120-002.md` | Adds implemented owner-gated authenticated production account snapshot GET path behavior. |
| V120-003 | PASS | `docs/rust-cutover/evidence/V120-003.md` | Adds redacted production account response-shape evidence. |
| V120-004 | PASS | `docs/rust-cutover/evidence/V120-004.md` | Adds local shadow portfolio runtime artifacts. |
| V120-005 | PASS | `docs/rust-cutover/evidence/V120-005.md` | Adds bounded shadow strategy session JSONL event artifacts. |
| V120-006 | PASS | `docs/rust-cutover/evidence/V120-006.md` | Adds local production read-only reconciliation classifications. |
| V120-007 | PASS | `docs/rust-cutover/evidence/V120-007.md` | Adds Dashboard v0.12 production shadow read-only panel. |
| V120-008 | PASS | `docs/rust-cutover/evidence/V120-008.md` | Wires v0.12 offline release gates and manual-online fail-closed preflight. |
| V120-009 | PASS CANDIDATE | `docs/rust-cutover/evidence/V120-009.md` | Prepares v0.12 readiness and release notes for owner decision. |
| V121-008 | PASS CANDIDATE | `docs/rust-cutover/evidence/V121-008.md` | Adds Decimal/string shadow notional preflight evidence and blocks f64 aggregation from live-alpha money-math claims. |

## Gate Evidence

Required local validation for the v0.12 readiness material:

```bash
scripts/ai/verify_release.sh v12-offline-release-gates v12-manual-online-preflight
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

The v0.12 manual-online preflight verifies:

```text
manual-online request path remains blocked without NTPRO_V12_MANUAL_ONLINE=1
network_attempted=false
owner-run successful online proof is not required for CI or default release gates
```

## Release Closure Status

The V120 task queue is complete after V120-009, and the owner release decision
has moved the release package to the formal publication path:

```text
formal tag = ntpro-rust-only-v0.12.0
formal release name = NTPRO Rust-only v0.12.0
formal GitHub Release = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.12.0
```

## Next Track

`v0.13.0` is the earliest possible Guarded Live Alpha candidate. Guarded Live
Alpha is not part of v0.12.0 and must require a separate owner-approved scope
decision before any production order mutation or live trading capability can be
claimed.

## Final Verdict

The v0.12.0 release package is the formal publication package for
`ntpro-rust-only-v0.12.0`.

Do not describe this readiness PASS as production order submission readiness,
real-funds readiness, production trading readiness, automatic production
remediation readiness, production portfolio parity readiness, or Dashboard
order-control readiness.
