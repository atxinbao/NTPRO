# nautilus-databento

[![Rust Cutover Smoke](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml/badge.svg)](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml)
[![Documentation](https://img.shields.io/docsrs/nautilus-databento)](https://docs.rs/nautilus-databento/latest/nautilus-databento/)
[![crates.io version](https://img.shields.io/crates/v/nautilus-databento.svg)](https://crates.io/crates/nautilus-databento)
![license](https://img.shields.io/github/license/atxinbao/NTPRO?color=blue)

NTPRO adapter for [Databento](https://databento.com).

The `nautilus-databento` crate provides a complete integration with the Databento API for
accessing institutional-grade market data feeds across multiple venues and asset classes.

## NTPRO scope

NTPRO is an open-source, Rust-native
engine for multi-asset, multi-venue trading systems.

The system spans research, deterministic simulation, and live execution within a single
event-driven architecture, providing research-to-live semantic parity.

## Feature flags

This crate provides feature flags to control source code inclusion during compilation:

- `live` (default): Enables live data functionality including the `data`, `factories`, and `live` modules.
- `high-precision`: Enables [high-precision mode](https://github.com/atxinbao/NTPRO/blob/main/docs/getting_started/installation.md#precision-mode) to use 128-bit value types.

## Documentation

See [the docs](https://docs.rs/nautilus-databento) for more detailed usage.

## License

The NTPRO workspace retains NautilusTrader license lineage and is available under the [GNU Lesser General Public License v3.0](https://www.gnu.org/licenses/lgpl-3.0.en.html).

---

NTPRO is a Rust-only release workspace derived from NautilusTrader. It retains NautilusTrader license lineage; review the repository license files and release notes before distribution or operational use.

© 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
