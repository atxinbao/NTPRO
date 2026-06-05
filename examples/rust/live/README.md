# Rust Live Examples

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-010

Updated: 2026-06-06
Executor: Codex
Task ID: DRG-005

This directory is reserved for Rust-first live workflow examples driven by the
`nautilus live` CLI.

## CLI Command Contract

```bash
cargo run -q -p nautilus-cli -- live validate --config examples/rust/live/live_init_smoke.toml
cargo run -q -p nautilus-cli -- live run --config examples/rust/live/live_init_smoke.toml --run-id live-init-smoke --output runs/live-init-smoke
```

## Live Init Smoke

RHARD-005 adds a runnable Cargo smoke for live-node initialization and
shutdown:

```bash
cargo run -p nautilus-live --no-default-features --features node --example live-init-smoke
```

The equivalent owner-visible config is recorded in:

```text
examples/rust/live/live_init_smoke.toml
```

The smoke builds a Rust `LiveNode` in `Sandbox` mode, registers the
`nautilus_sandbox` simulated execution client, starts the node, confirms the
execution client and account cache are initialized, then stops the node. It
does not call a real trading endpoint and does not submit real orders.

## Current Runtime Boundary

`live validate` now validates the scoped live-init smoke config. `live run`
starts and stops a Rust `LiveNode` in `Sandbox` mode with the simulated
execution client registered, then writes owner-visible summary and event
artifacts. This path does not connect to a real venue, reconcile external
state, or submit real orders.

## Required Evidence For First Runnable Example

- The adapter used by the example is classified as supported for the example
  mode.
- The first example uses fixture, dry-run, sandbox, or explicitly scoped live
  evidence.
- The command exposes startup, reconciliation, stop, and shutdown status.
- The run does not import Python, require PyO3, or require Cython artifacts.
