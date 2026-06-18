# NTPRO v0.9.0 Strategy Runtime Readiness Report

Date: 2026-06-18
Executor: Codex
Milestone: `ntpro-rust-only-v0.9.0`
Status: RELEASE CANDIDATE - NOT PUBLISHED

## Summary

`v0.9.0` is ready for owner release-closure review as the Strategy Runtime
Foundation milestone. It proves the local headless strategy runtime path can
load a strategy session, consume fixture/mock market input, emit signal
artifacts, emit shadow order-intent artifacts, write shadow risk decisions,
surface supervisor status, and display read-only Dashboard artifacts.

Plain Chinese summary: v0.9.0 可以进入发布决策，但它只代表“策略运行时地基”
准备好了：本地 `ntpro-node` 能跑策略会话、吃 fixture/mock 行情、生成信号、
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
| V090-013 | PASS PENDING PR MERGE | `docs/rust-cutover/evidence/V090-013.md` | Readiness report and release notes prepared. |
| V090-014 | MANUAL GATE | `docs/rust-cutover/tasks/V090-014.md` | Tag and GitHub Release closure require explicit owner approval. |

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

V090-013 must also pass hosted PR checks before it can be treated as merged
readiness evidence.

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

V090-014 is not complete yet. Creating the `ntpro-rust-only-v0.9.0` tag or
publishing the GitHub Release remains manual-gated and requires explicit owner
approval.

Until V090-014 is approved and completed, this report is release-candidate
readiness evidence only, not a publication record.

## Final Verdict

`v0.9.0` is ready to enter release-closure review after V090-013 PR merge and
hosted checks pass.

Do not describe this PASS as Binance testnet order readiness, real-funds
readiness, production trading readiness, or Dashboard order-control readiness.
