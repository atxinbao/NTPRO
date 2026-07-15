# Tutorials

Step-by-step walkthroughs demonstrating specific features and workflows.

:::info
NTPRO tutorials are Rust-only product documentation. Legacy Jupytext Python
tutorial scripts have been removed from this repository.
:::

## Recommended order

New to NTPRO? Work through these in sequence:

1. [Run a Backtest (Rust)](../how_to/run_rust_backtest) - learn the
   `BacktestEngine` and `BacktestNode` paths.
2. [Write an Actor (Rust)](../how_to/write_rust_actor) - build a data actor.
3. [Write a Strategy (Rust)](../how_to/write_rust_strategy) - build a
   strategy with order management.
4. Pick a topic-specific Rust tutorial below.

## Strategy patterns

| Tutorial                                                                            | Description                                       | Data              |
|:------------------------------------------------------------------------------------|:--------------------------------------------------|:------------------|
| [On‑Chain Grid Market Making with Short‑Term Orders (dYdX)](grid_market_maker_dydx) | Grid MM on dYdX v4 perpetuals.                    | User‑provided     |

## Options

| Tutorial                                                                            | Description                                       | Data              |
|:------------------------------------------------------------------------------------|:--------------------------------------------------|:------------------|
| [Options Data and Greeks (Bybit)](options_data_bybit)                               | Stream Greeks and option chain snapshots.         | Live API          |
| [Delta‑Neutral Options Strategy (Bybit)](delta_neutral_options_bybit)               | Short strangle with perpetual delta hedging.      | Live API          |

## Rust

| Tutorial                                                                            | Description                                          | Data           |
|:------------------------------------------------------------------------------------|:-----------------------------------------------------|:---------------|
| [Book Imbalance Backtest (Betfair)](backtest_book_imbalance_betfair)                | Book imbalance actor on Betfair L2 data.             | User‑provided  |
| [Hurst/VPIN Directional Strategy (Kraken Futures)](hurst_vpin_kraken)               | Regime‑filtered informed‑flow strategy on PF_XBTUSD. | Tardis.dev     |
