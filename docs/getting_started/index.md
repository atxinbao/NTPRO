# Getting started

## 1. Install

Install the Rust toolchain and build the workspace with Cargo. See the
[Installation](installation) guide for platform support and development
dependencies. The supported v0.2.0 path is source-build first; local
`cargo install --path crates/cli --bin nautilus --locked --force` is supported
from a checked-out NTPRO repository, while prebuilt binaries and Docker
delivery remain deferred.

## 2. Run the Rust backtest guide

Start with [Run a Backtest (Rust)](../how_to/run_rust_backtest.md). It shows
the low-level `BacktestEngine` path and the high-level `BacktestNode` path
without relying on Python, PyO3, or Cython.

## 3. Choose your path

- **Backtesting** - start with [Run a Backtest (Rust)](../how_to/run_rust_backtest.md),
  then review the Rust tutorials for strategy pattern walkthroughs.
- **Live-node development** - start with the bounded
  [Rust sandbox live-init example](../../examples/rust/live/README.md). The
  v0.32.0 backend baseline does not authorize production venue execution.
- **Data workflows** - see the [how-to guides](../how_to/) for loading
  external data and setting up the Parquet data catalog.
- **Building adapters** - see the [Developer guide](../developer_guide/).

## Backtesting API levels

NTPRO exposes two Rust API levels for backtesting:

| API level                                      | Entry point     | Best for                                                          |
|:-----------------------------------------------|:----------------|:------------------------------------------------------------------|
| [Low-level Rust API](../how_to/run_rust_backtest.md#backtestengine-low-level-api) | `BacktestEngine` | Direct component access, library development |
| [High-level Rust API](../how_to/run_rust_backtest.md#backtestnode-high-level-api) | `BacktestNode` | Production workflows, easier transition to live trading |

The high‑level API requires a Parquet‑based data catalog. The low‑level API works with
in‑memory data but has no live‑trading path.

:::warning[One node per process]
Running multiple `BacktestNode` or `TradingNode` instances concurrently in the same
process is not supported due to global singleton state. Sequential execution with
proper disposal between runs is supported.

See [Processes and threads](../concepts/architecture.md#processes-and-threads) for
details.
:::

See the [Backtesting](../concepts/backtesting.md) concept guide for help choosing an
API level.

## Examples in the repository

NTPRO keeps Rust examples as the supported product surface:

| Directory                                      | Contains                                             |
|:-----------------------------------------------|:-----------------------------------------------------|
| [examples/rust/](../../examples/rust/)         | Rust example contracts and command walkthroughs      |
| [crates/backtest/examples/](../../crates/backtest/examples/) | Runnable Rust backtest Cargo examples |
| [crates/live/examples/](../../crates/live/examples/) | Runnable Rust live/sandbox Cargo examples |
| [docs/tutorials/](../tutorials/)               | Rust-only tutorials and migration walkthroughs       |

## Docker status

Docker and Jupyter images are not the default NTPRO v0.1.0 product delivery
path, and Docker delivery is not a v0.2.0 requirement. Use the Rust CLI,
Cargo examples, and local release verification scripts as the supported
starting point.
