# v0.9.0 Order Intent Artifact Contract

Date: 2026-06-18
Executor: Codex

## Purpose

V090-006 defines the auditable order intent artifact produced by the v0.9
Strategy Runtime Foundation. The order intent artifact is local-only and
records what a strategy would like to do before risk evaluation.

Plain Chinese summary: 这个文件只说明“策略想下什么单”的本地记录格式。它不是交易所订单，
不会调用 Binance，也不能作为 testnet 下单证明或真实交易证明。

## Artifact

```text
strategy/order_intent.jsonl
schema_version = ntpro.v09_order_intent.v1
```

Each line must be valid JSON and represents one local order intent derived from
a local strategy signal.

## Required Fields

```json
{
  "schema_version": "ntpro.v09_order_intent.v1",
  "session_id": "btc-ema-shadow-001",
  "strategy_id": "ema_cross_btcusdt_v1",
  "intent_id": "btc-ema-shadow-001:ema_cross_btcusdt_v1:5",
  "symbol": "BTCUSDT.BINANCE",
  "side": "buy",
  "order_type": "market",
  "quantity": 1.0,
  "source_signal": "long",
  "confidence": 0.51,
  "market_event_seq": 5,
  "signal_generated_at": "unix:1725000000000",
  "created_at": "unix:1725000000100",
  "submission_allowed": false,
  "submission_status": "blocked_by_v09_strategy_runtime_boundary"
}
```

`submission_allowed` must be `false` for every v0.9 order intent.

## Boundary

Order intents are not:

- exchange orders;
- Binance testnet order requests;
- cancel/replace/amend requests;
- execution adapter inputs;
- production trading evidence.

The v0.9 order intent flow is:

```text
local market event -> signal -> order intent -> risk decision
```

The flow must stop before any execution adapter or exchange API.

## Validation Rule

For V090-006, the acceptance check is:

- `strategy/order_intent.jsonl` is valid JSONL;
- every order intent includes `schema_version`, `session_id`, `strategy_id`,
  `intent_id`, `symbol`, `side`, `order_type`, `quantity`, `source_signal`,
  `confidence`, `market_event_seq`, `signal_generated_at`, `created_at`,
  `submission_allowed`, and `submission_status`;
- every order intent has `submission_allowed = false`;
- order intent records do not include exchange or venue order identifiers;
- no exchange order API is called.
