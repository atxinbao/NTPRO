# v0.9.0 Supervisor Strategy Session Status

Date: 2026-06-18
Executor: Codex
Task: V090-010

## Purpose

The local supervisor can now expose the Strategy Session view of a node, not
only the process/node lifecycle view.

This is a read-only status surface for the v0.9.0 Strategy Runtime Foundation.
It does not add order controls, exchange submission, production connectivity,
or Dashboard actions.

## Status Fields

`supervisor status` includes:

```text
strategy_session_state
strategy_id
market_state
risk_state
last_signal_at
last_rejection_reason
strategy_session_status
strategy_events
strategy_summary
```

If a node has no Strategy Session artifacts, these fields return `none` or
`unknown` instead of failing the supervisor status command.

## Metrics Fields

`supervisor metrics` includes:

```text
strategy_signal_count
strategy_rejection_count
```

For v0.9 strategy nodes these values come from the local status/metrics
artifacts produced by `ntpro-node`.

## Log Surface

`supervisor logs` includes the normal node logs plus Strategy Session artifact
paths when they exist:

```text
strategy_events
strategy_summary
```

## Boundaries

This status surface remains read-only:

- no order button;
- no cancel/replace/amend action;
- no exchange order API call;
- no production connectivity;
- no real funds;
- no production trading.

## Rollback

Revert the V090-010 PR to remove the supervisor-derived Strategy Session
fields and metrics while preserving base node status/metrics/log behavior.
