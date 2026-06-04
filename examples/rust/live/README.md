# Rust Live Examples

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-010

This directory is reserved for Rust-first live workflow examples driven by the
`nautilus live` CLI.

## CLI Command Contract

```bash
cargo run -q -p nautilus-cli -- live validate --config examples/rust/live/live_dry_run.toml
cargo run -q -p nautilus-cli -- live run --config examples/rust/live/live_dry_run.toml --run-id live-dry-run --output runs/live-dry-run
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

## Current Blocker

`live validate` and `live run` parse and expose help, but execution returns an
explicit blocker until Rust config parsing, adapter support classification, and
live-node runtime wiring are implemented. The RHARD-005 Cargo smoke is the
current executable live initialization path; it is not yet wired into
`nautilus live run`.

## Required Evidence For First Runnable Example

- The adapter used by the example is classified as supported for the example
  mode.
- The first example uses fixture, dry-run, sandbox, or explicitly scoped live
  evidence.
- The command exposes startup, reconciliation, stop, and shutdown status.
- The run does not import Python, require PyO3, or require Cython artifacts.
