# Rust Product Surface Inventory

Date: 2026-05-27
Executor: Codex
Task ID: RCTL-004

## Summary

This inventory records the current Rust-facing product surface for the NTPRO
Rust-first cutover. It is descriptive only: no runtime behavior, public API, or
build configuration changed in RCTL-004.

Current state:

- Cargo workspace packages: 42.
- Adapter packages: 17.
- Non-adapter packages: 25.
- Rust crate example files under `crates/**/examples`: 52.
- Rust crate test files under `crates/**/tests`: 114.
- Top-level Python example files under `examples/**`: 136.
- Documentation markdown/reStructuredText files under `docs/**`: 238.
- `nautilus-pyo3` remains present as a workspace package.

The existing Rust product surface is broad at the crate level, but the user
entrypoint is still incomplete for Rust-first product use: default
`nautilus-cli` currently exposes database operations, while DeFi blockchain
operations are feature-gated behind `defi`. Rust backtest, sandbox, and live
commands are not yet exposed as stable top-level CLI product commands.

## Workspace Packages

Generated from:

```bash
cargo metadata --no-deps --format-version=1
```

Packages:

- `nautilus-analysis` (`crates/analysis/Cargo.toml`)
- `nautilus-architect-ax` (`crates/adapters/architect_ax/Cargo.toml`)
- `nautilus-backtest` (`crates/backtest/Cargo.toml`)
- `nautilus-betfair` (`crates/adapters/betfair/Cargo.toml`)
- `nautilus-binance` (`crates/adapters/binance/Cargo.toml`)
- `nautilus-bitmex` (`crates/adapters/bitmex/Cargo.toml`)
- `nautilus-blockchain` (`crates/adapters/blockchain/Cargo.toml`)
- `nautilus-bybit` (`crates/adapters/bybit/Cargo.toml`)
- `nautilus-cli` (`crates/cli/Cargo.toml`)
- `nautilus-coinbase` (`crates/adapters/coinbase/Cargo.toml`)
- `nautilus-common` (`crates/common/Cargo.toml`)
- `nautilus-core` (`crates/core/Cargo.toml`)
- `nautilus-cryptography` (`crates/cryptography/Cargo.toml`)
- `nautilus-data` (`crates/data/Cargo.toml`)
- `nautilus-databento` (`crates/adapters/databento/Cargo.toml`)
- `nautilus-deribit` (`crates/adapters/deribit/Cargo.toml`)
- `nautilus-dydx` (`crates/adapters/dydx/Cargo.toml`)
- `nautilus-event-store` (`crates/event_store/Cargo.toml`)
- `nautilus-execution` (`crates/execution/Cargo.toml`)
- `nautilus-hyperliquid` (`crates/adapters/hyperliquid/Cargo.toml`)
- `nautilus-indicators` (`crates/indicators/Cargo.toml`)
- `nautilus-infrastructure` (`crates/infrastructure/Cargo.toml`)
- `nautilus-interactive-brokers` (`crates/adapters/interactive_brokers/Cargo.toml`)
- `nautilus-kraken` (`crates/adapters/kraken/Cargo.toml`)
- `nautilus-live` (`crates/live/Cargo.toml`)
- `nautilus-model` (`crates/model/Cargo.toml`)
- `nautilus-network` (`crates/network/Cargo.toml`)
- `nautilus-okx` (`crates/adapters/okx/Cargo.toml`)
- `nautilus-persistence` (`crates/persistence/Cargo.toml`)
- `nautilus-persistence-macros` (`crates/persistence/macros/Cargo.toml`)
- `nautilus-plugin` (`crates/plugin/Cargo.toml`)
- `nautilus-polymarket` (`crates/adapters/polymarket/Cargo.toml`)
- `nautilus-portfolio` (`crates/portfolio/Cargo.toml`)
- `nautilus-pyo3` (`crates/pyo3/Cargo.toml`)
- `nautilus-risk` (`crates/risk/Cargo.toml`)
- `nautilus-sandbox` (`crates/adapters/sandbox/Cargo.toml`)
- `nautilus-serialization` (`crates/serialization/Cargo.toml`)
- `nautilus-system` (`crates/system/Cargo.toml`)
- `nautilus-tardis` (`crates/adapters/tardis/Cargo.toml`)
- `nautilus-testkit` (`crates/testkit/Cargo.toml`)
- `nautilus-trader` (`crates/Cargo.toml`)
- `nautilus-trading` (`crates/trading/Cargo.toml`)

