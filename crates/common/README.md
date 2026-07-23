# nautilus-common

[![Rust Cutover Smoke](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml/badge.svg)](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml)
[![Documentation](https://img.shields.io/docsrs/nautilus-common)](https://docs.rs/nautilus-common/latest/nautilus-common/)
[![crates.io version](https://img.shields.io/crates/v/nautilus-common.svg)](https://crates.io/crates/nautilus-common)
![license](https://img.shields.io/github/license/atxinbao/NTPRO?color=blue)

Common componentry for NTPRO.

The `nautilus-common` crate provides shared components and utilities that form the system foundation for
NautilusTrader applications. This includes the actor system, message bus, caching layer, and other
essential services.

## NTPRO scope

NTPRO is an open-source, Rust-native
engine for multi-asset, multi-venue trading systems.

The system spans research, deterministic simulation, and live execution within a single
event-driven architecture, providing research-to-live semantic parity.

## Feature flags

This crate provides feature flags to control source code inclusion during compilation:

- `ffi`: Enables the C foreign function interface (FFI) from [cbindgen](https://github.com/mozilla/cbindgen).
- `defi`: Enables DeFi (Decentralized Finance) support.
- `indicators`: Includes the `nautilus-indicators` crate and indicator utilities.
- `live`: Enables the Tokio async runtime for live trading.
- `tracing-bridge`: Enables the `tracing` subscriber bridge for log integration.

## Documentation

See [the docs](https://docs.rs/nautilus-common) for more detailed usage.

## License

The NTPRO workspace retains NautilusTrader license lineage and is available under the [GNU Lesser General Public License v3.0](https://www.gnu.org/licenses/lgpl-3.0.en.html).

---

NTPRO is a Rust-only release workspace derived from NautilusTrader. It retains NautilusTrader license lineage; review the repository license files and release notes before distribution or operational use.

© 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
