# NTPRO v0.13.0 Guarded Live Alpha Preflight Readiness Report

Date: 2026-06-21
Executor: Codex
Milestone: `ntpro-rust-only-v0.13.0`
Status: PASS - RELEASED

## Summary

`v0.13.0` is the Guarded Live Alpha Preflight release. It closes the V130 task
queue by proving the system can collect live-alpha preflight evidence without
crossing into production order mutation, production order-state reads,
listenKey lifecycle, real funds, production trading, or Dashboard order
controls.

Plain Chinese summary: v0.13.0 可以理解成“实盘 alpha 前置检查包”，不是实盘
下单系统。它把 shadow session、只读 proof pack、kill switch、Dashboard 边界、
Decimal 金额边界和 no-production-mutation 门禁都补齐，但默认仍然离线、fail-closed，
不碰生产订单和真实资金。

## Product Claim

```text
capability = Guarded Live Alpha Preflight
current published release before v0.13.0 = ntpro-rust-only-v0.12.1
release tag = ntpro-rust-only-v0.13.0
release name = NTPRO Rust-only v0.13.0
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.13.0
default execution posture = offline fail-closed
shadow preflight session = local artifact evidence only
owner-run online proof pack = optional, read-only, owner-gated, fail-closed by default
kill switch = dry-run/manual approval artifact only
Dashboard controls = read-only/status and local ops boundary only
amount preflight = Decimal/string-only evidence
no-production-mutation release gate = included
production order submission = not included
production order mutation = not included
production order-state reads = not included
listenKey lifecycle = not included
real funds = not included
production trading = not included
risk/execution-grade live-alpha money math = not included
formal tag = ntpro-rust-only-v0.13.0
formal GitHub Release = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.13.0
```

## Included

```text
Guarded Live Alpha Preflight scope decision
local shadow preflight session heartbeat/stop/stale-data evidence
owner-gated production online read-only proof-pack wrapper
kill-switch dry-run/manual approval artifact
trader/ops Dashboard read-only/control boundary evidence
Decimal/string-only amount preflight evidence
v13 no-production-mutation PR/release gate
release-surface current guard update for v0.13.0
v0.13.0 readiness and release-note material
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
risk/execution-grade live-alpha money math
exchange-confirmed shadow fills or positions
raw account response, raw balances, raw credentials, signatures, signed query, or signed URL persistence
real funds
production trading
Dashboard order/cancel/replace/amend/retry/reconnect controls
Dashboard credential input
```

## Merged PR Accounting

| PR | Status | Classification | Evidence | Notes |
| --- | --- | --- | --- | --- |
| #440 | PASS | Source-tree hardening delta | PR #440 | Post-v0.12.1 release-publication guard fix. Included in source tree, not a v0.13 capability expansion. |
| #441 | PASS | V130-001 | `docs/rust-cutover/evidence/V130-001.md` | Guarded Live Alpha Preflight scope decision. |
| #442 | PASS | V130-002 | `docs/rust-cutover/evidence/V130-002.md` | Local shadow preflight session loop. |
| #443 | PASS | V130-003 | `docs/rust-cutover/evidence/V130-003.md` | Owner-run production online read-only proof-pack wrapper. |
| #444 | PASS | V130-004 | `docs/rust-cutover/evidence/V130-004.md` | Kill-switch dry-run/manual approval artifact. |
| #445 | PASS | V130-005 | `docs/rust-cutover/evidence/V130-005.md` | Trader/ops Dashboard control boundary. |
| #446 | PASS | V130-006 | `docs/rust-cutover/evidence/V130-006.md` | Decimal/string amount boundary. |
| #447 | PASS | V130-007 | `docs/rust-cutover/evidence/V130-007.md` | No-production-mutation PR/release gate. |

## Gate Evidence

Required local validation for the v0.13.0 readiness material:

```bash
scripts/ai/verify_release.sh release-surface-current-guard
scripts/ai/verify_release.sh v13-no-production-mutation-gate
scripts/ai/verify_release.sh v12-offline-release-gates
scripts/ai/verify_release.sh v12-manual-online-preflight
scripts/ai/verify_fast.sh
git diff --check
```

The v0.13 no-production-mutation gate verifies:

```text
network_default_offline=true
production_orders_submitted=0
production_order_mutations_attempted=0
dashboard_order_controls_enabled=false
production_order_submission_allowed=false
production_order_mutation_allowed=false
production_order_state_reads_allowed=false
listen_key_lifecycle_allowed=false
production_reconnect_allowed=false
live_alpha_money_math_ready=false
risk_or_execution_grade=false
```

Hosted evidence before release-surface closeout:

```text
PR #447 Rust Cutover Smoke = PASS
run = 27901502248
job = 82562520046
merge commit = 8b244ef178e47464843f54979e5f936205440287
```

Final release evidence must include the hosted Rust Cutover Release Gate for the
formal `ntpro-rust-only-v0.13.0` tag.

Formal release closure evidence:

```text
formal tag = ntpro-rust-only-v0.13.0
formal release name = NTPRO Rust-only v0.13.0
formal GitHub Release = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.13.0
GitHub Release draft = false
GitHub Release prerelease = false
tag commit = 31115c269e17ef7b42ac86807b408b642cfea867
origin/main commit at release-closure verification = 31115c269e17ef7b42ac86807b408b642cfea867
tag/main relationship after evidence-only PRs = tag remains reachable from origin/main
hosted Rust Cutover Release Gate run = 27905068904
hosted Rust Cutover Release Gate URL = https://github.com/atxinbao/NTPRO/actions/runs/27905068904
hosted Rust Cutover Release Gate conclusion = success
hosted Rust Cutover Release Gate jobs = 43/43 success
```

## Release Closure Status

The V130 task queue is complete after V130-007. The owner release decision has
moved the release package to the formal publication path:

```text
formal tag = ntpro-rust-only-v0.13.0
formal release name = NTPRO Rust-only v0.13.0
formal GitHub Release = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.13.0
```

## Final Verdict

The v0.13.0 release package is the formal publication package for
`ntpro-rust-only-v0.13.0`.

Do not describe this readiness PASS as production order submission readiness,
production order mutation readiness, production order-state read readiness,
listenKey lifecycle readiness, real-funds readiness, production trading
readiness, automatic production remediation readiness, production portfolio
parity readiness, risk/execution-grade live-alpha money-math readiness, or
Dashboard order-control readiness.