## Rust CLI Surface

Primary package: `nautilus-cli`.

Source:

- `crates/cli/Cargo.toml`
- `crates/cli/src/bin/cli.rs`
- `crates/cli/src/opt.rs`
- `crates/cli/README.md`

Default command surface:

```text
nautilus
  database
```

Default help was verified with:

```bash
cargo run -q -p nautilus-cli -- --help
cargo run -q -p nautilus-cli -- database --help
```

Feature-gated command surface:

```text
nautilus --features defi
  database
  blockchain
```

Feature-gated help was verified with:

```bash
cargo run -q -p nautilus-cli --features defi -- --help
```

Observed gap for later RPROD/RCLI work:

- No default top-level `backtest` command.
- No default top-level `sandbox` command.
- No default top-level `live` command.
- Current CLI is closer to database and operational tooling than a complete
  Rust-first trading product entrypoint.

## Rust Examples

Rust examples exist primarily under crate-local `examples/` directories:

- Adapter node/data/exec tester examples under `crates/adapters/*/examples`.
- Backtest examples under `crates/backtest/examples`.
- Plugin examples under `crates/plugin/examples`.
- Common actor example under `crates/common/examples`.

Representative Rust examples:

- `crates/backtest/examples/engine_ema_cross.rs`
- `crates/backtest/examples/node_ema_cross.rs`
- `crates/adapters/sandbox/examples/databento_cme.rs`
- `crates/plugin/examples/runtime_smoke_plugin.rs`
- `crates/common/examples/greeks_actor_example.rs`

Top-level `examples/**` remains Python-heavy:

- 136 Python example files were found under `examples/**`.
- These examples remain useful as migration references, but they are not a
  Rust-first public product path.

## Rust Documentation Surface

Existing Rust-facing documentation:

- `docs/concepts/rust.md`
- `docs/developer_guide/rust.md`
- `docs/how_to/run_rust_backtest.md`
- `docs/how_to/run_rust_live_trading.md`
- `docs/how_to/write_rust_actor.md`
- `docs/how_to/write_rust_strategy.md`

Existing API reference docs include Rust-relevant modules:

- `docs/api_reference/backtest.md`
- `docs/api_reference/live.md`
- `docs/api_reference/data.md`
- `docs/api_reference/execution.md`
- `docs/api_reference/risk.md`
- `docs/api_reference/portfolio.md`
- `docs/api_reference/trading.md`

Observed gap for later RPROD documentation work:

- Rust docs exist, but they are not yet tied to a complete stable Rust CLI
  contract for backtest, sandbox, and live workflows.

## Runnable Rust Tests

Current runnable Rust test entrypoints are primarily crate-scoped:

- `cargo test -p nautilus-backtest`
- `cargo test -p nautilus-live`
- `cargo test -p nautilus-data`
- `cargo test -p nautilus-execution`
- `cargo test -p nautilus-risk`
- `cargo test -p nautilus-portfolio`
- `cargo test -p nautilus-sandbox`
- `cargo test -p <adapter-crate>`

Repository-level Rust-first smoke remains:

```bash
scripts/ai/verify_fast.sh
```

Release-level Rust-only checks remain gated by later tasks:

```bash
scripts/ai/verify_full.sh
scripts/ai/verify_release.sh
scripts/ai/check_rust_only_runtime.sh
```

## Rust-First Cutover Implications

This inventory supports the following next steps:

- RPROD/RCLI tasks should define stable Rust CLI contracts for `backtest`,
  `sandbox`, and `live`.
- RCORE/RBTL tasks should prove Rust runtime smoke paths are callable from the
  Rust product surface.
- RADP tasks should classify adapter support and fixture/mock coverage before
  any adapter removal.
- RREM/RREL tasks must not remove Python, PyO3, or Cython surfaces until the
  Rust-only route is explicitly approved and release gates pass.
