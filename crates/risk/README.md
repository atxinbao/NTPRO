# nautilus-risk

[![Rust Cutover Smoke](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml/badge.svg)](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml)
[![Documentation](https://img.shields.io/docsrs/nautilus-risk)](https://docs.rs/nautilus-risk/latest/nautilus-risk/)
[![crates.io version](https://img.shields.io/crates/v/nautilus-risk.svg)](https://crates.io/crates/nautilus-risk)
![license](https://img.shields.io/github/license/atxinbao/NTPRO?color=blue)

Risk engine for NTPRO.

The `nautilus-risk` crate provides risk management capabilities including pre-trade
order validation, position sizing calculations, and trading controls. This system ensures
trading operations remain within defined risk parameters and regulatory constraints:

- **Risk engine**: Central risk management orchestration with configurable trading states.
- **Order validation**: Pre-trade checks for price, quantity, notional limits, and market conditions.
- **Position sizing**: Fixed-risk position sizing calculations with commission and exchange rate support.
- **Trading controls**: Rate limiting, balance validation, and exposure management.
- **Account protection**: Multi-currency balance checks and margin requirement validation.

## NTPRO scope

NTPRO is an open-source, Rust-native
engine for multi-asset, multi-venue trading systems.

The system spans research, deterministic simulation, and live execution within a single
event-driven architecture, providing research-to-live semantic parity.

## Feature flags

This crate provides feature flags to control source code inclusion during compilation:

## Documentation

See [the docs](https://docs.rs/nautilus-risk) for more detailed usage.

## License

The NTPRO workspace retains NautilusTrader license lineage and is available under the [GNU Lesser General Public License v3.0](https://www.gnu.org/licenses/lgpl-3.0.en.html).

---

NTPRO is a Rust-only release workspace derived from NautilusTrader. It retains NautilusTrader license lineage; review the repository license files and release notes before distribution or operational use.

© 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
