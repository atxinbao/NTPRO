# NTPRO v0.9.0 Strategy Runtime Boundary

Date: 2026-06-18
Executor: Codex
Status: ACTIVE PLANNING BOUNDARY

## Summary

`v0.9.0` is the Strategy Runtime Foundation milestone. It makes `ntpro-node` a
headless strategy runtime host for local fixture/mock/sandbox input and
read-only artifacts. It does not prove Binance testnet order lifecycle and does
not enable live order submission.

Plain Chinese summary: v0.9.0 只做“策略运行时地基”。目标是让 `ntpro-node`
能加载策略会话配置、跑本地 fixture/mock 输入、产出信号、订单意图、风控决策和审计文件。
它不是 Binance testnet 下单版本，不支持真实资金，不支持生产交易，也不允许 Dashboard 下单。

## Corrected Version Sequence

```text
v0.9.0  = Strategy Runtime Foundation
v0.10.0 = Binance Testnet Order Proof
v0.11.0 = Production Read-Only + Shadow
v0.12.0 = Guarded Live Alpha
```

The earlier idea that `v0.9.0` should be the Binance testnet order lifecycle
proof is superseded. Testnet order proof is now explicitly deferred to
`v0.10.0`.

## Included in v0.9.0

`v0.9.0` may include:

- strategy session configuration validation;
- `StrategySession` lifecycle state;
- local fixture/mock market stream input;
- built-in deterministic demo strategy runtime;
- signal artifacts;
- order intent artifacts;
- shadow-mode risk decision artifacts;
- strategy session audit/event artifacts;
- `ntpro-node` headless strategy runtime hosting;
- supervisor read-only strategy runtime status;
- Dashboard read-only strategy runtime status and artifact display;
- v0.9 strategy runtime smoke/release verification wiring;
- v0.9 readiness report and release notes.

## Excluded from v0.9.0

`v0.9.0` must not include:

- Binance testnet order submission;
- order cancel/replace/amend;
- production order submission;
- production Binance connectivity as a trading surface;
- real funds;
- production trading;
- strategy-driven live exchange execution;
- Dashboard order buttons or order controls;
- Dashboard credential input;
- automatic network or authenticated exchange probes.

## Strategy Session Lifecycle

The v0.9 strategy session lifecycle is a local runtime lifecycle, not an
exchange execution lifecycle:

```text
created
validated
starting
running
paused
risk_halted
stopping
stopped
failed
```

The lifecycle may write status and event artifacts that are consumed by the
supervisor and Dashboard. It must not directly create exchange orders.

## Shadow Mode

Shadow mode is the default and required v0.9 execution posture:

- strategies may emit signals;
- signals may become order intents;
- order intents must pass through risk decision;
- risk decision may explain what would have happened;
- actual submission remains disabled.

`order_submission = disabled` is the default v0.9 product rule.

## Artifact Contract

The v0.9 artifact surface is local, auditable, and read-only from Dashboard:

```text
strategy/session_status.json
strategy/events.jsonl
strategy/market_status.json
strategy/market_events.jsonl
strategy/signal.jsonl
strategy/order_intent.jsonl
strategy/risk_decision.jsonl
strategy/summary.json
```

Every artifact should include a schema version once implemented. Artifact
contents must describe local strategy runtime behavior and must not claim that
orders were submitted.

## Responsibility Boundaries

### `ntpro-node`

`ntpro-node` may host a local strategy session, drive fixture/mock market input,
run the demo strategy, and write artifacts.

`ntpro-node` must not submit exchange orders in v0.9.

### Supervisor

Supervisor may expose read-only strategy session state and artifact locations.
It must not provide order controls in v0.9.

### Dashboard

Dashboard may display read-only strategy runtime status and artifacts. It must
not provide order buttons, credential entry, live execution controls, or
Dashboard-started exchange probes in v0.9.

### Risk Gate

Risk gate may evaluate order intents and write decisions. In v0.9, actual order
submission remains disabled even when a decision records hypothetical acceptance.

## Deferred to v0.10.0

The following belongs to `v0.10.0` or later:

- Binance testnet order submission;
- order lifecycle proof;
- cancel/replace/amend proof;
- exchange execution adapter submission wiring;
- order lifecycle golden trace;
- testnet order redaction and audit evidence;
- any user-facing claim that NTPRO can place exchange orders.

## Release Rule

The v0.9 release may be prepared only after the strategy runtime smoke,
shadow-mode no-order gate, readiness report, and release notes are complete.

Creating the `ntpro-rust-only-v0.9.0` tag or publishing a GitHub Release remains
manual-gated and requires explicit owner approval.
