# Rust Sandbox Examples

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-010

This directory is reserved for Rust-first sandbox live-node examples driven by
the `nautilus sandbox` CLI.

## Command Contract

```bash
cargo run -q -p nautilus-cli -- sandbox validate --config examples/rust/sandbox/sandbox_smoke.toml
cargo run -q -p nautilus-cli -- sandbox run --config examples/rust/sandbox/sandbox_smoke.toml --run-id sandbox-smoke --output runs/sandbox-smoke
```

## Cargo Smoke

The current runnable Rust sandbox smoke constructs a `nautilus-live`
`LiveNode` in `Sandbox` mode without connecting to an external venue:

```bash
cargo run -p nautilus-live --no-default-features --features node --example sandbox-node-smoke
```

The smoke verifies the node starts in `Idle`, reports the configured trader ID
and environment, and records that no Python runtime or external venue
connection is required.

## Current Blocker

`sandbox validate` and `sandbox run` parse and expose help, but execution
returns an explicit blocker until Rust config parsing and sandbox live-node
runtime wiring are implemented.

## Required Evidence For First Runnable Example

- The example uses a sandbox or fixture data path.
- The command does not connect to a production venue.
- Startup and shutdown status are owner-visible.
- The run does not import Python, require PyO3, or require Cython artifacts.
