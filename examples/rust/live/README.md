# Rust Live Examples

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-010

This directory is reserved for Rust-first live workflow examples driven by the
`nautilus live` CLI.

## Command Contract

```bash
cargo run -q -p nautilus-cli -- live validate --config examples/rust/live/live_dry_run.toml
cargo run -q -p nautilus-cli -- live run --config examples/rust/live/live_dry_run.toml --run-id live-dry-run --output runs/live-dry-run
```

## Current Blocker

`live validate` and `live run` parse and expose help, but execution returns an
explicit blocker until Rust config parsing, adapter support classification, and
live-node runtime wiring are implemented.

## Required Evidence For First Runnable Example

- The adapter used by the example is classified as supported for the example
  mode.
- The first example uses fixture, dry-run, sandbox, or explicitly scoped live
  evidence.
- The command exposes startup, reconciliation, stop, and shutdown status.
- The run does not import Python, require PyO3, or require Cython artifacts.
