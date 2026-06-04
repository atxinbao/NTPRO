# Rust

NTPRO has a Rust-only implementation under the `crates/` directory. You can
write actors, strategies, run backtests, and build live/runtime paths through
the Rust workspace without Python, PyO3, or Cython.

Legacy upstream documentation described v1 Cython/Python and v2 PyO3 paths.
Those paths are not supported NTPRO product surfaces after the Rust-only
cutover. Retained references are historical or migration context only.

:::warning
The Rust API is under active development. Method signatures and trait
requirements may change between releases.
:::

## System implementation

NTPRO's supported implementation is the Rust workspace under `crates/`. The
workspace retains runtime crates for backtest, live, trading, data, execution,
risk, portfolio, model, persistence, system, and adapters.

Do not select a Python/PyO3/Cython implementation path for current NTPRO work.
Adapter and live-runtime availability must be read from current release evidence
and adapter support records, not from upstream Python compatibility tables.

### Adapter families

The repository includes Rust adapter crates for Architect AX, Betfair, Binance,
BitMEX, Blockchain, Bybit, Coinbase, Databento, Deribit, dYdX, Hyperliquid,
Interactive Brokers, Kraken, OKX, Polymarket, Sandbox, and Tardis. Support
status is not implied by upstream Python parity; each adapter needs current
Rust evidence before it is treated as supported for a release.

### Choosing a path

- **NTPRO Rust workspace** is the supported product path.
- **Legacy upstream Python/Cython/PyO3 paths** are unsupported for NTPRO and
  should only be cited as history, migration evidence, or removal evidence.
- **Future product work** must add Rust implementation evidence before docs
  describe it as available.

## Project setup

Use the repository workspace as the authoritative source for NTPRO. For local
development inside this repository, build and test the workspace through Cargo:

```bash
cargo check --workspace
cargo test -p nautilus-cli
```

When creating a local Rust example outside the workspace, use local `path`
dependencies that point at the checked-out NTPRO repository. Do not point NTPRO
examples at the upstream NautilusTrader `develop` branch.

```toml
[dependencies]
nautilus-backtest = { path = "/path/to/NTPRO/crates/backtest" }
nautilus-common = { path = "/path/to/NTPRO/crates/common" }
nautilus-execution = { path = "/path/to/NTPRO/crates/execution" }
nautilus-model = { path = "/path/to/NTPRO/crates/model", features = ["stubs"] }
nautilus-trading = { path = "/path/to/NTPRO/crates/trading", features = ["examples"] }
```

The minimum supported Rust version (MSRV) is **1.95.0**.

### Feature flags

| Flag             | Crate               | Effect                                                        |
|------------------|---------------------|---------------------------------------------------------------|
| `high-precision` | `nautilus-model`    | 16-digit fixed precision (default is 9). Required for crypto. |
| `stubs`          | `nautilus-model`    | Test instrument stubs (`audusd_sim`, etc.).                   |
| `examples`       | `nautilus-trading`  | Example strategies (`EmaCross`, `GridMarketMaker`).           |
| `streaming`      | `nautilus-backtest` | Catalog‑based data streaming via `BacktestNode`.              |
| `defi`           | `nautilus-model`    | DeFi data types. Implies `high-precision`.                    |

:::tip
Standard 9-digit precision handles most traditional finance instruments.
Enable `high-precision` for crypto venues where prices can have many decimal
places (e.g. `0.00000001`).
:::

## Actors

An actor receives market data, custom data/signals, and system events but does not manage orders.
Implement the `DataActor` trait and bind your struct to `DataActorCore` via
`Deref`/`DerefMut`. Your struct must also implement `Debug` (required by the
blanket `Component` impl). The core provides subscription methods, cache
access, and clock access directly on your struct.

### Handler methods

Override any handler on the `DataActor` trait to receive the corresponding
data or event. All handlers have default no-op implementations, so you only
override what you need.

| Handler                | Receives                  |
|------------------------|---------------------------|
| `on_start`             | Actor started.            |
| `on_stop`              | Actor stopped.            |
| `on_quote`             | `QuoteTick`               |
| `on_trade`             | `TradeTick`               |
| `on_bar`               | `Bar`                     |
| `on_book_deltas`       | `OrderBookDeltas`         |
| `on_book`              | `OrderBook` (at interval) |
| `on_instrument`        | `InstrumentAny`           |
| `on_mark_price`        | `MarkPriceUpdate`         |
| `on_index_price`       | `IndexPriceUpdate`        |
| `on_funding_rate`      | `FundingRateUpdate`       |
| `on_option_greeks`     | `OptionGreeks`            |
| `on_option_chain`      | `OptionChainSlice`        |
| `on_instrument_status` | `InstrumentStatus`        |
| `on_order_filled`      | `OrderFilled`             |
| `on_order_canceled`    | `OrderCanceled`           |
| `on_time_event`        | `TimeEvent`               |

For a step-by-step walkthrough, see the
[Write an Actor (Rust)](../how_to/write_rust_actor.md) how-to guide.
For a complete example, see
[`BookImbalanceActor`](../../crates/trading/src/examples/actors/imbalance).

## Strategies

A strategy extends an actor with order management. Implement both
`DataActor` (for data handling) and `Strategy` (for access to
`StrategyCore`). The `StrategyCore` wraps `DataActorCore` and adds an
`OrderFactory`, `OrderManager`, and portfolio integration.

### Order management

The `Strategy` trait provides order methods through `StrategyCore`:

