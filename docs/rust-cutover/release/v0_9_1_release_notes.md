# NTPRO Rust-only v0.9.1 Release Notes

Date: 2026-06-18
Executor: Codex
Status: PLANNING - NOT READY FOR RELEASE

## Summary

`v0.9.1` is the Strategy Runtime Semantics & Audit Hardening patch for the
published v0.9 Strategy Runtime batch foundation line. It does not add Binance
testnet order submission or production trading capability.

Plain Chinese summary: v0.9.1 不接 Binance 下单，也不是实盘版本。它要先把
v0.9.0 暴露出来的“策略运行时语义”补严：node 还在跑时，session/market/risk/heartbeat
状态不能互相打架；计数不能回退；kill switch 字段要表达清楚；策略工件要有 manifest
和一致性审计；Supervisor/Dashboard 要能看见损坏或缺失工件。v0.10.0 才是最早可以
规划 Binance testnet order proof 的版本。

## Changed

Planned changes:

- release surface correction after the `ntpro-rust-only-v0.9.0` publication;
- unified StrategyNode config validation shared by CLI and node runtime;
- persistent StrategySession lifecycle semantics;
- coherent node/session/market state mapping;
- shared runtime counters for heartbeat, NodeStatus, Supervisor, Dashboard, and
  summary artifacts;
- split kill-switch enabled and active semantics;
- Strategy Session manifest and child artifact integrity audit;
- degraded Supervisor/Dashboard health when artifacts or state conflict;
- integration, heartbeat, shutdown, and restart smoke coverage;
- final v0.9.1 readiness and release closure after all V091 tasks pass.

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
```

Default validation remains offline and credential-free. v0.9.1 does not require
Binance credentials and does not run online exchange probes.

## Release Status

This document is a planning draft for a possible `ntpro-rust-only-v0.9.1`
release decision after V091-002 through V091-010 complete.

This task does not create a tag and does not publish a GitHub Release.

If owner-approved publication happens later, the release name should remain:

```text
NTPRO Rust-only v0.9.1
```

and the release boundary must remain Strategy Runtime semantics and audit
hardening only: no Binance testnet order submission, no real funds, no
production trading, and no Dashboard order controls.
