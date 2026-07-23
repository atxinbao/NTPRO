# nautilus-portfolio

[![Rust Cutover Smoke](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml/badge.svg?branch=main)](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml)
[![Documentation](https://img.shields.io/docsrs/nautilus-portfolio)](https://docs.rs/nautilus-portfolio/latest/nautilus-portfolio/)
[![crates.io version](https://img.shields.io/crates/v/nautilus-portfolio.svg)](https://crates.io/crates/nautilus-portfolio)
![license](https://img.shields.io/github/license/atxinbao/NTPRO?color=blue)

Portfolio management and risk analysis for NTPRO.

The `nautilus-portfolio` crate provides portfolio management capabilities including
real-time position tracking, performance calculations, and risk management. This includes
sophisticated portfolio analytics and multi-currency support:

- **Portfolio tracking**: Real-time portfolio state management with position and balance monitoring.
- **Account management**: Support for cash and margin accounts across multiple venues.
- **Performance calculations**: Real-time unrealized PnL, realized PnL, and mark-to-market valuations.
- **Risk management**: Initial margin calculations, maintenance margin tracking, and exposure monitoring.
- **Multi-currency support**: Currency conversion and cross-currency risk exposure analysis.
- **Configuration options**: Flexible settings for price types, currency conversion, and portfolio behavior.

The crate handles complex portfolio scenarios including multi-venue trading, currency conversions,
and sophisticated margin calculations for both live trading and backtesting environments.

## NTPRO scope

NTPRO is an open-source, Rust-native
engine for multi-asset, multi-venue trading systems.

The system spans research, deterministic simulation, and live execution within a single
event-driven architecture, providing research-to-live semantic parity.

## Feature flags

This crate provides feature flags to control source code inclusion during compilation:

## Documentation

See [the docs](https://docs.rs/nautilus-portfolio) for more detailed usage.

## License

The NTPRO workspace retains NautilusTrader license lineage and is available under the [GNU Lesser General Public License v3.0](https://www.gnu.org/licenses/lgpl-3.0.en.html).

---

NTPRO is a Rust-only release workspace derived from NautilusTrader. It retains NautilusTrader license lineage; review the repository license files and release notes before distribution or operational use.

© 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
