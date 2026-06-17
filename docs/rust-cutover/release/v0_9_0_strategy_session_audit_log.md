# v0.9.0 Strategy Session Audit Log

Date: 2026-06-18
Executor: Codex

## Purpose

V090-008 defines the local audit surface for the v0.9 Strategy Runtime
Foundation. It records session lifecycle events, risk decision events, and a
summary artifact for local review.

Plain Chinese summary: 这个文件说明 v0.9.0 策略运行时会留下哪些本地审计记录。它只
记录本地 session、signal、order intent、risk decision 和 summary，不代表已经接入
Binance testnet 下单，也不代表可以实盘交易。

## Artifacts

```text
strategy/session_status.json
strategy/events.jsonl
strategy/signal.jsonl
strategy/order_intent.jsonl
strategy/risk_decision.jsonl
strategy/summary.json
```

Every artifact written by the Strategy Session runtime must include a
`schema_version`.

## Event Log

`strategy/events.jsonl` records local session events. In v0.9.0, it includes:

- session lifecycle transitions such as start, pause, stop, and failed states;
- risk decision rejection events for local order intents.

Risk decision events use:

```text
event_type = strategy_risk_decision_rejected
```

These events are local audit records only. They are not adapter events, exchange
events, venue order events, or production trading evidence.

## Summary

`strategy/summary.json` uses:

```text
schema_version = ntpro.v09_strategy_session_summary.v1
```

It records:

- `signal_count`;
- `intent_count`;
- `risk_decision_count`;
- `rejection_count`;
- `actual_submission_count`;
- local lifecycle and market-event counts.

For v0.9.0, `actual_submission_count` must be `0`.

## Boundary

The audit log is not:

- an execution adapter log;
- a Binance testnet order log;
- a live order lifecycle trace;
- account-state mutation evidence;
- production trading evidence.

The v0.9 strategy runtime flow remains:

```text
local market event -> signal -> order intent -> risk decision -> audit summary
```

The flow must still stop before execution adapter, exchange API, cancel/replace,
amend, real funds, or production trading paths.

## Validation Rule

For V090-008, the acceptance check is:

- start, stop, pause, and risk decision events are present in local artifacts;
- `summary.json` includes `signal_count`, `intent_count`, and
  `rejection_count`;
- every Strategy Session artifact includes `schema_version`;
- `actual_submission_count = 0`;
- no exchange order API is called.
