# Rust Sandbox Examples

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-010

This directory is reserved for Rust-first sandbox live-node examples driven by
the `nautilus sandbox` CLI.

## Minimal Sandbox Demo

```bash
cargo run -q -p nautilus-cli -- sandbox validate --config examples/rust/sandbox/sandbox_smoke.toml
cargo run -q -p nautilus-cli -- sandbox run --config examples/rust/sandbox/sandbox_smoke.toml --run-id sandbox-smoke --output runs/sandbox-smoke
```

The RHARD-004 CLI path is a local simulated sandbox demo. It parses
`sandbox_smoke.toml`, uses synthetic quote events, uses simulated order
submission, and writes owner-visible artifacts:

```text
runs/sandbox-smoke/summary.txt
runs/sandbox-smoke/events.log
```

The run reports node start, event flow, risk state, portfolio state, cache
state, and node stop. It never connects to a production venue and never submits
real orders.

## Cargo Smoke

The current runnable Rust sandbox smoke constructs a `nautilus-live`
`LiveNode` in `Sandbox` mode without connecting to an external venue:

```bash
cargo run -p nautilus-live --no-default-features --features node --example sandbox-node-smoke
```

The smoke verifies the node starts in `Idle`, reports the configured trader ID
and environment, and records that no Python runtime or external venue
connection is required.

## Current Runtime Boundary

`sandbox validate` and `sandbox run` now support the RHARD-004 local simulated
demo. Full live-node construction, adapter wiring, and production exchange
behavior remain deferred until a later scoped runtime or adapter task provides
evidence.

## Required Evidence For First Runnable Example

- The example uses a sandbox or fixture data path.
- The command does not connect to a production venue.
- Startup and shutdown status are owner-visible.
- The run does not import Python, require PyO3, or require Cython artifacts.
