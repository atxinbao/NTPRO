# Golden Trace Gate Evidence

Date: 2026-07-29
Executor: Codex
Task ID: RTRACE-008 / RREL-008 refresh / RREL-009 / DRG-009 / PAR-010

## Gate Status

The golden trace gate has an executable Rust validation spine. PAR-010 promotes
the final five schema-only release cases to executable Rust replay and makes
zero schema-only cases a release-runner invariant.

Current evidence proves that the trace schema is enforced locally and that
backtest, live/sandbox lifecycle, data source / market data, execution order
lifecycle, risk rejection, cache/message-bus, and adapter payload traces replay
through repeatable Rust commands. The read-model contract derives health and
blocking reasons from component lineage, source provenance, and freshness. The
envelope smoke constructs and serializes a Rust `QuoteTick`; it is no longer a
schema-only row.

Current release scope: 289 cases, 103 executable Rust replay, 186 executable
validator replay, and 0 schema-only.

## Standard Command

Run the gate evidence with:

```bash
scripts/ai/run_golden_traces.sh
```

The command currently validates all `tests/golden/*.jsonl` files and runs:

```bash
cargo test -p nautilus-testkit --test golden_trace_schema
cargo test -p nautilus-model --test golden_trace_market_data
cargo test -p nautilus-common --test golden_trace_cache_msgbus
cargo test -p nautilus-backtest --test golden_trace_backtest
cargo test -p nautilus-backtest --test backtest_live_semantic_parity
cargo test -p nautilus-live --test golden_trace_live_sandbox
cargo test -p nautilus-execution --test golden_trace_order_lifecycle
cargo test -p nautilus-risk --test golden_trace_risk_rejection
cargo test -p nautilus-okx --test golden_trace_adapter_payload
cargo test -p nautilus-cli --test golden_trace_live_alpha_reconciliation
cargo test -p nautilus-cli --test golden_trace_live_alpha_mutation_dry_run
cargo test -p nautilus-cli --test golden_trace_actual_cancel
cargo test -p nautilus-cli --test golden_trace_production_order_lifecycle
cargo test -p nautilus-cli --test golden_trace_read_model_projection
cargo test -p nautilus-cli --test golden_trace_schema_smoke_runtime
```

Final release mode runs the same validation plus:

```bash
REQUIRE_GOLDEN_REPLAY=1 scripts/ai/run_golden_traces.sh
```

When `GOLDEN_TRACE_REPLAY_COMMAND` is unset, final release mode validates:

```text
docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json
```

The manifest requires each `tests/golden/*.jsonl` case to be either
`executable_replay` or `validator_executable_replay`. Release mode rejects any
`schema_only_scoped` case.

## Original R2 Trace Inventory

| File | Rows | Category | Execution status |
| --- | ---: | --- | --- |
| `tests/golden/adapter_payload_schema.jsonl` | 1 | `adapter_payload` | Rust OKX adapter parser replay |
| `tests/golden/actual_cancel_schema.jsonl` | 10 | `execution` | Rust CLI actual-cancel fixture coverage |
| `tests/golden/backtest_live_semantic_parity_schema.jsonl` | 1 | `backtest_live` | Rust backtest/live scoped parity replay |
| `tests/golden/backtest_replay_schema.jsonl` | 1 | `backtest_live` | Rust backtest replay |
| `tests/golden/cache_msgbus_schema.jsonl` | 1 | `cache_msgbus` | Rust common cache/message-bus replay |
| `tests/golden/live_alpha_mutation_dry_run_schema.jsonl` | 9 | `execution` | Rust CLI live-alpha mutation dry-run replay |
| `tests/golden/live_alpha_reconciliation_schema.jsonl` | 7 | `execution` | Rust CLI live-alpha reconciliation replay |
| `tests/golden/live_sandbox_lifecycle_schema.jsonl` | 1 | `backtest_live` | Rust live/sandbox lifecycle replay |
| `tests/golden/market_data_schema.jsonl` | 6 | `market_data` | Rust market-data model replay |
| `tests/golden/order_lifecycle_schema.jsonl` | 6 | `order_lifecycle` | Rust execution lifecycle replay |
| `tests/golden/risk_rejection_schema.jsonl` | 1 | `risk` | Rust `RiskEngine` rejection replay |
| `tests/golden/schema_smoke.jsonl` | 1 | `market_data` | Rust `QuoteTick` envelope replay |

