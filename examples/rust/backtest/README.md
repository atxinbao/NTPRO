# Rust Backtest Examples

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-010

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

## Current Runtime Blocker

`backtest validate` and `backtest run --dry-run` now support the RHARD-006
metadata-only path. Full execution still returns an explicit blocker until Rust
strategy selection and backtest runtime wiring are implemented.

Do not replace this with Python backtest examples. The legacy
`examples/backtest` Python tree has been removed from NTPRO; this directory
tracks the Rust product surface.

## Required Evidence For First Minimal CLI Path

- `cargo run -q -p nautilus-cli -- backtest validate --config <path>` succeeds.
- `cargo run -q -p nautilus-cli -- backtest run --config <path> --dry-run --output <dir>` succeeds.
- The metadata-only run emits an owner-visible run ID and output path.
- The run does not import Python, require PyO3, or require Cython artifacts.
