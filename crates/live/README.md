# nautilus-live

[![Rust Cutover Smoke](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml/badge.svg)](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml)
[![Documentation](https://img.shields.io/docsrs/nautilus-live)](https://docs.rs/nautilus-live/latest/nautilus-live/)
[![crates.io version](https://img.shields.io/crates/v/nautilus-live.svg)](https://crates.io/crates/nautilus-live)
![license](https://img.shields.io/github/license/atxinbao/NTPRO?color=blue)

Live system node for NTPRO.

The `nautilus-live` crate provides high-level abstractions and infrastructure for running live trading
systems, including data streaming, execution management, and system lifecycle handling.
It builds on top of the system kernel to provide simplified interfaces for live deployment:

- `LiveNode` High-level abstraction for live system nodes.
- `LiveNodeConfig` Configuration for live node deployment.
- `AsyncRunner` for managing system real-time data flow.

## NTPRO scope

NTPRO is an open-source, Rust-native
engine for multi-asset, multi-venue trading systems.

The system spans research, deterministic simulation, and live execution within a single
event-driven architecture, providing research-to-live semantic parity.

## Feature flags

This crate provides feature flags to control source code inclusion during compilation:

- `ffi`: Enables the C foreign function interface (FFI) from [cbindgen](https://github.com/mozilla/cbindgen).
- `streaming`: Enables `persistence` dependency for streaming configuration.
- `defi`: Enables DeFi (Decentralized Finance) support.

## Documentation

See [the docs](https://docs.rs/nautilus-live) for more detailed usage.

## License

The NTPRO workspace retains NautilusTrader license lineage and is available under the [GNU Lesser General Public License v3.0](https://www.gnu.org/licenses/lgpl-3.0.en.html).

---

NTPRO is a Rust-only release workspace derived from NautilusTrader. It retains NautilusTrader license lineage; review the repository license files and release notes before distribution or operational use.

© 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
