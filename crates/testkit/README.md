# nautilus-testkit

[![Rust Cutover Smoke](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml/badge.svg)](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml)
[![Documentation](https://img.shields.io/docsrs/nautilus-testkit)](https://docs.rs/nautilus-testkit/latest/nautilus-testkit/)
[![crates.io version](https://img.shields.io/crates/v/nautilus-testkit.svg)](https://crates.io/crates/nautilus-testkit)
![license](https://img.shields.io/github/license/atxinbao/NTPRO?color=blue)

Test utilities and data management for NTPRO.

The `nautilus-testkit` crate provides testing utilities including test data management,
file handling, and common testing patterns. This crate supports testing workflows
across the entire NautilusTrader ecosystem with automated data downloads and validation:

- **Test data management**: Automated downloading and caching of test datasets.
- **File utilities**: File integrity verification with SHA-256 checksums.
- **Path resolution**: Platform-agnostic test data path management.
- **Precision handling**: Support for both 64-bit and 128-bit precision test data.
- **Common patterns**: Reusable test utilities and helper functions.

## NTPRO scope

NTPRO is an open-source, Rust-native
engine for multi-asset, multi-venue trading systems.

The system spans research, deterministic simulation, and live execution within a single
event-driven architecture, providing research-to-live semantic parity.

## Feature flags

This crate provides feature flags to control source code inclusion during compilation:

- `high-precision`: Enables [high-precision mode](https://github.com/atxinbao/NTPRO/blob/main/docs/getting_started/installation.md#precision-mode) to use 128-bit value types.

## Documentation

See [the docs](https://docs.rs/nautilus-testkit) for more detailed usage.

## License

The NTPRO workspace retains NautilusTrader license lineage and is available under the [GNU Lesser General Public License v3.0](https://www.gnu.org/licenses/lgpl-3.0.en.html).

---

NTPRO is a Rust-only release workspace derived from NautilusTrader. It retains NautilusTrader license lineage; review the repository license files and release notes before distribution or operational use.

© 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
