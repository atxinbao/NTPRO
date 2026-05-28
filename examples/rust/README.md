# NTPRO Rust Examples

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-010

This directory is the Rust-first examples entrypoint for NTPRO cutover work.
It separates Rust product workflows from the existing Python examples under
`examples/backtest`, `examples/live`, and `examples/sandbox`.

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

The CLI command surfaces exist, but runtime execution is intentionally blocked
until later runtime and config-parser tasks connect the commands to Rust models.

Supported help commands:

```bash
cargo run -q -p nautilus-cli -- backtest --help
cargo run -q -p nautilus-cli -- sandbox --help
cargo run -q -p nautilus-cli -- live --help
cargo run -q -p nautilus-cli -- data --help
cargo run -q -p nautilus-cli -- config --help
```

Execution commands currently return explicit blockers that point to their
product contracts. Examples in this directory must not use Python fallback
behavior to bypass those blockers.

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
Python, PyO3, or Cython artifacts. Until then, keep examples as command and
config-contract documentation with explicit blocker status.
