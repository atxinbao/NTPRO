# NTPRO Rust Examples

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-010

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

The CLI command surfaces exist. Backtest and sandbox now have narrow runnable
Rust CLI paths, while full runtime execution remains deferred until later
runtime and adapter tasks connect the commands to Rust models.

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

The backtest command supports the RHARD-006 metadata-only dry-run. The sandbox
command supports the RHARD-004 local simulated demo. The live crate supports
the RHARD-005 Cargo live init smoke. Live CLI, data, config, and full runtime
execution paths still return explicit blockers that point to their product
contracts. Examples in this directory must not use Python fallback behavior to
bypass those blockers.

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
