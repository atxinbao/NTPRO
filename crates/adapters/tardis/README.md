# nautilus-tardis

[![Rust Cutover Smoke](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml/badge.svg?branch=main)](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml)
[![Documentation](https://img.shields.io/docsrs/nautilus-tardis)](https://docs.rs/nautilus-tardis/latest/nautilus-tardis/)
[![crates.io version](https://img.shields.io/crates/v/nautilus-tardis.svg)](https://crates.io/crates/nautilus-tardis)
![license](https://img.shields.io/github/license/atxinbao/NTPRO?color=blue)

NTPRO adapter for [Tardis](https://tardis.dev).

The `nautilus-tardis` crate provides integration with the Tardis API for accessing
normalized historical and real-time market data across multiple exchanges.

## NTPRO scope

NTPRO is an open-source, Rust-native
engine for multi-asset, multi-venue trading systems.

The system spans research, deterministic simulation, and live execution within a single
event-driven architecture, providing research-to-live semantic parity.

## Feature flags

This crate provides feature flags to control source code inclusion during compilation,
depending on the intended use case:

- `replay` (default): Enables market data replay functionality.

[High-precision mode](https://github.com/atxinbao/NTPRO/blob/main/docs/getting_started/installation.md#precision-mode) (128-bit value types) is enabled by default.

## Documentation

See [the docs](https://docs.rs/nautilus-tardis) for more detailed usage.

## License

The NTPRO workspace retains NautilusTrader license lineage and is available under the [GNU Lesser General Public License v3.0](https://www.gnu.org/licenses/lgpl-3.0.en.html).

---

NTPRO is a Rust-only release workspace derived from NautilusTrader. It retains NautilusTrader license lineage; review the repository license files and release notes before distribution or operational use.

© 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
