# v0.9.0 Shadow Risk Decision Gate

Date: 2026-06-18
Executor: Codex

## Purpose

V090-007 defines the local shadow-mode risk decision gate for the v0.9 Strategy
Runtime Foundation. The gate evaluates every local order intent and emits a
risk decision artifact, but it never forwards an order to execution adapters or
exchange APIs.

Plain Chinese summary: 这个文件定义的是“风控怎么看策略意图”的本地记录格式。v0.9.0
阶段所有 risk decision 都默认拒绝真实提交，不能把它理解成 Binance testnet 下单、
实盘下单或生产交易能力。

## Artifact

```text
strategy/risk_decision.jsonl
schema_version = ntpro.v09_risk_decision.v1
```

Each line must be valid JSON and represents one local risk decision for one
local order intent.

## Required Fields

```json
{
  "schema_version": "ntpro.v09_risk_decision.v1",
  "session_id": "btc-ema-shadow-001",
  "strategy_id": "ema_cross_btcusdt_v1",
  "decision_id": "risk:btc-ema-shadow-001:ema_cross_btcusdt_v1:5",
  "intent_id": "btc-ema-shadow-001:ema_cross_btcusdt_v1:5",
  "symbol": "BTCUSDT.BINANCE",
  "decision": "rejected",
  "reasons": [
    "order_submission_disabled",
    "shadow_mode_actual_submission_disabled",
    "account_state_missing"
  ],
  "mode": "shadow",
  "order_submission": "disabled",
  "kill_switch": false,
  "account_state": "missing",
  "market_state": "available",
  "actual_submission": false,
  "evaluated_at": "unix:1725000000200"
}
```

`decision` must be `rejected` and `actual_submission` must be `false` for every
v0.9 risk decision.

## Reject Rules

The local shadow risk gate rejects when any of these conditions are true:

- `order_submission = disabled`;
- `kill_switch = true`;
- `mode = shadow`;
- `account_state = missing`;
- `market_state = missing`.

The v0.9 demo path has market state from the local fixture stream, but it still
rejects because order submission is disabled, mode is shadow, and account state
is intentionally missing.

## Boundary

Risk decisions are not:

- exchange orders;
- Binance testnet order requests;
- execution adapter commands;
- live account state changes;
- production trading evidence.

The v0.9 strategy runtime flow is:

```text
local market event -> signal -> order intent -> risk decision
```

The flow must stop at risk decision. It must not enter execution adapter,
exchange API, cancel/replace/amend, or production trading paths.

## Validation Rule

For V090-007, the acceptance check is:

- every order intent has exactly one risk decision;
- every risk decision includes `schema_version`, `decision_id`, `intent_id`,
  `decision`, `reasons`, `mode`, `order_submission`, `kill_switch`,
  `account_state`, `market_state`, `actual_submission`, and `evaluated_at`;
- every risk decision has `decision = rejected`;
- every risk decision has `actual_submission = false`;
- risk decision records do not include exchange or venue order identifiers;
- no exchange order API is called.