| Method                | Action                                    |
|-----------------------|-------------------------------------------|
| `submit_order`        | Submit a new order to the venue.          |
| `submit_order_list`   | Submit a list of contingent orders.       |
| `modify_order`        | Modify price, quantity, or trigger price. |
| `cancel_order`        | Cancel a specific order.                  |
| `cancel_orders`       | Cancel a filtered set of orders.          |
| `cancel_all_orders`   | Cancel all orders for an instrument.      |
| `close_position`      | Close a position with a market order.     |
| `close_all_positions` | Close all open positions.                 |

The `OrderFactory` (accessed via `self.core.order_factory()`) builds order
objects: `market`, `limit`, `stop_market`, `stop_limit`,
`market_if_touched`, `limit_if_touched`, and `trailing_stop_market`.

For a step-by-step walkthrough, see the
[Write a Strategy (Rust)](../how_to/write_rust_strategy.md) how-to guide.
For complete examples, see
[`EmaCross`](../../crates/trading/src/examples/strategies/ema_cross)
and
[`GridMarketMaker`](../../crates/trading/src/examples/strategies/grid_mm).

### Running Rust components

Rust strategies and actors run through the Rust path. The examples below use
strategies, but the same pattern applies to actors via `add_actor`.

#### Pure Rust

Write your strategy and `main` function in Rust, then build a standalone
binary with `cargo build`. This path requires no Python runtime.

```rust
let strategy = GridMarketMaker::new(config);
node.add_strategy(strategy)?;
node.run().await?;
```

See [Run Live Trading (Rust)](../how_to/run_rust_live_trading.md) for a
full walkthrough.

#### Legacy native config from Python

The upstream native-config-from-Python path is unsupported in NTPRO after the
Rust-only cutover. Do not use Python config objects, PyO3 wrappers, or
`add_native_strategy`/`add_native_actor` examples as current NTPRO product
entry points.

#### Plugin loading (planned)

A future plugin system will load compiled shared libraries at runtime.
Users compile strategies and actors as `cdylib` crates and the node
loads them without recompilation. This path is not yet available.

## Backtesting

For annotated walkthroughs of both APIs, see the
[Run a Backtest (Rust)](../how_to/run_rust_backtest.md) how-to guide.

### `BacktestEngine` (low-level API)

Construct the engine, add venues and instruments, load data, register
strategies, and run. See the full working example:

```bash
cargo run -p nautilus-backtest --features examples --example engine-ema-cross
```

Source:
[`crates/backtest/examples/engine_ema_cross.rs`](../../crates/backtest/examples/engine_ema_cross.rs)

### `BacktestNode` (high-level API)

Loads data from a `ParquetDataCatalog` and supports streaming in
configurable chunk sizes. Requires the `streaming` feature on
`nautilus-backtest`. See the full working example:

```bash
cargo run -p nautilus-backtest --features examples,streaming --example node-ema-cross
```

Source:
[`crates/backtest/examples/node_ema_cross.rs`](../../crates/backtest/examples/node_ema_cross.rs)

## Live trading

For an annotated walkthrough, see the
[Run Live Trading (Rust)](../how_to/run_rust_live_trading.md) how-to guide.

The `LiveNode` connects to real venues through adapter clients. The builder
pattern configures data and execution clients, then `run()` starts the async
event loop. Each adapter provides its own factory and config types.

| Adapter        | Example                                                  |
|----------------|----------------------------------------------------------|
| Architect AX   | `crates/adapters/architect_ax/examples/`                 |
| Betfair        | `crates/adapters/betfair/examples/`                      |
| Binance        | `crates/adapters/binance/examples/`                      |
| BitMEX         | `crates/adapters/bitmex/examples/`                       |
| Blockchain     | `crates/adapters/blockchain/examples/`                   |
| Bybit          | `crates/adapters/bybit/examples/`                        |
| Databento      | `crates/adapters/databento/examples/`                    |
| Deribit        | `crates/adapters/deribit/examples/`                      |
| dYdX           | `crates/adapters/dydx/examples/`                         |
| Hyperliquid    | `crates/adapters/hyperliquid/examples/`                  |
| Kraken         | `crates/adapters/kraken/examples/`                       |
| OKX            | `crates/adapters/okx/examples/`                          |
| Polymarket     | `crates/adapters/polymarket/examples/`                   |
| Sandbox        | `crates/adapters/sandbox/examples/`                      |
| Tardis         | `crates/adapters/tardis/examples/`                       |

Most adapters include `node_data_tester.rs` and `node_exec_tester.rs`
examples. These test data requests, streaming, and order execution
against live venues.

## Related guides

- [Write an Actor (Rust)](../how_to/write_rust_actor.md) - Step-by-step actor walkthrough.
- [Write a Strategy (Rust)](../how_to/write_rust_strategy.md) - Step-by-step strategy walkthrough.
- [Run a Backtest (Rust)](../how_to/run_rust_backtest.md) - BacktestEngine and BacktestNode usage.
- [Run Live Trading (Rust)](../how_to/run_rust_live_trading.md) - LiveNode setup and venue connection.
- [Architecture](architecture.md) - System design and data/execution flow.
- [Actors](actors.md) - Actor concepts for the Rust workspace.
- [Strategies](strategies.md) - Strategy concepts and handler reference.
- [Events](events.md) - Event types and handler dispatch.
- [Backtesting](backtesting.md) - Backtest concepts and matching engine behavior.
