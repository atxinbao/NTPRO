# nautilus-betfair

[![Rust Cutover Smoke](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml/badge.svg)](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml)
[![Documentation](https://img.shields.io/docsrs/nautilus-betfair)](https://docs.rs/nautilus-betfair/latest/nautilus-betfair/)
[![crates.io version](https://img.shields.io/crates/v/nautilus-betfair.svg)](https://crates.io/crates/nautilus-betfair)
![license](https://img.shields.io/github/license/atxinbao/NTPRO?color=blue)

NTPRO adapter for the [Betfair](https://www.betfair.com/) betting exchange.

The `nautilus-betfair` crate provides data and execution clients, streaming
and REST API models, and full NautilusTrader integration for the
[Betfair](https://www.betfair.com/) betting exchange.

The official API reference can be found at <https://docs.developer.betfair.com/>.

## NTPRO scope

NTPRO is an open-source, Rust-native
engine for multi-asset, multi-venue trading systems.

The system spans research, deterministic simulation, and live execution within a single
event-driven architecture, providing research-to-live semantic parity.

## Feature flags

This crate provides feature flags to control source code inclusion during compilation:

- `high-precision`: Enables [128-bit value types](https://github.com/atxinbao/NTPRO/blob/main/docs/getting_started/installation.md#precision-mode) from `nautilus-model`.

## Documentation

See [the docs](https://docs.rs/nautilus-betfair) for more detailed usage.

## License

The NTPRO workspace retains NautilusTrader license lineage and is available under the [GNU Lesser General Public License v3.0](https://www.gnu.org/licenses/lgpl-3.0.en.html).

---

NTPRO is a Rust-only release workspace derived from NautilusTrader. It retains NautilusTrader license lineage; review the repository license files and release notes before distribution or operational use.

© 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
