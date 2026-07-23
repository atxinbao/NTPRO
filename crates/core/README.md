# nautilus-core

[![Rust Cutover Smoke](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml/badge.svg?branch=main)](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml)
[![Documentation](https://img.shields.io/docsrs/nautilus-core)](https://docs.rs/nautilus-core/latest/nautilus-core/)
[![crates.io version](https://img.shields.io/crates/v/nautilus-core.svg)](https://crates.io/crates/nautilus-core)
![license](https://img.shields.io/github/license/atxinbao/NTPRO?color=blue)

Core foundational types and utilities for NTPRO.

The `nautilus-core` crate is designed to be lightweight, efficient, and to provide zero-cost abstractions
wherever possible. It supplies the essential building blocks used across the NautilusTrader
ecosystem, including:

- Time handling and atomic clock functionality.
- UUID generation and management.
- Mathematical functions and interpolation utilities.
- Correctness validation functions.
- Serialization traits and helpers.
- Cross-platform environment utilities.
- Abstractions over common collections.

## NTPRO scope

NTPRO is an open-source, Rust-native
engine for multi-asset, multi-venue trading systems.

The system spans research, deterministic simulation, and live execution within a single
event-driven architecture, providing research-to-live semantic parity.

## Feature flags

This crate provides feature flags to control source code inclusion during compilation:

- `ffi`: Enables the C foreign function interface (FFI) from [cbindgen](https://github.com/mozilla/cbindgen).

## Documentation

See [the docs](https://docs.rs/nautilus-core) for more detailed usage.

## License

The NTPRO workspace retains NautilusTrader license lineage and is available under the [GNU Lesser General Public License v3.0](https://www.gnu.org/licenses/lgpl-3.0.en.html).

---

NTPRO is a Rust-only release workspace derived from NautilusTrader. It retains NautilusTrader license lineage; review the repository license files and release notes before distribution or operational use.

© 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
