# NTPRO Rust-only v0.9.1 Release Notes

Date: 2026-06-19
Executor: Codex
Status: READY FOR OWNER RELEASE DECISION - NOT RELEASED

## Summary

`v0.9.1` is the Strategy Runtime Semantics & Audit Hardening patch for the
published v0.9 Strategy Runtime batch foundation line. It does not add Binance
testnet order submission or production trading capability.

Plain Chinese summary: v0.9.1 的开发队列已经完成，下一步需要 owner 明确决定是否
创建 `ntpro-rust-only-v0.9.1` tag 并发布 GitHub Release。这个版本不接 Binance 下单，
也不是实盘版本。它只把 v0.9.0 暴露出来的“策略运行时语义”补严：node、session、
market、risk、heartbeat、计数、kill switch、manifest、Supervisor 和 Dashboard 的
状态不能互相打架，工件损坏要能看见，同一个节点停止后也能重新启动。

## Changed

Delivered changes:

- release surface correction after the `ntpro-rust-only-v0.9.0` publication;
- unified StrategyNode config validation shared by CLI and node runtime;
- persistent StrategySession lifecycle semantics;
- coherent node/session/market state mapping;
- shared runtime counters for heartbeat, NodeStatus, Supervisor, Dashboard, and
  summary artifacts;
- split kill-switch enabled and active semantics;
- Strategy Session manifest and child artifact integrity audit;
- degraded Supervisor/Dashboard health when artifacts or state conflict;
- Supervisor restart identity hardening for same-node restart cycles;
- integration, heartbeat, shutdown, and restart smoke coverage;
- final v0.9.1 readiness and release-note material for owner decision.

## Boundary

Included:

```text
v0.9.0 release-surface correction
Strategy Runtime config contract unification
StrategySession persistent lifecycle semantics
node/session/market status coherence
runtime counter snapshot semantics
kill-switch enabled/active split
Strategy Session manifest and artifact audit
Supervisor/Dashboard degraded status visibility
Supervisor same-node restart hardening
integration and restart smoke coverage
final v0.9.1 readiness and release-note material
```

Not included:

```text
Binance testnet order placement
order cancel/replace/amend
account mutation
production Binance connectivity
production trading
real funds
Dashboard order controls
GitHub tag creation before owner approval
GitHub Release publication before owner approval
```

## Validation

Readiness evidence for this patch is recorded in:

```text
docs/rust-cutover/evidence/V091-001.md
docs/rust-cutover/evidence/V091-002.md
docs/rust-cutover/evidence/V091-003.md
docs/rust-cutover/evidence/V091-004.md
docs/rust-cutover/evidence/V091-005.md
docs/rust-cutover/evidence/V091-006.md
docs/rust-cutover/evidence/V091-007.md
docs/rust-cutover/evidence/V091-008.md
docs/rust-cutover/evidence/V091-009.md
docs/rust-cutover/evidence/V091-010.md
```

Default validation remains offline and credential-free. v0.9.1 does not require
Binance credentials and does not run online exchange probes.

Hosted validation for the final integration slice:

```text
PR #378 = merged
Rust Cutover Smoke / smoke = PASS
security-audit / changes = PASS
```

## Release Status

This document is ready for a possible `ntpro-rust-only-v0.9.1` release decision
after V091-010 merges.

This task does not create a tag and does not publish a GitHub Release.

If owner-approved publication happens later, the release name should remain:

```text
NTPRO Rust-only v0.9.1
```

and the release boundary must remain Strategy Runtime semantics and audit
hardening only: no Binance testnet order submission, no real funds, no
production trading, and no Dashboard order controls.
