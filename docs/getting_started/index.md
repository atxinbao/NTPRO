# Getting started

## 1. Install

Install the Rust toolchain and build the workspace with Cargo. See the
[Installation](installation) guide for platform support and development
dependencies.

## 2. Run the Rust backtest guide

Start with [Run a Backtest (Rust)](../how_to/run_rust_backtest.md). It shows
the low-level `BacktestEngine` path and the high-level `BacktestNode` path
without relying on Python, PyO3, or Cython.

## 3. Choose your path

- **Backtesting** - start with [Run a Backtest (Rust)](../how_to/run_rust_backtest.md),
  then review the Rust tutorials for strategy pattern walkthroughs.
- **Live trading** - see the
  [Configure a live trading node](../how_to/configure_live_trading.md) how-to
  and [Integrations](../integrations/) for supported venues.
- **Data workflows** - see the [how-to guides](../how_to/) for loading
  external data and setting up the Parquet data catalog.
- **Building adapters** - see the [Developer guide](../developer_guide/).

## Backtesting API levels

NautilusTrader provides two API levels for backtesting:

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

## Running in Docker

A self-contained dockerized Jupyter notebook server provides the fastest way to try
NautilusTrader with no local setup. Deleting the container deletes any data.

```bash
# Pull the latest image
docker pull ghcr.io/nautechsystems/jupyterlab:nightly --platform linux/amd64

# Run the container
docker run -p 8888:8888 ghcr.io/nautechsystems/jupyterlab:nightly
```

Then open <http://localhost:8888> in your browser. The legacy Python notebook
examples have been removed from this repository; use Rust guides for supported
local workflows.
