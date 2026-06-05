# CLI Capability Matrix

Date: 2026-06-05
Executor: Codex
Task ID: NAUDIT-002

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
| `nautilus backtest validate` | `metadata_only` | Parses and validates the RHARD-006 minimal TOML config. | Does not start `BacktestEngine`, load market data, or run a strategy. |
| `nautilus backtest run --dry-run` | `metadata_only` | Writes a metadata summary for the RHARD-006 minimal config. | `engine_started=false`; full run without `--dry-run` remains deferred. |
| `nautilus backtest run` without `--dry-run` | `deferred` | Returns an explicit runtime-wiring blocker. | No Python fallback and no Rust backtest runtime execution yet. |
| `nautilus sandbox validate` | `simulated_demo` | Validates the RHARD-004 sandbox demo config. | Does not start a real `LiveNode` or connect to any venue. |
| `nautilus sandbox run` | `simulated_demo` | Writes `summary.txt` and `events.log` for a deterministic simulated sandbox flow. | Reports `live_node_started=false`, `external_venue_connection=false`, and `real_orders_submitted=false`. |
| `nautilus live validate` | `deferred` | Returns an explicit live-validation blocker. | `config validate --kind live` can validate a scoped live-smoke TOML boundary, but `live validate` itself is not implemented. |
| `nautilus live run` | `deferred` | Returns an explicit live-runtime blocker. | No live node is started by the CLI. |
| `nautilus data inspect` | `metadata_only` | Inspects local file/directory catalog metadata for the GH-156 config shape. | Does not decode rows, query intervals, load data, or call adapters. |
| `nautilus data validate` | `metadata_only` | Validates local catalog readability and query shape. | Does not prove row-level catalog completeness. |
| `nautilus data load` | `deferred` | Returns an explicit data-load blocker. | No catalog writes or adapter-backed loads are implemented. |
| `nautilus config validate` | `implemented` | Validates scoped backtest, sandbox, live-smoke, and data/catalog TOML boundaries. | It validates configs only and does not run the selected workflow. |
| `nautilus database init/drop` | `implemented` | Runs the existing Rust database administration commands. | Database operations are operational utilities, not trading runtime execution. |

## Documentation Rule

User-facing docs may mention stubbed commands only with their matrix status.
Do not describe `metadata_only`, `simulated_demo`, or `deferred` commands as
complete trading, backtest, live, data-load, or node lifecycle runtime paths.

## Release Rule

Release notes must not claim v0.2 has a fully runnable CLI trading path until
the relevant matrix rows are promoted by implementation evidence and tests.
