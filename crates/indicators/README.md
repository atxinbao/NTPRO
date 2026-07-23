# nautilus-indicators

[![Rust Cutover Smoke](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml/badge.svg)](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml)
[![Documentation](https://img.shields.io/docsrs/nautilus-indicators)](https://docs.rs/nautilus-indicators/latest/nautilus-indicators/)
[![crates.io version](https://img.shields.io/crates/v/nautilus-indicators.svg)](https://crates.io/crates/nautilus-indicators)
![license](https://img.shields.io/github/license/atxinbao/NTPRO?color=blue)

Technical analysis indicators for NTPRO.

The `nautilus-indicators` crate provides a collection of technical analysis indicators
for quantitative trading and market research. This includes a wide variety of indicators
organized by category, with a unified trait-based architecture for consistent usage:

- **Moving averages**: SMA, EMA, DEMA, HMA, WMA, VWAP, adaptive averages, and linear regression.
- **Momentum indicators**: RSI, MACD, Aroon, Bollinger Bands, CCI, Stochastics, and rate of change.
- **Volatility indicators**: ATR, Donchian Channels, Keltner Channels, and volatility ratios.
- **Ratio analysis**: Efficiency ratios and spread analysis for relative performance.
- **Order book indicators**: Book imbalance ratio for analyzing market microstructure.
- **Common indicator trait**: Unified interface supporting bars, quotes, trades, and order book data.

All indicators are designed for high-performance real-time processing with bounded memory
usage and efficient circular buffer implementations. The crate supports both Rust-native
usage and legacy integration for strategy development and backtesting.

## NTPRO scope

NTPRO is an open-source, Rust-native
engine for multi-asset, multi-venue trading systems.

The system spans research, deterministic simulation, and live execution within a single
event-driven architecture, providing research-to-live semantic parity.

## Feature flags

This crate provides feature flags to control source code inclusion during compilation:

## Documentation

See [the docs](https://docs.rs/nautilus-indicators) for more detailed usage.

## License

The NTPRO workspace retains NautilusTrader license lineage and is available under the [GNU Lesser General Public License v3.0](https://www.gnu.org/licenses/lgpl-3.0.en.html).

---

NTPRO is a Rust-only release workspace derived from NautilusTrader. It retains NautilusTrader license lineage; review the repository license files and release notes before distribution or operational use.

© 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
