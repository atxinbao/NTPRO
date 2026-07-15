# NTPRO Rust Examples

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-010

Updated: 2026-07-15
Executor: Codex
Task ID: DEXG-002

This directory is the Rust-first examples entrypoint for NTPRO cutover work.
It is the only supported top-level example surface after the Python example
trees were removed from `examples/backtest`, `examples/live`,
`examples/sandbox`, and related legacy directories.

## Layout

```text
examples/rust/
  backtest/  Rust backtest CLI workflow examples
  sandbox/   Rust sandbox live-node workflow examples
  live/      Rust live workflow examples
  data/      Rust data/catalog workflow examples
  config/    Shared Rust config validation examples
```

## Current Status

The supported examples are narrow, local Rust workflows. Backtest, sandbox,
live-init, data/catalog, and shared config validation have runnable CLI paths.
They do not authorize external venue execution, production order submission,
or any other capability forbidden by the v0.32.0 backend freeze.

Supported help and demo commands:

```bash
cargo run -q -p nautilus-cli -- backtest --help
cargo run -q -p nautilus-cli -- backtest run --config examples/rust/backtest/minimal_dry_run.toml --dry-run --output runs/minimal-backtest-dry-run
cargo run -q -p nautilus-cli -- sandbox --help
cargo run -q -p nautilus-cli -- sandbox run --config examples/rust/sandbox/sandbox_smoke.toml --run-id sandbox-smoke --output runs/sandbox-smoke
cargo run -q -p nautilus-cli -- live --help
cargo run -p nautilus-live --no-default-features --features node --example live-init-smoke
cargo run -q -p nautilus-cli -- data --help
cargo run -q -p nautilus-cli -- config --help
```

The backtest command supports the RHARD-006 metadata-only dry-run and the
DRG-005 engine smoke. The sandbox command supports the RHARD-004 local
simulated demo. The live CLI and crate support the sandbox-only RHARD-005 live
init smoke. Data/catalog commands support local inspect, validation, and fixture
load paths. Shared config validation dispatches to these scoped Rust validators.

These examples remain bounded demonstrations. They do not connect to a real
venue, enable production submit or mutation, or convert the backend closeout
baseline into actual production go-live authority. Examples in this directory
must not use Python fallback behavior to bypass unsupported behavior.

## Contract Mapping

- Rust API entrypoints are recorded in
  `docs/rust-cutover/product/RUST_API_ENTRYPOINTS.md`.
- Backtest examples must follow
  `docs/rust-cutover/product/BACKTEST_CLI_CONTRACT.md`.
- Sandbox and live examples must follow
  `docs/rust-cutover/product/LIVE_SANDBOX_CLI_CONTRACT.md`.
- Data/catalog examples must follow
  `docs/rust-cutover/product/DATA_CATALOG_CLI_CONTRACT.md`.
- Shared config validation examples must follow
  `docs/rust-cutover/product/CONFIG_VALIDATION_CLI_CONTRACT.md`.

## Contribution Rule

Add runnable Rust source only after the matching command can execute without
Python, PyO3, or Cython artifacts. Keep unsupported flows as command and
config-contract documentation with explicit blocker status.
