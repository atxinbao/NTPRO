# Golden Trace Gate Evidence

Date: 2026-06-02
Executor: Codex
Task ID: RTRACE-008 / RREL-008 refresh

## Gate Status

The R2 golden trace gate has an executable Rust validation spine, but it is not
a final Rust-only release signoff. Current evidence proves that the trace schema
is enforced locally and that representative backtest/live/adapter traces replay
through Rust code. The final release command still requires a strict
`GOLDEN_TRACE_REPLAY_COMMAND` when `REQUIRE_GOLDEN_REPLAY=1`.

## Standard Command

Run the gate evidence with:

```bash
scripts/ai/run_golden_traces.sh
```

The command currently validates all `tests/golden/*.jsonl` files and runs:

```bash
cargo test -p nautilus-testkit --test golden_trace_schema
cargo test -p nautilus-common --test golden_trace_cache_msgbus
cargo test -p nautilus-backtest --test golden_trace_backtest
cargo test -p nautilus-live --test golden_trace_live_sandbox
cargo test -p nautilus-okx --test golden_trace_adapter_payload
```

## Current Trace Inventory

| File | Rows | Category | Execution status |
| --- | ---: | --- | --- |
| `tests/golden/adapter_payload_schema.jsonl` | 1 | `adapter_payload` | Rust OKX adapter parser replay |
| `tests/golden/backtest_live_semantic_parity_schema.jsonl` | 1 | `backtest_live` | Rust backtest/live scoped parity replay |
| `tests/golden/backtest_replay_schema.jsonl` | 1 | `backtest_live` | Rust backtest replay |
| `tests/golden/cache_msgbus_schema.jsonl` | 1 | `cache_msgbus` | Rust common cache/message-bus replay |
| `tests/golden/live_sandbox_lifecycle_schema.jsonl` | 1 | `backtest_live` | Rust live/sandbox lifecycle replay |
| `tests/golden/market_data_schema.jsonl` | 6 | `market_data` | Rust schema harness |
| `tests/golden/order_lifecycle_schema.jsonl` | 6 | `order_lifecycle` | Rust schema harness |
| `tests/golden/schema_smoke.jsonl` | 1 | `market_data` | Rust schema harness |

Total: 8 JSONL files, 18 trace rows.

## Executable Evidence

| Evidence | Rust entrypoint | Covered behavior |
| --- | --- | --- |
| RTRACE-004 | `nautilus-testkit::golden_trace_schema` | Enforces `golden-trace-v1` row fields, category allowlist, event envelopes, timestamp shape, payload objects, and tolerance objects. |
| RCORE-009 | `nautilus-common::golden_trace_cache_msgbus` | Replays deterministic common-cache quote storage, typed message-bus publish ordering, BusTap-before-subscriber capture, and common object dispose state. |
| RTRACE-005 | `nautilus-backtest::golden_trace_backtest` | Replays one deterministic quote through `BacktestEngine` and compares normalized `BacktestResult` output. |
| RTRACE-006 | `nautilus-live::golden_trace_live_sandbox` | Builds and stops one Rust sandbox `LiveNode`, comparing deterministic lifecycle states. |
| RTRACE-007 | `nautilus-okx::golden_trace_adapter_payload` | Parses one OKX WebSocket trade payload fixture through the Rust adapter parser into a normalized `TradeTick`. |
| RBTL-009 | `nautilus-backtest::backtest_live_semantic_parity` | Compares a scoped Rust backtest quote replay against a Rust live sandbox lifecycle summary. |

## Schema-Only Seed Evidence

The following categories currently have valid trace rows but do not yet claim
full runtime replay parity:

- `market_data`: quote, trade, bar, order book delta, instrument status, and
  catalog ordering fixtures exist.
- `order_lifecycle`: submit accept/reject, modify accept, cancel accept,
  triggered fill, and partial-to-filled fixtures exist.

These fixtures are intentionally useful before full replay hooks exist. They
are release blockers until later runtime tasks bind them to the scoped Rust
engines or record explicit scope decisions.

## Release Blockers

The golden trace gate is blocked for final Rust-only release until these gaps
are closed or explicitly scoped:

- `verify_release.sh`: final mode sets `REQUIRE_GOLDEN_REPLAY=1`, but no
  `GOLDEN_TRACE_REPLAY_COMMAND` is wired by default.
- `risk`: no executable Rust golden trace replay yet for risk accept/reject,
  rate limits, notional checks, or trading-state gates.
- `execution`: order routing and venue report replay is not fully bound beyond
  the current OKX adapter payload parser fixture.
- `position`: no executable position open/increase/reduce/close trace replay
  yet.
- `portfolio_pnl`: no executable account balance, margin, realized PnL,
  unrealized PnL, or equity replay yet.
- `cache_msgbus`: common-level cache/message-bus ordering now has one
  executable Rust replay; full kernel/event-store shutdown ordering and any
  backing database replay remain owned by later runtime/release gate tasks.
- `adapter_payload`: only one OKX market-data parser fixture is executable;
  broader official adapter payload parity remains owned by later `RADP-*`
  tasks.

## Removal Gate Impact

Python, PyO3, and Cython removal remains blocked by the broader Rust-only gate.
This document does not authorize removal. It records that R2 now has an
executable validation spine and explicit residual blockers for later gates.
