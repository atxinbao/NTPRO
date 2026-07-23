# nautilus-sandbox

[![Rust Cutover Smoke](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml/badge.svg?branch=main)](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml)
[![Documentation](https://img.shields.io/docsrs/nautilus-sandbox)](https://docs.rs/nautilus-sandbox/latest/nautilus-sandbox/)
[![crates.io version](https://img.shields.io/crates/v/nautilus-sandbox.svg)](https://crates.io/crates/nautilus-sandbox)
![license](https://img.shields.io/github/license/atxinbao/NTPRO?color=blue)

NTPRO sandbox execution adapter for paper trading.

The `nautilus-sandbox` crate provides a simulated execution client that uses the
`OrderMatchingEngine` to simulate order execution against live market data. This enables
paper trading and strategy testing in real-time without actual order execution on exchanges.

## Features

- Paper trading against live market data from any data source.
- Full order matching simulation using the deterministic `OrderMatchingEngine`.
- Support for all order types (market, limit, stop, etc.).
- Configurable fill models and fee models.
- Account balance and position tracking.
- Support for both cash and margin account types.

## NTPRO scope

NTPRO is an open-source, Rust-native
engine for multi-asset, multi-venue trading systems.

The system spans research, deterministic simulation, and live execution within a single
event-driven architecture, providing research-to-live semantic parity.

## Feature flags

This crate provides feature flags to control source code inclusion during compilation:

[High-precision mode](https://github.com/atxinbao/NTPRO/blob/main/docs/getting_started/installation.md#precision-mode) (128-bit value types) is enabled by default.

## Documentation

See [the docs](https://docs.rs/nautilus-sandbox) for more detailed usage.

## License

The NTPRO workspace retains NautilusTrader license lineage and is available under the [GNU Lesser General Public License v3.0](https://www.gnu.org/licenses/lgpl-3.0.en.html).

---

NTPRO is a Rust-only release workspace derived from NautilusTrader. It retains NautilusTrader license lineage; review the repository license files and release notes before distribution or operational use.

© 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
