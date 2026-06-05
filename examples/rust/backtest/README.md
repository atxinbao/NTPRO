# Rust Backtest Examples

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-010

Updated: 2026-06-06
Executor: Codex
Task ID: DRG-005

This directory is reserved for Rust-first backtest examples driven by the
`nautilus backtest` CLI.

## RHARD-006 Minimal CLI Path

```bash
cargo run -q -p nautilus-cli -- backtest validate --config examples/rust/backtest/minimal_dry_run.toml
cargo run -q -p nautilus-cli -- backtest run --config examples/rust/backtest/minimal_dry_run.toml --dry-run --output runs/minimal-backtest-dry-run
```

This path validates a small TOML config and writes a metadata-only summary file
to the output directory. It does not start `BacktestEngine`, load market data,
run a strategy, or change trading semantics.

Expected summary path:

```text
runs/minimal-backtest-dry-run/summary.txt
```

## Cargo Smoke

The current runnable Rust backtest smoke uses the `nautilus-backtest` Cargo
example directly:

```bash
cargo run -p nautilus-backtest --features examples --example engine-ema-cross
```

This smoke runs `crates/backtest/examples/engine_ema_cross.rs` with synthetic
AUD/USD quote data, a simulated venue, and the Rust `EmaCross` strategy from
`nautilus-trading`.

## Minimal Engine Smoke CLI Path

DRG-005 adds a scoped CLI path which starts the Rust `BacktestEngine` with
synthetic AUD/USD quotes and the Rust `EmaCross` strategy:

```bash
cargo run -q -p nautilus-cli -- backtest validate --config examples/rust/backtest/minimal_engine_smoke.toml
cargo run -q -p nautilus-cli -- backtest run --config examples/rust/backtest/minimal_engine_smoke.toml --output runs/minimal-backtest-engine-smoke
```

This is not a general strategy/data loader. The first supported CLI runtime
mode is intentionally limited to `run.mode = "engine-smoke"`,
`strategy.name = "ema-cross"`, and `data.instrument_id = "AUD/USD.SIM"`.

Do not replace this with Python backtest examples. The legacy
`examples/backtest` Python tree has been removed from NTPRO; this directory
tracks the Rust product surface.

## Required Evidence For First Minimal CLI Path

- `cargo run -q -p nautilus-cli -- backtest validate --config <path>` succeeds.
- `cargo run -q -p nautilus-cli -- backtest run --config <path> --dry-run --output <dir>` succeeds.
- `cargo run -q -p nautilus-cli -- backtest run --config <engine-smoke-path> --output <dir>` succeeds.
- The metadata-only run emits an owner-visible run ID and output path.
- The engine-smoke run emits `engine_started=true` and `runtime_status=completed`.
- The run does not import Python, require PyO3, or require Cython artifacts.
