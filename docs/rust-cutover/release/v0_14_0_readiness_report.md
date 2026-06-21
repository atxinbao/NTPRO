# NTPRO v0.14.0 Production Order-State Read-Only + Live Alpha Dry-Run Readiness Report

Date: 2026-06-22
Executor: Codex
Milestone: `ntpro-rust-only-v0.14.0`
Status: PASS - RELEASED

## Summary

`v0.14.0` is the Production Order-State Read-Only + Live Alpha Dry-Run release
line. It introduces a narrow owner-gated production order-state read-only proof
boundary and local live-alpha dry-run evidence, but it does not authorize
production order submission or production order mutation.

Plain Chinese summary: v0.14.0 可以开始把“生产订单状态只读查询”和“live-alpha
干跑证据”串起来了。它默认仍然离线、fail-closed。它不是实盘下单版本，不允许下单、
撤单、改单、重试、纠错、listenKey、真实资金交易，也不允许 Dashboard 下单控件。

## Product Claim

```text
capability = Production Order-State Read-Only + Live Alpha Dry-Run
release tag = ntpro-rust-only-v0.14.0
release name = NTPRO Rust-only v0.14.0
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.14.0
default execution posture = offline fail-closed
production order-state read proof = owner-gated GET only
live-alpha order flow = dry-run only, no submission
live-alpha risk preflight = local artifact evaluation only
Dashboard live-alpha panel = read-only evidence summary
production order submission = not included
production order mutation = not included
production cancel/replace/amend/retry/correction = not included
listenKey lifecycle = not included
real funds = not included
production trading = not included
Dashboard order controls = not included
```

## Included

```text
production order-state read-only boundary
owner-gated production order-state GET proof with default offline preflight
supervisor-managed shadow runtime evidence
live-alpha dry-run order gate artifact
live-alpha local risk preflight artifact
live-alpha reconciliation golden traces
Dashboard live-alpha dry-run read-only panel
v0.14 release gate wiring
v0.14 readiness and release-note material
```

## Not Included

```text
production order submission
production cancel, replace, amend, retry, correction, or flatten
production order-test submission
execution adapter calls for live-alpha dry-run
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

## Prerequisite Accounting

| PR | Status | Classification | Evidence | Capability expansion |
| --- | --- | --- | --- | --- |
| #459 | PASS | V131-007 v0.13.1 readiness and release notes | `docs/rust-cutover/evidence/V131-007.md` | No v0.14 capability; prerequisite release closure |

## Merged PR Accounting

| PR | Status | Classification | Evidence | Capability expansion |
| --- | --- | --- | --- | --- |
| #460 | PASS | V140-000 order-state read-only boundary | `docs/rust-cutover/evidence/V140-000.md` | Boundary only |
| #461 | PASS | V140-001 owner-gated production order-state GET proof | `docs/rust-cutover/evidence/V140-001.md` | Read-only owner-gated proof only |
| #462 | PASS | V140-002 supervisor shadow runtime evidence | `docs/rust-cutover/evidence/V140-002.md` | Local shadow lifecycle evidence only |
| #463 | PASS | V140-003 live-alpha dry-run order gate | `docs/rust-cutover/evidence/V140-003.md` | Dry-run intent only, no submission |
| #464 | PASS | V140-004 live-alpha risk preflight | `docs/rust-cutover/evidence/V140-004.md` | Local preflight only, no execution |
| #465 | PASS | V140-005 reconciliation golden traces | `docs/rust-cutover/evidence/V140-005.md` | Replay evidence only |
| #466 | PASS | V140-006 Dashboard live-alpha dry-run panel | `docs/rust-cutover/evidence/V140-006.md` | Read-only Dashboard status only |
| #467 | PASS | V140-007 v0.14 release gates | `docs/rust-cutover/evidence/V140-007.md` | Gate wiring only |
| #468 | PASS | V140-008 readiness and release notes | `docs/rust-cutover/evidence/V140-008.md` | Release accounting only |

## Gate Evidence

Local validation recorded across V140 evidence:

```text
scripts/ai/verify_v14_order_state_readonly_proof.sh = PASS
scripts/ai/verify_v14_supervisor_shadow_runtime.sh = PASS
scripts/ai/verify_v14_live_alpha_dry_run_order_gate.sh = PASS
scripts/ai/verify_v14_live_alpha_risk_preflight.sh = PASS
scripts/ai/verify_v14_release_gates.sh = PASS
scripts/ai/verify_release.sh v14-release-gates = PASS
python3 scripts/ai/golden_trace_runner.py tests/golden/live_alpha_reconciliation_schema.jsonl --mode validate-only = PASS
cargo test -p nautilus-cli --test golden_trace_live_alpha_reconciliation = PASS
cargo test -p nautilus-cli dashboard --lib = PASS
scripts/ai/verify_fast.sh = PASS
git diff --check = PASS
```

Hosted PR smoke evidence:

```text
PR #466 Rust Cutover Smoke = PASS, run 27918395268
PR #467 Rust Cutover Smoke = PASS, run 27919248124
PR #467 security-audit checks = PASS, run 27919248127
PR #468 Rust Cutover Smoke = PASS, run 27919653653
```

## Release Closure Status

The V140 task queue is complete after V140-008, and the owner release decision
has moved the release package to the formal publication path:

```text
formal tag = ntpro-rust-only-v0.14.0
formal release name = NTPRO Rust-only v0.14.0
formal GitHub Release = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.14.0
```

## Final Verdict

The v0.14.0 release package is the formal publication package for
`ntpro-rust-only-v0.14.0`.

Do not describe this readiness PASS as production order submission readiness,
production order mutation readiness, production cancel/replace/amend/retry/
correction readiness, listenKey lifecycle readiness, real-funds readiness,
production trading readiness, automatic production remediation readiness, or
Dashboard order-control readiness.
