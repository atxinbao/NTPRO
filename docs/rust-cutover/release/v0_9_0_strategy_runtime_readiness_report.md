# NTPRO v0.9.0 Strategy Runtime Readiness Report

Date: 2026-06-18
Executor: Codex
Milestone: `ntpro-rust-only-v0.9.0`
Status: RELEASED

## Summary

`v0.9.0` is released as the Strategy Runtime Foundation milestone. It proves
the local headless strategy runtime path can
load a strategy session, consume fixture/mock market input, emit signal
artifacts, emit shadow order-intent artifacts, write shadow risk decisions,
surface supervisor status, and display read-only Dashboard artifacts.

Plain Chinese summary: v0.9.0 已完成 owner-approved GitHub Release 发布闭环，
但它只代表“策略运行时地基”准备好了：本地 `ntpro-node` 能跑策略会话、吃 fixture/mock 行情、生成信号、
订单意图、风控拒绝决策和审计文件，并能被 supervisor / Dashboard 只读展示。它不是
Binance testnet 下单证明，不支持真实资金，不支持生产交易，也不允许 Dashboard 下单。

## Product Claim

```text
capability = local Strategy Runtime Foundation
runtime host = ntpro-node
market input = fixture/mock local stream
strategy output = signal artifacts
order planning output = shadow order-intent artifacts
risk output = shadow rejected risk-decision artifacts
status surface = supervisor read-only fields
dashboard surface = read-only artifact/status display
order submission = disabled
Binance testnet order lifecycle proof = deferred to v0.10.0
production Binance connectivity = not included
real funds = not included
production trading = not included
Dashboard order controls = not included
```

## Task Gate Matrix

| Task | Status | Evidence | Notes |
| --- | --- | --- | --- |
| V090-000 | PASS | `docs/rust-cutover/evidence/V090-000.md` | Corrected boundary: v0.9 is Strategy Runtime Foundation, v0.10 is Binance testnet order proof. |
| V090-001 | PASS | `docs/rust-cutover/evidence/V090-001.md` | Strategy session config contract and CLI validation path. |
| V090-002 | PASS | `docs/rust-cutover/evidence/V090-002.md` | `StrategySession` state machine. |
| V090-003 | PASS | `docs/rust-cutover/evidence/V090-003.md` | Built-in deterministic demo strategy runtime. |
| V090-004 | PASS | `docs/rust-cutover/evidence/V090-004.md` | Fixture/mock market stream adapter. |
| V090-005 | PASS | `docs/rust-cutover/evidence/V090-005.md` | Signal artifact JSONL contract. |
| V090-006 | PASS | `docs/rust-cutover/evidence/V090-006.md` | Shadow order-intent artifact contract with submission disabled. |
| V090-007 | PASS | `docs/rust-cutover/evidence/V090-007.md` | Shadow-mode risk decision gate. |
| V090-008 | PASS | `docs/rust-cutover/evidence/V090-008.md` | Strategy session event/audit log and summary. |
| V090-009 | PASS | `docs/rust-cutover/evidence/V090-009.md` | `ntpro-node` hosts the local Strategy Session and writes artifacts. |
| V090-010 | PASS | `docs/rust-cutover/evidence/V090-010.md` | Supervisor exposes read-only Strategy Session status. |
| V090-011 | PASS | `docs/rust-cutover/evidence/V090-011.md` | Dashboard displays Strategy Runtime artifacts in read-only mode. |
| V090-012 | PASS | `docs/rust-cutover/evidence/V090-012.md` | PR/release smoke gates for strategy runtime and no-order boundary. |
| V090-013 | PASS | `docs/rust-cutover/evidence/V090-013.md` | Readiness report and release notes prepared. |
| V090-014 | PASS | `docs/rust-cutover/evidence/V090-014.md` | Owner-approved tag and formal GitHub Release published. |

## Validation Evidence

The release candidate depends on these gates:

```bash
scripts/ai/verify_v09_strategy_runtime_smoke.sh
scripts/ai/verify_v09_shadow_mode_no_order_gate.sh
scripts/ai/verify_release.sh v09-strategy-runtime-smoke v09-shadow-mode-no-order-gate
scripts/ai/verify_fast.sh
```

Known local evidence from V090-012:

```text
v09_strategy_runtime_smoke_assertions status=ok signals=2 intents=2 risk_decisions=2 actual_submission_count=0
v09_shadow_mode_no_order_gate_assertions status=ok order_intents=2 risk_decisions=2 actual_submission_count=0
verify_release v09-strategy-runtime-smoke v09-shadow-mode-no-order-gate = PASS
```

Hosted PR evidence:

- V090-009 PR #364: Rust Cutover Smoke PASS and security-audit PASS.
- V090-010 PR #365: Rust Cutover Smoke PASS and security-audit PASS.
- V090-011 PR #366: Rust Cutover Smoke PASS and security-audit PASS.
- V090-012 PR #367: Rust Cutover Smoke PASS and security-audit PASS.
- V090-013 PR #368: Rust Cutover Smoke PASS and security-audit PASS.

Hosted release evidence:

```text
Workflow-dispatch release gate: https://github.com/atxinbao/NTPRO/actions/runs/27738080665
Tag-triggered release gate: https://github.com/atxinbao/NTPRO/actions/runs/27742550316
Both release gates passed 34/34 jobs at release commit 83b333503a5c8e8436c98f54a4d94c4a50f919a8.
```

## Out Of Scope For v0.9.0

```text
Binance testnet order submission
order cancel/replace/amend
production order submission
production Binance trading surface
real funds
production trading
strategy-driven live exchange execution
Dashboard order buttons or order controls
Dashboard credential input
automatic network or authenticated exchange probes
prebuilt binary or Docker delivery
```

## Release Closure

V090-014 is complete. The owner-approved release closure published:

```text
Tag: ntpro-rust-only-v0.9.0
Release name: NTPRO Rust-only v0.9.0
Release URL: https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.9.0
Release commit: 83b333503a5c8e8436c98f54a4d94c4a50f919a8
Published at: 2026-06-18T07:57:04Z
Draft: false
Prerelease: false
```

## Final Verdict

`v0.9.0` is released as the Strategy Runtime Foundation milestone.

Do not describe this PASS as Binance testnet order readiness, real-funds
readiness, production trading readiness, or Dashboard order-control readiness.
