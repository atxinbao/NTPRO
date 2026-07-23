# nautilus-data

[![Rust Cutover Smoke](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml/badge.svg?branch=main)](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml)
[![Documentation](https://img.shields.io/docsrs/nautilus-data)](https://docs.rs/nautilus-data/latest/nautilus-data/)
[![crates.io version](https://img.shields.io/crates/v/nautilus-data.svg)](https://crates.io/crates/nautilus-data)
![license](https://img.shields.io/github/license/atxinbao/NTPRO?color=blue)

Data engine and market data processing for NTPRO.

The `nautilus-data` crate provides a framework for handling market data ingestion,
processing, and aggregation within the NautilusTrader ecosystem. This includes real-time
data streaming, historical data management, and various aggregation methodologies:

- High-performance data engine for orchestrating data operations.
- Data client infrastructure for connecting to market data providers.
- Bar aggregation machinery supporting tick, volume, value, and time-based aggregation.
- Order book management and delta processing capabilities.
- Subscription management and data request handling.
- Configurable data routing and processing pipelines.

## NTPRO scope

NTPRO is an open-source, Rust-native
engine for multi-asset, multi-venue trading systems.

The system spans research, deterministic simulation, and live execution within a single
event-driven architecture, providing research-to-live semantic parity.

## Feature flags

This crate provides feature flags to control source code inclusion during compilation:

- `streaming`: Enables `persistence` dependency for catalog-based data streaming.
- `ffi`: Enables the C foreign function interface (FFI) from [cbindgen](https://github.com/mozilla/cbindgen).
- `high-precision`: Enables [high-precision mode](https://github.com/atxinbao/NTPRO/blob/main/docs/getting_started/installation.md#precision-mode) to use 128-bit value types.
- `defi`: Enables DeFi (Decentralized Finance) support.

## Documentation

See [the docs](https://docs.rs/nautilus-data) for more detailed usage.

## License

The NTPRO workspace retains NautilusTrader license lineage and is available under the [GNU Lesser General Public License v3.0](https://www.gnu.org/licenses/lgpl-3.0.en.html).

---

NTPRO is a Rust-only release workspace derived from NautilusTrader. It retains NautilusTrader license lineage; review the repository license files and release notes before distribution or operational use.

© 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
