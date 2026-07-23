# nautilus-backtest

[![Rust Cutover Smoke](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml/badge.svg)](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml)
[![Documentation](https://img.shields.io/docsrs/nautilus-backtest)](https://docs.rs/nautilus-backtest/latest/nautilus-backtest/)
[![crates.io version](https://img.shields.io/crates/v/nautilus-backtest.svg)](https://crates.io/crates/nautilus-backtest)
![license](https://img.shields.io/github/license/atxinbao/NTPRO?color=blue)

Backtest engine for NTPRO.

The `nautilus-backtest` crate provides an event-driven backtesting framework that allows
quantitative traders to test and validate trading strategies on historical data with high
fidelity market simulation. The system replicates real market conditions including:

- Event-driven backtesting engine with simulated exchanges.
- Market data replay with configurable latency and fill models.
- Order matching engines with realistic execution simulation.
- Multi-venue and multi-asset backtesting capabilities.
- Configuration and state management.

## NTPRO scope

NTPRO is an open-source, Rust-native
engine for multi-asset, multi-venue trading systems.

The system spans research, deterministic simulation, and live execution within a single
event-driven architecture, providing research-to-live semantic parity.

## Feature flags

This crate provides feature flags to control source code inclusion during compilation:

- `examples`: Enables example strategies and the EMA crossover backtest example.
- `streaming`: Enables `persistence` dependency for streaming configuration.
- `ffi`: Enables the C foreign function interface (FFI) from [cbindgen](https://github.com/mozilla/cbindgen).

## Documentation

See [the docs](https://docs.rs/nautilus-backtest) for more detailed usage.

## License

The NTPRO workspace retains NautilusTrader license lineage and is available under the [GNU Lesser General Public License v3.0](https://www.gnu.org/licenses/lgpl-3.0.en.html).

---

NTPRO is a Rust-only release workspace derived from NautilusTrader. It retains NautilusTrader license lineage; review the repository license files and release notes before distribution or operational use.

© 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
