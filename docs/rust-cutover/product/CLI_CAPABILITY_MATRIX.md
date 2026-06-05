# CLI Capability Matrix

Date: 2026-06-05
Executor: Codex
Task ID: NAUDIT-002

Updated: 2026-06-06
Executor: Codex
Task ID: DRG-005

## Purpose

This matrix records the current user-visible Rust CLI capability boundary. It
prevents NTPRO docs, help text, and release notes from presenting stubbed or
simulated paths as fully implemented runtime workflows.

## Status Definitions

| Status | Meaning |
| --- | --- |
| `implemented` | The command executes a scoped Rust workflow and can be treated as current product behavior for that scope. |
| `simulated_demo` | The command writes or validates deterministic demo artifacts, but it does not start the full runtime path it represents. |
| `metadata_only` | The command parses, validates, inspects, or writes metadata only. It does not run trading/runtime behavior. |
| `deferred` | The command exists as a help-stable contract, but execution returns an owner-visible Rust blocker. |

## Current Matrix

| Command | Status | Current behavior | Explicit boundary |
| --- | --- | --- | --- |
| `nautilus backtest validate` | `metadata_only` | Parses and validates the RHARD-006 dry-run config and DRG-005 engine-smoke config. | Does not start `BacktestEngine`, load market data, or run a strategy. |
| `nautilus backtest run --dry-run` | `metadata_only` | Writes a metadata summary for the RHARD-006 minimal config. | `engine_started=false`; the non-dry-run path supports only the scoped DRG-005 engine-smoke config. |
| `nautilus backtest run` without `--dry-run` | `implemented` | Runs the scoped DRG-005 Rust `BacktestEngine` smoke with synthetic AUD/USD quotes and the Rust `EmaCross` strategy. | Only `run.mode = "engine-smoke"`, `strategy.name = "ema-cross"`, and `data.instrument_id = "AUD/USD.SIM"` are supported. No arbitrary strategy/data loader yet. |
| `nautilus sandbox validate` | `simulated_demo` | Validates the RHARD-004 sandbox demo config. | Does not start a real `LiveNode` or connect to any venue. |
| `nautilus sandbox run` | `simulated_demo` | Writes `summary.txt` and `events.log` for a deterministic simulated sandbox flow. | Reports `live_node_started=false`, `external_venue_connection=false`, and `real_orders_submitted=false`. |
| `nautilus live validate` | `implemented` | Validates the DRG-005 `live-init-smoke` TOML boundary. | Requires sandbox environment, simulated execution client, disabled order submission, no external venue connection, and start/stop shutdown mode. |
| `nautilus live run` | `implemented` | Starts and stops a Rust `LiveNode` in sandbox mode with the simulated execution client registered, then writes summary/events artifacts. | No real venue connection, no reconciliation, no real orders, and no production adapter behavior. |
| `nautilus data inspect` | `metadata_only` | Inspects local file/directory catalog metadata for the GH-156 config shape. | Does not decode rows, query intervals, load data, or call adapters. |
| `nautilus data validate` | `metadata_only` | Validates local catalog readability and query shape. | Does not prove row-level catalog completeness. |
| `nautilus data load` | `implemented` | Loads a local `quote_tick_csv_v1` QuoteTick fixture by copying it into the configured Rust catalog directory and writing a summary artifact. | No adapter access, no Parquet row encoding, no row-level semantic decode, and no interval availability proof. |
| `nautilus config validate` | `implemented` | Validates scoped backtest, sandbox, live-smoke, and data/catalog TOML boundaries. | It validates configs only and does not run the selected workflow. |
| `nautilus database init/drop` | `implemented` | Runs the existing Rust database administration commands. | Database operations are operational utilities, not trading runtime execution. |

## Documentation Rule

User-facing docs may mention stubbed commands only with their matrix status.
Do not describe `metadata_only`, `simulated_demo`, or `deferred` commands as
complete trading, backtest, live, data-load, or node lifecycle runtime paths.

## Release Rule

Release notes may claim only the scoped DRG-005 minimal paths: backtest
engine-smoke, live-init sandbox start/stop smoke, and local QuoteTick fixture
load. They must not claim arbitrary strategy loading, production live trading,
adapter-backed data load, or a full trading workflow until those rows are
separately promoted by implementation evidence and tests.
