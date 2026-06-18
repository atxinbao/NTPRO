# NTPRO Rust-only v0.9.0 Release Notes

Date: 2026-06-18
Executor: Codex
Status: RELEASE CANDIDATE - NOT PUBLISHED

## Summary

`v0.9.0` is scoped to the Strategy Runtime Foundation.

Plain Chinese summary: 这个版本只证明 NTPRO 的本地策略运行时地基已经成型：
`ntpro-node` 可以启动策略会话，消费 fixture/mock 行情，生成 signal、order intent、
shadow risk decision 和审计 artifact，并把这些状态提供给 supervisor 和 Dashboard
只读展示。它不是 Binance testnet 下单版本，不支持真实资金，不支持生产交易，不支持
Dashboard 下单。

## Added

- Strategy Runtime boundary and corrected version sequence:
  - `v0.9.0 = Strategy Runtime Foundation`;
  - `v0.10.0 = Binance Testnet Order Proof`.
- Strategy session config contract and validation.
- `StrategySession` lifecycle state machine.
- Built-in deterministic demo strategy runtime.
- Fixture/mock market stream input for strategy sessions.
- Signal JSONL artifact contract.
- Shadow order-intent JSONL artifact contract.
- Shadow-mode risk decision artifact contract.
- Strategy session audit log and summary artifact.
- `ntpro-node` local Strategy Session host.
- Supervisor read-only Strategy Session status.
- Dashboard read-only Strategy Runtime artifact display.
- v0.9 PR/release gates:
  - `scripts/ai/verify_v09_strategy_runtime_smoke.sh`;
  - `scripts/ai/verify_v09_shadow_mode_no_order_gate.sh`;
  - `scripts/ai/verify_release.sh v09-strategy-runtime-smoke v09-shadow-mode-no-order-gate`.

## Boundary

Included:

```text
local Strategy Session runtime
fixture/mock market stream
signal artifacts
shadow order-intent artifacts
shadow rejected risk-decision artifacts
strategy audit artifacts
supervisor read-only status
Dashboard read-only artifact/status display
v0.9 runtime/no-order verification gates
```

Not included:

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
```

## Validation

Release validation includes:

```bash
scripts/ai/verify_v09_strategy_runtime_smoke.sh
scripts/ai/verify_v09_shadow_mode_no_order_gate.sh
scripts/ai/verify_release.sh v09-strategy-runtime-smoke v09-shadow-mode-no-order-gate
scripts/ai/verify_fast.sh
```

The v0.9 gates are local/offline and do not require Binance credentials.

## Release Status

This document is a release-candidate note. The formal tag and GitHub Release
are not published by V090-013.

Expected manual-gated release identity, if approved later:

```text
Tag: ntpro-rust-only-v0.9.0
Release name: NTPRO Rust-only v0.9.0
```

V090-014 owns final release closure. Do not create the tag or publish the
GitHub Release without explicit owner approval.

## Next Milestone

`v0.10.0` is the earliest milestone for Binance testnet order lifecycle proof.
That work must remain separate from the v0.9 Strategy Runtime Foundation.
