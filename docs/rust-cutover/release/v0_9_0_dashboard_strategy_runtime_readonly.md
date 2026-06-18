# v0.9.0 Dashboard Strategy Runtime Read-Only Surface

Date: 2026-06-18
Executor: Codex
Task: V090-011

## Purpose

The Dashboard can now display Strategy Runtime Foundation artifacts produced by
local v0.9 strategy sessions.

This is a read-only status surface. It helps operators inspect what the strategy
runtime did locally, but it does not turn the Dashboard into a trading control
panel.

## Displayed Fields

The Dashboard `Strategy Runtime` panel displays:

```text
node_id
session_id
session_state
strategy_id
symbol
market_stream_status
signal_count
latest_signal
latest_order_intent
latest_risk_decision
rejection_reason
order_submission_mode
actual_submission_count
session_status_path
signal_artifact_path
order_intent_artifact_path
risk_decision_artifact_path
summary_artifact_path
```

All values are derived from local files under the node artifact root:

```text
strategy/session_status.json
strategy/market_status.json
strategy/signal.jsonl
strategy/order_intent.jsonl
strategy/risk_decision.jsonl
strategy/summary.json
```

## Boundaries

This Dashboard surface remains read-only:

- no order button;
- no cancel/replace/amend action;
- no live-enable button;
- no credential input;
- no strategy hot reload;
- no production connect action;
- no exchange order API call;
- no real funds;
- no production trading.

Existing node lifecycle controls remain separate from this panel. The Strategy
Runtime panel itself contains no action buttons.

## Rollback

Revert the V090-011 PR to remove the Strategy Runtime Dashboard panel and the
`strategy_runtime` snapshot field. Existing Dashboard workflow, node, runtime
module, logs, metrics, and lifecycle-control surfaces remain unchanged.
