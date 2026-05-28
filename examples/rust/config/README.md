# Rust Config Validation Examples

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-010

This directory is reserved for shared Rust config validation examples driven by
the `nautilus config` CLI.

## Command Contract

```bash
cargo run -q -p nautilus-cli -- config validate --kind backtest --config examples/rust/backtest/ema_cross.toml
cargo run -q -p nautilus-cli -- config validate --kind sandbox --config examples/rust/sandbox/sandbox_smoke.toml
cargo run -q -p nautilus-cli -- config validate --kind live --config examples/rust/live/live_dry_run.toml
cargo run -q -p nautilus-cli -- config validate --kind data --config examples/rust/data/catalog_audit.toml
```

## Current Blocker

`config validate` parses and exposes help, but execution returns an explicit
blocker until a shared Rust config parser and workflow-specific validation
models are implemented.

## Required Evidence For First Runnable Example

- The selected `--kind` maps to a Rust config model.
- Validation reports failing section and field details when known.
- The command does not import Python, require PyO3, or require Cython artifacts.
