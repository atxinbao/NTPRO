# nautilus-persistence

[![Rust Cutover Smoke](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml/badge.svg)](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml)
[![Documentation](https://img.shields.io/docsrs/nautilus-persistence)](https://docs.rs/nautilus-persistence/latest/nautilus-persistence/)
[![crates.io version](https://img.shields.io/crates/v/nautilus-persistence.svg)](https://crates.io/crates/nautilus-persistence)
![license](https://img.shields.io/github/license/atxinbao/NTPRO?color=blue)

Data persistence and storage for NTPRO.

The `nautilus-persistence` crate provides data persistence capabilities including reading and writing
trading data to various storage backends. This includes Apache Parquet file support, streaming data
pipelines, and cloud storage integration for historical data management.

## NTPRO scope

NTPRO is an open-source, Rust-native
engine for multi-asset, multi-venue trading systems.

The system spans research, deterministic simulation, and live execution within a single
event-driven architecture, providing research-to-live semantic parity.

## Feature flags

This crate provides feature flags to control source code inclusion during compilation:

- `cloud`: Enables cloud storage backends (S3, Azure, GCP, HTTP) via `object_store`.
- `ffi`: Enables the C foreign function interface (FFI) from [cbindgen](https://github.com/mozilla/cbindgen).
- `high-precision`: Enables [high-precision mode](https://github.com/atxinbao/NTPRO/blob/main/docs/getting_started/installation.md#precision-mode) to use 128-bit value types.

## Documentation

See [the docs](https://docs.rs/nautilus-persistence) for more detailed usage.

## License

The NTPRO workspace retains NautilusTrader license lineage and is available under the [GNU Lesser General Public License v3.0](https://www.gnu.org/licenses/lgpl-3.0.en.html).

---

NTPRO is a Rust-only release workspace derived from NautilusTrader. It retains NautilusTrader license lineage; review the repository license files and release notes before distribution or operational use.

© 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
