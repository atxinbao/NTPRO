# Golden Trace Schema

Date: 2026-05-29
Executor: Codex
Task ID: RTRACE-001
Updated: 2026-05-29 by Codex for RTRACE-004 Rust harness binding

A golden trace is JSONL. Each line is one deterministic trace case. A trace
file must contain at least one row. Golden traces are the compatibility
contract for behavior that cannot be proven by help output or documentation
alone.

## Schema Version

Current schema version:

```text
golden-trace-v1
```

Every row must include `schema_version` so future migration tasks can reject or
upgrade stale trace fixtures intentionally.

## Required Row Fields

```json
{
  "schema_version": "golden-trace-v1",
  "case_id": "order_lifecycle.submit_accept_fill.001",
  "category": "order_lifecycle",
  "description": "Submit limit order and receive accepted/fill events",
  "input": {
    "events": []
  },
  "expected": {
    "events": []
  },
  "tolerances": {}
}
```

Required fields:

- `schema_version`: must be `golden-trace-v1`.
- `case_id`: stable unique string. Prefix it with the category when possible.
- `category`: one of the categories listed below.
- `description`: human-readable case summary.
- `input`: object containing an `events` array.
- `expected`: object containing an `events` array.
- `tolerances`: object. Use `{}` when no tolerance is allowed.

## Required Categories

The Rust cutover requires trace coverage, executable replay, or explicit scope
blockers for these categories:

| Category | Purpose | Later owner |
| --- | --- | --- |
| `market_data` | Quotes, trades, bars, order book deltas, status, and catalog input ordering. | `RTRACE-002` |
| `order_lifecycle` | Submit, accepted, rejected, canceled, expired, triggered, filled, and modified order events. | `RTRACE-003` |
| `risk` | Risk accept/reject decisions, rate limits, notional checks, and trading-state gates. | `RTRACE-003` and runtime parity tasks |
| `execution` | Execution routing, venue order reports, fill reports, and reconciliation events. | `RTRACE-003` and `RTRACE-007` |
| `position` | Position open, increase, reduce, close, flip, and hedge/netting distinctions. | runtime parity tasks |
| `portfolio_pnl` | Account balance, margin, realized PnL, unrealized PnL, and equity snapshots. | runtime parity tasks |
| `cache_msgbus` | Cache updates, message-bus ordering, command/event sequencing, and replay ordering. | runtime parity tasks |
| `backtest_live` | Backtest, sandbox, and live lifecycle parity for scoped workflows. | `RTRACE-005` and `RTRACE-006` |
| `adapter_payload` | Adapter raw payload parsing into Rust model, data, and execution events. | `RTRACE-007` |

## Event Envelope

Each event in `input.events` and `expected.events` must use the same envelope:

```json
{
  "event_type": "order.accepted",
  "ts_event": "0",
  "ts_init": "0",
  "instrument_id": "BTCUSDT.BINANCE",
  "venue": "BINANCE",
  "correlation_id": "case-local-id",
  "payload": {}
}
```

Required event fields:

- `event_type`: stable string describing the semantic event.
- `ts_event`: nanosecond timestamp as an integer or decimal string.
- `payload`: object containing category-specific fields.

Optional common event fields:

- `ts_init`: nanosecond initialization timestamp as an integer or decimal
  string.
- `instrument_id`: Nautilus instrument ID string when the event is
  instrument-scoped.
- `venue`: venue identifier when adapter or execution behavior is scoped.
- `actor_id`: actor, strategy, engine, or component identifier.
- `correlation_id`: deterministic ID for linking commands and resulting events.

Rules:

- Use strings for decimal values.
- Use nanosecond timestamps as integers or strings.
- Do not rely on wall-clock time.
- Any nondeterministic field must be normalized or excluded with a documented
  tolerance.
- Do not include secrets, API keys, account credentials, or production endpoint
  tokens in traces.
- Do not use Python/PyO3/Cython output as release evidence for Rust-only trace
  replay. Python-derived data may be used only as documented comparison input
  before the Rust replay harness exists.

## Tolerances

`tolerances` is intentionally explicit. Use `{}` when exact match is required.
Allowed tolerance entries must name the exact path they affect:

```json
{
  "events[0].payload.elapsed_ns": {
    "kind": "excluded",
    "reason": "runtime duration is nondeterministic"
  }
}
```

Tolerances must not hide trading-semantic differences such as price, quantity,
side, order state, position side, account balance, risk decision, or adapter
payload interpretation.

## Scope Blockers

If a category cannot be replayed yet, the task must record an owner-visible
blocker in its evidence file instead of weakening this schema. A blocker must
include:

- category;
- missing Rust replay hook or fixture source;
- command attempted;
- next task that owns the gap;
- whether release/removal gates are affected.

## Validation Command

Validate all golden trace files with:

```bash
scripts/ai/run_golden_traces.sh
```

The validation command runs the JSONL schema validator and the Rust harness:

```bash
cargo test -p nautilus-testkit --test golden_trace_schema
cargo test -p nautilus-backtest --test golden_trace_backtest
cargo test -p nautilus-live --test golden_trace_live_sandbox
```

Set `RUN_RUST_GOLDEN_TRACE_HARNESS=0` or
`RUN_RUST_BACKTEST_TRACE_REPLAY=0` or
`RUN_RUST_LIVE_SANDBOX_TRACE_REPLAY=0` only when documenting an explicit local
toolchain or scoped replay blocker in task evidence.

The final release gate may require replay by setting:

```bash
REQUIRE_GOLDEN_REPLAY=1 GOLDEN_TRACE_REPLAY_COMMAND='<command using {trace} and optionally {actual}>' scripts/ai/run_golden_traces.sh
```
