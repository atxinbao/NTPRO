# v0.9.0 ntpro-node Strategy Session Host

Date: 2026-06-18
Executor: Codex
Task: V090-009

## Purpose

`ntpro-node` can now host the local v0.9 Strategy Runtime Foundation path by
loading a strategy-session configuration, running the built-in
`ema_cross_demo` over fixture market bars, and writing Strategy Session
artifacts.

This is a headless local strategy runtime host. It is not a Binance testnet
order proof and it is not a production trading node.

## Supported Config

The supported v0.9 node config shape is:

```toml
[node]
node_id = "btc-ema-shadow-001"
mode = "shadow"

[strategy]
strategy_id = "ema_cross_btcusdt_v1"
strategy_package = "builtin"
strategy_runtime = "ema_cross_demo"

[market]
venue = "BINANCE_TESTNET"
symbols = ["BTCUSDT.BINANCE"]
data_mode = "fixture_stream"

[execution]
venue = "BINANCE_TESTNET"
order_submission = "disabled"
external_venue_connection = false

[risk]
kill_switch = true
```

The repository includes the runnable example:

```text
configs/nodes/btc-ema-shadow.toml
```

## Artifacts

Running `ntpro-node --config configs/nodes/btc-ema-shadow.toml` writes:

```text
summary.txt
status.json
metrics.json
events.log
logs/events.log
strategy/session_status.json
strategy/events.jsonl
strategy/market_status.json
strategy/market_events.jsonl
strategy/signal.jsonl
strategy/order_intent.jsonl
strategy/risk_decision.jsonl
strategy/summary.json
```

## Boundaries

v0.9.0 keeps these boundaries:

- no Binance testnet order submission;
- no cancel / replace / amend;
- no exchange execution adapter call;
- no production Binance;
- no real funds;
- no production trading;
- no Dashboard order controls.

The `order_intent` artifact remains local and `submission_allowed=false`.
The `risk_decision` artifact remains rejected and `actual_submission=false`.

## Rollback

Revert the V090-009 PR to remove the strategy-session host branch from
`ntpro-node`, the example config, tests, and evidence. The existing
`live-init-smoke` node path remains the fallback local node smoke.
