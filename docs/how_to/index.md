# How-to Guides

Goal-oriented recipes for common tasks. Each guide assumes familiarity with
Nautilus concepts and focuses on achieving a specific outcome.

New to Nautilus? Start with the [getting started](../getting_started/)
path and the Rust guides below first.

## Rust

| Guide                                                     | Description                                            |
|:----------------------------------------------------------|:-------------------------------------------------------|
| [Write an Actor (Rust)](write_rust_actor)                 | Build a data actor with subscriptions and handlers.    |
| [Write a Strategy (Rust)](write_rust_strategy)            | Build a strategy with order management.                |
| [Run a Backtest (Rust)](run_rust_backtest)                | Use BacktestEngine or BacktestNode with a catalog.     |
| [Run Live Trading (Rust)](run_rust_live_trading)          | Connect to a venue with LiveNode.                      |

:::warning[Backend freeze boundary]
The Rust live guide documents library construction, not production go-live
authorization. The v0.32.0 backend baseline authorizes only the bounded
[sandbox live-init example](../../examples/rust/live/README.md); external venue
execution remains separately scoped.
:::
