# Rust Backtest Examples

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-010

This directory is reserved for Rust-first backtest examples driven by the
`nautilus backtest` CLI.

## Command Contract

```bash
cargo run -q -p nautilus-cli -- backtest validate --config examples/rust/backtest/ema_cross.toml
cargo run -q -p nautilus-cli -- backtest run --config examples/rust/backtest/ema_cross.toml --run-id ema-cross --output runs/ema-cross
```

## Current Blocker

`backtest validate` and `backtest run` parse and expose help, but execution
returns an explicit blocker until Rust config parsing, strategy selection, and
backtest runtime wiring are implemented.

Do not replace this with Python backtest examples. Existing Python examples
remain under `examples/backtest` for upstream compatibility, while this
directory tracks the Rust product surface.

## Required Evidence For First Runnable Example

- `cargo run -q -p nautilus-cli -- backtest validate --config <path>` succeeds.
- `cargo run -q -p nautilus-cli -- backtest run --config <path>` succeeds.
- The run emits an owner-visible run ID and output path.
- The run does not import Python, require PyO3, or require Cython artifacts.
