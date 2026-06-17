# v0.9.0 Signal Artifact Contract

Date: 2026-06-18
Executor: Codex

## Purpose

V090-005 defines the auditable signal artifact produced by the v0.9 Strategy
Runtime Foundation. The signal artifact is local-only and is not an order,
execution request, or trading instruction.

## Artifact

```text
strategy/signal.jsonl
schema_version = ntpro.v09_strategy_signal.v1
```

Each line must be valid JSON and represents one strategy signal generated from
local fixture/mock market input.

## Required Fields

```json
{
  "schema_version": "ntpro.v09_strategy_signal.v1",
  "session_id": "btc-ema-shadow-001",
  "strategy_id": "ema_cross_btcusdt_v1",
  "symbol": "BTCUSDT.BINANCE",
  "signal": "long",
  "confidence": 0.51,
  "market_event_seq": 5,
  "generated_at": "unix:1725000000000"
}
```

The Rust struct may keep an additional `generated_at_unix_ms` field for numeric
processing, but the public JSONL contract requires `generated_at`.

## Boundary

Signals are evidence that a local strategy produced a decision from a local
market event stream. In v0.9 they must not be treated as:

- an order intent;
- an exchange order request;
- proof of Binance testnet order submission;
- production trading evidence.

Order intents and risk decisions are separate artifacts in later V090 tasks.

## Validation Rule

For V090-005, the acceptance check is:

- `strategy/signal.jsonl` is valid JSONL;
- every signal includes non-empty `session_id`, `strategy_id`, `symbol`, and
  `generated_at`;
- every signal uses `schema_version = ntpro.v09_strategy_signal.v1`;
- the artifact is produced from local fixture/mock market input only.