Total: 12 JSONL files, 45 trace rows.

## Executable Evidence

| Evidence | Rust entrypoint | Covered behavior |
| --- | --- | --- |
| RTRACE-004 | `nautilus-testkit::golden_trace_schema` | Enforces `golden-trace-v1` row fields, category allowlist, event envelopes, timestamp shape, payload objects, and tolerance objects. |
| DRG-009 | `nautilus-model::golden_trace_market_data` | Replays the six v0.2 market-data rows through Rust model constructors for quote, trade, bar, order book delta, instrument status, and catalog-style ordering. |
| RCORE-009 | `nautilus-common::golden_trace_cache_msgbus` | Replays deterministic common-cache quote storage, typed message-bus publish ordering, BusTap-before-subscriber capture, and common object dispose state. |
| RTRACE-005 | `nautilus-backtest::golden_trace_backtest` | Replays one deterministic quote through `BacktestEngine` and compares normalized `BacktestResult` output. |
| RTRACE-006 | `nautilus-live::golden_trace_live_sandbox` | Builds and stops one Rust sandbox `LiveNode`, comparing deterministic lifecycle states. |
| DRG-009 | `nautilus-execution::golden_trace_order_lifecycle` | Replays submit accept/reject, modify accept, cancel accept, triggered fill, and partial-to-filled lifecycle traces through Rust order event constructors and a deterministic execution lifecycle harness. |
| DRG-009 | `nautilus-risk::golden_trace_risk_rejection` | Replays a valid submit command through `RiskEngine` with halted trading state, proving one denial event and no forwarded execution command. |
| RTRACE-007 | `nautilus-okx::golden_trace_adapter_payload` | Parses one OKX WebSocket trade payload fixture through the Rust adapter parser into a normalized `TradeTick`. |
| RBTL-009 | `nautilus-backtest::backtest_live_semantic_parity` | Compares a scoped Rust backtest quote replay against a Rust live sandbox lifecycle summary. |
| V140-005 | `nautilus-cli::golden_trace_live_alpha_reconciliation` | Replays local live-alpha reconciliation scenarios for fresh/stale order state, account readability, kill switch, and risk-limit outcomes without production mutation. |
| V150-006 | `nautilus-cli::golden_trace_live_alpha_mutation_dry_run` | Replays local mutation dry-run scenarios for approval, kill switch, risk, network-disabled, and Dashboard-control boundaries without production mutation. |
| V190-009 | `nautilus-cli::golden_trace_actual_cancel` | Validates owner-approved actual-cancel success, blocked pre-send, failed, recovered, unknown, already-cancelled, and partial-fill trace outcomes plus request/response/readback/audit/provenance references. |
| PAR-010 | `nautilus-cli::golden_trace_read_model_projection` | Derives unified contract health and fail-closed reasons from Rust projection logic, and replays account provenance/redaction negative paths. |
| PAR-010 | `nautilus-cli::golden_trace_schema_smoke_runtime` | Constructs and serializes the deterministic envelope row through the Rust `QuoteTick` model. |

## Zero Schema-Only Decision

No schema-only case remains in
`docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json`. Release mode
contains a direct zero-count guard, and
`scripts/ai/test_golden_trace_zero_schema_only_guard.sh` proves that a
schema-only regression is rejected with and without an external replay command,
and that release mode cannot disable either PAR-010 Rust harness.

## Residual Scoped Gaps

The golden trace gate now has executable local release-mode evidence for the G8
product-claimed areas. The following gaps remain explicitly scoped and should
be expanded by later runtime, adapter, and release tasks:

- `risk`: halted-state rejection is executable; rate limits, notional checks,
  and broader trading-state gates remain later expansion scope.
- `execution`: scoped lifecycle replay is executable; full order routing,
  matching-engine semantics, and venue report replay remain later expansion
  scope.
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

Python, PyO3, and Cython removal is not authorized by this document. It records
that R2 now has an executable validation spine, final release-mode scope
classification, and explicit residual gaps for later gates.
