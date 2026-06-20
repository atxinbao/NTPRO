# NTPRO v0.12.0 Production Online Read-Only + Persistent Shadow Readiness Report

Date: 2026-06-21
Executor: Codex
Milestone: `v0.12.0`
Status: READY FOR OWNER RELEASE DECISION

## Summary

`v0.12.0` is the Production Online Read-Only + Persistent Shadow candidate
release line after the published `v0.11.0` Production Read-Only Contract +
Offline Shadow Portfolio milestone. It advances the source tree from offline
contracts into owner-gated production `GET` read-only proof paths and local
persistent shadow evidence.

Plain Chinese summary: v0.12.0 不是实盘交易版本。它只把生产环境的“只读 GET 证明”
和本地 persistent shadow 证据打通：公开只读探测、认证账户快照只读、response shape
脱敏检查、shadow portfolio runtime、shadow strategy session、read-only
reconciliation 和 Dashboard 只读展示。它不下生产订单，不撤单、不改单、不碰真实资金，
Dashboard 也没有下单按钮。

## Product Claim

```text
capability = Production Online Read-Only + Persistent Shadow
current published release before v0.12.0 = ntpro-rust-only-v0.11.0
candidate tag = ntpro-rust-only-v0.12.0
default execution posture = offline fail-closed
production public online read = owner-gated GET-only proof path
production authenticated account snapshot online read = owner-gated GET-only proof path
production read-only response shape = redacted and bounded evidence only
shadow portfolio runtime = local artifact only
shadow strategy session = local persistent event artifact only
production read-only reconciliation = local classification only
Dashboard production shadow status = read-only display only
production order submission = not included
production order mutation = not included
production order-state reads = not included
listenKey lifecycle = not included
real funds = not included
production trading = not included
Dashboard order controls = not included
tag creation = not performed by this readiness PR
GitHub Release publication = not performed by this readiness PR
```

## Included

```text
v0.12 production online read-only boundary
owner-gated production public GET read-only probe
owner-gated authenticated production account snapshot GET proof
redacted account response-shape validation
local shadow portfolio runtime artifact
local persistent shadow strategy session event stream
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
exchange-confirmed shadow fills or positions
raw account response, raw balances, raw credentials, signatures, signed query, or signed URL persistence
real funds
production trading
Dashboard order/cancel/replace/amend/retry/reconnect controls
Dashboard credential input
tag creation
GitHub Release publication
```

## Task Accounting

| Task | Status | Evidence | Notes |
| --- | --- | --- | --- |
| V120-000 | PASS | `docs/rust-cutover/evidence/V120-000.md` | Defines the Production Online Read-Only + Persistent Shadow boundary. |
| V120-001 | PASS | `docs/rust-cutover/evidence/V120-001.md` | Adds owner-gated production public read-only GET probe behavior. |
| V120-002 | PASS | `docs/rust-cutover/evidence/V120-002.md` | Adds owner-gated authenticated production account snapshot GET proof. |
| V120-003 | PASS | `docs/rust-cutover/evidence/V120-003.md` | Adds redacted production account response-shape evidence. |
| V120-004 | PASS | `docs/rust-cutover/evidence/V120-004.md` | Adds local shadow portfolio runtime artifacts. |
| V120-005 | PASS | `docs/rust-cutover/evidence/V120-005.md` | Adds persistent shadow strategy session event artifacts. |
| V120-006 | PASS | `docs/rust-cutover/evidence/V120-006.md` | Adds local production read-only reconciliation classifications. |
| V120-007 | PASS | `docs/rust-cutover/evidence/V120-007.md` | Adds Dashboard v0.12 production shadow read-only panel. |
| V120-008 | PASS | `docs/rust-cutover/evidence/V120-008.md` | Wires v0.12 offline release gates and manual-online fail-closed preflight. |
| V120-009 | PASS CANDIDATE | `docs/rust-cutover/evidence/V120-009.md` | Prepares v0.12 readiness and release notes for owner decision. |

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
local persistent shadow strategy session
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
owner-gated online proof is not required for CI
```

## Release Decision

This report is readiness material only. It does not create the
`ntpro-rust-only-v0.12.0` tag and does not publish a GitHub Release.

Before publishing `ntpro-rust-only-v0.12.0`, the owner should verify:

```text
main includes all V120-000 through V120-009 PRs
GitHub hosted Rust Cutover Smoke is PASS on the release candidate commit
README / ROADMAP / versioning / release notes agree on v0.12 scope
GitHub open PR count is acceptable for release closure
worktree is clean
```

## Next Track

`v0.13.0` is the earliest possible Guarded Live Alpha candidate. Guarded Live
Alpha is not part of v0.12.0 and must require a separate owner-approved scope
decision before any production order mutation or live trading capability can be
claimed.

## Final Verdict

`v0.12.0` is ready for owner release decision as a Production Online Read-Only
and Persistent Shadow candidate after V120-000 through V120-009 pass and merge.

Do not describe this readiness PASS as production order submission readiness,
real-funds readiness, production trading readiness, automatic production
remediation readiness, production portfolio parity readiness, or Dashboard
order-control readiness.
